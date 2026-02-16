use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{PrmError, PrmResult};
use crate::model::{Circle, Id, Person, User};

pub fn insert(conn: &Connection, owner_id: Id<User>, circle: &Circle) -> PrmResult<()> {
    conn.execute(
        "INSERT INTO circles (id, network_owner_id, name, description, archived) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            circle.id.value.to_string(),
            owner_id.value.to_string(),
            circle.name,
            circle.description,
            circle.archived as i32,
        ],
    )?;

    for member_id in &circle.member_ids {
        conn.execute(
            "INSERT INTO circle_members (circle_id, person_id) VALUES (?1, ?2)",
            params![circle.id.value.to_string(), member_id.value.to_string()],
        )?;
    }

    Ok(())
}

pub fn update(conn: &Connection, circle: &Circle) -> PrmResult<()> {
    conn.execute(
        "UPDATE circles SET name = ?1, description = ?2, archived = ?3 WHERE id = ?4",
        params![
            circle.name,
            circle.description,
            circle.archived as i32,
            circle.id.value.to_string(),
        ],
    )?;
    Ok(())
}

pub fn set_members(conn: &Connection, circle_id: Id<Circle>, member_ids: &[Id<Person>]) -> PrmResult<()> {
    conn.execute(
        "DELETE FROM circle_members WHERE circle_id = ?1",
        params![circle_id.value.to_string()],
    )?;

    for member_id in member_ids {
        conn.execute(
            "INSERT INTO circle_members (circle_id, person_id) VALUES (?1, ?2)",
            params![circle_id.value.to_string(), member_id.value.to_string()],
        )?;
    }

    Ok(())
}

pub fn add_members(conn: &Connection, circle_id: Id<Circle>, member_ids: &[Id<Person>]) -> PrmResult<()> {
    for member_id in member_ids {
        conn.execute(
            "INSERT OR IGNORE INTO circle_members (circle_id, person_id) VALUES (?1, ?2)",
            params![circle_id.value.to_string(), member_id.value.to_string()],
        )?;
    }
    Ok(())
}

pub fn remove_members(conn: &Connection, circle_id: Id<Circle>, member_ids: &[Id<Person>]) -> PrmResult<()> {
    for member_id in member_ids {
        conn.execute(
            "DELETE FROM circle_members WHERE circle_id = ?1 AND person_id = ?2",
            params![circle_id.value.to_string(), member_id.value.to_string()],
        )?;
    }
    Ok(())
}

pub fn delete(conn: &Connection, circle_id: Id<Circle>) -> PrmResult<()> {
    conn.execute(
        "DELETE FROM circle_members WHERE circle_id = ?1",
        params![circle_id.value.to_string()],
    )?;
    conn.execute(
        "DELETE FROM circles WHERE id = ?1",
        params![circle_id.value.to_string()],
    )?;
    Ok(())
}

pub fn find_by_id(conn: &Connection, id: Id<Circle>) -> PrmResult<Option<Circle>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, archived FROM circles WHERE id = ?1",
    )?;

    let result = stmt.query_row(params![id.value.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let name: String = row.get(1)?;
        let description: Option<String> = row.get(2)?;
        let archived: i32 = row.get(3)?;
        Ok((id_str, name, description, archived))
    });

    match result {
        Ok((id_str, name, description, archived)) => {
            let circle_id = Id::new(
                Uuid::parse_str(&id_str)
                    .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
            );
            let member_ids = find_member_ids(conn, circle_id)?;
            Ok(Some(Circle {
                id: circle_id,
                name,
                description,
                member_ids,
                archived: archived != 0,
            }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn find_by_owner(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<Circle>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, archived FROM circles WHERE network_owner_id = ?1 ORDER BY name",
    )?;

    let rows: Vec<(String, String, Option<String>, i32)> = stmt
        .query_map(params![owner_id.value.to_string()], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut circles = Vec::new();
    for (id_str, name, description, archived) in rows {
        let circle_id = Id::new(
            Uuid::parse_str(&id_str)
                .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
        );
        let member_ids = find_member_ids(conn, circle_id)?;
        circles.push(Circle {
            id: circle_id,
            name,
            description,
            member_ids,
            archived: archived != 0,
        });
    }

    Ok(circles)
}

pub fn find_active_by_owner(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<Circle>> {
    Ok(find_by_owner(conn, owner_id)?
        .into_iter()
        .filter(|c| !c.archived)
        .collect())
}

pub fn find_archived_by_owner(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<Circle>> {
    Ok(find_by_owner(conn, owner_id)?
        .into_iter()
        .filter(|c| c.archived)
        .collect())
}

pub fn find_by_name(
    conn: &Connection,
    owner_id: Id<User>,
    name: &str,
) -> PrmResult<Option<Circle>> {
    let circles = find_by_owner(conn, owner_id)?;
    Ok(circles
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case(name)))
}

pub fn find_circles_for_person(
    conn: &Connection,
    owner_id: Id<User>,
    person_id: Id<Person>,
) -> PrmResult<Vec<Circle>> {
    let circles = find_active_by_owner(conn, owner_id)?;
    Ok(circles
        .into_iter()
        .filter(|c| c.member_ids.contains(&person_id))
        .collect())
}

fn find_member_ids(conn: &Connection, circle_id: Id<Circle>) -> PrmResult<Vec<Id<Person>>> {
    let mut stmt = conn.prepare(
        "SELECT person_id FROM circle_members WHERE circle_id = ?1",
    )?;

    let ids = stmt
        .query_map(params![circle_id.value.to_string()], |row| {
            let id_str: String = row.get(0)?;
            Ok(id_str)
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|s| {
            Ok(Id::new(
                Uuid::parse_str(&s)
                    .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
            ))
        })
        .collect::<PrmResult<Vec<_>>>()?;

    Ok(ids)
}
