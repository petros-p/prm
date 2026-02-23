use serde::{Deserialize, Serialize};

use super::ids::Id;
use super::person::Person;

/// Describes the nature of your connection to another person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipLabel {
    pub id: Id<RelationshipLabel>,
    pub name: String,
    pub archived: bool,
}

impl RelationshipLabel {
    pub fn create(name: String) -> Self {
        Self {
            id: Id::generate(),
            name,
            archived: false,
        }
    }

    /// Default labels to start with.
    pub fn defaults() -> Vec<RelationshipLabel> {
        [
            "me",
            "friend",
            "family",
            "coworker",
            "acquaintance",
            "mentor",
            "mentee",
            "neighbor",
            "former coworker",
            "romantic partner",
            "former romantic partner",
        ]
        .iter()
        .map(|name| RelationshipLabel::create(name.to_string()))
        .collect()
    }
}

/// Your relationship to another person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub person_id: Id<Person>,
    pub labels: Vec<Id<RelationshipLabel>>,
    pub reminder_days: Option<i32>,
}

impl Relationship {
    pub fn create(person_id: Id<Person>) -> Self {
        Self {
            person_id,
            labels: Vec::new(),
            reminder_days: None,
        }
    }
}
