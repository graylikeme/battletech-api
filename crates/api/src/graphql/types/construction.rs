use async_graphql::{Enum, Object, SimpleObject, ID};
use rust_decimal::prelude::ToPrimitive;

use crate::db::models::{
    DbArmorType, DbCockpitType, DbEngineType, DbGyroType, DbHeatsinkType,
    DbMyomerType, DbStructureType,
};

// ── Filter Enums ─────────────────────────────────────────────────────────────

/// Technology base filter for construction reference queries.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum TechBaseFilter {
    /// Inner Sphere technology.
    InnerSphere,
    /// Clan technology.
    Clan,
    /// Mixed Inner Sphere / Clan technology.
    Mixed,
    /// Primitive (pre-Star League) technology.
    Primitive,
}

impl TechBaseFilter {
    pub fn as_db_str(self) -> &'static str {
        match self {
            Self::InnerSphere => "inner_sphere",
            Self::Clan => "clan",
            Self::Mixed => "mixed",
            Self::Primitive => "primitive",
        }
    }
}

/// Rules level filter for construction reference queries.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum RulesLevelFilter {
    /// Basic game rules, limited equipment options.
    Introductory,
    /// Standard tournament-legal rules.
    Standard,
    /// Advanced rules with expanded options.
    Advanced,
    /// Experimental/prototype equipment.
    Experimental,
    /// Fan-created or unofficial equipment.
    Unofficial,
}

impl RulesLevelFilter {
    pub fn as_db_str(self) -> &'static str {
        match self {
            Self::Introductory => "introductory",
            Self::Standard => "standard",
            Self::Advanced => "advanced",
            Self::Experimental => "experimental",
            Self::Unofficial => "unofficial",
        }
    }
}

// ── Engine Type ──────────────────────────────────────────────────────────────

pub struct EngineTypeGql(pub DbEngineType);

/// A fusion or combustion engine type used in BattleMech construction.
/// Weight = engine_weight_table[rating].standard_weight * weight_multiplier.
#[Object]
impl EngineTypeGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "xl-is", "standard-fusion").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Human-readable engine name (e.g. "XL Engine (IS)").
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// Technology base required for this engine type.
    async fn tech_base(&self) -> &str {
        &self.0.tech_base
    }

    /// Minimum rules level needed to use this engine type.
    async fn rules_level(&self) -> &str {
        &self.0.rules_level
    }

    /// Multiplier applied to standard engine weight from engine_weight_table.
    async fn weight_multiplier(&self) -> f64 {
        self.0.weight_multiplier.to_f64().unwrap_or(1.0)
    }

    /// Critical slots consumed in the center torso.
    async fn ct_crits(&self) -> i32 {
        self.0.ct_crits as i32
    }

    /// Critical slots consumed in each side torso.
    async fn st_crits(&self) -> i32 {
        self.0.st_crits as i32
    }

    /// In-universe year when this engine type was first produced.
    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }
}

// ── Armor Type ───────────────────────────────────────────────────────────────

pub struct ArmorTypeGql(pub DbArmorType);

/// An armor technology used in BattleMech construction.
/// Total armor weight = total_armor_points / points_per_ton.
#[Object]
impl ArmorTypeGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "ferro-fibrous-is").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Human-readable armor name (e.g. "Ferro-Fibrous (IS)").
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// Technology base required for this armor type.
    async fn tech_base(&self) -> &str {
        &self.0.tech_base
    }

    /// Minimum rules level needed to use this armor type.
    async fn rules_level(&self) -> &str {
        &self.0.rules_level
    }

    /// Armor points provided per ton of armor weight.
    async fn points_per_ton(&self) -> f64 {
        self.0.points_per_ton.to_f64().unwrap_or(16.0)
    }

    /// Total critical slots consumed across all locations.
    async fn crits(&self) -> i32 {
        self.0.crits as i32
    }

    /// In-universe year when this armor type was first produced.
    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }
}

// ── Structure Type ───────────────────────────────────────────────────────────

pub struct StructureTypeGql(pub DbStructureType);

/// An internal structure technology used in BattleMech construction.
/// Structure weight = mech_tonnage * weight_fraction.
#[Object]
impl StructureTypeGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "endo-steel-is").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Human-readable structure name (e.g. "Endo Steel (IS)").
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// Technology base required for this structure type.
    async fn tech_base(&self) -> &str {
        &self.0.tech_base
    }

    /// Minimum rules level needed to use this structure type.
    async fn rules_level(&self) -> &str {
        &self.0.rules_level
    }

    /// Fraction of mech tonnage consumed by the internal structure.
    async fn weight_fraction(&self) -> f64 {
        self.0.weight_fraction.to_f64().unwrap_or(0.1)
    }

    /// Total critical slots consumed across all locations.
    async fn crits(&self) -> i32 {
        self.0.crits as i32
    }

    /// In-universe year when this structure type was first produced.
    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }
}

// ── Heatsink Type ────────────────────────────────────────────────────────────

pub struct HeatsinkTypeGql(pub DbHeatsinkType);

/// A heat sink technology used in BattleMech construction.
#[Object]
impl HeatsinkTypeGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "double-is").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Human-readable heat sink name (e.g. "Double Heat Sink (IS)").
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// Technology base required for this heat sink type.
    async fn tech_base(&self) -> &str {
        &self.0.tech_base
    }

    /// Minimum rules level needed to use this heat sink type.
    async fn rules_level(&self) -> &str {
        &self.0.rules_level
    }

    /// Heat points dissipated per heat sink per turn.
    async fn dissipation(&self) -> i32 {
        self.0.dissipation as i32
    }

    /// Critical slots consumed per heat sink.
    async fn crits(&self) -> i32 {
        self.0.crits as i32
    }

    /// Weight per heat sink in tons.
    async fn weight(&self) -> f64 {
        self.0.weight.to_f64().unwrap_or(1.0)
    }

    /// In-universe year when this heat sink type was first produced.
    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }
}

// ── Gyro Type ────────────────────────────────────────────────────────────────

pub struct GyroTypeGql(pub DbGyroType);

/// A gyroscope type used in BattleMech construction.
/// Weight = ceil(engine_rating / 100) * weight_multiplier (standard BT rounding: 0.5 rounds up).
#[Object]
impl GyroTypeGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "xl", "compact").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Human-readable gyro name (e.g. "XL Gyro").
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// Technology base required. Null means tech-base-agnostic.
    async fn tech_base(&self) -> Option<&str> {
        self.0.tech_base.as_deref()
    }

    /// Minimum rules level needed to use this gyro type.
    async fn rules_level(&self) -> &str {
        &self.0.rules_level
    }

    /// Multiplier applied to base gyro weight (ceil(engine_rating / 100)).
    async fn weight_multiplier(&self) -> f64 {
        self.0.weight_multiplier.to_f64().unwrap_or(1.0)
    }

    /// Critical slots consumed in the center torso.
    async fn crits(&self) -> i32 {
        self.0.crits as i32
    }

    /// True if this gyro type is restricted to superheavy mechs.
    async fn is_superheavy_only(&self) -> bool {
        self.0.is_superheavy_only
    }

    /// In-universe year when this gyro type was first produced.
    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }
}

// ── Cockpit Type ─────────────────────────────────────────────────────────────

pub struct CockpitTypeGql(pub DbCockpitType);

/// A cockpit type used in BattleMech construction.
#[Object]
impl CockpitTypeGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "small", "command-console").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Human-readable cockpit name (e.g. "Small Cockpit").
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// Technology base required. Null means tech-base-agnostic.
    async fn tech_base(&self) -> Option<&str> {
        self.0.tech_base.as_deref()
    }

    /// Minimum rules level needed to use this cockpit type.
    async fn rules_level(&self) -> &str {
        &self.0.rules_level
    }

    /// Weight in tons.
    async fn weight(&self) -> i32 {
        self.0.weight as i32
    }

    /// Total critical slots consumed.
    async fn crits(&self) -> i32 {
        self.0.crits as i32
    }

    /// In-universe year when this cockpit type was first produced.
    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }
}

// ── Myomer Type ──────────────────────────────────────────────────────────────

pub struct MyomerTypeGql(pub DbMyomerType);

/// A myomer (muscle fiber) type used in BattleMech construction.
#[Object]
impl MyomerTypeGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "tsm", "masc").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Human-readable myomer name (e.g. "Triple-Strength Myomer").
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// Technology base required. Null means tech-base-agnostic.
    async fn tech_base(&self) -> Option<&str> {
        self.0.tech_base.as_deref()
    }

    /// Minimum rules level needed to use this myomer type.
    async fn rules_level(&self) -> &str {
        &self.0.rules_level
    }

    /// In-universe year when this myomer type was first produced.
    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }

    /// Type-specific properties as a JSON object (e.g. {"tonnage_fraction": 0.05} for MASC).
    async fn properties(&self) -> &serde_json::Value {
        &self.0.properties
    }
}

// ── Engine Weight ────────────────────────────────────────────────────────────

/// Standard fusion engine weight entry from TechManual.
/// Actual engine weight = standard_weight * engine_type.weight_multiplier.
#[derive(SimpleObject)]
pub struct EngineWeightGql {
    /// Engine rating (10-400 in increments of 5).
    pub rating: i32,
    /// Standard fusion engine weight in tons for this rating.
    pub standard_weight: f64,
}

// ── Internal Structure ───────────────────────────────────────────────────────

/// Internal structure hit points per location for a given mech tonnage.
/// Max armor per location = 2 * structure_points (except head max = 9).
#[derive(SimpleObject)]
pub struct InternalStructureGql {
    /// Mech tonnage this row applies to.
    pub tonnage: i32,
    /// Head structure points (always 3).
    pub head: i32,
    /// Center torso structure points.
    pub center_torso: i32,
    /// Side torso structure points (left and right are equal).
    pub side_torso: i32,
    /// Arm structure points (left and right are equal).
    pub arm: i32,
    /// Leg structure points (left and right are equal).
    pub leg: i32,
}

// ── Construction Reference (mega-query) ──────────────────────────────────────

pub struct ConstructionReferenceGql {
    pub engine_types: Vec<EngineTypeGql>,
    pub armor_types: Vec<ArmorTypeGql>,
    pub structure_types: Vec<StructureTypeGql>,
    pub heatsink_types: Vec<HeatsinkTypeGql>,
    pub gyro_types: Vec<GyroTypeGql>,
    pub cockpit_types: Vec<CockpitTypeGql>,
    pub myomer_types: Vec<MyomerTypeGql>,
    pub engine_weights: Vec<EngineWeightGql>,
    pub internal_structure: Vec<InternalStructureGql>,
}

/// All construction reference data in a single response. Fetch once and cache locally for builder initialization.
#[Object]
impl ConstructionReferenceGql {
    /// All available engine types.
    async fn engine_types(&self) -> &[EngineTypeGql] {
        &self.engine_types
    }

    /// All available armor types.
    async fn armor_types(&self) -> &[ArmorTypeGql] {
        &self.armor_types
    }

    /// All available internal structure types.
    async fn structure_types(&self) -> &[StructureTypeGql] {
        &self.structure_types
    }

    /// All available heat sink types.
    async fn heatsink_types(&self) -> &[HeatsinkTypeGql] {
        &self.heatsink_types
    }

    /// All available gyroscope types.
    async fn gyro_types(&self) -> &[GyroTypeGql] {
        &self.gyro_types
    }

    /// All available cockpit types.
    async fn cockpit_types(&self) -> &[CockpitTypeGql] {
        &self.cockpit_types
    }

    /// All available myomer types.
    async fn myomer_types(&self) -> &[MyomerTypeGql] {
        &self.myomer_types
    }

    /// Engine weight lookup table (rating → standard weight).
    async fn engine_weights(&self) -> &[EngineWeightGql] {
        &self.engine_weights
    }

    /// Internal structure points by mech tonnage.
    async fn internal_structure(&self) -> &[InternalStructureGql] {
        &self.internal_structure
    }
}
