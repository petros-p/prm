use rusqlite::Connection;

use crate::db::{circle_repo, person_repo};
use crate::error::{PrmError, PrmResult};
use crate::model::{Circle, Id, Person, User};
use crate::validation::{self, trim_optional};

pub fn create_circle(
    conn: &Connection,
    owner_id: Id<User>,
    name: &str,
    description: Option<&str>,
    member_ids: Vec<Id<Person>>,
) -> PrmResult<Circle> {
    let valid_name = validation::non_blank(name, "name")?;

    // Filter to valid person IDs
    let valid_members: Vec<Id<Person>> = member_ids
        .into_iter()
        .filter(|id| person_repo::find_by_id(conn, *id).ok().flatten().is_some())
        .collect();

    let mut circle = Circle::create(valid_name, trim_optional(description));
    circle.member_ids = valid_members;

    circle_repo::insert(conn, owner_id, &circle)?;
    Ok(circle)
}

pub fn update_circle(
    conn: &Connection,
    circle_id: Id<Circle>,
    name: Option<&str>,
    description: Option<Option<&str>>,
) -> PrmResult<Circle> {
    let mut circle = circle_repo::find_by_id(conn, circle_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Circle".into(),
            id: circle_id.to_string(),
        })?;

    if let Some(n) = name {
        circle.name = validation::non_blank(n, "name")?;
    }
    if let Some(desc) = description {
        circle.description = trim_optional(desc);
    }

    circle_repo::update(conn, &circle)?;
    Ok(circle)
}

pub fn add_members(
    conn: &Connection,
    circle_id: Id<Circle>,
    person_ids: Vec<Id<Person>>,
) -> PrmResult<Circle> {
    let circle = circle_repo::find_by_id(conn, circle_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Circle".into(),
            id: circle_id.to_string(),
        })?;

    circle_repo::add_members(conn, circle_id, &person_ids)?;

    // Re-fetch to get updated member list
    Ok(circle_repo::find_by_id(conn, circle_id)?.unwrap_or(circle))
}

pub fn remove_members(
    conn: &Connection,
    circle_id: Id<Circle>,
    person_ids: Vec<Id<Person>>,
) -> PrmResult<Circle> {
    let circle = circle_repo::find_by_id(conn, circle_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Circle".into(),
            id: circle_id.to_string(),
        })?;

    circle_repo::remove_members(conn, circle_id, &person_ids)?;

    Ok(circle_repo::find_by_id(conn, circle_id)?.unwrap_or(circle))
}

pub fn archive_circle(conn: &Connection, circle_id: Id<Circle>) -> PrmResult<Circle> {
    let mut circle = circle_repo::find_by_id(conn, circle_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Circle".into(),
            id: circle_id.to_string(),
        })?;

    circle.archived = true;
    circle_repo::update(conn, &circle)?;
    Ok(circle)
}

pub fn unarchive_circle(conn: &Connection, circle_id: Id<Circle>) -> PrmResult<Circle> {
    let mut circle = circle_repo::find_by_id(conn, circle_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Circle".into(),
            id: circle_id.to_string(),
        })?;

    circle.archived = false;
    circle_repo::update(conn, &circle)?;
    Ok(circle)
}

pub fn delete_circle(conn: &Connection, circle_id: Id<Circle>) -> PrmResult<()> {
    circle_repo::find_by_id(conn, circle_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Circle".into(),
            id: circle_id.to_string(),
        })?;

    circle_repo::delete(conn, circle_id)
}
