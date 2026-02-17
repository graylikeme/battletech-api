use async_graphql::{Context, Object, ID};

use crate::{db::models::DbFaction, error::AppError, state::AppState};

use super::era::EraGql;

pub struct FactionGql(pub DbFaction);

#[Object]
impl FactionGql {
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    async fn slug(&self) -> &str {
        &self.0.slug
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn short_name(&self) -> Option<&str> {
        self.0.short_name.as_deref()
    }

    async fn faction_type(&self) -> &str {
        &self.0.faction_type
    }

    async fn is_clan(&self) -> bool {
        self.0.is_clan
    }

    async fn founding_year(&self) -> Option<i32> {
        self.0.founding_year
    }

    async fn dissolution_year(&self) -> Option<i32> {
        self.0.dissolution_year
    }

    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

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
