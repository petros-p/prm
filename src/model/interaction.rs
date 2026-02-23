use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::ids::Id;

/// How the interaction took place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionMedium {
    InPerson,
    Text,
    PhoneCall,
    VideoCall,
    SocialMedia,
}

impl InteractionMedium {
    pub const ALL: &'static [InteractionMedium] = &[
        InteractionMedium::InPerson,
        InteractionMedium::Text,
        InteractionMedium::PhoneCall,
        InteractionMedium::VideoCall,
        InteractionMedium::SocialMedia,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            InteractionMedium::InPerson => "In Person",
            InteractionMedium::Text => "Text",
            InteractionMedium::PhoneCall => "Phone Call",
            InteractionMedium::VideoCall => "Video Call",
            InteractionMedium::SocialMedia => "Social Media",
        }
    }

    /// Parse from database string representation.
    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "InPerson" => Some(InteractionMedium::InPerson),
            "Text" => Some(InteractionMedium::Text),
            "PhoneCall" => Some(InteractionMedium::PhoneCall),
            "VideoCall" => Some(InteractionMedium::VideoCall),
            "SocialMedia" => Some(InteractionMedium::SocialMedia),
            _ => None,
        }
    }

    /// Convert to database string representation.
    pub fn to_db_str(&self) -> &'static str {
        match self {
            InteractionMedium::InPerson => "InPerson",
            InteractionMedium::Text => "Text",
            InteractionMedium::PhoneCall => "PhoneCall",
            InteractionMedium::VideoCall => "VideoCall",
            InteractionMedium::SocialMedia => "SocialMedia",
        }
    }
}

/// A single interaction between you and another person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: Id<Interaction>,
    pub date: NaiveDate,
    pub medium: InteractionMedium,
    pub my_location: String,
    pub their_location: Option<String>,
    pub topics: Vec<String>,
    pub note: Option<String>,
}

impl Interaction {
    /// Creates a new in-person interaction.
    pub fn create_in_person(
        location: String,
        topics: Vec<String>,
        note: Option<String>,
        date: NaiveDate,
    ) -> Self {
        Self {
            id: Id::generate(),
            date,
            medium: InteractionMedium::InPerson,
            my_location: location.clone(),
            their_location: Some(location),
            topics,
            note,
        }
    }

    /// Creates a new remote interaction.
    pub fn create_remote(
        medium: InteractionMedium,
        my_location: String,
        their_location: Option<String>,
        topics: Vec<String>,
        note: Option<String>,
        date: NaiveDate,
    ) -> Self {
        assert_ne!(
            medium,
            InteractionMedium::InPerson,
            "Use create_in_person for in-person interactions"
        );
        Self {
            id: Id::generate(),
            date,
            medium,
            my_location,
            their_location,
            topics,
            note,
        }
    }
}
