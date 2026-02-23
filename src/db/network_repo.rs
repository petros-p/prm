use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{PrmError, PrmResult};
use crate::model::{Id, Person, User};

pub fn insert_user(conn: &Connection, user: &User) -> PrmResult<()> {
    conn.execute(
        "INSERT INTO users (id, name, email) VALUES (?1, ?2, ?3)",
        params![user.id.value.to_string(), user.name, user.email],
    )?;
    Ok(())
}

pub fn find_user(conn: &Connection, user_id: Id<User>) -> PrmResult<Option<User>> {
    let mut stmt = conn.prepare("SELECT id, name, email FROM users WHERE id = ?1")?;

    let result = stmt.query_row(params![user_id.value.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let name: String = row.get(1)?;
        let email: String = row.get(2)?;
        Ok((id_str, name, email))
    });

    match result {
        Ok((id_str, name, email)) => Ok(Some(User {
            id: Id::new(
                Uuid::parse_str(&id_str)
                    .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
            ),
            name,
            email,
        })),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn find_first_user(conn: &Connection) -> PrmResult<Option<User>> {
    let mut stmt = conn.prepare("SELECT id, name, email FROM users LIMIT 1")?;

    let result = stmt.query_row([], |row| {
        let id_str: String = row.get(0)?;
        let name: String = row.get(1)?;
        let email: String = row.get(2)?;
        Ok((id_str, name, email))
    });

    match result {
        Ok((id_str, name, email)) => Ok(Some(User {
            id: Id::new(
                Uuid::parse_str(&id_str)
                    .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
            ),
            name,
            email,
        })),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set_network_metadata(
    conn: &Connection,
    owner_id: Id<User>,
    self_id: Id<Person>,
) -> PrmResult<()> {
    conn.execute(
        "INSERT INTO network_metadata (owner_id, self_id) VALUES (?1, ?2)
         ON CONFLICT(owner_id) DO UPDATE SET self_id = excluded.self_id",
        params![owner_id.value.to_string(), self_id.value.to_string()],
    )?;
    Ok(())
}

pub fn get_self_id(conn: &Connection, owner_id: Id<User>) -> PrmResult<Option<Id<Person>>> {
    let mut stmt = conn.prepare("SELECT self_id FROM network_metadata WHERE owner_id = ?1")?;

    let result = stmt.query_row(params![owner_id.value.to_string()], |row| {
        let id_str: String = row.get(0)?;
        Ok(id_str)
    });

    match result {
        Ok(id_str) => Ok(Some(Id::new(
            Uuid::parse_str(&id_str)
                .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
        ))),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
