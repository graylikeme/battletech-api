CREATE TABLE unit_mech_data (
    unit_id         INTEGER NOT NULL PRIMARY KEY REFERENCES units (id) ON DELETE CASCADE,
    config          TEXT NOT NULL,  -- "Biped", "Quad", "Tripod", "LAM"
    is_omnimech     BOOLEAN NOT NULL DEFAULT FALSE,
    engine_rating   INTEGER,        -- e.g. 300
    engine_type     TEXT,           -- "Fusion Engine", "XL Engine", etc.
    walk_mp         INTEGER,
    jump_mp         INTEGER,
    heat_sink_count INTEGER,
    heat_sink_type  TEXT,           -- "Single", "Double", "Clan Double"
    structure_type  TEXT,           -- "Standard", "Endo Steel", etc.
    armor_type      TEXT,           -- "Standard Armor", "Ferro-Fibrous", etc.
    gyro_type       TEXT,           -- "Standard Gyro", "XL Gyro", etc.
    cockpit_type    TEXT,           -- "Standard Cockpit", "Small Cockpit", etc.
    myomer_type     TEXT            -- "Standard", "Triple-Strength Myomer", etc.
);

CREATE INDEX idx_unit_mech_data_config   ON unit_mech_data (config);
CREATE INDEX idx_unit_mech_data_omnimech ON unit_mech_data (is_omnimech);
CREATE INDEX idx_unit_mech_data_engine   ON unit_mech_data (engine_type);
CREATE INDEX idx_unit_mech_data_jump     ON unit_mech_data (jump_mp);
