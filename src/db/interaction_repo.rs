use chrono::NaiveDate;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{PrmError, PrmResult};
use crate::model::{Id, Interaction, InteractionMedium, Person};

pub fn insert(conn: &Connection, person_id: Id<Person>, interaction: &Interaction) -> PrmResult<()> {
    conn.execute(
        "INSERT INTO interactions (id, relationship_person_id, date, medium, my_location, their_location, note)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            interaction.id.value.to_string(),
            person_id.value.to_string(),
            interaction.date.to_string(),
            interaction.medium.to_db_str(),
            interaction.my_location,
            interaction.their_location,
            interaction.note,
        ],
    )?;

    for topic in &interaction.topics {
        conn.execute(
            "INSERT INTO interaction_topics (interaction_id, topic) VALUES (?1, ?2)",
            params![interaction.id.value.to_string(), topic],
        )?;
    }

    Ok(())
}

pub fn find_by_person(
    conn: &Connection,
    person_id: Id<Person>,
) -> PrmResult<Vec<Interaction>> {
    let mut stmt = conn.prepare(
        "SELECT id, date, medium, my_location, their_location, note
         FROM interactions WHERE relationship_person_id = ?1
         ORDER BY date DESC, created_at DESC",
    )?;

    let rows: Vec<(String, String, String, String, Option<String>, Option<String>)> = stmt
        .query_map(params![person_id.value.to_string()], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut interactions = Vec::new();
    for (id_str, date_str, medium_str, my_loc, their_loc, note) in rows {
        let id = Id::new(
            Uuid::parse_str(&id_str)
                .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
        );
        let topics = find_topics(conn, id)?;
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|e| PrmError::Other(format!("Invalid date: {}", e)))?;
        let medium = InteractionMedium::from_db_str(&medium_str)
            .ok_or_else(|| PrmError::Other(format!("Unknown medium: {}", medium_str)))?;

        interactions.push(Interaction {
            id,
            date,
            medium,
            my_location: my_loc,
            their_location: their_loc,
            topics,
            note,
        });
    }

    Ok(interactions)
}

pub fn find_last_interaction_date(
    conn: &Connection,
    person_id: Id<Person>,
) -> PrmResult<Option<NaiveDate>> {
    let mut stmt = conn.prepare(
        "SELECT date FROM interactions WHERE relationship_person_id = ?1
         ORDER BY date DESC LIMIT 1",
    )?;

    let result = stmt.query_row(params![person_id.value.to_string()], |row| {
        let date_str: String = row.get(0)?;
        Ok(date_str)
    });

    match result {
        Ok(date_str) => {
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|e| PrmError::Other(format!("Invalid date: {}", e)))?;
            Ok(Some(date))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn find_in_date_range(
    conn: &Connection,
    owner_id: Id<crate::model::User>,
    from: NaiveDate,
    to: NaiveDate,
) -> PrmResult<Vec<(Id<Person>, Interaction)>> {
    let mut stmt = conn.prepare(
        "SELECT i.id, i.relationship_person_id, i.date, i.medium, i.my_location, i.their_location, i.note
         FROM interactions i
         JOIN relationships r ON i.relationship_person_id = r.person_id
         WHERE r.network_owner_id = ?1 AND i.date >= ?2 AND i.date <= ?3
         ORDER BY i.date DESC",
    )?;

    let rows: Vec<(String, String, String, String, String, Option<String>, Option<String>)> = stmt
        .query_map(
            params![
                owner_id.value.to_string(),
                from.to_string(),
                to.to_string()
            ],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

    let mut results = Vec::new();
    for (id_str, pid_str, date_str, medium_str, my_loc, their_loc, note) in rows {
        let id = Id::new(
            Uuid::parse_str(&id_str)
                .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
        );
        let person_id = Id::new(
            Uuid::parse_str(&pid_str)
                .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
        );
        let topics = find_topics(conn, id)?;
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|e| PrmError::Other(format!("Invalid date: {}", e)))?;
        let medium = InteractionMedium::from_db_str(&medium_str)
            .ok_or_else(|| PrmError::Other(format!("Unknown medium: {}", medium_str)))?;

        results.push((
            person_id,
            Interaction {
                id,
                date,
                medium,
                my_location: my_loc,
                their_location: their_loc,
                topics,
                note,
            },
        ));
    }

    Ok(results)
}

pub fn count_by_owner(conn: &Connection, owner_id: Id<crate::model::User>) -> PrmResult<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM interactions i
         JOIN relationships r ON i.relationship_person_id = r.person_id
         WHERE r.network_owner_id = ?1",
        params![owner_id.value.to_string()],
        |row| row.get(0),
    )?;
    Ok(count)
}

fn find_topics(conn: &Connection, interaction_id: Id<Interaction>) -> PrmResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT topic FROM interaction_topics WHERE interaction_id = ?1 ORDER BY topic",
    )?;

    let topics = stmt
        .query_map(params![interaction_id.value.to_string()], |row| {
            row.get(0)
        })?
        .collect::<Result<Vec<String>, _>>()?;

    Ok(topics)
}
