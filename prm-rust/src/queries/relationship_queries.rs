use rusqlite::Connection;

use crate::db::{person_repo, relationship_repo};
use crate::error::PrmResult;
use crate::model::{Id, Person, Relationship, RelationshipLabel, User};

pub fn get_relationship(
    conn: &Connection,
    person_id: Id<Person>,
) -> PrmResult<Option<Relationship>> {
    relationship_repo::find_by_person(conn, person_id)
}

pub fn labels_for(
    conn: &Connection,
    person_id: Id<Person>,
) -> PrmResult<Vec<RelationshipLabel>> {
    let rel = match relationship_repo::find_by_person(conn, person_id)? {
        Some(r) => r,
        None => return Ok(Vec::new()),
    };

    let mut labels = Vec::new();
    for label_id in &rel.labels {
        if let Some(label) = relationship_repo::find_label_by_id(conn, *label_id)? {
            labels.push(label);
        }
    }

    Ok(labels)
}

pub fn people_with_label(
    conn: &Connection,
    owner_id: Id<User>,
    label_id: Id<RelationshipLabel>,
) -> PrmResult<Vec<Person>> {
    let rels = relationship_repo::find_by_owner(conn, owner_id)?;
    let mut people = Vec::new();

    for rel in rels {
        if rel.labels.contains(&label_id) {
            if let Some(person) = person_repo::find_by_id(conn, rel.person_id)? {
                if !person.archived {
                    people.push(person);
                }
            }
        }
    }

    people.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(people)
}

pub fn active_labels(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<RelationshipLabel>> {
    relationship_repo::find_active_labels(conn, owner_id)
}

pub fn archived_labels(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<RelationshipLabel>> {
    relationship_repo::find_archived_labels(conn, owner_id)
}

pub fn find_label_by_name(
    conn: &Connection,
    owner_id: Id<User>,
    name: &str,
) -> PrmResult<Option<RelationshipLabel>> {
    relationship_repo::find_label_by_name(conn, owner_id, name)
}

pub fn people_with_label_name(
    conn: &Connection,
    owner_id: Id<User>,
    label_name: &str,
) -> PrmResult<Vec<Person>> {
    let lower = label_name.to_lowercase();
    let labels = relationship_repo::find_active_labels(conn, owner_id)?;
    match labels.into_iter().find(|l| l.name.to_lowercase() == lower) {
        Some(label) => people_with_label(conn, owner_id, label.id),
        None => Ok(Vec::new()),
    }
}

pub fn find_active_label_by_name(
    conn: &Connection,
    owner_id: Id<User>,
    query: &str,
) -> PrmResult<Vec<RelationshipLabel>> {
    let lower = query.to_lowercase();
    let labels = relationship_repo::find_active_labels(conn, owner_id)?;
    Ok(labels
        .into_iter()
        .filter(|l| l.name.to_lowercase().contains(&lower))
        .collect())
}
