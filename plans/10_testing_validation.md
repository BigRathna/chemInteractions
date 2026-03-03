# Phase 5 — Testing & Validation

## Goal

Ensure the system is accurate, stable, and safe through automated tests and chemical benchmarks.

---

## 1. Automated Unit Tests

### ML Brain Tests (`tests/test_ml_brain.rs`)

- Verify ONNX session loads successfully.
- Test tokenizer encodes/decodes complex SMILES.
- Benchmark inference time (<1s per beam search).
- Assert top-1 accuracy on canonical simple reactions (e.g. esterification).

### Rule Brain Tests (`tests/test_rule_brain.rs`)

- Functional group detector: Test "CC(=O)O" matches "carboxylic_acid".
- KB matching: Assert rule "Fischer Esterification" matches carboxylic_acid + alcohol.
- Conditions scoring: Test that inhibited conditions correctly zero-out the probability.

---

## 2. API Integration Tests (`tests/test_api.rs`)

- Test `POST /api/v1/predict`:
  - Valid input → 200 OK + JSON.
  - Missing reactants → 400 Bad Request.
  - Malformed SMILES → 400 Bad Request.
- Test concurrent requests handling (Rust/Axum is excellent at this).
- Test PubChem resolve endpoint (mock the external network request).

---

## 3. Chemical Benchmarks (The "Truth" Set)

Validate that the system correctly predicts these textbook interactions with reasonable probabilities.

| Reactants             | Conditions    | Expected Reaction                    | Target P |
| --------------------- | ------------- | ------------------------------------ | -------- |
| Acetic Acid + Ethanol | H2SO4, Reflux | Fischer Esterification               | > 0.80   |
| HCl + NaOH            | -             | Acid-Base Neutralization             | > 0.95   |
| NaCl + AgNO3          | Aqueous       | Precipitation (AgCl)                 | > 0.90   |
| Methane + Cl2         | UV Light      | Radical Halogenation                 | > 0.70   |
| Propene + HCl         | -             | Electrophilic Addition (Markovnikov) | > 0.85   |
| C6H6 + HNO3           | H2SO4, Heat   | Nitration (EAS)                      | > 0.75   |

---

## 4. Stability & Performance

- **Memory Leak Test**: Run 1000 sequential predictions and monitor RAM (important for ONNX runtime).
- **Cold Boot Time**: Measure time from `cargo run` to first 200 OK (models can take 2-5s to load).
- **Graceful Failure**: If PubChem is down, ensure the system still works for SMILES/Cached inputs.

---

## 5. Safety Validation

- **Manual Audit**: Ensure that mixing dangerous household chemicals (e.g. Bleach + Ammonia) triggers a prominent hazard warning: "⚠️ Risk of toxic Chloramine gas formation".
- **Physical Plausibility**: Check that mass is conserved (atoms in != atoms out unless specified).

---

## Checklist

- [ ] `cargo test` runs without failures.
- [ ] Inference benchmarks are consistent on target hardware.
- [ ] All 6 Benchmark reactions pass with P > target.
- [ ] Hazard flags appear for known dangerous mixtures.
- [ ] Rust coverage reports (optional) show >70% coverage.
