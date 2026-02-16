use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{PrmError, PrmResult};
use crate::model::{Id, Person, Relationship, RelationshipLabel, User};

pub fn upsert(conn: &Connection, owner_id: Id<User>, rel: &Relationship) -> PrmResult<()> {
    conn.execute(
        "INSERT INTO relationships (person_id, network_owner_id, reminder_days)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(person_id) DO UPDATE SET reminder_days = excluded.reminder_days",
        params![
            rel.person_id.value.to_string(),
            owner_id.value.to_string(),
            rel.reminder_days,
        ],
    )?;

    // Replace label assignments
    conn.execute(
        "DELETE FROM relationship_label_assignments WHERE relationship_person_id = ?1",
        params![rel.person_id.value.to_string()],
    )?;

    for label_id in &rel.labels {
        conn.execute(
            "INSERT INTO relationship_label_assignments (relationship_person_id, label_id)
             VALUES (?1, ?2)",
            params![rel.person_id.value.to_string(), label_id.value.to_string()],
        )?;
    }

    Ok(())
}

pub fn find_by_person(conn: &Connection, person_id: Id<Person>) -> PrmResult<Option<Relationship>> {
    let mut stmt = conn.prepare(
        "SELECT person_id, reminder_days FROM relationships WHERE person_id = ?1",
    )?;

    let result = stmt.query_row(params![person_id.value.to_string()], |row| {
        let pid_str: String = row.get(0)?;
        let reminder_days: Option<i32> = row.get(1)?;
        Ok((pid_str, reminder_days))
    });

    match result {
        Ok((pid_str, reminder_days)) => {
            let pid = Id::new(
                Uuid::parse_str(&pid_str)
                    .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
            );
            let labels = find_label_ids(conn, pid)?;
            Ok(Some(Relationship {
                person_id: pid,
                labels,
                reminder_days,
            }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn find_by_owner(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<Relationship>> {
    let mut stmt = conn.prepare(
        "SELECT person_id, reminder_days FROM relationships WHERE network_owner_id = ?1",
    )?;

    let rows: Vec<(String, Option<i32>)> = stmt
        .query_map(params![owner_id.value.to_string()], |row| {
            let pid_str: String = row.get(0)?;
            let reminder_days: Option<i32> = row.get(1)?;
            Ok((pid_str, reminder_days))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut rels = Vec::new();
    for (pid_str, reminder_days) in rows {
        let pid = Id::new(
            Uuid::parse_str(&pid_str)
                .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
        );
        let labels = find_label_ids(conn, pid)?;
        rels.push(Relationship {
            person_id: pid,
            labels,
            reminder_days,
        });
    }

    Ok(rels)
}

pub fn update_reminder(
    conn: &Connection,
    person_id: Id<Person>,
    days: Option<i32>,
) -> PrmResult<()> {
    conn.execute(
        "UPDATE relationships SET reminder_days = ?1 WHERE person_id = ?2",
        params![days, person_id.value.to_string()],
    )?;
    Ok(())
}

fn find_label_ids(conn: &Connection, person_id: Id<Person>) -> PrmResult<Vec<Id<RelationshipLabel>>> {
    let mut stmt = conn.prepare(
        "SELECT label_id FROM relationship_label_assignments WHERE relationship_person_id = ?1",
    )?;

    let ids = stmt
        .query_map(params![person_id.value.to_string()], |row| {
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

// --- Label CRUD ---

pub fn insert_label(
    conn: &Connection,
    owner_id: Id<User>,
    label: &RelationshipLabel,
) -> PrmResult<()> {
    conn.execute(
        "INSERT INTO relationship_labels (id, network_owner_id, name, archived) VALUES (?1, ?2, ?3, ?4)",
        params![
            label.id.value.to_string(),
            owner_id.value.to_string(),
            label.name,
            label.archived as i32,
        ],
    )?;
    Ok(())
}

pub fn update_label_row(conn: &Connection, label: &RelationshipLabel) -> PrmResult<()> {
    conn.execute(
        "UPDATE relationship_labels SET name = ?1, archived = ?2 WHERE id = ?3",
        params![label.name, label.archived as i32, label.id.value.to_string()],
    )?;
    Ok(())
}

pub fn find_label_by_id(
    conn: &Connection,
    id: Id<RelationshipLabel>,
) -> PrmResult<Option<RelationshipLabel>> {
    let mut stmt =
        conn.prepare("SELECT id, name, archived FROM relationship_labels WHERE id = ?1")?;

    let result = stmt.query_row(params![id.value.to_string()], |row| {
        Ok(row_to_label(row))
    });

    match result {
        Ok(label) => Ok(Some(label?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn find_labels_by_owner(
    conn: &Connection,
    owner_id: Id<User>,
) -> PrmResult<Vec<RelationshipLabel>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, archived FROM relationship_labels WHERE network_owner_id = ?1 ORDER BY name",
    )?;

    let labels = stmt
        .query_map(params![owner_id.value.to_string()], |row| {
            Ok(row_to_label(row))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(labels)
}

pub fn find_active_labels(
    conn: &Connection,
    owner_id: Id<User>,
) -> PrmResult<Vec<RelationshipLabel>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, archived FROM relationship_labels
         WHERE network_owner_id = ?1 AND archived = 0 ORDER BY name",
    )?;

    let labels = stmt
        .query_map(params![owner_id.value.to_string()], |row| {
            Ok(row_to_label(row))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(labels)
}

pub fn find_archived_labels(
    conn: &Connection,
    owner_id: Id<User>,
) -> PrmResult<Vec<RelationshipLabel>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, archived FROM relationship_labels
         WHERE network_owner_id = ?1 AND archived = 1 ORDER BY name",
    )?;

    let labels = stmt
        .query_map(params![owner_id.value.to_string()], |row| {
            Ok(row_to_label(row))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(labels)
}

pub fn find_label_by_name(
    conn: &Connection,
    owner_id: Id<User>,
    name: &str,
) -> PrmResult<Option<RelationshipLabel>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, archived FROM relationship_labels
         WHERE network_owner_id = ?1 AND name = ?2 COLLATE NOCASE",
    )?;

    let result = stmt.query_row(params![owner_id.value.to_string(), name], |row| {
        Ok(row_to_label(row))
    });

    match result {
        Ok(label) => Ok(Some(label?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn row_to_label(row: &rusqlite::Row) -> PrmResult<RelationshipLabel> {
    let id_str: String = row.get(0).map_err(rusqlite::Error::from)?;
    Ok(RelationshipLabel {
        id: Id::new(
            Uuid::parse_str(&id_str)
                .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
        ),
        name: row.get(1).map_err(rusqlite::Error::from)?,
        archived: row.get::<_, i32>(2).map_err(rusqlite::Error::from)? != 0,
    })
}
