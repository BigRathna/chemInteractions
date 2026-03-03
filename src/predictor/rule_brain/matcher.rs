use purr::read::read;
use purr::feature::{AtomKind, Aliphatic, BondKind};
use purr::graph::Builder;

pub fn detect_functional_groups(smiles: &str) -> Vec<String> {
    let mut groups = Vec::new();

    // 1. Parse SMILES into a graph-like structure
    let mut builder = Builder::new();
    if let Err(_) = read(smiles, &mut builder, None) {
        return vec![]; // Invalid SMILES
    }
    
    let atoms = match builder.build() {
        Ok(a) => a,
        Err(_) => return vec![],
    };

    // 2. Identify functional groups by inspecting atoms and their neighbors
    for (i, atom) in atoms.iter().enumerate() {
        match &atom.kind {
            // Check for Carbonyl-based groups (Carboxylic Acid, Ester, Aldehyde, Ketone)
            AtomKind::Aliphatic(Aliphatic::C) => {
                let mut has_double_o = false;
                let mut has_oh = false;
                let mut has_oc = false;

                for bond in &atom.bonds {
                    let other_idx = bond.tid;
                    let other_atom = &atoms[other_idx];
                    let kind = &bond.kind;

                    if matches!(kind, BondKind::Double) {
                        if matches!(other_atom.kind, AtomKind::Aliphatic(Aliphatic::O)) ||
                           matches!(other_atom.kind, AtomKind::Aromatic(purr::feature::Aromatic::O)) {
                            has_double_o = true;
                        }
                    } else if matches!(kind, BondKind::Single | BondKind::Elided) {
                        if matches!(other_atom.kind, AtomKind::Aliphatic(Aliphatic::O)) ||
                           matches!(other_atom.kind, AtomKind::Aromatic(purr::feature::Aromatic::O)) {
                            // Check if this O has an H or a C
                            let mut o_has_c = false;
                            for o_bond in &other_atom.bonds {
                                if o_bond.tid == i { continue; } // Skip back-edge
                                if matches!(atoms[o_bond.tid].kind, AtomKind::Aliphatic(Aliphatic::C)) {
                                    o_has_c = true;
                                }
                            }
                            // In SMILES, if O is not connected to another C, it's likely an OH (implicit)
                            if o_has_c { has_oc = true; } else { has_oh = true; }
                        }
                    }
                }

                if has_double_o {
                    if has_oh { groups.push("carboxylic_acid".to_string()); }
                    else if has_oc { groups.push("ester".to_string()); }
                    else { groups.push("ketone".to_string()); }
                }
            },
            // Check for Alcohols (O-H not on a carbonyl)
            AtomKind::Aliphatic(Aliphatic::O) => {
                let mut is_carbonyl = false;
                for bond in &atom.bonds {
                    if matches!(bond.kind, BondKind::Double) {
                        if matches!(atoms[bond.tid].kind, AtomKind::Aliphatic(Aliphatic::C)) {
                            is_carbonyl = true;
                        }
                    }
                }
                if !is_carbonyl {
                    groups.push("alcohol".to_string());
                }
            },
            // Check for Amines
            AtomKind::Aliphatic(Aliphatic::N) => {
                groups.push("amine".to_string());
            },
            _ => {}
        }
    }

    groups.sort();
    groups.dedup();
    groups
}
