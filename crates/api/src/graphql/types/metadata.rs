use async_graphql::{Object, SimpleObject};
use chrono::NaiveDate;

use crate::db::models::{DbMetadata, DbRuleset};

pub struct DatasetMetadataGql(pub DbMetadata);

/// Metadata about the imported MegaMek dataset (version, schema, release date).
#[Object]
impl DatasetMetadataGql {
    /// MegaMek version string the data was imported from (e.g. "0.50.11").
    async fn version(&self) -> &str {
        &self.0.version
    }

    /// Database schema version number. Used by the /ready endpoint for compatibility checks.
    async fn schema_version(&self) -> i32 {
        self.0.schema_version
    }

    /// Flavor text or summary of the dataset contents.
    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

    /// Date the MegaMek release was published, formatted as YYYY-MM-DD. Null if unknown.
    async fn release_date(&self) -> Option<String> {
        self.0.release_date.map(|d: NaiveDate| d.to_string())
    }
}

/// A game ruleset level defining which units and equipment are permitted.
#[derive(SimpleObject)]
pub struct RulesetGql {
    /// Lowercase, hyphen-separated identifier for the ruleset.
    pub slug: String,
    /// Human-readable ruleset name (e.g. "Standard").
    pub name: String,
    /// Rules level in snake_case. One of: introductory, standard, advanced, experimental, unofficial.
    pub level: String,
    /// Description of what this ruleset permits.
    pub description: Option<String>,
    /// Source book defining this ruleset.
    pub source_book: Option<String>,
}

impl From<DbRuleset> for RulesetGql {
    fn from(r: DbRuleset) -> Self {
        Self {
            slug: r.slug,
            name: r.name,
            level: r.level,
            description: r.description,
            source_book: r.source_book,
        }
    }
}
