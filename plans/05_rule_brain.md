# Phase 2 — Rule Brain: Functional Group Matching

## Goal

Detect functional groups in input SMILES strings and match them against knowledge base rules to produce scored, annotated rule matches.

---

## Overview

The Rule Brain runs in **parallel with the ML Brain**. It does not run inference — it looks up structured rules from the SQLite knowledge base based on what functional groups it detects in the input SMILES.

```
Input SMILES: ["CC(=O)O", "CCO"]
    ↓
Functional Group Detector
    → ["carboxylic_acid"] from CC(=O)O
    → ["alcohol"] from CCO
    ↓
Rule Matcher (SQLite query)
    → Rule: Fischer Esterification (reactant_classes: [carboxylic_acid, alcohol])
    → Score: 0.88 × conditions_score(0.95) = 0.836
    ↓
Return: Vec<KbMatch> with scores, byproducts, mechanism, citations
```

---

## Functional Group Detector (`src/predictor/rule_brain/matcher.rs`)

Since we aren't using RDKit, we match SMARTS patterns as **regex-like string patterns** against SMILES. For simple patterns this is effective; for complex SMARTS we use a curated lookup table.

### Strategy: Pattern → String Heuristic

```rust
pub fn detect_functional_groups(smiles: &str) -> Vec<String> {
    let mut groups = Vec::new();

    // Carboxylic acid: contains C(=O)O followed by H (not ester)
    if smiles.contains("C(=O)O") && !smiles.contains("C(=O)OC") {
        groups.push("carboxylic_acid".to_string());
    }
    // Ester: C(=O)OC pattern
    if smiles.contains("C(=O)OC") {
        groups.push("ester".to_string());
    }
    // Alcohol: isolated OH not attached to carbonyl
    if smiles.contains("CO") || smiles.contains("[OH]") {
        if !smiles.contains("C(=O)O") {
            groups.push("alcohol".to_string());
        }
    }
    // Aldehyde: specific CHO pattern
    if smiles.contains("C=O") && !smiles.contains("CC=O") {
        groups.push("aldehyde".to_string());
    }
    // Alkyl halide
    if smiles.contains("Cl") || smiles.contains("Br") || smiles.contains("[I]") {
        groups.push("alkyl_halide".to_string());
    }
    // Alkene
    if smiles.contains("C=C") { groups.push("alkene".to_string()); }
    // Alkyne
    if smiles.contains("C#C") { groups.push("alkyne".to_string()); }
    // Amine
    if smiles.contains("N") && !smiles.contains("N(=O)") {
        groups.push("amine".to_string());
    }
    // Nitro
    if smiles.contains("N(=O)=O") || smiles.contains("[N+](=O)[O-]") {
        groups.push("nitro".to_string());
    }
    // Aromatic
    if smiles.contains("c1") || smiles.contains("c2") {
        groups.push("aromatic".to_string());
    }
    // Mineral acids (special compounds)
    if smiles == "Cl" || smiles == "Br" || smiles.contains("OS(=O)(=O)O") {
        groups.push("mineral_acid".to_string());
    }
    // Hydroxide / strong base
    if smiles.contains("[OH-]") || smiles.contains("[Na+].[OH-]") {
        groups.push("hydroxide".to_string());
    }

    groups.dedup();
    groups
}
```

> **Note:** For v2 we can integrate the `rdkit-sys` crate or a SMARTS interpreter for more accurate matching.

---

## Rule Matching Query (`src/predictor/rule_brain/db.rs`)

```rust
pub async fn find_matching_rules(
    pool: &SqlitePool,
    detected_groups: &[String],
    conditions: &Conditions,
) -> anyhow::Result<Vec<KbMatch>> {

    // Load all rules (cached in memory after first startup)
    let rules = sqlx::query_as!(
        RuleRow,
        "SELECT * FROM reaction_rules"
    ).fetch_all(pool).await?;

    let mut matches = Vec::new();

    for rule in rules {
        let rule_classes: Vec<String> = serde_json::from_str(&rule.reactant_classes)?;

        // Check if ALL rule reactant classes are present in detected groups
        let all_present = rule_classes.iter()
            .all(|rc| detected_groups.contains(rc));

        if !all_present { continue; }

        // Score conditions
        let conditions_score = score_conditions(&rule, conditions);

        // Skip if conditions are actively inhibited
        if conditions_score < 0.0 { continue; }

        let kb_score = rule.kb_probability_modifier * conditions_score;

        matches.push(KbMatch {
            rule_id: rule.id,
            name: rule.name,
            kb_score,
            conditions_score,
            byproducts: serde_json::from_str(&rule.byproducts)?,
            hazards: serde_json::from_str(&rule.hazards)?,
            mechanism_summary: rule.mechanism_summary,
            references: serde_json::from_str(&rule.references)?,
        });
    }

    // Sort by kb_score descending
    matches.sort_by(|a, b| b.kb_score.partial_cmp(&a.kb_score).unwrap());
    Ok(matches)
}
```

---

## Conditions Scoring

```rust
fn score_conditions(rule: &RuleRow, conditions: &Conditions) -> f32 {
    let favored: Vec<String> = serde_json::from_str(&rule.conditions_favored).unwrap_or_default();
    let inhibited: Vec<String> = serde_json::from_str(&rule.conditions_inhibited).unwrap_or_default();

    let user_conditions = conditions.to_tokens(); // → Vec<String> of canonical tokens

    // If any inhibiting condition is present → block this rule
    for inh in &inhibited {
        if user_conditions.contains(inh) { return -1.0; }
    }

    // Score based on how many favored conditions are present
    if favored.is_empty() { return 1.0; } // no preference → neutral

    let matched = favored.iter().filter(|f| user_conditions.contains(f)).count();
    matched as f32 / favored.len() as f32
}
```

---

## Output Types

```rust
pub struct KbMatch {
    pub rule_id: String,
    pub name: String,
    pub kb_score: f32,          // final score after conditions weighting
    pub conditions_score: f32,  // raw conditions match ratio
    pub byproducts: Vec<Byproduct>,
    pub hazards: Vec<String>,
    pub mechanism_summary: String,
    pub references: Vec<String>,
}

pub struct Byproduct {
    pub smiles: String,
    pub name: String,
}
```

---

## Checklist

- [ ] `detect_functional_groups("CC(=O)O")` returns `["carboxylic_acid"]`
- [ ] `detect_functional_groups("CCO")` returns `["alcohol"]`
- [ ] Rule matching finds Fischer Esterification for [carboxylic_acid, alcohol]
- [ ] Conditions scoring boosts score when acid_catalyst + heat provided
- [ ] Inhibited conditions block rule from returning
- [ ] `HCl + NaOH` input returns neutralization rule with score ≥ 0.90
