//! Tag response types for frontend serialization

use crate::db::repositories::tag_repository::{DbTag, TagWithCount};
use serde::{Deserialize, Serialize};

/// Tag for JSON serialization (frontend-friendly)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TagResponse {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub usage_count: i32,
}

impl From<DbTag> for TagResponse {
    fn from(db_tag: DbTag) -> Self {
        Self {
            id: db_tag.id,
            name: db_tag.name,
            category: db_tag.category,
            usage_count: db_tag.usage_count,
        }
    }
}

impl From<TagWithCount> for TagResponse {
    fn from(tag: TagWithCount) -> Self {
        Self {
            id: tag.id,
            name: tag.name,
            category: tag.category,
            usage_count: tag.usage_count,
        }
    }
}

impl PartialEq<str> for TagResponse {
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}

impl PartialEq<&str> for TagResponse {
    fn eq(&self, other: &&str) -> bool {
        self.name == *other
    }
}
