# Phase 3 — Fusion Layer & Validation

## Goal

Implement the logic to merge ML Brain outputs and Rule Brain matches into a single ranked list, assign confidence tiers, and validate physical constraints.

---

## Score Fusion Logic (`src/predictor/fusion.rs`)

The Fusion Layer merges a candidate product SMILES from the ML Brain with a potential rule match from the Rule Brain.

```rust
pub struct FusionResult {
    pub raw_prediction: MlCandidate,      // the product SMILES + ML confidence
    pub matched_rule: Option<KbMatch>,    // if KB found a match for this product
    pub final_probability: f32,
    pub confidence_tier: ConfidenceTier,
}

pub enum ConfidenceTier {
    Corroborated,  // ML + KB agree
    MlPredicted,   // ML predicted it, no KB match (novel/complex)
    RuleBased,     // KB matched it, but ML didn't predict it in top beams
}

pub fn fuse_outputs(
    ml_results: Vec<MlCandidate>,
    kb_results: Vec<KbMatch>,
    alpha: f32, // e.g. 0.65
) -> Vec<FusionResult> {
    let mut results = Vec::new();

    // 1. Process ML results (ML-dominant)
    for ml in ml_results {
        // Try to find a KB rule that would produce this ML product
        // (Matched by looking at the functional groups involved in the rule)
        let matched_rule = kb_results.iter().find(|kb| {
            // Logic: Is this ML product a reasonable output for this KB rule?
            // Simple heuristic: if rule name matches a classification of the ML product
            is_rule_valid_for_product(kb, &ml.smiles)
        });

        let p_kb = matched_rule.as_ref().map(|r| r.kb_score).unwrap_or(0.0);
        let p_ml = ml.confidence;

        let final_p = (alpha * p_ml) + ((1.0 - alpha) * p_kb);

        results.push(FusionResult {
            raw_prediction: ml,
            matched_rule: matched_rule.cloned(),
            final_probability: final_p,
            confidence_tier: if matched_rule.is_some() {
                ConfidenceTier::Corroborated
            } else {
                ConfidenceTier::MlPredicted
            },
        });
    }

    // 2. Add KB results that ML missed entirely
    for kb in kb_results {
        if !results.iter().any(|r| r.matched_rule.as_ref().map(|rule| &rule.rule_id) == Some(&kb.rule_id)) {
            // This is a "rule-only" prediction
            results.push(FusionResult {
                raw_prediction: MlCandidate {
                    smiles: "".to_string(), // we might not know the exact SMILES without ML
                    confidence: 0.0,
                    rank: 0
                },
                matched_rule: Some(kb.clone()),
                final_probability: (1.0 - alpha) * kb.kb_score,
                confidence_tier: ConfidenceTier::RuleBased,
            });
        }
    }

    results.sort_by(|a, b| b.final_probability.partial_cmp(&a.final_probability).unwrap());
    results
}
```

---

## Physical Constraint Validation (`src/predictor/validator.rs`)

Before returning results, we filter out physically impossible products.

```rust
pub fn validate_constraints(product_smiles: &str, reactant_smiles: &[String]) -> bool {
    // 1. Atom Conservation
    //    Count atoms in reactants vs product + byproducts.
    //    Allow for small discrepancies if assuming solvent/air interaction (e.g. O2, H2O).
    let reactant_atoms = count_atoms_batch(reactant_smiles);
    let product_atoms = count_atoms(product_smiles);

    if product_atoms.total > reactant_atoms.total + 10 { // sanity check threshold
        return false;
    }

    // 2. Valence Check
    //    Are there 5-valent carbons or other impossible structures?
    if has_impossible_valency(product_smiles) {
        return false;
    }

    // 3. Charge Balance
    //    Total charge of reactants == Total charge of products
    //    (Unless in electrochemical system)
    true
}
```

---

## Tier Logic & Tagging

| Tier             | P Formula           | Visual UI Badge     |
| ---------------- | ------------------- | ------------------- |
| **Corroborated** | `0.65*ML + 0.35*KB` | Green "Verified"    |
| **ML Predicted** | `0.65*ML`           | Blue "Experimental" |
| **Rule-Based**   | `0.35*KB`           | Amber "Theoretical" |

---

## Tuning the Alpha (α)

Alpha represents the system's "Trust in ML" vs "Trust in Rules".

- **α = 1.0**: Pure ReactionT5 mode. High accuracy on benchmarks, but no byproduct/mechanism/safety grounding.
- **α = 0.0**: Pure KB mode. Reliable for textbook reactions, but fails on any compound not in a known class.
- **α = 0.65**: Balanced hybrid. Leverages ML's predictive power while using KB to boost "known good" paths.

---

## Checklist

- [ ] Fusion layer correctly combines scores for corroborated cases.
- [ ] Tier assignment correctly labels ML-only and KB-only results.
- [ ] Physical validator blocks "hallucinated" products (e.g. product has 10 carbons but reactants only had 2).
- [ ] Sorted output always has the most probable/corroborated result first.
- [ ] Alpha value is configurable via environment variables.
