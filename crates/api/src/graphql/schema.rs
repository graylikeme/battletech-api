use async_graphql::{EmptyMutation, EmptySubscription, Schema};

use crate::{graphql::query::QueryRoot, state::AppState};

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build(state: AppState) -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(state)
        .limit_depth(20)
        .limit_complexity(500)
        .finish()
}
