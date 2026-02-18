use std::collections::HashMap;

use async_graphql::dataloader::Loader;

use crate::db::{models::DbMechData, units};

pub struct MechDataLoader {
    pub pool: sqlx::PgPool,
}

impl Loader<i32> for MechDataLoader {
    type Value = DbMechData;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, DbMechData>, async_graphql::Error> {
        let rows = units::get_mech_data_batch(&self.pool, keys).await?;
        Ok(rows.into_iter().map(|r| (r.unit_id, r)).collect())
    }
}
