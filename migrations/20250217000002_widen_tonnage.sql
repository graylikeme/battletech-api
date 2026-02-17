-- Widen tonnage columns to support large units (dropships, jumpships, warships).
-- NUMERIC(6,1) maxes at 99999.9 tons; NUMERIC(10,1) supports up to 999999999.9.
ALTER TABLE unit_chassis ALTER COLUMN tonnage TYPE NUMERIC(10,1);
ALTER TABLE units        ALTER COLUMN tonnage TYPE NUMERIC(10,1);
