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

/// A single edge in a paginated unit list, pairing a cursor with its unit node.
#[derive(SimpleObject)]
pub struct UnitEdge {
    /// Opaque pagination cursor for this edge. Can be used as the "after" parameter.
    pub cursor: String,
    /// The unit at this position in the result set.
    pub node: UnitGql,
}

pub struct UnitConnection {
    pub edges: Vec<UnitEdge>,
    pub page_info: PageInfo,
}

/// Relay-style paginated list of units with cursor-based navigation.
#[Object]
impl UnitConnection {
    /// List of unit edges in this page.
    async fn edges(&self) -> &[UnitEdge] {
        &self.edges
    }
    /// Pagination metadata including cursors and total count.
    async fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

/// A single edge in a paginated equipment list, pairing a cursor with its equipment node.
#[derive(SimpleObject)]
pub struct EquipmentEdge {
    /// Opaque pagination cursor for this edge. Can be used as the "after" parameter.
    pub cursor: String,
    /// The equipment item at this position in the result set.
    pub node: EquipmentGql,
}

pub struct EquipmentConnection {
    pub edges: Vec<EquipmentEdge>,
    pub page_info: PageInfo,
}

/// Relay-style paginated list of equipment items with cursor-based navigation.
#[Object]
impl EquipmentConnection {
    /// List of equipment edges in this page.
    async fn edges(&self) -> &[EquipmentEdge] {
        &self.edges
    }
    /// Pagination metadata including cursors and total count.
    async fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

// ── Query Root ─────────────────────────────────────────────────────────────

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    // ── Metadata ────────────────────────────────────────────────────────────

    /// Current dataset metadata (MegaMek version, schema version, release date).
    async fn metadata(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<DatasetMetadataGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = metadata::get_latest(&state.pool).await?;
        Ok(row.map(DatasetMetadataGql))
    }

    /// All available game rulesets (Introductory, Standard, Advanced, etc.).
    async fn rulesets(&self, ctx: &Context<'_>) -> Result<Vec<RulesetGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = metadata::list_rulesets(&state.pool).await?;
        Ok(rows.into_iter().map(RulesetGql::from).collect())
    }

    // ── Units ───────────────────────────────────────────────────────────────

    /// Look up a single unit variant by its slug.
    async fn unit(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Lowercase, hyphen-separated unit identifier (e.g. \"atlas-as7-d\").")] slug: String,
    ) -> Result<Option<UnitGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = units::get_by_slug(&state.pool, &slug).await?;
        Ok(row.map(UnitGql))
    }

    /// Batch lookup of units by their slugs. Returns units in the order of the input slugs.
    async fn units_by_ids(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Array of lowercase, hyphen-separated unit slugs. Maximum 24 slugs per call.")] slugs: Vec<String>,
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

    /// Paginated, filterable search across all unit variants. Returns a cursor-based connection.
    async fn units(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Items per page. Default 20, max 100.")] first: Option<i32>,
        #[graphql(desc = "Opaque cursor from a previous pageInfo.endCursor. Omit for the first page.")] after: Option<String>,
        #[graphql(desc = "Case-insensitive substring match against the unit's full name.")] name_search: Option<String>,
        #[graphql(desc = "Filter by technology base. One of: inner_sphere, clan, mixed, primitive.")] tech_base: Option<String>,
        #[graphql(desc = "Filter by rules level. One of: introductory, standard, advanced, experimental, unofficial.")] rules_level: Option<String>,
        #[graphql(desc = "Minimum tonnage filter (inclusive). Weight in metric tons.")] tonnage_min: Option<f64>,
        #[graphql(desc = "Maximum tonnage filter (inclusive). Weight in metric tons.")] tonnage_max: Option<f64>,
        #[graphql(desc = "Filter to units available to this faction. Lowercase, hyphen-separated slug (e.g. \"clan-wolf\").")] faction_slug: Option<String>,
        #[graphql(desc = "Filter to units available in this era. Lowercase, hyphen-separated slug (e.g. \"clan-invasion\").")] era_slug: Option<String>,
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

    /// Look up a single chassis by its slug. A chassis groups all variants of a unit design.
    async fn chassis(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Lowercase, hyphen-separated chassis identifier (e.g. \"atlas\").")] slug: String,
    ) -> Result<Option<UnitChassisGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = units::get_chassis_by_slug(&state.pool, &slug).await?;
        Ok(row.map(UnitChassisGql))
    }

    /// List all chassis, optionally filtered by unit type and/or technology base.
    async fn all_chassis(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Filter by unit type (e.g. \"BattleMech\", \"Vehicle\", \"AeroSpaceFighter\").")] unit_type: Option<String>,
        #[graphql(desc = "Filter by technology base. One of: inner_sphere, clan, mixed, primitive.")] tech_base: Option<String>,
    ) -> Result<Vec<UnitChassisGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows =
            units::list_chassis(&state.pool, unit_type.as_deref(), tech_base.as_deref()).await?;
        Ok(rows.into_iter().map(UnitChassisGql).collect())
    }

    // ── Equipment ───────────────────────────────────────────────────────────

    /// Look up a single equipment item by its slug.
    async fn equipment(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Lowercase, hyphen-separated equipment identifier (e.g. \"medium-laser\").")] slug: String,
    ) -> Result<Option<EquipmentGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = equipment::get_by_slug(&state.pool, &slug).await?;
        Ok(row.map(EquipmentGql))
    }

    /// Paginated, filterable search across all equipment items. Returns a cursor-based connection.
    async fn all_equipment(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Items per page. Default 20, max 100.")] first: Option<i32>,
        #[graphql(desc = "Opaque cursor from a previous pageInfo.endCursor. Omit for the first page.")] after: Option<String>,
        #[graphql(desc = "Case-insensitive substring match against the equipment name.")] name_search: Option<String>,
        #[graphql(desc = "Filter by equipment category in snake_case. One of: energy_weapon, ballistic_weapon, missile_weapon, ammo, physical_weapon, equipment, armor, structure, engine, targeting_system, myomer, heat_sink, jump_jet, communications.")] category: Option<String>,
        #[graphql(desc = "Filter by technology base. One of: inner_sphere, clan, mixed, primitive.")] tech_base: Option<String>,
        #[graphql(desc = "Filter by rules level. One of: introductory, standard, advanced, experimental, unofficial.")] rules_level: Option<String>,
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

    /// Look up a single faction by its slug.
    async fn faction(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Lowercase, hyphen-separated faction identifier (e.g. \"clan-wolf\").")] slug: String,
    ) -> Result<Option<FactionGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = factions::get_by_slug(&state.pool, &slug).await?;
        Ok(row.map(FactionGql))
    }

    /// List all factions, optionally filtered by type, clan status, or era.
    async fn all_factions(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Filter by faction classification. One of: great_house, clan, periphery, mercenary, other.")] faction_type: Option<String>,
        #[graphql(desc = "Filter by clan status. True returns only Clans; false returns only non-Clans.")] is_clan: Option<bool>,
        #[graphql(desc = "Filter to factions active in this era. Lowercase, hyphen-separated slug (e.g. \"clan-invasion\").")] era_slug: Option<String>,
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

    /// Look up a single era by its slug.
    async fn era(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Lowercase, hyphen-separated era identifier (e.g. \"clan-invasion\").")] slug: String,
    ) -> Result<Option<EraGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let row = eras::get_by_slug(&state.pool, &slug).await?;
        Ok(row.map(EraGql))
    }

    /// List all eras in chronological order.
    async fn all_eras(&self, ctx: &Context<'_>) -> Result<Vec<EraGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = eras::list_all(&state.pool).await?;
        Ok(rows.into_iter().map(EraGql).collect())
    }

    /// Find eras that contain the given in-universe year. May return multiple eras if they overlap.
    async fn era_by_year(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "In-universe BattleTech year to look up (e.g. 3055). Not a real-world date.")] year: i32,
    ) -> Result<Vec<EraGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = eras::get_by_year(&state.pool, year).await?;
        Ok(rows.into_iter().map(EraGql).collect())
    }
}
