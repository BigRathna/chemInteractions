# Phase 3 — Explainer & Byproduct Annotation

## Goal

Enrich raw product predictions with natural-language mechanisms, textbook citations, and side-reaction byproducts from the knowledge base.

---

## Byproduct Annotation (`src/predictor/byproducts.rs`)

Reaction models typically predict the _primary_ product (e.g. the ester). The Knowledge Base provides the _byproducts_ (e.g. water, NaCl, CO2) which are critical for yield and purification calculations.

```rust
pub fn annotate_byproducts(prediction: &mut FusionResult) {
    if let Some(rule) = &prediction.matched_rule {
        // Rule found → append byproducts defined in the KB
        // rule.byproducts is a Vec<{name: "Water", smiles: "O"}>
        for bp in &rule.byproducts {
            prediction.add_byproduct(bp.clone());
        }
    } else {
        // No KB match → try to infer byproduct?
        // Simple heuristic: if Condensation but no byproducts, add Water
        if is_condensation(prediction) {
            prediction.add_byproduct(Byproduct { name: "Water".into(), smiles: "O".into() });
        }
    }
}
```

---

## Mechanism Generation (`src/predictor/explainer.rs`)

The explainer takes the matched Knowledge Base rule and formats its `mechanism_summary` with reactant-specific context.

### Template Processing

```rust
pub fn generate_explanation(
    rule_name: &str,
    mechanism_template: &str,
    reactants: &[String],
    references: &[String]
) -> String {
    let mut explanation = format!("### Mechanism: {}\n\n", rule_name);

    // Replace placeholders in template with specific reactant names
    // e.g. "Protonation of [reactant0]..." -> "Protonation of acetic acid..."
    let mut text = mechanism_template.to_string();
    for (i, r) in reactants.iter().enumerate() {
        text = text.replace(&format!("[reactant{}]", i), r);
    }

    explanation.push_str(&text);
    explanation.push_str("\n\n**Sources:**\n");
    for ref_link in references {
        explanation.push_str(&format!("- {}\n", ref_link));
    }

    explanation
}
```

---

## Hazard Awareness

If the KB rule contains hazardous components (e.g. `toxicity`, `exothermic`, `explosive`), these are surfaced as prominent UI alerts.

```rust
pub fn check_hazards(prediction: &mut FusionResult) {
    if let Some(rule) = &prediction.matched_rule {
        for h in &rule.hazards {
            prediction.alerts.push(format!("⚠️ Hazard: {}", h));
        }
    }

    // Hardcoded safety checks
    let smiles = &prediction.raw_prediction.smiles;
    if smiles == "[Cl-]" && is_acidic(prediction) {
        prediction.alerts.push("❗ Risk of toxic Chlorine gas formation if oxidized.".into());
    }
}
```

---

## Result View Object

The final object sent to the API:

```rust
pub struct DetailedResult {
    pub name: String,
    pub probability: f32,
    pub tier: String,             // Corroborated, etc.
    pub products: Vec<Product>,   // Primary + Byproducts
    pub mechanism: String,        // Formatted Markdown
    pub citations: Vec<String>,
    pub alerts: Vec<String>,
}
```

---

## Example Explanation Snippet

> **Mechanism: Fischer Esterification**
>
> Protonation of the carbonyl oxygen in **Acetic Acid** by H+ increases its electrophilicity. **Ethanol** then acts as a nucleophile, attacking the activated carbonyl. This leads to a tetrahedral intermediate, followed by proton transfers and the eventual loss of **Water** to yield **Ethyl Acetate**.
>
> **Sources:**
>
> - McMurry Organic Chemistry, 9th Ed., Ch.21
> - Clayden Organic Chemistry, 2nd Ed., Ch.12

---

## Checklist

- [ ] Byproducts correctly appended from KB rules to predictions.
- [ ] Template replacement correctly inserts reactant names into mechanisms.
- [ ] Hazards from KB are correctly surfaced as alerts.
- [ ] Multiple citations are listed clearly in a markdown list.
- [ ] Inference of common byproducts (like water) works even for ML-only predictions.
