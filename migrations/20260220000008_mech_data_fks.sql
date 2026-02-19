-- ============================================================================
-- Link unit_mech_data to construction reference tables via FKs.
-- Also creates alias mapping tables for MegaMek text → reference table matching.
-- ============================================================================

-- ── FK columns on unit_mech_data ─────────────────────────────────────────────

ALTER TABLE unit_mech_data ADD COLUMN engine_type_id    INT REFERENCES engine_types(id);
ALTER TABLE unit_mech_data ADD COLUMN armor_type_id     INT REFERENCES armor_types(id);
ALTER TABLE unit_mech_data ADD COLUMN structure_type_id  INT REFERENCES structure_types(id);
ALTER TABLE unit_mech_data ADD COLUMN heatsink_type_id   INT REFERENCES heatsink_types(id);
ALTER TABLE unit_mech_data ADD COLUMN gyro_type_id       INT REFERENCES gyro_types(id);
ALTER TABLE unit_mech_data ADD COLUMN cockpit_type_id    INT REFERENCES cockpit_types(id);
ALTER TABLE unit_mech_data ADD COLUMN myomer_type_id     INT REFERENCES myomer_types(id);

-- ── Alias Mapping Tables ─────────────────────────────────────────────────────
-- Map MegaMek text values to reference table IDs.

CREATE TABLE engine_type_aliases (
  alias           TEXT PRIMARY KEY,
  engine_type_id  INT NOT NULL REFERENCES engine_types(id)
);

INSERT INTO engine_type_aliases (alias, engine_type_id) VALUES
  ('Fusion Engine',           (SELECT id FROM engine_types WHERE slug = 'standard-fusion')),
  ('Fusion',                  (SELECT id FROM engine_types WHERE slug = 'standard-fusion')),
  ('XL Engine',               (SELECT id FROM engine_types WHERE slug = 'xl-is')),
  ('XL Fusion Engine',        (SELECT id FROM engine_types WHERE slug = 'xl-is')),
  ('XL (Clan) Engine',        (SELECT id FROM engine_types WHERE slug = 'xl-clan')),
  ('Clan XL Engine',          (SELECT id FROM engine_types WHERE slug = 'xl-clan')),
  ('Light Engine',            (SELECT id FROM engine_types WHERE slug = 'light')),
  ('Light Fusion Engine',     (SELECT id FROM engine_types WHERE slug = 'light')),
  ('Compact Engine',          (SELECT id FROM engine_types WHERE slug = 'compact')),
  ('Compact Fusion Engine',   (SELECT id FROM engine_types WHERE slug = 'compact')),
  ('XXL Engine',              (SELECT id FROM engine_types WHERE slug = 'xxl-is')),
  ('XXL Fusion Engine',       (SELECT id FROM engine_types WHERE slug = 'xxl-is')),
  ('ICE',                     (SELECT id FROM engine_types WHERE slug = 'ice')),
  ('I.C.E.',                  (SELECT id FROM engine_types WHERE slug = 'ice')),
  ('ICE Engine',              (SELECT id FROM engine_types WHERE slug = 'ice')),
  ('Fuel Cell',               (SELECT id FROM engine_types WHERE slug = 'fuel-cell')),
  ('Fuel Cell Engine',        (SELECT id FROM engine_types WHERE slug = 'fuel-cell')),
  ('Fuel-Cell Engine',        (SELECT id FROM engine_types WHERE slug = 'fuel-cell')),
  ('Primitive Fusion Engine', (SELECT id FROM engine_types WHERE slug = 'primitive-fusion')),
  ('Primitive Engine',        (SELECT id FROM engine_types WHERE slug = 'primitive-fusion'))
ON CONFLICT (alias) DO UPDATE SET engine_type_id = EXCLUDED.engine_type_id;

CREATE TABLE armor_type_aliases (
  alias          TEXT PRIMARY KEY,
  armor_type_id  INT NOT NULL REFERENCES armor_types(id)
);

INSERT INTO armor_type_aliases (alias, armor_type_id) VALUES
  ('Standard',              (SELECT id FROM armor_types WHERE slug = 'standard')),
  ('Standard Armor',        (SELECT id FROM armor_types WHERE slug = 'standard')),
  ('Ferro-Fibrous',         (SELECT id FROM armor_types WHERE slug = 'ferro-fibrous-is')),
  ('Ferro-Fibrous (Inner Sphere)', (SELECT id FROM armor_types WHERE slug = 'ferro-fibrous-is')),
  ('Ferro-Fibrous (Clan)',  (SELECT id FROM armor_types WHERE slug = 'ferro-fibrous-clan')),
  ('Clan Ferro-Fibrous',    (SELECT id FROM armor_types WHERE slug = 'ferro-fibrous-clan')),
  ('Light Ferro-Fibrous',   (SELECT id FROM armor_types WHERE slug = 'light-ferro')),
  ('Heavy Ferro-Fibrous',   (SELECT id FROM armor_types WHERE slug = 'heavy-ferro')),
  ('Stealth',               (SELECT id FROM armor_types WHERE slug = 'stealth')),
  ('Stealth Armor',         (SELECT id FROM armor_types WHERE slug = 'stealth')),
  ('Reactive',              (SELECT id FROM armor_types WHERE slug = 'reactive')),
  ('Reactive Armor',        (SELECT id FROM armor_types WHERE slug = 'reactive')),
  ('Hardened',              (SELECT id FROM armor_types WHERE slug = 'hardened')),
  ('Hardened Armor',        (SELECT id FROM armor_types WHERE slug = 'hardened')),
  ('Primitive',             (SELECT id FROM armor_types WHERE slug = 'primitive')),
  ('Primitive Armor',       (SELECT id FROM armor_types WHERE slug = 'primitive'))
ON CONFLICT (alias) DO UPDATE SET armor_type_id = EXCLUDED.armor_type_id;

CREATE TABLE structure_type_aliases (
  alias              TEXT PRIMARY KEY,
  structure_type_id  INT NOT NULL REFERENCES structure_types(id)
);

INSERT INTO structure_type_aliases (alias, structure_type_id) VALUES
  ('Standard',                  (SELECT id FROM structure_types WHERE slug = 'standard')),
  ('Standard Structure',        (SELECT id FROM structure_types WHERE slug = 'standard')),
  ('Endo Steel',                (SELECT id FROM structure_types WHERE slug = 'endo-steel-is')),
  ('Endo Steel (Inner Sphere)', (SELECT id FROM structure_types WHERE slug = 'endo-steel-is')),
  ('IS Endo Steel',             (SELECT id FROM structure_types WHERE slug = 'endo-steel-is')),
  ('Endo Steel (Clan)',         (SELECT id FROM structure_types WHERE slug = 'endo-steel-clan')),
  ('Clan Endo Steel',           (SELECT id FROM structure_types WHERE slug = 'endo-steel-clan')),
  ('Composite',                 (SELECT id FROM structure_types WHERE slug = 'composite')),
  ('Composite Structure',       (SELECT id FROM structure_types WHERE slug = 'composite')),
  ('Reinforced',                (SELECT id FROM structure_types WHERE slug = 'reinforced')),
  ('Reinforced Structure',      (SELECT id FROM structure_types WHERE slug = 'reinforced')),
  ('Endo-Composite',            (SELECT id FROM structure_types WHERE slug = 'endo-composite-is')),
  ('IS Endo-Composite',         (SELECT id FROM structure_types WHERE slug = 'endo-composite-is'))
ON CONFLICT (alias) DO UPDATE SET structure_type_id = EXCLUDED.structure_type_id;

CREATE TABLE heatsink_type_aliases (
  alias              TEXT PRIMARY KEY,
  heatsink_type_id   INT NOT NULL REFERENCES heatsink_types(id)
);

INSERT INTO heatsink_type_aliases (alias, heatsink_type_id) VALUES
  ('Single',                     (SELECT id FROM heatsink_types WHERE slug = 'single')),
  ('Single Heat Sink',           (SELECT id FROM heatsink_types WHERE slug = 'single')),
  ('Double',                     (SELECT id FROM heatsink_types WHERE slug = 'double-is')),
  ('Double Heat Sink',           (SELECT id FROM heatsink_types WHERE slug = 'double-is')),
  ('IS Double Heat Sink',        (SELECT id FROM heatsink_types WHERE slug = 'double-is')),
  ('Double (Inner Sphere)',      (SELECT id FROM heatsink_types WHERE slug = 'double-is')),
  ('Clan Double Heat Sink',      (SELECT id FROM heatsink_types WHERE slug = 'double-clan')),
  ('Double (Clan)',              (SELECT id FROM heatsink_types WHERE slug = 'double-clan')),
  ('Compact',                    (SELECT id FROM heatsink_types WHERE slug = 'compact')),
  ('Compact Heat Sink',          (SELECT id FROM heatsink_types WHERE slug = 'compact'))
ON CONFLICT (alias) DO UPDATE SET heatsink_type_id = EXCLUDED.heatsink_type_id;

CREATE TABLE gyro_type_aliases (
  alias         TEXT PRIMARY KEY,
  gyro_type_id  INT NOT NULL REFERENCES gyro_types(id)
);

INSERT INTO gyro_type_aliases (alias, gyro_type_id) VALUES
  ('Standard Gyro',    (SELECT id FROM gyro_types WHERE slug = 'standard')),
  ('Standard',         (SELECT id FROM gyro_types WHERE slug = 'standard')),
  ('XL Gyro',          (SELECT id FROM gyro_types WHERE slug = 'xl')),
  ('Compact Gyro',     (SELECT id FROM gyro_types WHERE slug = 'compact')),
  ('Heavy Duty Gyro',  (SELECT id FROM gyro_types WHERE slug = 'heavy-duty')),
  ('Heavy-Duty Gyro',  (SELECT id FROM gyro_types WHERE slug = 'heavy-duty')),
  ('Superheavy Gyro',  (SELECT id FROM gyro_types WHERE slug = 'superheavy'))
ON CONFLICT (alias) DO UPDATE SET gyro_type_id = EXCLUDED.gyro_type_id;

CREATE TABLE cockpit_type_aliases (
  alias            TEXT PRIMARY KEY,
  cockpit_type_id  INT NOT NULL REFERENCES cockpit_types(id)
);

INSERT INTO cockpit_type_aliases (alias, cockpit_type_id) VALUES
  ('Standard Cockpit',       (SELECT id FROM cockpit_types WHERE slug = 'standard')),
  ('Standard',               (SELECT id FROM cockpit_types WHERE slug = 'standard')),
  ('Small Cockpit',          (SELECT id FROM cockpit_types WHERE slug = 'small')),
  ('Small',                  (SELECT id FROM cockpit_types WHERE slug = 'small')),
  ('Command Console',        (SELECT id FROM cockpit_types WHERE slug = 'command-console')),
  ('Torso Cockpit',          (SELECT id FROM cockpit_types WHERE slug = 'torso-mounted')),
  ('Torso-Mounted Cockpit',  (SELECT id FROM cockpit_types WHERE slug = 'torso-mounted')),
  ('Industrial Cockpit',     (SELECT id FROM cockpit_types WHERE slug = 'industrial')),
  ('Industrial',             (SELECT id FROM cockpit_types WHERE slug = 'industrial')),
  ('Primitive Cockpit',      (SELECT id FROM cockpit_types WHERE slug = 'primitive')),
  ('Primitive',              (SELECT id FROM cockpit_types WHERE slug = 'primitive'))
ON CONFLICT (alias) DO UPDATE SET cockpit_type_id = EXCLUDED.cockpit_type_id;

CREATE TABLE myomer_type_aliases (
  alias           TEXT PRIMARY KEY,
  myomer_type_id  INT NOT NULL REFERENCES myomer_types(id)
);

INSERT INTO myomer_type_aliases (alias, myomer_type_id) VALUES
  ('Standard',                  (SELECT id FROM myomer_types WHERE slug = 'standard')),
  ('Standard Myomer',           (SELECT id FROM myomer_types WHERE slug = 'standard')),
  ('MASC',                      (SELECT id FROM myomer_types WHERE slug = 'masc')),
  ('Triple Strength Myomer',    (SELECT id FROM myomer_types WHERE slug = 'tsm')),
  ('Triple-Strength Myomer',    (SELECT id FROM myomer_types WHERE slug = 'tsm')),
  ('TSM',                       (SELECT id FROM myomer_types WHERE slug = 'tsm')),
  ('Industrial',                (SELECT id FROM myomer_types WHERE slug = 'industrial')),
  ('Industrial Myomer',         (SELECT id FROM myomer_types WHERE slug = 'industrial'))
ON CONFLICT (alias) DO UPDATE SET myomer_type_id = EXCLUDED.myomer_type_id;

-- ── Populate FK columns from alias tables ────────────────────────────────────

UPDATE unit_mech_data md SET engine_type_id = ea.engine_type_id
FROM engine_type_aliases ea
WHERE lower(trim(md.engine_type)) = lower(ea.alias)
  AND md.engine_type_id IS NULL;

UPDATE unit_mech_data md SET armor_type_id = aa.armor_type_id
FROM armor_type_aliases aa
WHERE lower(trim(md.armor_type)) = lower(aa.alias)
  AND md.armor_type_id IS NULL;

UPDATE unit_mech_data md SET structure_type_id = sa.structure_type_id
FROM structure_type_aliases sa
WHERE lower(trim(md.structure_type)) = lower(sa.alias)
  AND md.structure_type_id IS NULL;

UPDATE unit_mech_data md SET heatsink_type_id = ha.heatsink_type_id
FROM heatsink_type_aliases ha
WHERE lower(trim(md.heat_sink_type)) = lower(ha.alias)
  AND md.heatsink_type_id IS NULL;

UPDATE unit_mech_data md SET gyro_type_id = ga.gyro_type_id
FROM gyro_type_aliases ga
WHERE lower(trim(md.gyro_type)) = lower(ga.alias)
  AND md.gyro_type_id IS NULL;

UPDATE unit_mech_data md SET cockpit_type_id = ca.cockpit_type_id
FROM cockpit_type_aliases ca
WHERE lower(trim(md.cockpit_type)) = lower(ca.alias)
  AND md.cockpit_type_id IS NULL;

UPDATE unit_mech_data md SET myomer_type_id = ma.myomer_type_id
FROM myomer_type_aliases ma
WHERE lower(trim(md.myomer_type)) = lower(ma.alias)
  AND md.myomer_type_id IS NULL;
