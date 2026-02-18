use async_graphql::{Context, Object, ID};

use crate::{db::models::DbFaction, error::AppError, state::AppState};

use super::era::EraGql;

pub struct FactionGql(pub DbFaction);

/// A BattleTech faction (e.g. Great House, Clan, mercenary company, or periphery state).
#[Object]
impl FactionGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "clan-wolf", "house-steiner").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Full faction name (e.g. "Clan Wolf", "Lyran Commonwealth").
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// Abbreviated faction name (e.g. "CW", "LC"). Null if no abbreviation exists.
    async fn short_name(&self) -> Option<&str> {
        self.0.short_name.as_deref()
    }

    /// Faction classification. One of: great_house, clan, periphery, mercenary, other.
    async fn faction_type(&self) -> &str {
        &self.0.faction_type
    }

    /// True if this faction is a Clan; false otherwise.
    async fn is_clan(&self) -> bool {
        self.0.is_clan
    }

    /// In-universe BattleTech year when the faction was founded. Not a real-world date.
    async fn founding_year(&self) -> Option<i32> {
        self.0.founding_year
    }

    /// In-universe BattleTech year when the faction was dissolved. Null if still active.
    async fn dissolution_year(&self) -> Option<i32> {
        self.0.dissolution_year
    }

    /// Flavor text or lore description of the faction.
    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

    /// Eras during which this faction was active, ordered by start year.
    #[graphql(complexity = 5)]
    async fn eras(&self, ctx: &Context<'_>) -> Result<Vec<EraGql>, AppError> {
        let state = ctx.data::<AppState>().unwrap();
        let rows = sqlx::query_as!(
            crate::db::models::DbEra,
            r#"SELECT e.id, e.slug, e.name, e.start_year, e.end_year, e.description
               FROM eras e
               JOIN faction_eras fe ON fe.era_id = e.id
               WHERE fe.faction_id = $1
               ORDER BY e.start_year"#,
            self.0.id
        )
        .fetch_all(&state.pool)
        .await?;
        Ok(rows.into_iter().map(EraGql).collect())
    }
}
