use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::ids::Id;

/// A structured physical address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

/// A user-defined contact type (e.g., "Discord", "LinkedIn").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomContactType {
    pub id: Id<CustomContactType>,
    pub name: String,
}

impl CustomContactType {
    pub fn create(name: String) -> Self {
        Self {
            id: Id::generate(),
            name,
        }
    }
}

/// The type of contact information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContactType {
    Phone,
    Email,
    PhysicalAddress,
    Custom { type_id: Id<CustomContactType> },
}

/// The value of a contact entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ContactValue {
    StringValue { value: String },
    AddressValue { value: Address },
}

/// A single contact entry for a person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactEntry {
    pub id: Id<ContactEntry>,
    pub contact_type: ContactType,
    pub value: ContactValue,
    pub label: Option<String>,
}

impl ContactEntry {
    pub fn phone(number: String, label: Option<String>) -> Self {
        Self {
            id: Id::generate(),
            contact_type: ContactType::Phone,
            value: ContactValue::StringValue { value: number },
            label,
        }
    }

    pub fn email(address: String, label: Option<String>) -> Self {
        Self {
            id: Id::generate(),
            contact_type: ContactType::Email,
            value: ContactValue::StringValue { value: address },
            label,
        }
    }

    pub fn address(address: Address, label: Option<String>) -> Self {
        Self {
            id: Id::generate(),
            contact_type: ContactType::PhysicalAddress,
            value: ContactValue::AddressValue { value: address },
            label,
        }
    }

    pub fn custom(type_id: Id<CustomContactType>, value: String, label: Option<String>) -> Self {
        Self {
            id: Id::generate(),
            contact_type: ContactType::Custom { type_id },
            value: ContactValue::StringValue { value },
            label,
        }
    }
}

/// A person in your network (including yourself).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: Id<Person>,
    pub name: String,
    pub nickname: Option<String>,
    pub how_we_met: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub notes: Option<String>,
    pub location: Option<String>,
    pub is_self: bool,
    pub archived: bool,
}

impl Person {
    pub fn create(name: String) -> Self {
        Self {
            id: Id::generate(),
            name,
            nickname: None,
            how_we_met: None,
            birthday: None,
            notes: None,
            location: None,
            is_self: false,
            archived: false,
        }
    }

    pub fn create_self(name: String) -> Self {
        let mut p = Self::create(name);
        p.is_self = true;
        p
    }
}
