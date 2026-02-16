use rusqlite::Connection;

use crate::db::{person_repo, relationship_repo};
use crate::error::{PrmError, PrmResult};
use crate::model::{Id, Person, Relationship, RelationshipLabel, User};
use crate::validation;

pub fn set_relationship(
    conn: &Connection,
    owner_id: Id<User>,
    person_id: Id<Person>,
    labels: Vec<Id<RelationshipLabel>>,
    reminder_days: Option<i32>,
) -> PrmResult<Relationship> {
    person_repo::find_by_id(conn, person_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Person".into(),
            id: person_id.to_string(),
        })?;

    validation::optional_positive(reminder_days, "reminderDays")?;

    let mut rel = relationship_repo::find_by_person(conn, person_id)?
        .unwrap_or_else(|| Relationship::create(person_id));

    rel.labels = labels;
    rel.reminder_days = reminder_days;

    relationship_repo::upsert(conn, owner_id, &rel)?;
    Ok(rel)
}

pub fn set_labels(
    conn: &Connection,
    owner_id: Id<User>,
    person_id: Id<Person>,
    labels: Vec<Id<RelationshipLabel>>,
) -> PrmResult<Relationship> {
    person_repo::find_by_id(conn, person_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Person".into(),
            id: person_id.to_string(),
        })?;

    let mut rel = relationship_repo::find_by_person(conn, person_id)?
        .unwrap_or_else(|| Relationship::create(person_id));

    rel.labels = labels;
    relationship_repo::upsert(conn, owner_id, &rel)?;
    Ok(rel)
}

pub fn add_labels(
    conn: &Connection,
    owner_id: Id<User>,
    person_id: Id<Person>,
    labels: Vec<Id<RelationshipLabel>>,
) -> PrmResult<Relationship> {
    let mut rel = relationship_repo::find_by_person(conn, person_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Relationship".into(),
            id: person_id.to_string(),
        })?;

    for label_id in labels {
        if !rel.labels.contains(&label_id) {
            rel.labels.push(label_id);
        }
    }

    relationship_repo::upsert(conn, owner_id, &rel)?;
    Ok(rel)
}

pub fn remove_labels(
    conn: &Connection,
    owner_id: Id<User>,
    person_id: Id<Person>,
    labels: Vec<Id<RelationshipLabel>>,
) -> PrmResult<Relationship> {
    let mut rel = relationship_repo::find_by_person(conn, person_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Relationship".into(),
            id: person_id.to_string(),
        })?;

    rel.labels.retain(|id| !labels.contains(id));

    relationship_repo::upsert(conn, owner_id, &rel)?;
    Ok(rel)
}

pub fn set_reminder(
    conn: &Connection,
    person_id: Id<Person>,
    days: Option<i32>,
) -> PrmResult<()> {
    validation::optional_positive(days, "days")?;

    relationship_repo::find_by_person(conn, person_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Relationship".into(),
            id: person_id.to_string(),
        })?;

    relationship_repo::update_reminder(conn, person_id, days)
}
