pub mod ids;
pub mod user;
pub mod person;
pub mod interaction;
pub mod relationship;
pub mod circle;

// Re-exports for convenience
pub use ids::Id;
pub use user::User;
pub use person::{Person, ContactEntry, ContactType, ContactValue, Address, CustomContactType};
pub use interaction::{Interaction, InteractionMedium};
pub use relationship::{Relationship, RelationshipLabel};
pub use circle::Circle;
