use async_graphql::{dataloader::DataLoader, Context, Object, SimpleObject, ID};
use rust_decimal::prelude::ToPrimitive;

use crate::{
    db::models::{DbMechData, DbUnit, DbUnitChassis},
    error::AppError,
    graphql::loaders::MechDataLoader,
    state::AppState,
};

// ── Quirk ──────────────────────────────────────────────────────────────────

/// A quirk affecting a specific unit variant, providing a positive or negative gameplay modifier.
#[derive(SimpleObject)]
pub struct QuirkGql {
    /// Lowercase, hyphen-separated identifier (e.g. "improved-sensors").
    pub slug: String,
    /// Human-readable quirk name.
    pub name: String,
    /// True if this quirk is beneficial; false if it is a drawback.
    pub is_positive: bool,
    /// Explanation of the quirk's gameplay effect, if available.
    pub description: Option<String>,
}

// ── Location ───────────────────────────────────────────────────────────────

/// An armor/structure location on a unit (e.g. head, center_torso, left_arm).
#[derive(SimpleObject)]
pub struct LocationGql {
    /// Body location name in snake_case (e.g. "center_torso", "left_arm", "head").
    pub location: String,
    /// Front armor points at this location. Null if the location has no armor.
    pub armor_points: Option<i32>,
    /// Rear armor points at this location. Only applicable to torso locations; null otherwise.
    pub rear_armor: Option<i32>,
    /// Internal structure points at this location. Null if not applicable.
    pub structure_points: Option<i32>,
}

// ── Loadout Entry ──────────────────────────────────────────────────────────

/// A single equipment item mounted on a unit at a specific location.
#[derive(SimpleObject)]
pub struct LoadoutEntryGql {
    /// Lowercase, hyphen-separated identifier of the equipment item (e.g. "medium-laser").
    pub equipment_slug: String,
    /// Human-readable name of the equipment item.
    pub equipment_name: String,
    /// Body location where this equipment is mounted (e.g. "right_arm"). Null if location is unspecified.
    pub location: Option<String>,
    /// Number of this equipment item mounted at this location.
    pub quantity: i32,
    /// True if the weapon is rear-facing (fires into the rear arc).
    pub is_rear_facing: bool,
    /// Additional notes about this loadout entry, if any.
    pub notes: Option<String>,
}

// ── Unit Chassis ───────────────────────────────────────────────────────────

pub struct UnitChassisGql(pub DbUnitChassis);

/// A chassis (design family) grouping all variants of a unit. For example, the "Atlas" chassis contains AS7-D, AS7-K, etc.
#[Object]
impl UnitChassisGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "atlas").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Human-readable chassis name (e.g. "Atlas").
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// Unit type category (e.g. "BattleMech", "Vehicle", "AeroSpaceFighter", "DropShip").
    async fn unit_type(&self) -> &str {
        &self.0.unit_type
    }

    /// Technology base. One of: inner_sphere, clan, mixed, primitive.
    async fn tech_base(&self) -> &str {
        &self.0.tech_base
    }

    /// Weight in metric tons. Ranges from 20 (light mechs) to 500,000+ (jumpships).
    async fn tonnage(&self) -> f64 {
        self.0.tonnage.to_f64().unwrap_or(0.0)
    }

    /// In-universe BattleTech year when this chassis was first produced (e.g. 3025). Not a real-world date.
    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }

    /// Flavor text or lore description of the chassis.
    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

    /// All unit variants belonging to this chassis, ordered by variant designation.
    #[graphql(complexity = 5)]
    async fn variants(&self, ctx: &Context<'_>) -> Result<Vec<UnitGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = sqlx::query_as!(
            DbUnit,
            r#"SELECT u.id, u.slug, u.chassis_id, u.variant, u.full_name,
                      u.tech_base::text AS "tech_base!", u.rules_level::text AS "rules_level!",
                      u.tonnage, u.bv, u.cost, u.intro_year, u.extinction_year,
                      u.reintro_year, u.source_book, u.description,
                      u.mul_id, u.role, u.clan_name, NULL::bigint AS total_count
               FROM units u WHERE u.chassis_id = $1 ORDER BY u.variant"#,
            self.0.id
        )
        .fetch_all(&state.pool)
        .await?;
        Ok(rows.into_iter().map(UnitGql).collect())
    }
}

// ── Mech Data ─────────────────────────────────────────────────────────────

pub struct MechDataGql(DbMechData);

/// Mech-specific technical data: engine, movement, heat management, armor/structure type, and chassis configuration.
#[Object]
impl MechDataGql {
    /// Chassis layout: "Biped", "Quad", "Tripod", or "LAM".
    async fn config(&self) -> &str {
        &self.0.config
    }

    /// True if this mech is an OmniMech with configurable pod space.
    async fn is_omnimech(&self) -> bool {
        self.0.is_omnimech
    }

    /// Engine power rating (e.g. 300 for a 100-ton mech with walk 3).
    async fn engine_rating(&self) -> Option<i32> {
        self.0.engine_rating
    }

    /// Engine type name (e.g. "Fusion Engine", "XL Engine", "Light Engine").
    async fn engine_type(&self) -> Option<&str> {
        self.0.engine_type.as_deref()
    }

    /// Walking movement points per turn.
    async fn walk_mp(&self) -> Option<i32> {
        self.0.walk_mp
    }

    /// Running movement points per turn. Computed as ceil(walkMp * 1.5).
    async fn run_mp(&self) -> Option<i32> {
        self.0.walk_mp.map(|w| ((w as f64) * 1.5).ceil() as i32)
    }

    /// Jump movement points per turn. 0 means no jump capability.
    async fn jump_mp(&self) -> Option<i32> {
        self.0.jump_mp
    }

    /// Total number of heat sinks installed.
    async fn heat_sink_count(&self) -> Option<i32> {
        self.0.heat_sink_count
    }

    /// Heat sink technology: "Single", "Double", or "Clan Double Heat Sink".
    async fn heat_sink_type(&self) -> Option<&str> {
        self.0.heat_sink_type.as_deref()
    }

    /// Internal structure technology (e.g. "Standard", "Endo Steel").
    async fn structure_type(&self) -> Option<&str> {
        self.0.structure_type.as_deref()
    }

    /// Armor technology (e.g. "Standard Armor", "Ferro-Fibrous").
    async fn armor_type(&self) -> Option<&str> {
        self.0.armor_type.as_deref()
    }

    /// Gyroscope type (e.g. "Standard Gyro", "XL Gyro", "Compact Gyro").
    async fn gyro_type(&self) -> Option<&str> {
        self.0.gyro_type.as_deref()
    }

    /// Cockpit type (e.g. "Standard Cockpit", "Small Cockpit").
    async fn cockpit_type(&self) -> Option<&str> {
        self.0.cockpit_type.as_deref()
    }

    /// Myomer type (e.g. "Standard", "Triple-Strength Myomer").
    async fn myomer_type(&self) -> Option<&str> {
        self.0.myomer_type.as_deref()
    }
}

// ── Unit ───────────────────────────────────────────────────────────────────

pub struct UnitGql(pub DbUnit);

/// A specific unit variant (e.g. "Atlas AS7-D") with its stats, loadout, armor, quirks, and faction availability.
#[Object]
impl UnitGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "atlas-as7-d").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Variant designation (e.g. "AS7-D", "Prime", "A").
    async fn variant(&self) -> &str {
        &self.0.variant
    }

    /// Full display name combining chassis and variant (e.g. "Atlas AS7-D").
    async fn full_name(&self) -> &str {
        &self.0.full_name
    }

    /// Technology base. One of: inner_sphere, clan, mixed, primitive.
    async fn tech_base(&self) -> &str {
        &self.0.tech_base
    }

    /// Rules level governing which game modes allow this unit. One of: introductory, standard, advanced, experimental, unofficial.
    async fn rules_level(&self) -> &str {
        &self.0.rules_level
    }

    /// Weight in metric tons. Ranges from 20 (light mechs) to 500,000+ (jumpships).
    async fn tonnage(&self) -> f64 {
        self.0.tonnage.to_f64().unwrap_or(0.0)
    }

    /// Battle Value — composite combat effectiveness score used for game balancing. Null if not computed.
    async fn bv(&self) -> Option<i32> {
        self.0.bv
    }

    /// Construction cost in C-bills (in-universe currency). Null if not available.
    async fn cost(&self) -> Option<i64> {
        self.0.cost
    }

    /// In-universe BattleTech year when this variant was first produced (e.g. 3025). Not a real-world date.
    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }

    /// In-universe BattleTech year when this variant went extinct. Null if still in production.
    async fn extinction_year(&self) -> Option<i32> {
        self.0.extinction_year
    }

    /// In-universe BattleTech year when this variant was reintroduced after extinction. Null if never reintroduced.
    async fn reintro_year(&self) -> Option<i32> {
        self.0.reintro_year
    }

    /// Source book or technical readout where this unit is published.
    async fn source_book(&self) -> Option<&str> {
        self.0.source_book.as_deref()
    }

    /// Flavor text or lore description of the unit variant.
    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

    /// Master Unit List numeric ID. Null for units not found in MUL.
    async fn mul_id(&self) -> Option<i32> {
        self.0.mul_id
    }

    /// Tactical role (e.g. "Juggernaut", "Sniper", "Striker"). Null if unassigned.
    async fn role(&self) -> Option<&str> {
        self.0.role.as_deref()
    }

    /// Alternate Clan/IS reporting name (e.g. "Fire Moth A" for "Dasher A"). Null for units without dual names.
    async fn clan_name(&self) -> Option<&str> {
        self.0.clan_name.as_deref()
    }

    /// Parent chassis this variant belongs to.
    async fn chassis(&self, ctx: &Context<'_>) -> Result<Option<UnitChassisGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = sqlx::query_as!(
            DbUnitChassis,
            r#"SELECT id, slug, name, unit_type, tech_base::text AS "tech_base!",
                      tonnage, intro_year, description
               FROM unit_chassis WHERE id = $1"#,
            self.0.chassis_id
        )
        .fetch_optional(&state.pool)
        .await?;
        Ok(row.map(UnitChassisGql))
    }

    /// Armor and internal structure values for each body location.
    #[graphql(complexity = 5)]
    async fn locations(&self, ctx: &Context<'_>) -> Result<Vec<LocationGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = crate::db::units::get_locations(&state.pool, self.0.id).await?;
        Ok(rows
            .into_iter()
            .map(|l| LocationGql {
                location: l.location,
                armor_points: l.armor_points,
                rear_armor: l.rear_armor,
                structure_points: l.structure_points,
            })
            .collect())
    }

    /// All equipment items mounted on this unit, grouped by location.
    #[graphql(complexity = 10)]
    async fn loadout(&self, ctx: &Context<'_>) -> Result<Vec<LoadoutEntryGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = crate::db::units::get_loadout(&state.pool, self.0.id).await?;
        Ok(rows
            .into_iter()
            .map(|e| LoadoutEntryGql {
                equipment_slug: e.equipment_slug,
                equipment_name: e.equipment_name,
                location: e.location,
                quantity: e.quantity,
                is_rear_facing: e.is_rear_facing,
                notes: e.notes,
            })
            .collect())
    }

    /// Positive and negative quirks unique to this unit variant.
    #[graphql(complexity = 3)]
    async fn quirks(&self, ctx: &Context<'_>) -> Result<Vec<QuirkGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = crate::db::units::get_quirks(&state.pool, self.0.id).await?;
        Ok(rows
            .into_iter()
            .map(|q| QuirkGql {
                slug: q.slug,
                name: q.name,
                is_positive: q.is_positive,
                description: q.description,
            })
            .collect())
    }

    /// Faction and era availability records for this unit.
    #[graphql(complexity = 5)]
    async fn availability(&self, ctx: &Context<'_>) -> Result<Vec<AvailabilityGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = sqlx::query!(
            r#"SELECT f.slug AS faction_slug, f.name AS faction_name,
                      e.slug AS era_slug, e.name AS era_name,
                      ua.availability_code, ua.notes
               FROM unit_availability ua
               JOIN factions f ON f.id = ua.faction_id
               JOIN eras e ON e.id = ua.era_id
               WHERE ua.unit_id = $1
               ORDER BY e.start_year, f.name"#,
            self.0.id
        )
        .fetch_all(&state.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| AvailabilityGql {
                faction_slug: r.faction_slug,
                faction_name: r.faction_name,
                era_slug: r.era_slug,
                era_name: r.era_name,
                availability_code: r.availability_code,
                notes: r.notes,
            })
            .collect())
    }

    /// Mech-specific technical data. Null for non-mech units (vehicles, aerospace, etc.).
    #[graphql(complexity = 5)]
    async fn mech_data(&self, ctx: &Context<'_>) -> Result<Option<MechDataGql>, AppError> {
        let loader = ctx.data::<DataLoader<MechDataLoader>>().unwrap();
        let data = loader
            .load_one(self.0.id)
            .await
            .map_err(|e| AppError::Internal(e.message))?;
        Ok(data.map(MechDataGql))
    }

    /// Reserved for future vehicle-specific data (motive type, turret, etc.).
    async fn vehicle_data(&self) -> Option<bool> {
        None
    }
}

/// A record of a unit's availability to a specific faction during a specific era.
#[derive(SimpleObject)]
pub struct AvailabilityGql {
    /// Lowercase, hyphen-separated faction identifier (e.g. "clan-wolf").
    pub faction_slug: String,
    /// Human-readable faction name.
    pub faction_name: String,
    /// Lowercase, hyphen-separated era identifier (e.g. "clan-invasion").
    pub era_slug: String,
    /// Human-readable era name.
    pub era_name: String,
    /// MegaMek availability rating code (e.g. "A", "B", "F"). Null if unrated.
    pub availability_code: Option<String>,
    /// Additional notes about this availability entry.
    pub notes: Option<String>,
}
