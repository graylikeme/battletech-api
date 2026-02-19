use std::collections::HashMap;

use async_graphql::dataloader::Loader;

use crate::db::{
    models::{DbEquipment, DbMechData},
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
