# Phase 1 — Knowledge Base Design

## Goal

Design the SQLite schema and JSON rule format for the textbook-sourced reaction knowledge base, and implement seeding on startup.

---

## SQLite Schema (`db/schema.sql`)

```sql
CREATE TABLE IF NOT EXISTS reaction_rules (
    id                    TEXT PRIMARY KEY,
    name                  TEXT NOT NULL,
    reaction_type         TEXT NOT NULL,  -- 'organic' | 'inorganic' | 'physical'
    category              TEXT NOT NULL,  -- 'neutralization' | 'condensation' | etc.
    reactant_classes      TEXT NOT NULL,  -- JSON array of strings
    conditions_favored    TEXT NOT NULL,  -- JSON array of strings
    conditions_inhibited  TEXT NOT NULL,  -- JSON array of strings
    byproducts            TEXT NOT NULL,  -- JSON array of {smiles, name}
    hazards               TEXT NOT NULL,  -- JSON array of strings
    kb_probability_modifier REAL NOT NULL DEFAULT 0.75,
    mechanism_summary     TEXT NOT NULL,
    references            TEXT NOT NULL,  -- JSON array of strings
    created_at            DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS functional_groups (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,           -- e.g. "carboxylic_acid"
    smarts      TEXT NOT NULL,           -- SMARTS pattern for matching
    aliases     TEXT NOT NULL DEFAULT '[]'  -- JSON array of alternate names
);

CREATE TABLE IF NOT EXISTS compounds_cache (
    name_query  TEXT PRIMARY KEY,        -- original user input
    smiles      TEXT NOT NULL,           -- canonical SMILES from PubChem
    formula     TEXT,
    iupac_name  TEXT,
    cid         INTEGER,
    cached_at   DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## JSON Rule Format

Rules are stored in `knowledge_base/inorganic.json`, `organic.json`, `physical.json`.

```json
[
  {
    "id": "rxn_org_001",
    "name": "Fischer Esterification",
    "reaction_type": "organic",
    "category": "condensation",
    "reactant_classes": ["carboxylic_acid", "alcohol"],
    "conditions_favored": ["acid_catalyst", "heat", "reflux"],
    "conditions_inhibited": ["basic_pH", "strong_base"],
    "byproducts": [{ "smiles": "O", "name": "Water" }],
    "hazards": [],
    "kb_probability_modifier": 0.88,
    "mechanism_summary": "Protonation of the carbonyl oxygen by H+ increases its electrophilicity. The alcohol oxygen attacks nucleophilically, followed by proton transfers and loss of water to yield the ester.",
    "references": [
      "McMurry Organic Chemistry, 9th Ed., Ch.21",
      "Clayden Organic Chemistry, 2nd Ed., Ch.12"
    ]
  }
]
```

---

## Reaction Classes — Full Taxonomy

### Inorganic Rules (~50)

| Category              | Examples                                           |
| --------------------- | -------------------------------------------------- |
| `neutralization`      | Strong acid + strong base, weak acid + strong base |
| `precipitation`       | AgNO₃ + NaCl → AgCl↓, BaCl₂ + Na₂SO₄ → BaSO₄↓      |
| `redox`               | Zn + H₂SO₄, Fe + CuSO₄, KMnO₄ oxidations           |
| `decomposition`       | H₂O₂ → H₂O + O₂, CaCO₃ → CaO + CO₂                 |
| `combination`         | 2H₂ + O₂ → 2H₂O, N₂ + 3H₂ → 2NH₃                   |
| `single_displacement` | Zn + CuSO₄, Na + H₂O                               |
| `double_displacement` | NaCl + AgNO₃, BaCl₂ + H₂SO₄                        |
| `gas_evolution`       | HCl + Na₂CO₃ → CO₂↑, NH₄Cl + NaOH → NH₃↑           |

### Organic Rules (~80)

| Category                 | Examples                                     |
| ------------------------ | -------------------------------------------- |
| `substitution_sn2`       | Primary alkyl halide + nucleophile           |
| `substitution_sn1`       | Tertiary alkyl halide + weak nucleophile     |
| `elimination_e2`         | Alkyl halide + strong base                   |
| `elimination_e1`         | Tertiary alkyl halide + weak base, heat      |
| `electrophilic_addition` | Alkene + HX, Alkene + X₂, Markovnikov        |
| `radical_halogenation`   | Alkane + X₂, UV light                        |
| `condensation`           | Esterification, Aldol, Amide formation       |
| `hydrolysis`             | Ester + H₂O (acid/base), Amide hydrolysis    |
| `grignard`               | RMgX + carbonyl → alcohol                    |
| `oxidation`              | Primary alcohol → aldehyde → carboxylic acid |
| `reduction`              | Carbonyl → alcohol (NaBH₄, LiAlH₄)           |
| `aromatic_sub`           | EAS: nitration, halogenation, Friedel-Crafts |

### Physical Chemistry Rules (~20)

| Category            | Examples                                                    |
| ------------------- | ----------------------------------------------------------- |
| `equilibrium_shift` | Le Chatelier — temperature, pressure, concentration changes |
| `electrochemistry`  | Galvanic cell reaction, electrolysis                        |
| `thermodynamic`     | ΔG-based spontaneity classification                         |
| `acid_base_equil`   | Buffer reactions, pH-dependent equilibria                   |

---

## Functional Group SMARTS Patterns

```json
[
  {
    "id": "fg_001",
    "name": "carboxylic_acid",
    "smarts": "[CX3](=[OX1])[OX2H1]"
  },
  { "id": "fg_002", "name": "alcohol", "smarts": "[OX2H]" },
  { "id": "fg_003", "name": "amine", "smarts": "[NX3;H2,H1,H0;!$(NC=O)]" },
  { "id": "fg_004", "name": "aldehyde", "smarts": "[CX3H1](=[OX1])" },
  { "id": "fg_005", "name": "ketone", "smarts": "[CX3](=[OX1])([#6])[#6]" },
  { "id": "fg_006", "name": "ester", "smarts": "[CX3](=[OX1])[OX2][#6]" },
  { "id": "fg_007", "name": "alkyl_halide", "smarts": "[CX4][F,Cl,Br,I]" },
  { "id": "fg_008", "name": "alkene", "smarts": "[CX3]=[CX3]" },
  { "id": "fg_009", "name": "alkyne", "smarts": "[CX2]#[CX2]" },
  { "id": "fg_010", "name": "aromatic", "smarts": "c1ccccc1" },
  {
    "id": "fg_011",
    "name": "mineral_acid",
    "smarts": "[$(S(=O)(=O)[OX2H]),$([NH4+]),$([OH][Cl,Br,I,N])]"
  },
  {
    "id": "fg_012",
    "name": "hydroxide",
    "smarts": "[OX2H][$(C=O),$([OH][Na,K,Ca,Ba])]"
  },
  { "id": "fg_013", "name": "amide", "smarts": "[NX3][CX3](=[OX1])" },
  {
    "id": "fg_014",
    "name": "nitro",
    "smarts": "[$([NX3](=O)=O),$([NX3+](=O)[O-])]"
  }
]
```

---

## Seeding Logic (`db/seed.rs`)

```rust
// Pseudocode — actual implementation in src/db/seed.rs
pub async fn load_rules(pool: &SqlitePool, dir: &str) -> anyhow::Result<()> {
    for file in ["inorganic.json", "organic.json", "physical.json"] {
        let path = format!("{}/{}", dir, file);
        let rules: Vec<ReactionRule> = serde_json::from_str(&fs::read_to_string(path)?)?;
        for rule in rules {
            sqlx::query!(
                "INSERT OR IGNORE INTO reaction_rules (id, name, ...) VALUES (?, ?, ...)",
                rule.id, rule.name, /* ... */
            ).execute(pool).await?;
        }
    }
    Ok(())
}
```

On every startup: rules are inserted with `INSERT OR IGNORE`, so re-seeding is idempotent.

---

## Conditions Mapping

User-provided conditions are normalized to these canonical tokens before KB matching:

| User Input             | Canonical Token   |
| ---------------------- | ----------------- |
| "H2SO4", "HCl", "acid" | `acid_catalyst`   |
| "NaOH", "KOH", "base"  | `strong_base`     |
| "reflux", ">100°C"     | `heat`, `reflux`  |
| "UV", "light"          | `uv_light`        |
| "water", "aqueous"     | `aqueous`         |
| "anhydrous", "dry"     | `anhydrous`       |
| "NaBH4", "LiAlH4"      | `reducing_agent`  |
| "KMnO4", "O3"          | `oxidizing_agent` |
