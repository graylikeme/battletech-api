use async_graphql::{Object, ID};

use crate::db::models::DbEra;

pub struct EraGql(pub DbEra);

#[Object]
impl EraGql {
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    async fn slug(&self) -> &str {
        &self.0.slug
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn start_year(&self) -> i32 {
        self.0.start_year
    }

    async fn end_year(&self) -> Option<i32> {
        self.0.end_year
    }

    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }
}
