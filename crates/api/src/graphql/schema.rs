use async_graphql::{dataloader::DataLoader, EmptyMutation, EmptySubscription, Schema};

use crate::{graphql::{loaders::MechDataLoader, query::QueryRoot}, state::AppState};

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build(state: AppState) -> AppSchema {
    let mech_loader = DataLoader::new(
        MechDataLoader { pool: state.pool.clone() },
        tokio::spawn,
    );
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(state)
        .data(mech_loader)
        .limit_depth(20)
        .limit_complexity(500)
        .finish()
}
