# Phase 1 — PubChem Integration

## Goal

Resolve compound names (common names, IUPAC, formulas) to canonical SMILES using the PubChem REST API in Rust, with caching.

---

## PubChem REST API

Base URL: `https://pubchem.ncbi.nlm.nih.gov/rest/pug`

### Useful Endpoints

| Query                    | Endpoint                                                                       |
| ------------------------ | ------------------------------------------------------------------------------ |
| Name → CID               | `/compound/name/{name}/cids/JSON`                                              |
| CID → SMILES             | `/compound/cid/{cid}/property/CanonicalSMILES,MolecularFormula,IUPACName/JSON` |
| Name → SMILES (combined) | `/compound/name/{name}/property/CanonicalSMILES,MolecularFormula/JSON`         |
| CAS → info               | `/compound/name/{cas_number}/JSON`                                             |

### Example Request

```
GET https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/name/acetic%20acid/property/CanonicalSMILES,MolecularFormula,IUPACName/JSON

Response:
{
  "PropertyTable": {
    "Properties": [{
      "CID": 176,
      "CanonicalSMILES": "CC(=O)O",
      "MolecularFormula": "C2H4O2",
      "IUPACName": "acetic acid"
    }]
  }
}
```

---

## Rust Implementation (`src/predictor/pubchem.rs`)

```rust
use reqwest::Client;
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct CompoundInfo {
    pub query: String,
    pub smiles: String,
    pub formula: String,
    pub iupac_name: String,
    pub cid: i64,
}

// PubChem response deserialization
#[derive(Deserialize)]
struct PubChemResponse {
    #[serde(rename = "PropertyTable")]
    property_table: PropertyTable,
}

#[derive(Deserialize)]
struct PropertyTable {
    #[serde(rename = "Properties")]
    properties: Vec<PropertyEntry>,
}

#[derive(Deserialize)]
struct PropertyEntry {
    #[serde(rename = "CID")]
    cid: i64,
    #[serde(rename = "CanonicalSMILES")]
    smiles: String,
    #[serde(rename = "MolecularFormula")]
    formula: String,
    #[serde(rename = "IUPACName")]
    iupac_name: Option<String>,
}

pub struct PubChemClient {
    client: Client,
    base_url: String,
    db: SqlitePool,
}

impl PubChemClient {
    pub fn new(db: SqlitePool) -> Self {
        Self {
            client: Client::builder()
                .user_agent("ChemInteractions/1.0")
                .timeout(std::time::Duration::from_secs(10))
                .build().unwrap(),
            base_url: "https://pubchem.ncbi.nlm.nih.gov/rest/pug".into(),
            db,
        }
    }

    // Resolve with cache-first lookup
    pub async fn resolve(&self, name: &str) -> anyhow::Result<CompoundInfo> {
        // 1. Check cache
        if let Some(cached) = self.check_cache(name).await? {
            return Ok(cached);
        }

        // 2. If looks like SMILES already (contains '=' or '#' or digits),
        //    skip PubChem and return as-is
        if is_smiles(name) {
            return Ok(CompoundInfo {
                query: name.to_string(),
                smiles: name.to_string(),
                formula: "".to_string(),
                iupac_name: name.to_string(),
                cid: -1,
            });
        }

        // 3. PubChem lookup
        let url = format!(
            "{}/compound/name/{}/property/CanonicalSMILES,MolecularFormula,IUPACName/JSON",
            self.base_url,
            urlencoding::encode(name)
        );
        let resp: PubChemResponse = self.client.get(&url).send().await?.json().await?;
        let entry = resp.property_table.properties.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Compound not found: {}", name))?;

        let info = CompoundInfo {
            query: name.to_string(),
            smiles: entry.smiles,
            formula: entry.formula,
            iupac_name: entry.iupac_name.unwrap_or_default(),
            cid: entry.cid,
        };

        // 4. Store in cache
        self.cache(&info).await?;
        Ok(info)
    }

    async fn check_cache(&self, name: &str) -> anyhow::Result<Option<CompoundInfo>> {
        let row = sqlx::query!(
            "SELECT smiles, formula, iupac_name, cid FROM compounds_cache WHERE name_query = ?",
            name
        ).fetch_optional(&self.db).await?;

        Ok(row.map(|r| CompoundInfo {
            query: name.to_string(),
            smiles: r.smiles,
            formula: r.formula.unwrap_or_default(),
            iupac_name: r.iupac_name.unwrap_or_default(),
            cid: r.cid.unwrap_or(-1),
        }))
    }

    async fn cache(&self, info: &CompoundInfo) -> anyhow::Result<()> {
        sqlx::query!(
            "INSERT OR REPLACE INTO compounds_cache (name_query, smiles, formula, iupac_name, cid)
             VALUES (?, ?, ?, ?, ?)",
            info.query, info.smiles, info.formula, info.iupac_name, info.cid
        ).execute(&self.db).await?;
        Ok(())
    }
}

fn is_smiles(s: &str) -> bool {
    // Simple heuristic: contains chemistry-specific chars not in English names
    s.contains('=') || s.contains('#') || s.contains('[') || s.contains('@')
}
```

---

## Input Normalization Flow

```
User types "acetic acid"
    ↓
is_smiles("acetic acid")? → false
    ↓
cache hit? → if yes, return cached SMILES
    ↓
PubChem API → CanonicalSMILES: "CC(=O)O"
    ↓
Store in compounds_cache
    ↓
Return CompoundInfo { smiles: "CC(=O)O", formula: "C2H4O2", ... }
```

---

## Common Name Reference (Pre-loaded Cache Entries)

Pre-seed the cache with these common compounds to avoid API calls:

| Name              | SMILES                                 | Formula |
| ----------------- | -------------------------------------- | ------- |
| Water             | O                                      | H₂O     |
| Hydrochloric acid | Cl                                     | HCl     |
| Sodium hydroxide  | [Na+].[OH-]                            | NaOH    |
| Sulfuric acid     | OS(=O)(=O)O                            | H₂SO₄   |
| Ethanol           | CCO                                    | C₂H₅OH  |
| Acetic acid       | CC(=O)O                                | CH₃COOH |
| Methane           | C                                      | CH₄     |
| Ammonia           | N                                      | NH₃     |
| Glucose           | OC[C@H]1OC(O)[C@H](O)[C@@H](O)[C@@H]1O | C₆H₁₂O₆ |
| Carbon dioxide    | O=C=O                                  | CO₂     |

---

## Autocomplete API (for frontend)

```
GET /api/compound/search?q=acet&limit=5

Response:
[
  { "name": "acetic acid", "formula": "C2H4O2", "smiles": "CC(=O)O" },
  { "name": "acetone", "formula": "C3H6O", "smiles": "CC(C)=O" },
  ...
]
```

Implemented by searching `compounds_cache.iupac_name LIKE ?` after the user has searched before, with a PubChem autocomplete fallback.

---

## Checklist

- [ ] `resolve("acetic acid")` returns `smiles: "CC(=O)O"`
- [ ] `resolve("CC(=O)O")` (SMILES input) passes through unchanged
- [ ] Cache hit avoids second API call
- [ ] Unknown compound returns a clear error (not a panic)
- [ ] Rate limiting: add 200ms delay between PubChem requests if batching
