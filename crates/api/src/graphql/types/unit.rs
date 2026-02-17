use async_graphql::{Context, Object, SimpleObject, ID};
use rust_decimal::prelude::ToPrimitive;

use crate::{db::models::{DbUnit, DbUnitChassis}, error::AppError, state::AppState};

// ── Quirk ──────────────────────────────────────────────────────────────────

#[derive(SimpleObject)]
pub struct QuirkGql {
    pub slug: String,
    pub name: String,
    pub is_positive: bool,
    pub description: Option<String>,
}

// ── Location ───────────────────────────────────────────────────────────────

#[derive(SimpleObject)]
pub struct LocationGql {
    pub location: String,
    pub armor_points: Option<i32>,
    pub rear_armor: Option<i32>,
    pub structure_points: Option<i32>,
}

// ── Loadout Entry ──────────────────────────────────────────────────────────

#[derive(SimpleObject)]
pub struct LoadoutEntryGql {
    pub equipment_slug: String,
    pub equipment_name: String,
    pub location: Option<String>,
    pub quantity: i32,
    pub is_rear_facing: bool,
    pub notes: Option<String>,
}

// ── Unit Chassis ───────────────────────────────────────────────────────────

pub struct UnitChassisGql(pub DbUnitChassis);

#[Object]
impl UnitChassisGql {
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    async fn slug(&self) -> &str {
        &self.0.slug
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn unit_type(&self) -> &str {
        &self.0.unit_type
    }

    async fn tech_base(&self) -> &str {
        &self.0.tech_base
    }

    async fn tonnage(&self) -> f64 {
        self.0.tonnage.to_f64().unwrap_or(0.0)
    }

    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }

    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

    #[graphql(complexity = 5)]
    async fn variants(&self, ctx: &Context<'_>) -> Result<Vec<UnitGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = sqlx::query_as!(
            DbUnit,
            r#"SELECT u.id, u.slug, u.chassis_id, u.variant, u.full_name,
                      u.tech_base::text AS "tech_base!", u.rules_level::text AS "rules_level!",
                      u.tonnage, u.bv, u.cost, u.intro_year, u.extinction_year,
                      u.reintro_year, u.source_book, u.description, NULL::bigint AS total_count
               FROM units u WHERE u.chassis_id = $1 ORDER BY u.variant"#,
            self.0.id
        )
        .fetch_all(&state.pool)
        .await?;
        Ok(rows.into_iter().map(UnitGql).collect())
    }
}

// ── Unit ───────────────────────────────────────────────────────────────────

pub struct UnitGql(pub DbUnit);

#[Object]
impl UnitGql {
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    async fn slug(&self) -> &str {
        &self.0.slug
    }

    async fn variant(&self) -> &str {
        &self.0.variant
    }

    async fn full_name(&self) -> &str {
        &self.0.full_name
    }

    async fn tech_base(&self) -> &str {
        &self.0.tech_base
    }

    async fn rules_level(&self) -> &str {
        &self.0.rules_level
    }

    async fn tonnage(&self) -> f64 {
        self.0.tonnage.to_f64().unwrap_or(0.0)
    }

    async fn bv(&self) -> Option<i32> {
        self.0.bv
    }

    async fn cost(&self) -> Option<i64> {
        self.0.cost
    }

    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }

    async fn extinction_year(&self) -> Option<i32> {
        self.0.extinction_year
    }

    async fn reintro_year(&self) -> Option<i32> {
        self.0.reintro_year
    }

    async fn source_book(&self) -> Option<&str> {
        self.0.source_book.as_deref()
    }

    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

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

    /// Stub — populated in Milestone B
    async fn mech_data(&self) -> Option<bool> {
        None
    }

    /// Stub — populated in Milestone B
    async fn vehicle_data(&self) -> Option<bool> {
        None
    }
}

#[derive(SimpleObject)]
pub struct AvailabilityGql {
    pub faction_slug: String,
    pub faction_name: String,
    pub era_slug: String,
    pub era_name: String,
    pub availability_code: Option<String>,
    pub notes: Option<String>,
}
