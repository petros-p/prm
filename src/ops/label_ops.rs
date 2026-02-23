use rusqlite::Connection;

use crate::db::relationship_repo;
use crate::error::{PrmError, PrmResult};
use crate::model::{Id, RelationshipLabel, User};
use crate::validation;

pub fn add_label(
    conn: &Connection,
    owner_id: Id<User>,
    name: &str,
) -> PrmResult<RelationshipLabel> {
    let valid_name = validation::non_blank(name, "name")?;

    if relationship_repo::find_label_by_name(conn, owner_id, &valid_name)?.is_some() {
        return Err(PrmError::AlreadyExists {
            entity_type: "Label".into(),
            identifier: valid_name,
        });
    }

    let label = RelationshipLabel::create(valid_name);
    relationship_repo::insert_label(conn, owner_id, &label)?;
    Ok(label)
}

pub fn update_label(
    conn: &Connection,
    owner_id: Id<User>,
    label_id: Id<RelationshipLabel>,
    name: Option<&str>,
) -> PrmResult<RelationshipLabel> {
    let mut label = relationship_repo::find_label_by_id(conn, label_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Label".into(),
            id: label_id.to_string(),
        })?;

    if let Some(n) = name {
        let valid_name = validation::non_blank(n, "name")?;

        // Check for duplicate (excluding self)
        if let Some(existing) = relationship_repo::find_label_by_name(conn, owner_id, &valid_name)? {
            if existing.id != label_id {
                return Err(PrmError::AlreadyExists {
                    entity_type: "Label".into(),
                    identifier: valid_name,
                });
            }
        }

        label.name = valid_name;
    }

    relationship_repo::update_label_row(conn, &label)?;
    Ok(label)
}

pub fn archive_label(conn: &Connection, label_id: Id<RelationshipLabel>) -> PrmResult<RelationshipLabel> {
    let mut label = relationship_repo::find_label_by_id(conn, label_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Label".into(),
            id: label_id.to_string(),
        })?;

    label.archived = true;
    relationship_repo::update_label_row(conn, &label)?;
    Ok(label)
}

pub fn unarchive_label(conn: &Connection, label_id: Id<RelationshipLabel>) -> PrmResult<RelationshipLabel> {
    let mut label = relationship_repo::find_label_by_id(conn, label_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Label".into(),
            id: label_id.to_string(),
        })?;

    label.archived = false;
    relationship_repo::update_label_row(conn, &label)?;
    Ok(label)
}
