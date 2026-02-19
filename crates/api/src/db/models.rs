#![allow(dead_code)] // Fields populated by sqlx FromRow may not all be read in Rust

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
    pub mul_id: Option<i32>,
    pub role: Option<String>,
    pub clan_name: Option<String>,
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

#[derive(Debug, Clone, FromRow)]
pub struct DbMechData {
    pub unit_id: i32,
    pub config: String,
    pub is_omnimech: bool,
    pub engine_rating: Option<i32>,
    pub engine_type: Option<String>,
    pub walk_mp: Option<i32>,
    pub jump_mp: Option<i32>,
    pub heat_sink_count: Option<i32>,
    pub heat_sink_type: Option<String>,
    pub structure_type: Option<String>,
    pub armor_type: Option<String>,
    pub gyro_type: Option<String>,
    pub cockpit_type: Option<String>,
    pub myomer_type: Option<String>,
}

// ── Construction Reference ───────────────────────────────────────────────

#[derive(Debug, Clone, FromRow)]
pub struct DbEngineType {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub tech_base: String,
    pub rules_level: String,
    pub weight_multiplier: rust_decimal::Decimal,
    pub ct_crits: i16,
    pub st_crits: i16,
    pub intro_year: Option<i32>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbArmorType {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub tech_base: String,
    pub rules_level: String,
    pub points_per_ton: rust_decimal::Decimal,
    pub crits: i16,
    pub intro_year: Option<i32>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbStructureType {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub tech_base: String,
    pub rules_level: String,
    pub weight_fraction: rust_decimal::Decimal,
    pub crits: i16,
    pub intro_year: Option<i32>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbHeatsinkType {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub tech_base: String,
    pub rules_level: String,
    pub dissipation: i16,
    pub crits: i16,
    pub weight: rust_decimal::Decimal,
    pub intro_year: Option<i32>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbGyroType {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub tech_base: Option<String>,
    pub rules_level: String,
    pub weight_multiplier: rust_decimal::Decimal,
    pub crits: i16,
    pub is_superheavy_only: bool,
    pub intro_year: Option<i32>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbCockpitType {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub tech_base: Option<String>,
    pub rules_level: String,
    pub weight: i16,
    pub crits: i16,
    pub intro_year: Option<i32>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbMyomerType {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub tech_base: Option<String>,
    pub rules_level: String,
    pub intro_year: Option<i32>,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbEngineWeight {
    pub rating: i16,
    pub standard_weight: rust_decimal::Decimal,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbInternalStructure {
    pub tonnage: i16,
    pub head: i16,
    pub center_torso: i16,
    pub side_torso: i16,
    pub arm: i16,
    pub leg: i16,
}
