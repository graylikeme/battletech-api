use async_graphql::{dataloader::DataLoader, EmptyMutation, EmptySubscription, Schema};

use crate::{
    graphql::{
        loaders::{
            AmmoForLoader, AmmoTypesLoader, ArmorTypeLoader, CockpitTypeLoader, EngineTypeLoader,
            GyroTypeLoader, HeatsinkTypeLoader, MechDataLoader, MyomerTypeLoader,
            StructureTypeLoader,
        },
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
    let engine_type_loader = DataLoader::new(EngineTypeLoader { pool: pool.clone() }, tokio::spawn);
    let armor_type_loader = DataLoader::new(ArmorTypeLoader { pool: pool.clone() }, tokio::spawn);
    let structure_type_loader = DataLoader::new(StructureTypeLoader { pool: pool.clone() }, tokio::spawn);
    let heatsink_type_loader = DataLoader::new(HeatsinkTypeLoader { pool: pool.clone() }, tokio::spawn);
    let gyro_type_loader = DataLoader::new(GyroTypeLoader { pool: pool.clone() }, tokio::spawn);
    let cockpit_type_loader = DataLoader::new(CockpitTypeLoader { pool: pool.clone() }, tokio::spawn);
    let myomer_type_loader = DataLoader::new(MyomerTypeLoader { pool: pool.clone() }, tokio::spawn);

    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(state)
        .data(mech_loader)
        .data(ammo_for_loader)
        .data(ammo_types_loader)
        .data(engine_type_loader)
        .data(armor_type_loader)
        .data(structure_type_loader)
        .data(heatsink_type_loader)
        .data(gyro_type_loader)
        .data(cockpit_type_loader)
        .data(myomer_type_loader)
        .limit_depth(20)
        .limit_complexity(500)
        .finish()
}
