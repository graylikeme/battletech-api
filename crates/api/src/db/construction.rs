use sqlx::PgPool;

use crate::{
    db::models::{
        DbArmorType, DbCockpitType, DbEngineType, DbEngineWeight, DbGyroType, DbHeatsinkType,
        DbInternalStructure, DbMyomerType, DbStructureType,
    },
    error::AppError,
};

pub async fn list_engine_types(
    pool: &PgPool,
    tech_base: Option<&str>,
    rules_level: Option<&str>,
) -> Result<Vec<DbEngineType>, AppError> {
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT id, slug, name, tech_base::text AS tech_base,
                  rules_level::text AS rules_level,
                  weight_multiplier, ct_crits, st_crits, intro_year
           FROM engine_types WHERE TRUE"#,
    );
    if let Some(tb) = tech_base {
        builder.push(" AND tech_base::text = ");
        builder.push_bind(tb);
    }
    if let Some(rl) = rules_level {
        builder.push(" AND rules_level::text = ");
        builder.push_bind(rl);
    }
    builder.push(" ORDER BY name");
    Ok(builder.build_query_as::<DbEngineType>().fetch_all(pool).await?)
}

pub async fn list_armor_types(
    pool: &PgPool,
    tech_base: Option<&str>,
    rules_level: Option<&str>,
) -> Result<Vec<DbArmorType>, AppError> {
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT id, slug, name, tech_base::text AS tech_base,
                  rules_level::text AS rules_level,
                  points_per_ton, crits, intro_year
           FROM armor_types WHERE TRUE"#,
    );
    if let Some(tb) = tech_base {
        builder.push(" AND tech_base::text = ");
        builder.push_bind(tb);
    }
    if let Some(rl) = rules_level {
        builder.push(" AND rules_level::text = ");
        builder.push_bind(rl);
    }
    builder.push(" ORDER BY name");
    Ok(builder.build_query_as::<DbArmorType>().fetch_all(pool).await?)
}

pub async fn list_structure_types(
    pool: &PgPool,
    tech_base: Option<&str>,
    rules_level: Option<&str>,
) -> Result<Vec<DbStructureType>, AppError> {
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT id, slug, name, tech_base::text AS tech_base,
                  rules_level::text AS rules_level,
                  weight_fraction, crits, intro_year
           FROM structure_types WHERE TRUE"#,
    );
    if let Some(tb) = tech_base {
        builder.push(" AND tech_base::text = ");
        builder.push_bind(tb);
    }
    if let Some(rl) = rules_level {
        builder.push(" AND rules_level::text = ");
        builder.push_bind(rl);
    }
    builder.push(" ORDER BY name");
    Ok(builder.build_query_as::<DbStructureType>().fetch_all(pool).await?)
}

pub async fn list_heatsink_types(
    pool: &PgPool,
    tech_base: Option<&str>,
    rules_level: Option<&str>,
) -> Result<Vec<DbHeatsinkType>, AppError> {
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT id, slug, name, tech_base::text AS tech_base,
                  rules_level::text AS rules_level,
                  dissipation, crits, weight, intro_year
           FROM heatsink_types WHERE TRUE"#,
    );
    if let Some(tb) = tech_base {
        builder.push(" AND tech_base::text = ");
        builder.push_bind(tb);
    }
    if let Some(rl) = rules_level {
        builder.push(" AND rules_level::text = ");
        builder.push_bind(rl);
    }
    builder.push(" ORDER BY name");
    Ok(builder.build_query_as::<DbHeatsinkType>().fetch_all(pool).await?)
}

pub async fn list_gyro_types(
    pool: &PgPool,
    rules_level: Option<&str>,
) -> Result<Vec<DbGyroType>, AppError> {
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT id, slug, name, tech_base::text AS tech_base,
                  rules_level::text AS rules_level,
                  weight_multiplier, crits, is_superheavy_only, intro_year
           FROM gyro_types WHERE TRUE"#,
    );
    if let Some(rl) = rules_level {
        builder.push(" AND rules_level::text = ");
        builder.push_bind(rl);
    }
    builder.push(" ORDER BY name");
    Ok(builder.build_query_as::<DbGyroType>().fetch_all(pool).await?)
}

pub async fn list_cockpit_types(
    pool: &PgPool,
    rules_level: Option<&str>,
) -> Result<Vec<DbCockpitType>, AppError> {
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT id, slug, name, tech_base::text AS tech_base,
                  rules_level::text AS rules_level,
                  weight, crits, intro_year
           FROM cockpit_types WHERE TRUE"#,
    );
    if let Some(rl) = rules_level {
        builder.push(" AND rules_level::text = ");
        builder.push_bind(rl);
    }
    builder.push(" ORDER BY name");
    Ok(builder.build_query_as::<DbCockpitType>().fetch_all(pool).await?)
}

pub async fn list_myomer_types(
    pool: &PgPool,
    rules_level: Option<&str>,
) -> Result<Vec<DbMyomerType>, AppError> {
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT id, slug, name, tech_base::text AS tech_base,
                  rules_level::text AS rules_level,
                  intro_year, properties
           FROM myomer_types WHERE TRUE"#,
    );
    if let Some(rl) = rules_level {
        builder.push(" AND rules_level::text = ");
        builder.push_bind(rl);
    }
    builder.push(" ORDER BY name");
    Ok(builder.build_query_as::<DbMyomerType>().fetch_all(pool).await?)
}

pub async fn list_engine_weights(
    pool: &PgPool,
    rating: Option<i16>,
) -> Result<Vec<DbEngineWeight>, AppError> {
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        "SELECT rating, standard_weight FROM engine_weight_table WHERE TRUE",
    );
    if let Some(r) = rating {
        builder.push(" AND rating = ");
        builder.push_bind(r);
    }
    builder.push(" ORDER BY rating");
    Ok(builder.build_query_as::<DbEngineWeight>().fetch_all(pool).await?)
}

pub async fn get_internal_structure(
    pool: &PgPool,
    tonnage: i16,
) -> Result<Option<DbInternalStructure>, AppError> {
    let row = sqlx::query_as::<_, DbInternalStructure>(
        "SELECT tonnage, head, center_torso, side_torso, arm, leg
         FROM mech_internal_structure WHERE tonnage = $1",
    )
    .bind(tonnage)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_all_internal_structure(
    pool: &PgPool,
) -> Result<Vec<DbInternalStructure>, AppError> {
    let rows = sqlx::query_as::<_, DbInternalStructure>(
        "SELECT tonnage, head, center_torso, side_torso, arm, leg
         FROM mech_internal_structure ORDER BY tonnage",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
