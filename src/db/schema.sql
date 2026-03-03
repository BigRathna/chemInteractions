CREATE TABLE IF NOT EXISTS reaction_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    reactant_classes TEXT NOT NULL, -- JSON array
    products TEXT NOT NULL,         -- JSON array
    byproducts TEXT NOT NULL,       -- JSON array
    citation TEXT
);

CREATE TABLE IF NOT EXISTS compounds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    smiles TEXT NOT NULL,
    formula TEXT
);
