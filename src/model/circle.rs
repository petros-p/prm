use serde::{Deserialize, Serialize};

use super::ids::Id;
use super::person::Person;

/// An organizational grouping of people.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circle {
    pub id: Id<Circle>,
    pub name: String,
    pub description: Option<String>,
    pub member_ids: Vec<Id<Person>>,
    pub archived: bool,
}

impl Circle {
    pub fn create(name: String, description: Option<String>) -> Self {
        Self {
            id: Id::generate(),
            name,
            description,
            member_ids: Vec::new(),
            archived: false,
        }
    }
}
