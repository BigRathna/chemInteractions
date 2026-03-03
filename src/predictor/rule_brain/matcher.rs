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
            // Check for Carbonyl-based groups and others on Carbon
            AtomKind::Aliphatic(Aliphatic::C) | AtomKind::Aromatic(purr::feature::Aromatic::C) => {
                let mut has_double_o = false;
                let mut has_oh = false;
                let mut has_oc = false;
                let mut has_n = false;
                let mut carbon_neighbor_count = 0;
                // SMILES might not have explicit H

                for bond in &atom.bonds {
                    let other_idx = bond.tid;
                    let other_atom = &atoms[other_idx];
                    let kind = &bond.kind;

                    if matches!(kind, BondKind::Double) {
                        if matches!(other_atom.kind, AtomKind::Aliphatic(Aliphatic::O) | AtomKind::Aromatic(_)) {
                            has_double_o = true;
                        } else if matches!(other_atom.kind, AtomKind::Aliphatic(Aliphatic::C) | AtomKind::Aromatic(purr::feature::Aromatic::C)) {
                            groups.push("alkene".to_string());
                        }
                    } else if matches!(kind, BondKind::Triple) {
                        if matches!(other_atom.kind, AtomKind::Aliphatic(Aliphatic::N)) {
                            groups.push("nitrile".to_string());
                        } else if matches!(other_atom.kind, AtomKind::Aliphatic(Aliphatic::C) | AtomKind::Aromatic(purr::feature::Aromatic::C)) {
                            groups.push("alkyne".to_string());
                        }
                    } else if matches!(kind, BondKind::Single | BondKind::Elided) {
                        match &other_atom.kind {
                            AtomKind::Aliphatic(Aliphatic::O) | AtomKind::Aromatic(purr::feature::Aromatic::O) => {
                                let mut o_has_c = false;
                                for o_bond in &other_atom.bonds {
                                    if o_bond.tid == i { continue; }
                                    if matches!(atoms[o_bond.tid].kind, AtomKind::Aliphatic(Aliphatic::C) | AtomKind::Aromatic(purr::feature::Aromatic::C)) {
                                        o_has_c = true;
                                    }
                                }
                                if o_has_c { has_oc = true; } else { has_oh = true; }
                            },
                            AtomKind::Aliphatic(Aliphatic::N) | AtomKind::Aromatic(purr::feature::Aromatic::N) => {
                                has_n = true;
                            },
                            AtomKind::Aliphatic(Aliphatic::C) | AtomKind::Aromatic(purr::feature::Aromatic::C) => {
                                carbon_neighbor_count += 1;
                            },
                            AtomKind::Aliphatic(Aliphatic::F) | AtomKind::Aliphatic(Aliphatic::Cl) |
                            AtomKind::Aliphatic(Aliphatic::Br) | AtomKind::Aliphatic(Aliphatic::I) => {
                                groups.push("halide".to_string());
                            },
                            _ => {}
                        }
                    }
                }

                if has_double_o {
                    if has_oh { groups.push("carboxylic_acid".to_string()); }
                    else if has_oc { groups.push("ester".to_string()); }
                    else if has_n { groups.push("amide".to_string()); }
                    else if carbon_neighbor_count < 2 {
                        // If it has < 2 carbon neighbors, it's likely an aldehyde (one H) or formaldehyde (two H)
                        groups.push("aldehyde".to_string());
                    }
                    else { groups.push("ketone".to_string()); }
                }
            },
            // Check for Alcohols and Ethers
            AtomKind::Aliphatic(Aliphatic::O) | AtomKind::Aromatic(purr::feature::Aromatic::O) => {
                let mut is_carbonyl_related = false;
                let mut neighbor_carbons = 0;
                
                for bond in &atom.bonds {
                    if matches!(bond.kind, BondKind::Double) {
                        is_carbonyl_related = true; // It's the O of C=O
                        break;
                    }
                    
                    let other = &atoms[bond.tid];
                    if matches!(other.kind, AtomKind::Aliphatic(Aliphatic::C) | AtomKind::Aromatic(purr::feature::Aromatic::C)) {
                        neighbor_carbons += 1;
                        // Check if the neighbor carbon is part of a carbonyl
                        for c_bond in &other.bonds {
                            if matches!(c_bond.kind, BondKind::Double) {
                                let potential_o = &atoms[c_bond.tid];
                                if matches!(potential_o.kind, AtomKind::Aliphatic(Aliphatic::O) | AtomKind::Aromatic(purr::feature::Aromatic::O)) {
                                    is_carbonyl_related = true;
                                }
                            }
                        }
                    }
                }
                
                if !is_carbonyl_related {
                    if neighbor_carbons >= 2 {
                        groups.push("ether".to_string());
                    } else if neighbor_carbons == 1 {
                        groups.push("alcohol".to_string());
                    }
                }
            },
            // Check for Amines and Nitro groups
            AtomKind::Aliphatic(Aliphatic::N) => {
                let mut double_o_count = 0;
                for bond in &atom.bonds {
                    if matches!(bond.kind, BondKind::Double) {
                        let other = &atoms[bond.tid];
                        if matches!(other.kind, AtomKind::Aliphatic(Aliphatic::O)) {
                            double_o_count += 1;
                        }
                    }
                }
                if double_o_count >= 2 {
                    groups.push("nitro".to_string());
                } else {
                    // Check if part of amide (already handled in Carbon search, but let's be safe)
                    let mut is_amide = false;
                    for bond in &atom.bonds {
                        let other = &atoms[bond.tid];
                        if matches!(other.kind, AtomKind::Aliphatic(Aliphatic::C)) {
                            for c_bond in &other.bonds {
                                if c_bond.tid == i { continue; }
                                if matches!(c_bond.kind, BondKind::Double) {
                                    if matches!(atoms[c_bond.tid].kind, AtomKind::Aliphatic(Aliphatic::O)) {
                                        is_amide = true;
                                    }
                                }
                            }
                        }
                    }
                    if !is_amide {
                        groups.push("amine".to_string());
                    }
                }
            },
            },
            AtomKind::Aromatic(_) => {
                groups.push("aromatic_ring".to_string());
            }
            _ => {}
        }
    }

    groups.sort();
    groups.dedup();
    groups
}
