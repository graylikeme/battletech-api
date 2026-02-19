-- Add Master Unit List (MUL) fields to units table.
ALTER TABLE units ADD COLUMN mul_id INTEGER UNIQUE;
ALTER TABLE units ADD COLUMN role TEXT;
ALTER TABLE units ADD COLUMN bv_source TEXT;
ALTER TABLE units ADD COLUMN intro_year_source TEXT;
ALTER TABLE units ADD COLUMN last_mul_import_at TIMESTAMPTZ;

CREATE INDEX idx_units_mul_id ON units (mul_id) WHERE mul_id IS NOT NULL;
CREATE INDEX idx_units_role ON units (role) WHERE role IS NOT NULL;
