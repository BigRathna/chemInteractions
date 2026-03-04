use purr::read::read;
use purr::feature::{AtomKind, Aliphatic, BondKind, BracketSymbol, Element, Aromatic, BracketAromatic};
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
        // Aromatic ring check: Use the built-in is_aromatic() or manual check
        if atom.kind.is_aromatic() {
            groups.push("aromatic_ring".to_string());
        }

        match &atom.kind {
            // Check for Carbonyl-based groups and others on Carbon (Aliphatic, Aromatic, or Bracketed)
            AtomKind::Aliphatic(Aliphatic::C) | 
            AtomKind::Aromatic(Aromatic::C) | 
            AtomKind::Bracket { symbol: BracketSymbol::Element(Element::C), .. } |
            AtomKind::Bracket { symbol: BracketSymbol::Aromatic(BracketAromatic::C), .. } => {
                let mut has_double_o = false;
                let mut has_oh = false;
                let mut has_oc = false;
                let mut has_n = false;
                let mut carbon_neighbor_count = 0;

                for bond in &atom.bonds {
                    let other_idx = bond.tid;
                    let other_atom = &atoms[other_idx];
                    let kind = &bond.kind;

                    if matches!(kind, BondKind::Double) {
                        if is_element(&other_atom.kind, Element::O) || is_aromatic_element(&other_atom.kind, BracketAromatic::O) {
                            has_double_o = true;
                        } else if is_element(&other_atom.kind, Element::C) || is_aromatic_element(&other_atom.kind, BracketAromatic::C) {
                            groups.push("alkene".to_string());
                        }
                    } else if matches!(kind, BondKind::Triple) {
                        if is_element(&other_atom.kind, Element::N) || is_aromatic_element(&other_atom.kind, BracketAromatic::N) {
                            groups.push("nitrile".to_string());
                        } else if is_element(&other_atom.kind, Element::C) || is_aromatic_element(&other_atom.kind, BracketAromatic::C) {
                            groups.push("alkyne".to_string());
                        }
                    } else if matches!(kind, BondKind::Single | BondKind::Elided) {
                        if is_element(&other_atom.kind, Element::O) || is_aromatic_element(&other_atom.kind, BracketAromatic::O) {
                            let mut o_has_c = false;
                            for o_bond in &other_atom.bonds {
                                if o_bond.tid == i { continue; }
                                let o_neighbor = &atoms[o_bond.tid];
                                if is_element(&o_neighbor.kind, Element::C) || is_aromatic_element(&o_neighbor.kind, BracketAromatic::C) {
                                    o_has_c = true;
                                }
                            }
                            if o_has_c { has_oc = true; } else { has_oh = true; }
                        } else if is_element(&other_atom.kind, Element::N) || is_aromatic_element(&other_atom.kind, BracketAromatic::N) {
                            has_n = true;
                        } else if is_element(&other_atom.kind, Element::C) || is_aromatic_element(&other_atom.kind, BracketAromatic::C) {
                            carbon_neighbor_count += 1;
                        } else if is_halide(&other_atom.kind) {
                            groups.push("halide".to_string());
                        }
                    }
                }

                if has_double_o {
                    if has_oh { groups.push("carboxylic_acid".to_string()); }
                    else if has_oc { groups.push("ester".to_string()); }
                    else if has_n { groups.push("amide".to_string()); }
                    else if carbon_neighbor_count < 2 {
                        groups.push("aldehyde".to_string());
                    }
                    else { groups.push("ketone".to_string()); }
                }
            },
            // Check for Alcohols and Ethers (Oxygens)
            _ if is_element(&atom.kind, Element::O) || is_aromatic_element(&atom.kind, BracketAromatic::O) => {
                let mut is_carbonyl_related = false;
                let mut neighbor_carbons = 0;
                
                for bond in &atom.bonds {
                    if matches!(bond.kind, BondKind::Double) {
                        is_carbonyl_related = true; // It's the O of C=O
                        break;
                    }
                    
                    let other = &atoms[bond.tid];
                    if is_element(&other.kind, Element::C) || is_aromatic_element(&other.kind, BracketAromatic::C) {
                        neighbor_carbons += 1;
                        for c_bond in &other.bonds {
                            if matches!(c_bond.kind, BondKind::Double) {
                                let potential_o = &atoms[c_bond.tid];
                                if is_element(&potential_o.kind, Element::O) || is_aromatic_element(&potential_o.kind, BracketAromatic::O) {
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
            // Check for Amines and Nitro groups (Nitrogens)
            _ if is_element(&atom.kind, Element::N) || is_aromatic_element(&atom.kind, BracketAromatic::N) => {
                let mut o_bond_count = 0;
                for bond in &atom.bonds {
                    let other = &atoms[bond.tid];
                    if is_element(&other.kind, Element::O) || is_aromatic_element(&other.kind, BracketAromatic::O) {
                        o_bond_count += 1;
                    }
                }
                if o_bond_count >= 2 {
                    groups.push("nitro".to_string());
                } else {
                    let mut is_amide = false;
                    for bond in &atom.bonds {
                        let other = &atoms[bond.tid];
                        if is_element(&other.kind, Element::C) || is_aromatic_element(&other.kind, BracketAromatic::C) {
                            for c_bond in &other.bonds {
                                if c_bond.tid == i { continue; }
                                if matches!(c_bond.kind, BondKind::Double) {
                                    let potential_o = &atoms[c_bond.tid];
                                    if is_element(&potential_o.kind, Element::O) || is_aromatic_element(&potential_o.kind, BracketAromatic::O) {
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
            _ => {}
        }
    }

    groups.sort();
    groups.dedup();
    groups
}

fn is_element(kind: &AtomKind, element: Element) -> bool {
    match kind {
        AtomKind::Aliphatic(al) => match (al, &element) {
            (Aliphatic::C, Element::C) => true,
            (Aliphatic::N, Element::N) => true,
            (Aliphatic::O, Element::O) => true,
            (Aliphatic::F, Element::F) => true,
            (Aliphatic::Cl, Element::Cl) => true,
            (Aliphatic::Br, Element::Br) => true,
            (Aliphatic::I, Element::I) => true,
            _ => false,
        },
        AtomKind::Aromatic(ar) => match (ar, &element) {
            (Aromatic::C, Element::C) => true,
            (Aromatic::N, Element::N) => true,
            (Aromatic::O, Element::O) => true,
            _ => false,
        },
        AtomKind::Bracket { symbol: BracketSymbol::Element(el), .. } => el == &element,
        _ => false,
    }
}

fn is_aromatic_element(kind: &AtomKind, element: BracketAromatic) -> bool {
    match kind {
        AtomKind::Aromatic(ar) => match (ar, &element) {
            (Aromatic::C, BracketAromatic::C) => true,
            (Aromatic::N, BracketAromatic::N) => true,
            (Aromatic::O, BracketAromatic::O) => true,
            _ => false,
        },
        AtomKind::Bracket { symbol: BracketSymbol::Aromatic(ar), .. } => ar == &element,
        _ => false,
    }
}

fn is_halide(kind: &AtomKind) -> bool {
    is_element(kind, Element::F) || is_element(kind, Element::Cl) || is_element(kind, Element::Br) || is_element(kind, Element::I)
}
