use chrono::NaiveDate;
use rusqlite::Connection;

use crate::db::{interaction_repo, person_repo, relationship_repo};
use crate::error::{PrmError, PrmResult};
use crate::model::{Id, Interaction, InteractionMedium, Person, Relationship, User};
use crate::validation::{self, trim_optional};

pub fn log_in_person(
    conn: &Connection,
    owner_id: Id<User>,
    person_id: Id<Person>,
    location: &str,
    topics: Vec<String>,
    note: Option<&str>,
    date: NaiveDate,
) -> PrmResult<Interaction> {
    person_repo::find_by_id(conn, person_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Person".into(),
            id: person_id.to_string(),
        })?;

    let valid_location = validation::non_blank(location, "location")?;
    let valid_topics: Vec<String> = topics
        .into_iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    validation::non_empty_set(&valid_topics, "topics")?;

    // Ensure relationship exists
    ensure_relationship(conn, owner_id, person_id)?;

    let interaction = Interaction::create_in_person(
        valid_location,
        valid_topics,
        trim_optional(note),
        date,
    );

    interaction_repo::insert(conn, person_id, &interaction)?;
    Ok(interaction)
}

pub fn log_remote(
    conn: &Connection,
    owner_id: Id<User>,
    person_id: Id<Person>,
    medium: InteractionMedium,
    my_location: &str,
    their_location: Option<&str>,
    topics: Vec<String>,
    note: Option<&str>,
    date: NaiveDate,
) -> PrmResult<Interaction> {
    if medium == InteractionMedium::InPerson {
        return Err(PrmError::UseInPersonMethod);
    }

    person_repo::find_by_id(conn, person_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Person".into(),
            id: person_id.to_string(),
        })?;

    let valid_my_location = validation::non_blank(my_location, "myLocation")?;
    let valid_topics: Vec<String> = topics
        .into_iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    validation::non_empty_set(&valid_topics, "topics")?;

    ensure_relationship(conn, owner_id, person_id)?;

    let interaction = Interaction::create_remote(
        medium,
        valid_my_location,
        trim_optional(their_location),
        valid_topics,
        trim_optional(note),
        date,
    );

    interaction_repo::insert(conn, person_id, &interaction)?;
    Ok(interaction)
}

fn ensure_relationship(
    conn: &Connection,
    owner_id: Id<User>,
    person_id: Id<Person>,
) -> PrmResult<()> {
    if relationship_repo::find_by_person(conn, person_id)?.is_none() {
        let rel = Relationship::create(person_id);
        relationship_repo::upsert(conn, owner_id, &rel)?;
    }
    Ok(())
}
