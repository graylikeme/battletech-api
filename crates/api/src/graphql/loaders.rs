use std::collections::HashMap;

use async_graphql::dataloader::Loader;

use crate::db::{
    models::{
        DbArmorType, DbCockpitType, DbEngineType, DbEquipment, DbGyroType, DbHeatsinkType,
        DbMechData, DbMyomerType, DbStructureType,
    },
    units,
};

// ── MechData Loader ──────────────────────────────────────────────────────────

pub struct MechDataLoader {
    pub pool: sqlx::PgPool,
}

impl Loader<i32> for MechDataLoader {
    type Value = DbMechData;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, DbMechData>, async_graphql::Error> {
        let rows = units::get_mech_data_batch(&self.pool, keys).await?;
        Ok(rows.into_iter().map(|r| (r.unit_id, r)).collect())
    }
}

// ── Ammo-For Loader (ammo → weapon) ─────────────────────────────────────────

pub struct AmmoForLoader {
    pub pool: sqlx::PgPool,
}

impl Loader<i32> for AmmoForLoader {
    type Value = DbEquipment;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[i32],
    ) -> Result<HashMap<i32, DbEquipment>, async_graphql::Error> {
        let rows = sqlx::query_as::<_, DbEquipment>(
            r#"SELECT id, slug, name,
                      category::text AS category, tech_base::text AS tech_base,
                      rules_level::text AS rules_level,
                      tonnage, crits, damage, heat,
                      range_min, range_short, range_medium, range_long, bv, intro_year,
                      source_book, description,
                      observed_locations, ammo_for_id, stats_source,
                      NULL::bigint AS total_count
               FROM equipment WHERE id = ANY($1)"#,
        )
        .bind(keys)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.id, r)).collect())
    }
}

// ── Ammo-Types Loader (weapon → ammo list) ──────────────────────────────────

pub struct AmmoTypesLoader {
    pub pool: sqlx::PgPool,
}

impl Loader<i32> for AmmoTypesLoader {
    type Value = Vec<DbEquipment>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[i32],
    ) -> Result<HashMap<i32, Vec<DbEquipment>>, async_graphql::Error> {
        let rows = sqlx::query_as::<_, DbEquipment>(
            r#"SELECT id, slug, name,
                      category::text AS category, tech_base::text AS tech_base,
                      rules_level::text AS rules_level,
                      tonnage, crits, damage, heat,
                      range_min, range_short, range_medium, range_long, bv, intro_year,
                      source_book, description,
                      observed_locations, ammo_for_id, stats_source,
                      NULL::bigint AS total_count
               FROM equipment WHERE ammo_for_id = ANY($1)
               ORDER BY name"#,
        )
        .bind(keys)
        .fetch_all(&self.pool)
        .await?;

        let mut map: HashMap<i32, Vec<DbEquipment>> = HashMap::new();
        for row in rows {
            if let Some(weapon_id) = row.ammo_for_id {
                map.entry(weapon_id).or_default().push(row);
            }
        }
        Ok(map)
    }
}

// ── Component Type Loaders (for MechData FK resolution) ──────────────────────

pub struct EngineTypeLoader {
    pub pool: sqlx::PgPool,
}

impl Loader<i32> for EngineTypeLoader {
    type Value = DbEngineType;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, DbEngineType>, async_graphql::Error> {
        let rows = sqlx::query_as::<_, DbEngineType>(
            r#"SELECT id, slug, name, tech_base::text AS tech_base,
                      rules_level::text AS rules_level,
                      weight_multiplier, ct_crits, st_crits, intro_year
               FROM engine_types WHERE id = ANY($1)"#,
        )
        .bind(keys)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.id, r)).collect())
    }
}

pub struct ArmorTypeLoader {
    pub pool: sqlx::PgPool,
}

impl Loader<i32> for ArmorTypeLoader {
    type Value = DbArmorType;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, DbArmorType>, async_graphql::Error> {
        let rows = sqlx::query_as::<_, DbArmorType>(
            r#"SELECT id, slug, name, tech_base::text AS tech_base,
                      rules_level::text AS rules_level,
                      points_per_ton, crits, intro_year
               FROM armor_types WHERE id = ANY($1)"#,
        )
        .bind(keys)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.id, r)).collect())
    }
}

pub struct StructureTypeLoader {
    pub pool: sqlx::PgPool,
}

impl Loader<i32> for StructureTypeLoader {
    type Value = DbStructureType;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, DbStructureType>, async_graphql::Error> {
        let rows = sqlx::query_as::<_, DbStructureType>(
            r#"SELECT id, slug, name, tech_base::text AS tech_base,
                      rules_level::text AS rules_level,
                      weight_fraction, crits, intro_year
               FROM structure_types WHERE id = ANY($1)"#,
        )
        .bind(keys)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.id, r)).collect())
    }
}

pub struct HeatsinkTypeLoader {
    pub pool: sqlx::PgPool,
}

impl Loader<i32> for HeatsinkTypeLoader {
    type Value = DbHeatsinkType;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, DbHeatsinkType>, async_graphql::Error> {
        let rows = sqlx::query_as::<_, DbHeatsinkType>(
            r#"SELECT id, slug, name, tech_base::text AS tech_base,
                      rules_level::text AS rules_level,
                      dissipation, crits, weight, intro_year
               FROM heatsink_types WHERE id = ANY($1)"#,
        )
        .bind(keys)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.id, r)).collect())
    }
}

pub struct GyroTypeLoader {
    pub pool: sqlx::PgPool,
}

impl Loader<i32> for GyroTypeLoader {
    type Value = DbGyroType;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, DbGyroType>, async_graphql::Error> {
        let rows = sqlx::query_as::<_, DbGyroType>(
            r#"SELECT id, slug, name, tech_base::text AS tech_base,
                      rules_level::text AS rules_level,
                      weight_multiplier, crits, is_superheavy_only, intro_year
               FROM gyro_types WHERE id = ANY($1)"#,
        )
        .bind(keys)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.id, r)).collect())
    }
}

pub struct CockpitTypeLoader {
    pub pool: sqlx::PgPool,
}

impl Loader<i32> for CockpitTypeLoader {
    type Value = DbCockpitType;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, DbCockpitType>, async_graphql::Error> {
        let rows = sqlx::query_as::<_, DbCockpitType>(
            r#"SELECT id, slug, name, tech_base::text AS tech_base,
                      rules_level::text AS rules_level,
                      weight, crits, intro_year
               FROM cockpit_types WHERE id = ANY($1)"#,
        )
        .bind(keys)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.id, r)).collect())
    }
}

pub struct MyomerTypeLoader {
    pub pool: sqlx::PgPool,
}

impl Loader<i32> for MyomerTypeLoader {
    type Value = DbMyomerType;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, DbMyomerType>, async_graphql::Error> {
        let rows = sqlx::query_as::<_, DbMyomerType>(
            r#"SELECT id, slug, name, tech_base::text AS tech_base,
                      rules_level::text AS rules_level,
                      intro_year, properties
               FROM myomer_types WHERE id = ANY($1)"#,
        )
        .bind(keys)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.id, r)).collect())
    }
}
