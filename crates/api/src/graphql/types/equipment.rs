use async_graphql::{Object, ID};

use crate::db::models::DbEquipment;

pub struct EquipmentGql(pub DbEquipment);

/// An equipment item (weapon, ammo, armor, engine, etc.) that can be mounted on units.
#[Object]
impl EquipmentGql {
    /// Unique identifier (same as slug).
    async fn id(&self) -> ID {
        ID(self.0.slug.clone())
    }

    /// Lowercase, hyphen-separated identifier (e.g. "medium-laser", "clan-er-large-laser").
    async fn slug(&self) -> &str {
        &self.0.slug
    }

    /// Human-readable equipment name (e.g. "Medium Laser").
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// Equipment category in snake_case. One of: energy_weapon, ballistic_weapon, missile_weapon, ammo, physical_weapon, equipment, armor, structure, engine, targeting_system, myomer, heat_sink, jump_jet, communications.
    async fn category(&self) -> &str {
        &self.0.category
    }

    /// Technology base. One of: inner_sphere, clan, mixed, primitive.
    async fn tech_base(&self) -> &str {
        &self.0.tech_base
    }

    /// Rules level governing which game modes allow this equipment. One of: introductory, standard, advanced, experimental, unofficial.
    async fn rules_level(&self) -> &str {
        &self.0.rules_level
    }

    /// Weight in metric tons. Null for zero-weight or non-applicable items.
    async fn tonnage(&self) -> Option<f64> {
        self.0.tonnage.map(|d| {
            use rust_decimal::prelude::ToPrimitive;
            d.to_f64().unwrap_or(0.0)
        })
    }

    /// Number of critical hit slots consumed when mounted. Null if not applicable.
    async fn crits(&self) -> Option<i32> {
        self.0.crits
    }

    /// Damage value as a string (may contain special formats like "2/hit" or "10+5"). Null for non-damaging equipment.
    async fn damage(&self) -> Option<&str> {
        self.0.damage.as_deref()
    }

    /// Heat generated when fired, in heat points. Null for non-heat-generating equipment.
    async fn heat(&self) -> Option<i32> {
        self.0.heat
    }

    /// Minimum range in tabletop hexes. Attacks within this range suffer a penalty. Null if no minimum range.
    async fn range_min(&self) -> Option<i32> {
        self.0.range_min
    }

    /// Short range bracket in tabletop hexes. Null if not a ranged weapon.
    async fn range_short(&self) -> Option<i32> {
        self.0.range_short
    }

    /// Medium range bracket in tabletop hexes. Null if not a ranged weapon.
    async fn range_medium(&self) -> Option<i32> {
        self.0.range_medium
    }

    /// Long range bracket in tabletop hexes. Null if not a ranged weapon.
    async fn range_long(&self) -> Option<i32> {
        self.0.range_long
    }

    /// Battle Value — composite combat effectiveness score used for game balancing. Null if not computed.
    async fn bv(&self) -> Option<i32> {
        self.0.bv
    }

    /// In-universe BattleTech year when this equipment was first produced (e.g. 3025). Not a real-world date.
    async fn intro_year(&self) -> Option<i32> {
        self.0.intro_year
    }

    /// Source book or technical readout where this equipment is published.
    async fn source_book(&self) -> Option<&str> {
        self.0.source_book.as_deref()
    }

    /// Flavor text or technical description of the equipment item.
    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }
}
