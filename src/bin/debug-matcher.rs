use chem_interactions::predictor::rule_brain::matcher;

fn main() {
    let test_cases = vec![
        ("Nitrobenzene", "c1ccc(cc1)[N+](=O)[O-]"),
        ("Benzene", "c1ccccc1"),
        ("Hydrogen", "[H][H]"),
        ("Acetic Acid", "CC(=O)O"),
        ("Ethanol", "CCO"),
    ];

    for (name, smiles) in test_cases {
        let groups = matcher::detect_functional_groups(smiles);
        println!("SMILES: {} ({}) -> Groups: {:?}", smiles, name, groups);
    }
}
