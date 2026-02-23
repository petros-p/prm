use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use uuid::Uuid;

/// Type-safe identifier wrapper. The phantom type parameter `T` prevents
/// mixing IDs from different entity types (e.g., Person ID vs Circle ID).
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id<T> {
    pub value: Uuid,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new(value: Uuid) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    pub fn generate() -> Self {
        Self::new(Uuid::new_v4())
    }

    /// Parse from a UUID string.
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self::new(Uuid::parse_str(s)?))
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({})", self.value)
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Foo;

    #[test]
    fn generate_creates_unique_ids() {
        let id1 = Id::<Foo>::generate();
        let id2 = Id::<Foo>::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn ids_with_same_uuid_are_equal() {
        let uuid = Uuid::new_v4();
        let id1 = Id::<Foo>::new(uuid);
        let id2 = Id::<Foo>::new(uuid);
        assert_eq!(id1, id2);
    }

    #[test]
    fn parse_roundtrips() {
        let id = Id::<Foo>::generate();
        let s = id.value.to_string();
        let parsed = Id::<Foo>::parse(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn serde_roundtrip() {
        let id = Id::<Foo>::generate();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: Id<Foo> = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }
}
