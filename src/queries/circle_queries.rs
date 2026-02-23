use rusqlite::Connection;

use crate::db::{circle_repo, person_repo};
use crate::error::PrmResult;
use crate::model::{Circle, Id, Person, User};

pub fn active_circles(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<Circle>> {
    circle_repo::find_active_by_owner(conn, owner_id)
}

pub fn archived_circles(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<Circle>> {
    circle_repo::find_archived_by_owner(conn, owner_id)
}

pub fn circle_members(conn: &Connection, circle_id: Id<Circle>) -> PrmResult<Vec<Person>> {
    let circle = match circle_repo::find_by_id(conn, circle_id)? {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };

    let mut members = Vec::new();
    for member_id in &circle.member_ids {
        if let Some(person) = person_repo::find_by_id(conn, *member_id)? {
            members.push(person);
        }
    }

    members.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(members)
}

pub fn circles_for_person(
    conn: &Connection,
    owner_id: Id<User>,
    person_id: Id<Person>,
) -> PrmResult<Vec<Circle>> {
    circle_repo::find_circles_for_person(conn, owner_id, person_id)
}

pub fn find_circle_by_name(
    conn: &Connection,
    owner_id: Id<User>,
    name: &str,
) -> PrmResult<Option<Circle>> {
    circle_repo::find_by_name(conn, owner_id, name)
}

pub fn find_active_circle_by_name(
    conn: &Connection,
    owner_id: Id<User>,
    query: &str,
) -> PrmResult<Vec<Circle>> {
    let lower = query.to_lowercase();
    let circles = circle_repo::find_active_by_owner(conn, owner_id)?;
    Ok(circles
        .into_iter()
        .filter(|c| c.name.to_lowercase().contains(&lower))
        .collect())
}
