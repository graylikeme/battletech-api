use sqlx::PgPool;

use crate::{db::models::DbEquipment, error::AppError};

pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbEquipment>, AppError> {
    let row = sqlx::query_as::<_, DbEquipment>(
        r#"SELECT id, slug, name,
                  category::text AS category, tech_base::text AS tech_base,
                  rules_level::text AS rules_level,
                  tonnage, crits, damage, heat,
                  range_min, range_short, range_medium, range_long, bv, intro_year,
                  source_book, description,
                  observed_locations, ammo_for_id, stats_source, shots_per_ton,
                  NULL::bigint AS total_count
           FROM equipment WHERE slug = $1"#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub struct EquipmentFilter<'a> {
    pub name_search: Option<&'a str>,
    pub category: Option<&'a str>,
    pub tech_base: Option<&'a str>,
    pub rules_level: Option<&'a str>,
    pub max_tonnage: Option<f64>,
    pub max_crits: Option<i32>,
    pub observed_location: Option<&'a str>,
    pub ammo_for_slug: Option<&'a str>,
}

pub async fn search(
    pool: &PgPool,
    filter: EquipmentFilter<'_>,
    first: i64,
    after_cursor: Option<(&str, i32)>,
) -> Result<(Vec<DbEquipment>, i64, bool), AppError> {
    // Pre-resolve ammo_for_slug to ID
    let resolved_ammo_for_id = if let Some(slug) = filter.ammo_for_slug {
        sqlx::query_scalar::<_, i32>("SELECT id FROM equipment WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    let has_cursor = after_cursor.is_some();

    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new("");

    if has_cursor {
        builder.push("WITH filtered AS MATERIALIZED (");
    }

    builder.push(
        r#"SELECT id, slug, name, category::text AS category, tech_base::text AS tech_base,
                  rules_level::text AS rules_level, tonnage, crits, damage, heat,
                  range_min, range_short, range_medium, range_long, bv, intro_year,
                  source_book, description,
                  observed_locations, ammo_for_id, stats_source, shots_per_ton,
                  COUNT(*) OVER() AS total_count
           FROM equipment WHERE TRUE"#,
    );

    if let Some(n) = filter.name_search {
        builder.push(" AND name ILIKE '%' || ");
        builder.push_bind(n);
        builder.push(" || '%'");
    }
    if let Some(c) = filter.category {
        builder.push(" AND category::text = ");
        builder.push_bind(c);
    }
    if let Some(tb) = filter.tech_base {
        builder.push(" AND tech_base::text = ");
        builder.push_bind(tb);
    }
    if let Some(rl) = filter.rules_level {
        builder.push(" AND rules_level <= ");
        builder.push_bind(rl);
        builder.push("::rules_level_enum");
    }
    if let Some(max_t) = filter.max_tonnage {
        builder.push(" AND tonnage IS NOT NULL AND tonnage <= ");
        builder.push_bind(rust_decimal::Decimal::try_from(max_t).unwrap_or_default());
    }
    if let Some(max_c) = filter.max_crits {
        builder.push(" AND crits IS NOT NULL AND crits <= ");
        builder.push_bind(max_c);
    }
    if let Some(loc) = filter.observed_location {
        builder.push(" AND observed_locations @> ARRAY[");
        builder.push_bind(loc);
        builder.push("]");
    }
    if let Some(weapon_id) = resolved_ammo_for_id {
        builder.push(" AND ammo_for_id = ");
        builder.push_bind(weapon_id);
    }

    if let Some((sort_val, after_id)) = after_cursor {
        builder.push(") SELECT * FROM filtered WHERE (name > ");
        builder.push_bind(sort_val);
        builder.push(" OR (name = ");
        builder.push_bind(sort_val);
        builder.push(" AND id > ");
        builder.push_bind(after_id);
        builder.push("))");
    }

    builder.push(" ORDER BY name, id LIMIT ");
    builder.push_bind(first + 1);

    let mut rows = builder
        .build_query_as::<DbEquipment>()
        .fetch_all(pool)
        .await?;

    let total_count = rows.first().and_then(|r| r.total_count).unwrap_or(0);
    let has_next = rows.len() as i64 > first;
    rows.truncate(first as usize);

    Ok((rows, total_count, has_next))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Insert equipment in reverse-alpha order so IDs don't match name sort.
    async fn seed_equipment(pool: &PgPool) {
        let items = [
            ("zzz-laser", "ZZZ Laser"),
            ("death-ray", "Death Ray"),
            ("medium-laser", "Medium Laser"),
            ("beam-rifle", "Beam Rifle"),
            ("alpha-cannon", "Alpha Cannon"),
        ];
        for (slug, name) in &items {
            sqlx::query(
                "INSERT INTO equipment (slug, name, category, tech_base, rules_level)
                 VALUES ($1, $2, 'energy_weapon', 'inner_sphere', 'standard')",
            )
            .bind(slug)
            .bind(name)
            .execute(pool)
            .await
            .unwrap();
        }
    }

    fn empty_filter() -> EquipmentFilter<'static> {
        EquipmentFilter {
            name_search: None,
            category: None,
            tech_base: None,
            rules_level: None,
            max_tonnage: None,
            max_crits: None,
            observed_location: None,
            ammo_for_slug: None,
        }
    }

    /// Regression: same keyset pagination bug as units — IDs vs name order.
    #[sqlx::test(migrations = "../../migrations")]
    async fn keyset_pagination_no_duplicates_or_gaps(pool: PgPool) {
        seed_equipment(&pool).await;

        let mut all_names: Vec<String> = vec![];
        let mut cursor: Option<(String, i32)> = None;

        loop {
            let cursor_ref = cursor.as_ref().map(|(s, id)| (s.as_str(), *id));
            let (rows, total, has_next) =
                search(&pool, empty_filter(), 2, cursor_ref).await.unwrap();

            assert_eq!(total, 5, "totalCount must be stable across all pages");

            for row in &rows {
                all_names.push(row.name.clone());
            }

            if !has_next {
                break;
            }
            let last = rows.last().unwrap();
            cursor = Some((last.name.clone(), last.id));
        }

        assert_eq!(
            all_names,
            ["Alpha Cannon", "Beam Rifle", "Death Ray", "Medium Laser", "ZZZ Laser"],
            "items must appear in alphabetical order with no duplicates or gaps"
        );
    }

    /// Regression: totalCount must not shrink on page 2.
    #[sqlx::test(migrations = "../../migrations")]
    async fn total_count_stable_across_pages(pool: PgPool) {
        seed_equipment(&pool).await;

        let (page1, total1, _) = search(&pool, empty_filter(), 2, None).await.unwrap();
        let last = page1.last().unwrap();
        let cursor = (last.name.as_str(), last.id);

        let (_, total2, _) = search(&pool, empty_filter(), 2, Some(cursor)).await.unwrap();

        assert_eq!(total1, total2, "totalCount must not change between pages");
    }
}
