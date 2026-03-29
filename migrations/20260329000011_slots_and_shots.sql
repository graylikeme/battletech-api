-- Add critical slot positions to loadout entries (from MTF location sections)
ALTER TABLE unit_loadout ADD COLUMN slots INTEGER[];

-- Add shots-per-ton for ammunition equipment
ALTER TABLE equipment ADD COLUMN shots_per_ton INTEGER;
