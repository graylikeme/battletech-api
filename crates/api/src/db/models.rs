use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct DbMetadata {
    pub id: i32,
    pub version: String,
    pub schema_version: i32,
    pub description: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbRuleset {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub level: String,
    pub description: Option<String>,
    pub source_book: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbEra {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub start_year: i32,
    pub end_year: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbFaction {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub short_name: Option<String>,
    pub faction_type: String,
    pub is_clan: bool,
    pub founding_year: Option<i32>,
    pub dissolution_year: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbFactionEra {
    pub id: i32,
    pub faction_id: i32,
    pub era_id: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbUnitChassis {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub unit_type: String,
    pub tech_base: String,
    pub tonnage: rust_decimal::Decimal,
    pub intro_year: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbUnit {
    pub id: i32,
    pub slug: String,
    pub chassis_id: i32,
    pub variant: String,
    pub full_name: String,
    pub tech_base: String,
    pub rules_level: String,
    pub tonnage: rust_decimal::Decimal,
    pub bv: Option<i32>,
    pub cost: Option<i64>,
    pub intro_year: Option<i32>,
    pub extinction_year: Option<i32>,
    pub reintro_year: Option<i32>,
    pub source_book: Option<String>,
    pub description: Option<String>,
    /// Used by COUNT(*) OVER() window function when fetching paginated results
    pub total_count: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbEquipment {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub category: String,
    pub tech_base: String,
    pub rules_level: String,
    pub tonnage: Option<rust_decimal::Decimal>,
    pub crits: Option<i32>,
    pub damage: Option<String>,
    pub heat: Option<i32>,
    pub range_min: Option<i32>,
    pub range_short: Option<i32>,
    pub range_medium: Option<i32>,
    pub range_long: Option<i32>,
    pub bv: Option<i32>,
    pub intro_year: Option<i32>,
    pub source_book: Option<String>,
    pub description: Option<String>,
    pub total_count: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbLocation {
    pub id: i32,
    pub unit_id: i32,
    pub location: String,
    pub armor_points: Option<i32>,
    pub rear_armor: Option<i32>,
    pub structure_points: Option<i32>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbLoadoutEntry {
    pub id: i32,
    pub unit_id: i32,
    pub equipment_id: i32,
    pub location: Option<String>,
    pub quantity: i32,
    pub is_rear_facing: bool,
    pub notes: Option<String>,
    // Joined from equipment
    pub equipment_slug: String,
    pub equipment_name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbQuirk {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub is_positive: bool,
    pub description: Option<String>,
}
