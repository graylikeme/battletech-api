use async_graphql::{Object, ID};

use crate::db::models::DbEquipment;

pub struct EquipmentGql(pub DbEquipment);

#[Object]
impl EquipmentGql {
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    async fn slug(&self) -> &str {
        &self.0.slug
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn category(&self) -> &str {
        &self.0.category
    }

    async fn tech_base(&self) -> &str {
        &self.0.tech_base
    }

    async fn rules_level(&self) -> &str {
        &self.0.rules_level
    }

    async fn tonnage(&self) -> Option<f64> {
        self.0.tonnage.map(|d| {
            use rust_decimal::prelude::ToPrimitive;
            d.to_f64().unwrap_or(0.0)
        })
    }

    async fn crits(&self) -> Option<i32> {
        self.0.crits
    }

    async fn damage(&self) -> Option<&str> {
        self.0.damage.as_deref()
    }

    async fn heat(&self) -> Option<i32> {
        self.0.heat
    }

    async fn range_min(&self) -> Option<i32> {
        self.0.range_min
    }

    async fn range_short(&self) -> Option<i32> {
        self.0.range_short
    }

    async fn range_medium(&self) -> Option<i32> {
        self.0.range_medium
    }

    async fn range_long(&self) -> Option<i32> {
        self.0.range_long
    }

    async fn bv(&self) -> Option<i32> {
        self.0.bv
    }

    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }

    async fn source_book(&self) -> Option<&str> {
        self.0.source_book.as_deref()
    }

    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }
}
