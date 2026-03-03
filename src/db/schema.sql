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
    "references"          TEXT NOT NULL,  -- JSON array of strings
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

CREATE TABLE IF NOT EXISTS compounds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    smiles TEXT NOT NULL,
    formula TEXT
);
