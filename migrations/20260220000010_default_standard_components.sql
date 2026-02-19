-- Backfill standard gyro/cockpit/myomer FKs where MegaMek omitted the field.
-- MegaMek doesn't store gyro_type/cockpit_type/myomer_type when they are standard,
-- leaving them NULL. Default these to the standard component type.

UPDATE unit_mech_data SET gyro_type_id = (
    SELECT gyro_type_id FROM gyro_type_aliases WHERE alias = 'Standard Gyro'
) WHERE gyro_type_id IS NULL;

UPDATE unit_mech_data SET cockpit_type_id = (
    SELECT cockpit_type_id FROM cockpit_type_aliases WHERE alias = 'Standard Cockpit'
) WHERE cockpit_type_id IS NULL;

UPDATE unit_mech_data SET myomer_type_id = (
    SELECT myomer_type_id FROM myomer_type_aliases WHERE alias = 'Standard'
) WHERE myomer_type_id IS NULL;
