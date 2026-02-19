use async_graphql::{dataloader::DataLoader, EmptyMutation, EmptySubscription, Schema};

use crate::{
    graphql::{
        loaders::{AmmoForLoader, AmmoTypesLoader, MechDataLoader},
        query::QueryRoot,
    },
    state::AppState,
};

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build(state: AppState) -> AppSchema {
    let pool = &state.pool;
    let mech_loader = DataLoader::new(MechDataLoader { pool: pool.clone() }, tokio::spawn);
    let ammo_for_loader = DataLoader::new(AmmoForLoader { pool: pool.clone() }, tokio::spawn);
    let ammo_types_loader = DataLoader::new(AmmoTypesLoader { pool: pool.clone() }, tokio::spawn);

    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(state)
        .data(mech_loader)
        .data(ammo_for_loader)
        .data(ammo_types_loader)
        .limit_depth(20)
        .limit_complexity(500)
        .finish()
}
