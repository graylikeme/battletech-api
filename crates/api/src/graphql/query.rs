use async_graphql::{Context, Object, SimpleObject};

use crate::{
    db::{equipment, eras, factions, metadata, units},
    error::AppError,
    graphql::{
        pagination::{decode_cursor, encode_cursor, PageInfo},
        types::{
            equipment::EquipmentGql,
            era::EraGql,
            faction::FactionGql,
            metadata::{DatasetMetadataGql, RulesetGql},
            unit::{UnitChassisGql, UnitGql},
        },
    },
    state::AppState,
};

// ── Connection types ───────────────────────────────────────────────────────

#[derive(SimpleObject)]
pub struct UnitEdge {
    pub cursor: String,
    pub node: UnitGql,
}

pub struct UnitConnection {
    pub edges: Vec<UnitEdge>,
    pub page_info: PageInfo,
}

#[Object]
impl UnitConnection {
    async fn edges(&self) -> &[UnitEdge] {
        &self.edges
    }
    async fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

#[derive(SimpleObject)]
pub struct EquipmentEdge {
    pub cursor: String,
    pub node: EquipmentGql,
}

pub struct EquipmentConnection {
    pub edges: Vec<EquipmentEdge>,
    pub page_info: PageInfo,
}

#[Object]
impl EquipmentConnection {
    async fn edges(&self) -> &[EquipmentEdge] {
        &self.edges
    }
    async fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

// ── Query Root ─────────────────────────────────────────────────────────────

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    // ── Metadata ────────────────────────────────────────────────────────────

    async fn metadata(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<DatasetMetadataGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = metadata::get_latest(&state.pool).await?;
        Ok(row.map(DatasetMetadataGql))
    }

    async fn rulesets(&self, ctx: &Context<'_>) -> Result<Vec<RulesetGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = metadata::list_rulesets(&state.pool).await?;
        Ok(rows.into_iter().map(RulesetGql::from).collect())
    }

    // ── Units ───────────────────────────────────────────────────────────────

    async fn unit(&self, ctx: &Context<'_>, slug: String) -> Result<Option<UnitGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = units::get_by_slug(&state.pool, &slug).await?;
        Ok(row.map(UnitGql))
    }

    async fn units_by_ids(
        &self,
        ctx: &Context<'_>,
        slugs: Vec<String>,
    ) -> Result<Vec<UnitGql>, AppError> {
        if slugs.len() > 24 {
            return Err(AppError::Validation(
                "unitsByIds accepts at most 24 slugs".into(),
            ));
        }
        let state = ctx.data::<AppState>().unwrap();
        let rows = units::get_by_ids(&state.pool, &slugs).await?;
        Ok(rows.into_iter().map(UnitGql).collect())
    }

    async fn units(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
        name_search: Option<String>,
        tech_base: Option<String>,
        rules_level: Option<String>,
        tonnage_min: Option<f64>,
        tonnage_max: Option<f64>,
        faction_slug: Option<String>,
        era_slug: Option<String>,
    ) -> Result<UnitConnection, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let first = first.unwrap_or(20).clamp(1, 100) as i64;
        let after_id = after
            .as_deref()
            .and_then(decode_cursor)
            .map(|(_, id)| id);

        let filter = units::UnitFilter {
            name_search: name_search.as_deref(),
            tech_base: tech_base.as_deref(),
            rules_level: rules_level.as_deref(),
            tonnage_min,
            tonnage_max,
            faction_slug: faction_slug.as_deref(),
            era_slug: era_slug.as_deref(),
        };

        let (rows, total_count, has_next) =
            units::search(&state.pool, filter, first, after_id).await?;

        let edges: Vec<UnitEdge> = rows
            .into_iter()
            .map(|u| {
                let cursor = encode_cursor(&u.full_name, u.id);
                UnitEdge {
                    cursor,
                    node: UnitGql(u),
                }
            })
            .collect();

        let start_cursor = edges.first().map(|e| e.cursor.clone());
        let end_cursor = edges.last().map(|e| e.cursor.clone());

        Ok(UnitConnection {
            edges,
            page_info: PageInfo {
                has_next_page: has_next,
                has_previous_page: after_id.is_some(),
                start_cursor,
                end_cursor,
                total_count,
            },
        })
    }

    // ── Chassis ─────────────────────────────────────────────────────────────

    async fn chassis(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> Result<Option<UnitChassisGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = units::get_chassis_by_slug(&state.pool, &slug).await?;
        Ok(row.map(UnitChassisGql))
    }

    async fn all_chassis(
        &self,
        ctx: &Context<'_>,
        unit_type: Option<String>,
        tech_base: Option<String>,
    ) -> Result<Vec<UnitChassisGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows =
            units::list_chassis(&state.pool, unit_type.as_deref(), tech_base.as_deref()).await?;
        Ok(rows.into_iter().map(UnitChassisGql).collect())
    }

    // ── Equipment ───────────────────────────────────────────────────────────

    async fn equipment(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> Result<Option<EquipmentGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = equipment::get_by_slug(&state.pool, &slug).await?;
        Ok(row.map(EquipmentGql))
    }

    async fn all_equipment(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
        name_search: Option<String>,
        category: Option<String>,
        tech_base: Option<String>,
        rules_level: Option<String>,
    ) -> Result<EquipmentConnection, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let first = first.unwrap_or(20).clamp(1, 100) as i64;
        let after_id = after
            .as_deref()
            .and_then(decode_cursor)
            .map(|(_, id)| id);

        let (rows, total_count, has_next) = equipment::search(
            &state.pool,
            name_search.as_deref(),
            category.as_deref(),
            tech_base.as_deref(),
            rules_level.as_deref(),
            first,
            after_id,
        )
        .await?;

        let edges: Vec<EquipmentEdge> = rows
            .into_iter()
            .map(|e| {
                let cursor = encode_cursor(&e.name, e.id);
                EquipmentEdge {
                    cursor,
                    node: EquipmentGql(e),
                }
            })
            .collect();

        let start_cursor = edges.first().map(|e| e.cursor.clone());
        let end_cursor = edges.last().map(|e| e.cursor.clone());

        Ok(EquipmentConnection {
            edges,
            page_info: PageInfo {
                has_next_page: has_next,
                has_previous_page: after_id.is_some(),
                start_cursor,
                end_cursor,
                total_count,
            },
        })
    }

    // ── Factions ────────────────────────────────────────────────────────────

    async fn faction(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> Result<Option<FactionGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = factions::get_by_slug(&state.pool, &slug).await?;
        Ok(row.map(FactionGql))
    }

    async fn all_factions(
        &self,
        ctx: &Context<'_>,
        faction_type: Option<String>,
        is_clan: Option<bool>,
        era_slug: Option<String>,
    ) -> Result<Vec<FactionGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = factions::list(
            &state.pool,
            faction_type.as_deref(),
            is_clan,
            era_slug.as_deref(),
        )
        .await?;
        Ok(rows.into_iter().map(FactionGql).collect())
    }

    // ── Eras ─────────────────────────────────────────────────────────────────

    async fn era(&self, ctx: &Context<'_>, slug: String) -> Result<Option<EraGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = eras::get_by_slug(&state.pool, &slug).await?;
        Ok(row.map(EraGql))
    }

    async fn all_eras(&self, ctx: &Context<'_>) -> Result<Vec<EraGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = eras::list_all(&state.pool).await?;
        Ok(rows.into_iter().map(EraGql).collect())
    }

    async fn era_by_year(
        &self,
        ctx: &Context<'_>,
        year: i32,
    ) -> Result<Vec<EraGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = eras::get_by_year(&state.pool, year).await?;
        Ok(rows.into_iter().map(EraGql).collect())
    }
}
