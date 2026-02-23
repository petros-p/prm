use serde::{Deserialize, Serialize};

use super::ids::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Id<User>,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn create(name: String, email: String) -> Self {
        Self {
            id: Id::generate(),
            name,
            email,
        }
    }
}
