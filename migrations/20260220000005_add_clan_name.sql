ALTER TABLE units ADD COLUMN clan_name TEXT;
CREATE INDEX idx_units_clan_name ON units (clan_name) WHERE clan_name IS NOT NULL;
