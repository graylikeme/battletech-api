use async_graphql::{Object, SimpleObject};
use chrono::NaiveDate;

use crate::db::models::{DbMetadata, DbRuleset};

pub struct DatasetMetadataGql(pub DbMetadata);

#[Object]
impl DatasetMetadataGql {
    async fn version(&self) -> &str {
        &self.0.version
    }

    async fn schema_version(&self) -> i32 {
        self.0.schema_version
    }

    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

    async fn release_date(&self) -> Option<String> {
        self.0.release_date.map(|d: NaiveDate| d.to_string())
    }
}

#[derive(SimpleObject)]
pub struct RulesetGql {
    pub slug: String,
    pub name: String,
    pub level: String,
    pub description: Option<String>,
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
