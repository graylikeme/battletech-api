#!/usr/bin/env bash
set -euo pipefail

# Load BattleTech seed data from compressed SQL dump.
# Requires: psql, gunzip
#
# Usage:
#   ./seed/load.sh                          # uses DATABASE_URL from env / .env
#   ./seed/load.sh postgres://user:pass@host:5432/battletech

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DUMP_FILE="$SCRIPT_DIR/data.sql.gz"

if [ ! -f "$DUMP_FILE" ]; then
  echo "Error: seed dump not found at $DUMP_FILE" >&2
  exit 1
fi

# Resolve database URL: argument > env var > .env file
if [ $# -ge 1 ]; then
  DB_URL="$1"
elif [ -n "${DATABASE_URL:-}" ]; then
  DB_URL="$DATABASE_URL"
elif [ -f "$SCRIPT_DIR/../.env" ]; then
  DB_URL=$(grep -E '^DATABASE_URL=' "$SCRIPT_DIR/../.env" | head -1 | cut -d= -f2-)
  if [ -z "$DB_URL" ]; then
    echo "Error: DATABASE_URL not found in .env" >&2
    exit 1
  fi
else
  echo "Error: no DATABASE_URL provided (pass as argument, env var, or set in .env)" >&2
  exit 1
fi

echo "==> Truncating tables..."
psql "$DB_URL" -q -c "
  TRUNCATE
    unit_quirks,
    unit_loadout,
    unit_locations,
    unit_availability,
    units,
    unit_chassis,
    equipment,
    quirks,
    faction_eras,
    factions,
    eras,
    rulesets,
    dataset_metadata
  CASCADE;
"

echo "==> Loading seed data..."
gunzip -c "$DUMP_FILE" | psql "$DB_URL" -q -o /dev/null

echo "==> Resetting sequences..."
psql "$DB_URL" -q -o /dev/null -c "
  SELECT setval('dataset_metadata_id_seq', COALESCE((SELECT MAX(id) FROM dataset_metadata), 0) + 1, false);
  SELECT setval('eras_id_seq',             COALESCE((SELECT MAX(id) FROM eras), 0) + 1, false);
  SELECT setval('factions_id_seq',         COALESCE((SELECT MAX(id) FROM factions), 0) + 1, false);
  SELECT setval('unit_chassis_id_seq',     COALESCE((SELECT MAX(id) FROM unit_chassis), 0) + 1, false);
  SELECT setval('units_id_seq',            COALESCE((SELECT MAX(id) FROM units), 0) + 1, false);
  SELECT setval('equipment_id_seq',        COALESCE((SELECT MAX(id) FROM equipment), 0) + 1, false);
  SELECT setval('unit_locations_id_seq',   COALESCE((SELECT MAX(id) FROM unit_locations), 0) + 1, false);
  SELECT setval('unit_loadout_id_seq',     COALESCE((SELECT MAX(id) FROM unit_loadout), 0) + 1, false);
  SELECT setval('quirks_id_seq',           COALESCE((SELECT MAX(id) FROM quirks), 0) + 1, false);
  SELECT setval('unit_quirks_id_seq',      COALESCE((SELECT MAX(id) FROM unit_quirks), 0) + 1, false);
  SELECT setval('rulesets_id_seq',         COALESCE((SELECT MAX(id) FROM rulesets), 0) + 1, false);
  SELECT setval('faction_eras_id_seq',     COALESCE((SELECT MAX(id) FROM faction_eras), 0) + 1, false);
  SELECT setval('unit_availability_id_seq', COALESCE((SELECT MAX(id) FROM unit_availability), 0) + 1, false);
"

echo "==> Done. Verifying row counts..."
psql "$DB_URL" -t -c "
  SELECT 'unit_chassis:  ' || COUNT(*) FROM unit_chassis
  UNION ALL SELECT 'units:         ' || COUNT(*) FROM units
  UNION ALL SELECT 'equipment:     ' || COUNT(*) FROM equipment
  UNION ALL SELECT 'unit_loadout:  ' || COUNT(*) FROM unit_loadout
  UNION ALL SELECT 'unit_locations:' || COUNT(*) FROM unit_locations
  UNION ALL SELECT 'quirks:        ' || COUNT(*) FROM quirks
  UNION ALL SELECT 'unit_quirks:   ' || COUNT(*) FROM unit_quirks
  UNION ALL SELECT 'eras:          ' || COUNT(*) FROM eras
  UNION ALL SELECT 'factions:      ' || COUNT(*) FROM factions;
"
