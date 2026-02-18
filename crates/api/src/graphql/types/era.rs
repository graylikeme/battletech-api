use async_graphql::{Object, ID};

use crate::db::models::DbEra;

pub struct EraGql(pub DbEra);

/// A historical era in the BattleTech timeline (e.g. "Clan Invasion", "Star League").
#[Object]
impl EraGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "clan-invasion", "star-league").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Human-readable era name (e.g. "Clan Invasion").
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// In-universe BattleTech year when this era begins (e.g. 3049). Not a real-world date.
    async fn start_year(&self) -> i32 {
        self.0.start_year
    }

    /// In-universe BattleTech year when this era ends. Null if the era is ongoing.
    async fn end_year(&self) -> Option<i32> {
        self.0.end_year
    }

    /// Flavor text or lore description of the era.
    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }
}
