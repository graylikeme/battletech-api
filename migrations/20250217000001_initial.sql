-- BattleTech Data API — Milestone A initial schema
-- Extensions
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ── Enums ──────────────────────────────────────────────────────────────────
CREATE TYPE tech_base_enum AS ENUM (
    'inner_sphere',
    'clan',
    'mixed',
    'primitive'
);

CREATE TYPE rules_level_enum AS ENUM (
    'introductory',
    'standard',
    'advanced',
    'experimental',
    'unofficial'
);

CREATE TYPE equipment_category_enum AS ENUM (
    'energy_weapon',
    'ballistic_weapon',
    'missile_weapon',
    'physical_weapon',
    'ammunition',
    'equipment',
    'armor',
    'structure',
    'engine',
    'gyro',
    'cockpit',
    'actuator',
    'heat_sink',
    'jump_jet',
    'targeting_computer'
);

CREATE TYPE location_name_enum AS ENUM (
    'head',
    'center_torso',
    'left_torso',
    'right_torso',
    'left_arm',
    'right_arm',
    'left_leg',
    'right_leg',
    'front',
    'rear',
    'left_side',
    'right_side',
    'turret',
    'body'
);

-- ── Dataset Metadata ───────────────────────────────────────────────────────
CREATE TABLE dataset_metadata (
    id              SERIAL PRIMARY KEY,
    version         TEXT NOT NULL,
    schema_version  INTEGER NOT NULL DEFAULT 1,
    description     TEXT,
    release_date    DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Rulesets ───────────────────────────────────────────────────────────────
CREATE TABLE rulesets (
    id          SERIAL PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    level       rules_level_enum NOT NULL,
    description TEXT,
    source_book TEXT
);

-- ── Eras ───────────────────────────────────────────────────────────────────
CREATE TABLE eras (
    id          SERIAL PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    start_year  INTEGER NOT NULL,
    end_year    INTEGER,
    description TEXT
);

CREATE INDEX idx_eras_start_year ON eras (start_year);
CREATE INDEX idx_eras_end_year   ON eras (end_year);

-- ── Factions ───────────────────────────────────────────────────────────────
CREATE TABLE factions (
    id              SERIAL PRIMARY KEY,
    slug            TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    short_name      TEXT,
    faction_type    TEXT NOT NULL DEFAULT 'inner_sphere',
    is_clan         BOOLEAN NOT NULL DEFAULT FALSE,
    founding_year   INTEGER,
    dissolution_year INTEGER,
    description     TEXT
);

CREATE INDEX idx_factions_faction_type ON factions (faction_type);
CREATE INDEX idx_factions_is_clan      ON factions (is_clan);

-- ── Faction Eras ───────────────────────────────────────────────────────────
CREATE TABLE faction_eras (
    id          SERIAL PRIMARY KEY,
    faction_id  INTEGER NOT NULL REFERENCES factions (id) ON DELETE CASCADE,
    era_id      INTEGER NOT NULL REFERENCES eras (id) ON DELETE CASCADE,
    notes       TEXT,
    UNIQUE (faction_id, era_id)
);

CREATE INDEX idx_faction_eras_faction ON faction_eras (faction_id);
CREATE INDEX idx_faction_eras_era     ON faction_eras (era_id);

-- ── Unit Chassis ───────────────────────────────────────────────────────────
CREATE TABLE unit_chassis (
    id              SERIAL PRIMARY KEY,
    slug            TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    unit_type       TEXT NOT NULL,          -- 'mech', 'vehicle', 'aerospace', etc.
    tech_base       tech_base_enum NOT NULL,
    tonnage         NUMERIC(6,1) NOT NULL,
    intro_year      INTEGER,
    description     TEXT
);

CREATE INDEX idx_unit_chassis_unit_type  ON unit_chassis (unit_type);
CREATE INDEX idx_unit_chassis_tech_base  ON unit_chassis (tech_base);
CREATE INDEX idx_unit_chassis_tonnage    ON unit_chassis (tonnage);
CREATE INDEX idx_unit_chassis_name_trgm  ON unit_chassis USING gin (name gin_trgm_ops);

-- ── Units ─────────────────────────────────────────────────────────────────
CREATE TABLE units (
    id              SERIAL PRIMARY KEY,
    slug            TEXT NOT NULL UNIQUE,
    chassis_id      INTEGER NOT NULL REFERENCES unit_chassis (id) ON DELETE CASCADE,
    variant         TEXT NOT NULL,
    full_name       TEXT NOT NULL,
    tech_base       tech_base_enum NOT NULL,
    rules_level     rules_level_enum NOT NULL,
    tonnage         NUMERIC(6,1) NOT NULL,
    bv              INTEGER,
    cost            BIGINT,
    intro_year      INTEGER,
    extinction_year INTEGER,
    reintro_year    INTEGER,
    source_book     TEXT,
    description     TEXT
);

CREATE INDEX idx_units_chassis_id      ON units (chassis_id);
CREATE INDEX idx_units_tech_base       ON units (tech_base);
CREATE INDEX idx_units_rules_level     ON units (rules_level);
CREATE INDEX idx_units_tonnage         ON units (tonnage);
CREATE INDEX idx_units_bv              ON units (bv);
CREATE INDEX idx_units_intro_year      ON units (intro_year);
CREATE INDEX idx_units_full_name_trgm  ON units USING gin (full_name gin_trgm_ops);

-- ── Equipment ─────────────────────────────────────────────────────────────
CREATE TABLE equipment (
    id              SERIAL PRIMARY KEY,
    slug            TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    category        equipment_category_enum NOT NULL,
    tech_base       tech_base_enum NOT NULL,
    rules_level     rules_level_enum NOT NULL,
    tonnage         NUMERIC(6,2),
    crits           INTEGER,
    damage          TEXT,
    heat            INTEGER,
    range_min       INTEGER,
    range_short     INTEGER,
    range_medium    INTEGER,
    range_long      INTEGER,
    bv              INTEGER,
    intro_year      INTEGER,
    source_book     TEXT,
    description     TEXT
);

CREATE INDEX idx_equipment_category      ON equipment (category);
CREATE INDEX idx_equipment_tech_base     ON equipment (tech_base);
CREATE INDEX idx_equipment_rules_level   ON equipment (rules_level);
CREATE INDEX idx_equipment_name_trgm     ON equipment USING gin (name gin_trgm_ops);

-- ── Unit Availability ─────────────────────────────────────────────────────
CREATE TABLE unit_availability (
    id          SERIAL PRIMARY KEY,
    unit_id     INTEGER NOT NULL REFERENCES units (id) ON DELETE CASCADE,
    faction_id  INTEGER NOT NULL REFERENCES factions (id) ON DELETE CASCADE,
    era_id      INTEGER NOT NULL REFERENCES eras (id) ON DELETE CASCADE,
    availability_code TEXT,
    notes       TEXT,
    UNIQUE (unit_id, faction_id, era_id)
);

CREATE INDEX idx_unit_avail_unit    ON unit_availability (unit_id);
CREATE INDEX idx_unit_avail_faction ON unit_availability (faction_id);
CREATE INDEX idx_unit_avail_era     ON unit_availability (era_id);

-- ── Unit Locations ─────────────────────────────────────────────────────────
CREATE TABLE unit_locations (
    id              SERIAL PRIMARY KEY,
    unit_id         INTEGER NOT NULL REFERENCES units (id) ON DELETE CASCADE,
    location        location_name_enum NOT NULL,
    armor_points    INTEGER,
    rear_armor      INTEGER,
    structure_points INTEGER
);

CREATE INDEX idx_unit_locations_unit ON unit_locations (unit_id);

-- ── Unit Loadout ───────────────────────────────────────────────────────────
CREATE TABLE unit_loadout (
    id              SERIAL PRIMARY KEY,
    unit_id         INTEGER NOT NULL REFERENCES units (id) ON DELETE CASCADE,
    equipment_id    INTEGER NOT NULL REFERENCES equipment (id) ON DELETE CASCADE,
    location        location_name_enum,
    quantity        INTEGER NOT NULL DEFAULT 1,
    is_rear_facing  BOOLEAN NOT NULL DEFAULT FALSE,
    notes           TEXT
);

CREATE INDEX idx_unit_loadout_unit      ON unit_loadout (unit_id);
CREATE INDEX idx_unit_loadout_equipment ON unit_loadout (equipment_id);

-- ── Quirks ────────────────────────────────────────────────────────────────
CREATE TABLE quirks (
    id          SERIAL PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    is_positive BOOLEAN NOT NULL DEFAULT TRUE,
    description TEXT
);

-- ── Unit Quirks ───────────────────────────────────────────────────────────
CREATE TABLE unit_quirks (
    id          SERIAL PRIMARY KEY,
    unit_id     INTEGER NOT NULL REFERENCES units (id) ON DELETE CASCADE,
    quirk_id    INTEGER NOT NULL REFERENCES quirks (id) ON DELETE CASCADE,
    notes       TEXT,
    UNIQUE (unit_id, quirk_id)
);

CREATE INDEX idx_unit_quirks_unit  ON unit_quirks (unit_id);
CREATE INDEX idx_unit_quirks_quirk ON unit_quirks (quirk_id);
