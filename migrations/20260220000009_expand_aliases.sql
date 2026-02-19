-- Expand component type reference data and alias tables to cover MegaMek naming patterns.
--
-- MegaMek uses patterns like:
--   "XL Engine(IS)", "Fusion (Clan) Engine(IS)", "Standard(Inner Sphere)",
--   "IS Double", "Clan Double", "IS Standard", "Triple-Strength", "ISMASC"
-- that don't match the initial alias set. This migration adds:
--   1. Missing component types (XXL Clan, Fission, Industrial armor/structure, etc.)
--   2. Comprehensive aliases covering observed MegaMek naming conventions.

-- ═════════════════════════════════════════════════════════════════════════════
-- 1. New component types
-- ═════════════════════════════════════════════════════════════════════════════

-- XXL Clan engine (different st_crits from XXL IS)
INSERT INTO engine_types (slug, name, tech_base, rules_level, weight_multiplier, ct_crits, st_crits, intro_year)
VALUES ('xxl-clan', 'XXL Engine (Clan)', 'clan', 'experimental', 0.333, 6, 4, 3055)
ON CONFLICT (slug) DO NOTHING;

-- Fission engine (rare, experimental)
INSERT INTO engine_types (slug, name, tech_base, rules_level, weight_multiplier, ct_crits, st_crits, intro_year)
VALUES ('fission', 'Fission Engine', 'inner_sphere', 'experimental', 1.750, 6, 0, NULL)
ON CONFLICT (slug) DO NOTHING;

-- Industrial armor (same protection as standard, different fluff)
INSERT INTO armor_types (slug, name, tech_base, rules_level, points_per_ton, crits, intro_year)
VALUES ('industrial', 'Industrial Armor', 'inner_sphere', 'introductory', 16.00, 0, 2439)
ON CONFLICT (slug) DO NOTHING;

-- Heavy Industrial armor (half protection)
INSERT INTO armor_types (slug, name, tech_base, rules_level, points_per_ton, crits, intro_year)
VALUES ('heavy-industrial', 'Heavy Industrial Armor', 'inner_sphere', 'standard', 8.00, 0, 2460)
ON CONFLICT (slug) DO NOTHING;

-- Commercial armor (light civilian protection)
INSERT INTO armor_types (slug, name, tech_base, rules_level, points_per_ton, crits, intro_year)
VALUES ('commercial', 'Commercial Armor', 'inner_sphere', 'introductory', 8.00, 0, 2400)
ON CONFLICT (slug) DO NOTHING;

-- Reflective / Laser-Reflective armor (IS)
INSERT INTO armor_types (slug, name, tech_base, rules_level, points_per_ton, crits, intro_year)
VALUES ('reflective-is', 'Reflective Armor (IS)', 'inner_sphere', 'experimental', 16.00, 10, 3058)
ON CONFLICT (slug) DO NOTHING;

-- Reflective / Laser-Reflective armor (Clan)
INSERT INTO armor_types (slug, name, tech_base, rules_level, points_per_ton, crits, intro_year)
VALUES ('reflective-clan', 'Reflective Armor (Clan)', 'clan', 'experimental', 16.00, 5, 3061)
ON CONFLICT (slug) DO NOTHING;

-- Ferro-Lamellor armor (Clan experimental)
INSERT INTO armor_types (slug, name, tech_base, rules_level, points_per_ton, crits, intro_year)
VALUES ('ferro-lamellor', 'Ferro-Lamellor Armor', 'clan', 'experimental', 17.92, 12, 3070)
ON CONFLICT (slug) DO NOTHING;

-- Industrial structure
INSERT INTO structure_types (slug, name, tech_base, rules_level, weight_fraction, crits, intro_year)
VALUES ('industrial', 'Industrial Structure', 'inner_sphere', 'introductory', 0.10, 0, 2350)
ON CONFLICT (slug) DO NOTHING;

-- Clan Endo-Composite
INSERT INTO structure_types (slug, name, tech_base, rules_level, weight_fraction, crits, intro_year)
VALUES ('endo-composite-clan', 'Endo-Composite (Clan)', 'clan', 'advanced', 0.075, 4, 3073)
ON CONFLICT (slug) DO NOTHING;

-- Clan Reinforced
INSERT INTO structure_types (slug, name, tech_base, rules_level, weight_fraction, crits, intro_year)
VALUES ('reinforced-clan', 'Reinforced Structure (Clan)', 'clan', 'advanced', 0.20, 0, 3065)
ON CONFLICT (slug) DO NOTHING;

-- Laser Heat Sink (advanced, Dark Age era)
INSERT INTO heatsink_types (slug, name, tech_base, rules_level, dissipation, crits, weight, intro_year)
VALUES ('laser', 'Laser Heat Sink', 'clan', 'experimental', 2, 2, 1.00, 3075)
ON CONFLICT (slug) DO NOTHING;


-- ═════════════════════════════════════════════════════════════════════════════
-- 2. Engine type aliases
-- ═════════════════════════════════════════════════════════════════════════════

-- MegaMek pattern: "(IS)" suffix means Inner Sphere chassis, not engine tech.
-- "Fusion (Clan) Engine" = Standard Fusion produced by Clan.

INSERT INTO engine_type_aliases (alias, engine_type_id) VALUES
  -- Standard Fusion variants
  ('Fusion Engine(IS)',            (SELECT id FROM engine_types WHERE slug = 'standard-fusion')),
  ('Fusion (Clan) Engine',        (SELECT id FROM engine_types WHERE slug = 'standard-fusion')),
  ('Fusion (Clan) Engine(IS)',    (SELECT id FROM engine_types WHERE slug = 'standard-fusion')),
  ('Fusion Engine (Clan)',        (SELECT id FROM engine_types WHERE slug = 'standard-fusion')),
  -- XL IS variants
  ('XL Engine(IS)',               (SELECT id FROM engine_types WHERE slug = 'xl-is')),
  -- XL Clan variants
  ('XL (Clan) Engine(IS)',        (SELECT id FROM engine_types WHERE slug = 'xl-clan')),
  ('Clan XL Engine(IS)',          (SELECT id FROM engine_types WHERE slug = 'xl-clan')),
  -- Light variants
  ('Light Engine(IS)',            (SELECT id FROM engine_types WHERE slug = 'light')),
  ('Light Fusion Engine(IS)',     (SELECT id FROM engine_types WHERE slug = 'light')),
  -- Compact variants
  ('Compact Engine(IS)',          (SELECT id FROM engine_types WHERE slug = 'compact')),
  ('Compact Fusion Engine(IS)',   (SELECT id FROM engine_types WHERE slug = 'compact')),
  -- XXL IS variants
  ('XXL Engine(IS)',              (SELECT id FROM engine_types WHERE slug = 'xxl-is')),
  ('XXL Fusion Engine(IS)',       (SELECT id FROM engine_types WHERE slug = 'xxl-is')),
  -- XXL Clan variants
  ('XXL (Clan) Engine',          (SELECT id FROM engine_types WHERE slug = 'xxl-clan')),
  ('XXL (Clan) Engine(IS)',      (SELECT id FROM engine_types WHERE slug = 'xxl-clan')),
  ('Clan XXL Engine',            (SELECT id FROM engine_types WHERE slug = 'xxl-clan')),
  ('Clan XXL Engine(IS)',        (SELECT id FROM engine_types WHERE slug = 'xxl-clan')),
  -- ICE variants
  ('ICE Engine(IS)',             (SELECT id FROM engine_types WHERE slug = 'ice')),
  ('I.C.E. Engine',             (SELECT id FROM engine_types WHERE slug = 'ice')),
  ('I.C.E. Engine(IS)',         (SELECT id FROM engine_types WHERE slug = 'ice')),
  -- Fuel Cell variants
  ('Fuel Cell Engine(IS)',       (SELECT id FROM engine_types WHERE slug = 'fuel-cell')),
  ('Fuel-Cell Engine(IS)',       (SELECT id FROM engine_types WHERE slug = 'fuel-cell')),
  -- Primitive variants
  ('Primitive Fusion Engine(IS)',(SELECT id FROM engine_types WHERE slug = 'primitive-fusion')),
  ('Primitive Engine(IS)',       (SELECT id FROM engine_types WHERE slug = 'primitive-fusion')),
  -- Fission
  ('Fission Engine',             (SELECT id FROM engine_types WHERE slug = 'fission')),
  ('Fission Engine(IS)',         (SELECT id FROM engine_types WHERE slug = 'fission'))
ON CONFLICT (alias) DO NOTHING;


-- ═════════════════════════════════════════════════════════════════════════════
-- 3. Armor type aliases
-- ═════════════════════════════════════════════════════════════════════════════

-- MegaMek pattern: "Type(Inner Sphere)", "Type(Clan)", "Type(IS/Clan)"

INSERT INTO armor_type_aliases (alias, armor_type_id) VALUES
  -- Standard
  ('Standard(Inner Sphere)',            (SELECT id FROM armor_types WHERE slug = 'standard')),
  ('Standard(Clan)',                    (SELECT id FROM armor_types WHERE slug = 'standard')),
  ('Standard(IS/Clan)',                 (SELECT id FROM armor_types WHERE slug = 'standard')),
  ('Standard Armor(Inner Sphere)',      (SELECT id FROM armor_types WHERE slug = 'standard')),
  -- Ferro-Fibrous IS
  ('Ferro-Fibrous(Inner Sphere)',       (SELECT id FROM armor_types WHERE slug = 'ferro-fibrous-is')),
  -- Ferro-Fibrous Clan
  ('Ferro-Fibrous(Clan)',              (SELECT id FROM armor_types WHERE slug = 'ferro-fibrous-clan')),
  -- Light Ferro-Fibrous
  ('Light Ferro-Fibrous(Inner Sphere)',(SELECT id FROM armor_types WHERE slug = 'light-ferro')),
  ('Light Ferro-Fibrous(Clan)',        (SELECT id FROM armor_types WHERE slug = 'light-ferro')),
  -- Heavy Ferro-Fibrous
  ('Heavy Ferro-Fibrous(Inner Sphere)',(SELECT id FROM armor_types WHERE slug = 'heavy-ferro')),
  -- Stealth
  ('Stealth(Inner Sphere)',            (SELECT id FROM armor_types WHERE slug = 'stealth')),
  ('Stealth Armor(Inner Sphere)',      (SELECT id FROM armor_types WHERE slug = 'stealth')),
  -- Reactive
  ('Reactive(Inner Sphere)',           (SELECT id FROM armor_types WHERE slug = 'reactive')),
  ('Reactive(Clan)',                   (SELECT id FROM armor_types WHERE slug = 'reactive')),
  ('Reactive Armor(Inner Sphere)',     (SELECT id FROM armor_types WHERE slug = 'reactive')),
  -- Hardened
  ('Hardened(Inner Sphere)',           (SELECT id FROM armor_types WHERE slug = 'hardened')),
  ('Hardened(Clan)',                   (SELECT id FROM armor_types WHERE slug = 'hardened')),
  ('Hardened Armor(Inner Sphere)',     (SELECT id FROM armor_types WHERE slug = 'hardened')),
  -- Primitive
  ('Primitive(Inner Sphere)',          (SELECT id FROM armor_types WHERE slug = 'primitive')),
  ('Primitive Armor(Inner Sphere)',    (SELECT id FROM armor_types WHERE slug = 'primitive')),
  -- Industrial
  ('Industrial(Inner Sphere)',         (SELECT id FROM armor_types WHERE slug = 'industrial')),
  ('Industrial (Inner Sphere)',        (SELECT id FROM armor_types WHERE slug = 'industrial')),
  ('Industrial Armor(Inner Sphere)',   (SELECT id FROM armor_types WHERE slug = 'industrial')),
  -- Heavy Industrial
  ('Heavy Industrial(Inner Sphere)',   (SELECT id FROM armor_types WHERE slug = 'heavy-industrial')),
  ('Heavy Industrial(Clan)',           (SELECT id FROM armor_types WHERE slug = 'heavy-industrial')),
  ('Heavy Industrial Armor',           (SELECT id FROM armor_types WHERE slug = 'heavy-industrial')),
  -- Commercial
  ('Commercial(Inner Sphere)',         (SELECT id FROM armor_types WHERE slug = 'commercial')),
  ('Commercial Armor',                 (SELECT id FROM armor_types WHERE slug = 'commercial')),
  ('Commercial Armor(Inner Sphere)',   (SELECT id FROM armor_types WHERE slug = 'commercial')),
  -- Reflective IS
  ('Reflective(Inner Sphere)',         (SELECT id FROM armor_types WHERE slug = 'reflective-is')),
  ('Reflective Armor(Inner Sphere)',   (SELECT id FROM armor_types WHERE slug = 'reflective-is')),
  ('Laser-Reflective(Inner Sphere)',   (SELECT id FROM armor_types WHERE slug = 'reflective-is')),
  -- Reflective Clan
  ('Reflective(Clan)',                 (SELECT id FROM armor_types WHERE slug = 'reflective-clan')),
  ('Laser-Reflective(Clan)',           (SELECT id FROM armor_types WHERE slug = 'reflective-clan')),
  -- Ferro-Lamellor
  ('Ferro-Lamellor(Clan)',             (SELECT id FROM armor_types WHERE slug = 'ferro-lamellor')),
  ('Ferro-Lamellor Armor',             (SELECT id FROM armor_types WHERE slug = 'ferro-lamellor'))
ON CONFLICT (alias) DO NOTHING;


-- ═════════════════════════════════════════════════════════════════════════════
-- 4. Structure type aliases
-- ═════════════════════════════════════════════════════════════════════════════

INSERT INTO structure_type_aliases (alias, structure_type_id) VALUES
  -- Standard prefixed variants
  ('IS Standard',                      (SELECT id FROM structure_types WHERE slug = 'standard')),
  ('Clan Standard',                    (SELECT id FROM structure_types WHERE slug = 'standard')),
  -- Industrial
  ('Industrial',                       (SELECT id FROM structure_types WHERE slug = 'industrial')),
  ('IS Industrial',                    (SELECT id FROM structure_types WHERE slug = 'industrial')),
  ('Clan Industrial',                  (SELECT id FROM structure_types WHERE slug = 'industrial')),
  ('Industrial Structure',             (SELECT id FROM structure_types WHERE slug = 'industrial')),
  -- Endo-Steel IS variants
  ('Endo-Steel',                       (SELECT id FROM structure_types WHERE slug = 'endo-steel-is')),
  ('IS Endo-Steel',                    (SELECT id FROM structure_types WHERE slug = 'endo-steel-is')),
  ('IS Endo-Steel Prototype',          (SELECT id FROM structure_types WHERE slug = 'endo-steel-is')),
  ('Endo Steel Prototype',             (SELECT id FROM structure_types WHERE slug = 'endo-steel-is')),
  -- Endo-Steel Clan
  ('Clan Endo-Steel',                  (SELECT id FROM structure_types WHERE slug = 'endo-steel-clan')),
  -- Composite
  ('IS Composite',                     (SELECT id FROM structure_types WHERE slug = 'composite')),
  -- Reinforced IS
  ('IS Reinforced',                    (SELECT id FROM structure_types WHERE slug = 'reinforced')),
  -- Reinforced Clan
  ('Clan Reinforced',                  (SELECT id FROM structure_types WHERE slug = 'reinforced-clan')),
  -- Endo-Composite IS variants
  ('IS Endo-Composite',               (SELECT id FROM structure_types WHERE slug = 'endo-composite-is')),
  -- Endo-Composite Clan
  ('Clan Endo-Composite',             (SELECT id FROM structure_types WHERE slug = 'endo-composite-clan')),
  ('Clan Endo Composite',             (SELECT id FROM structure_types WHERE slug = 'endo-composite-clan'))
ON CONFLICT (alias) DO NOTHING;


-- ═════════════════════════════════════════════════════════════════════════════
-- 5. Heatsink type aliases
-- ═════════════════════════════════════════════════════════════════════════════

INSERT INTO heatsink_type_aliases (alias, heatsink_type_id) VALUES
  ('IS Double',                (SELECT id FROM heatsink_types WHERE slug = 'double-is')),
  ('Clan Double',              (SELECT id FROM heatsink_types WHERE slug = 'double-clan')),
  ('IS Double Heat Sink',      (SELECT id FROM heatsink_types WHERE slug = 'double-is')),
  ('Clan Double Heat Sink',    (SELECT id FROM heatsink_types WHERE slug = 'double-clan')),
  ('Laser',                    (SELECT id FROM heatsink_types WHERE slug = 'laser')),
  ('Laser Heat Sink',          (SELECT id FROM heatsink_types WHERE slug = 'laser'))
ON CONFLICT (alias) DO NOTHING;


-- ═════════════════════════════════════════════════════════════════════════════
-- 6. Myomer type aliases
-- ═════════════════════════════════════════════════════════════════════════════

INSERT INTO myomer_type_aliases (alias, myomer_type_id) VALUES
  ('Triple-Strength',              (SELECT id FROM myomer_types WHERE slug = 'tsm')),
  ('Industrial Triple-Strength',   (SELECT id FROM myomer_types WHERE slug = 'tsm')),
  ('ISMASC',                       (SELECT id FROM myomer_types WHERE slug = 'masc')),
  ('CLMASC',                       (SELECT id FROM myomer_types WHERE slug = 'masc')),
  ('IS MASC',                      (SELECT id FROM myomer_types WHERE slug = 'masc')),
  ('Clan MASC',                    (SELECT id FROM myomer_types WHERE slug = 'masc'))
ON CONFLICT (alias) DO NOTHING;


-- ═════════════════════════════════════════════════════════════════════════════
-- 7. Re-populate FK columns with expanded aliases
-- ═════════════════════════════════════════════════════════════════════════════

UPDATE unit_mech_data md SET engine_type_id = ea.engine_type_id
FROM engine_type_aliases ea
WHERE md.engine_type_id IS NULL
  AND lower(trim(md.engine_type)) = lower(ea.alias);

UPDATE unit_mech_data md SET armor_type_id = aa.armor_type_id
FROM armor_type_aliases aa
WHERE md.armor_type_id IS NULL
  AND lower(trim(md.armor_type)) = lower(aa.alias);

UPDATE unit_mech_data md SET structure_type_id = sa.structure_type_id
FROM structure_type_aliases sa
WHERE md.structure_type_id IS NULL
  AND lower(trim(md.structure_type)) = lower(sa.alias);

UPDATE unit_mech_data md SET heatsink_type_id = ha.heatsink_type_id
FROM heatsink_type_aliases ha
WHERE md.heatsink_type_id IS NULL
  AND lower(trim(md.heat_sink_type)) = lower(ha.alias);

UPDATE unit_mech_data md SET gyro_type_id = ga.gyro_type_id
FROM gyro_type_aliases ga
WHERE md.gyro_type_id IS NULL
  AND lower(trim(md.gyro_type)) = lower(ga.alias);

UPDATE unit_mech_data md SET cockpit_type_id = ca.cockpit_type_id
FROM cockpit_type_aliases ca
WHERE md.cockpit_type_id IS NULL
  AND lower(trim(md.cockpit_type)) = lower(ca.alias);

UPDATE unit_mech_data md SET myomer_type_id = ma.myomer_type_id
FROM myomer_type_aliases ma
WHERE md.myomer_type_id IS NULL
  AND lower(trim(md.myomer_type)) = lower(ma.alias);
