use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::PrmResult;
use crate::model::{Id, Person, User};

pub fn insert(conn: &Connection, owner_id: Id<User>, person: &Person) -> PrmResult<()> {
    conn.execute(
        "INSERT INTO people (id, network_owner_id, name, nickname, how_we_met, birthday, notes, location, is_self, archived)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            person.id.value.to_string(),
            owner_id.value.to_string(),
            person.name,
            person.nickname,
            person.how_we_met,
            person.birthday.map(|d| d.to_string()),
            person.notes,
            person.location,
            person.is_self as i32,
            person.archived as i32,
        ],
    )?;
    Ok(())
}

pub fn update(conn: &Connection, person: &Person) -> PrmResult<()> {
    conn.execute(
        "UPDATE people SET name = ?1, nickname = ?2, how_we_met = ?3, birthday = ?4, notes = ?5,
         location = ?6, is_self = ?7, archived = ?8, updated_at = datetime('now')
         WHERE id = ?9",
        params![
            person.name,
            person.nickname,
            person.how_we_met,
            person.birthday.map(|d| d.to_string()),
            person.notes,
            person.location,
            person.is_self as i32,
            person.archived as i32,
            person.id.value.to_string(),
        ],
    )?;
    Ok(())
}

pub fn find_by_id(conn: &Connection, id: Id<Person>) -> PrmResult<Option<Person>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, nickname, how_we_met, birthday, notes, location, is_self, archived
         FROM people WHERE id = ?1",
    )?;

    let result = stmt.query_row(params![id.value.to_string()], |row| {
        Ok(row_to_person(row))
    });

    match result {
        Ok(person) => Ok(Some(person?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn find_by_owner(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<Person>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, nickname, how_we_met, birthday, notes, location, is_self, archived
         FROM people WHERE network_owner_id = ?1 ORDER BY name",
    )?;

    let people = stmt
        .query_map(params![owner_id.value.to_string()], |row| {
            Ok(row_to_person(row))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(people)
}

pub fn find_active_by_owner(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<Person>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, nickname, how_we_met, birthday, notes, location, is_self, archived
         FROM people WHERE network_owner_id = ?1 AND archived = 0 ORDER BY name",
    )?;

    let people = stmt
        .query_map(params![owner_id.value.to_string()], |row| {
            Ok(row_to_person(row))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(people)
}

pub fn find_archived_by_owner(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<Person>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, nickname, how_we_met, birthday, notes, location, is_self, archived
         FROM people WHERE network_owner_id = ?1 AND archived = 1 ORDER BY name",
    )?;

    let people = stmt
        .query_map(params![owner_id.value.to_string()], |row| {
            Ok(row_to_person(row))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(people)
}

pub fn find_by_name(
    conn: &Connection,
    owner_id: Id<User>,
    query: &str,
) -> PrmResult<Vec<Person>> {
    let pattern = format!("%{}%", query.to_lowercase());
    let mut stmt = conn.prepare(
        "SELECT id, name, nickname, how_we_met, birthday, notes, location, is_self, archived
         FROM people WHERE network_owner_id = ?1
         AND (LOWER(name) LIKE ?2 OR LOWER(COALESCE(nickname, '')) LIKE ?2)
         ORDER BY name",
    )?;

    let people = stmt
        .query_map(params![owner_id.value.to_string(), pattern], |row| {
            Ok(row_to_person(row))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(people)
}

pub fn find_self(conn: &Connection, owner_id: Id<User>) -> PrmResult<Option<Person>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, nickname, how_we_met, birthday, notes, location, is_self, archived
         FROM people WHERE network_owner_id = ?1 AND is_self = 1",
    )?;

    let result = stmt.query_row(params![owner_id.value.to_string()], |row| {
        Ok(row_to_person(row))
    });

    match result {
        Ok(person) => Ok(Some(person?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn row_to_person(row: &rusqlite::Row) -> PrmResult<Person> {
    let id_str: String = row.get(0).map_err(rusqlite::Error::from)?;
    let birthday_str: Option<String> = row.get(4).map_err(rusqlite::Error::from)?;

    Ok(Person {
        id: Id::new(Uuid::parse_str(&id_str).map_err(|e| {
            crate::error::PrmError::Other(format!("Invalid UUID: {}", e))
        })?),
        name: row.get(1).map_err(rusqlite::Error::from)?,
        nickname: row.get(2).map_err(rusqlite::Error::from)?,
        how_we_met: row.get(3).map_err(rusqlite::Error::from)?,
        birthday: birthday_str
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
        notes: row.get(5).map_err(rusqlite::Error::from)?,
        location: row.get(6).map_err(rusqlite::Error::from)?,
        is_self: row.get::<_, i32>(7).map_err(rusqlite::Error::from)? != 0,
        archived: row.get::<_, i32>(8).map_err(rusqlite::Error::from)? != 0,
    })
}
