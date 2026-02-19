-- ============================================================================
-- Equipment Builder Enhancements
-- Adds observed locations, ammo linkage, and stats provenance to equipment.
-- ============================================================================

-- ── Observed Locations ───────────────────────────────────────────────────────
-- Empirically derived from existing loadout data. NULL = unknown/unrestricted.

ALTER TABLE equipment ADD COLUMN observed_locations TEXT[];
CREATE INDEX idx_equipment_observed_locations ON equipment USING GIN (observed_locations);

-- Populate from existing loadout data
UPDATE equipment e SET observed_locations = sub.locs
FROM (
  SELECT ul.equipment_id,
         array_agg(DISTINCT ul.location::text ORDER BY ul.location::text) AS locs
  FROM unit_loadout ul
  WHERE ul.location IS NOT NULL
  GROUP BY ul.equipment_id
) sub
WHERE e.id = sub.equipment_id;

-- ── Ammo-to-Weapon Linkage ───────────────────────────────────────────────────

ALTER TABLE equipment ADD COLUMN ammo_for_id INT REFERENCES equipment(id) ON DELETE SET NULL;
CREATE INDEX idx_equipment_ammo_for ON equipment(ammo_for_id) WHERE ammo_for_id IS NOT NULL;

-- ── Stats Provenance ─────────────────────────────────────────────────────────

ALTER TABLE equipment ADD COLUMN stats_source TEXT;
ALTER TABLE equipment ADD COLUMN stats_updated_at TIMESTAMPTZ;
