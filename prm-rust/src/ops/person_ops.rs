use chrono::NaiveDate;
use rusqlite::Connection;

use crate::db::{person_repo, relationship_repo};
use crate::error::{PrmError, PrmResult};
use crate::model::{Id, Person, Relationship, User};
use crate::validation::{self, trim_optional};

pub fn add_person(
    conn: &Connection,
    owner_id: Id<User>,
    name: &str,
    nickname: Option<&str>,
    how_we_met: Option<&str>,
    birthday: Option<NaiveDate>,
    notes: Option<&str>,
    location: Option<&str>,
) -> PrmResult<Person> {
    let valid_name = validation::non_blank(name, "name")?;

    let mut person = Person::create(valid_name);
    person.nickname = trim_optional(nickname);
    person.how_we_met = trim_optional(how_we_met);
    person.birthday = birthday;
    person.notes = trim_optional(notes);
    person.location = trim_optional(location);

    person_repo::insert(conn, owner_id, &person)?;

    // Create a default relationship
    let rel = Relationship::create(person.id);
    relationship_repo::upsert(conn, owner_id, &rel)?;

    Ok(person)
}

pub fn update_person(
    conn: &Connection,
    person_id: Id<Person>,
    name: Option<&str>,
    nickname: Option<Option<&str>>,
    how_we_met: Option<Option<&str>>,
    birthday: Option<Option<NaiveDate>>,
    notes: Option<Option<&str>>,
    location: Option<Option<&str>>,
) -> PrmResult<Person> {
    let person = person_repo::find_by_id(conn, person_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Person".into(),
            id: person_id.to_string(),
        })?;

    let mut updated = person;

    if let Some(n) = name {
        updated.name = validation::non_blank(n, "name")?;
    }
    if let Some(nick) = nickname {
        updated.nickname = trim_optional(nick);
    }
    if let Some(hwm) = how_we_met {
        updated.how_we_met = trim_optional(hwm);
    }
    if let Some(bd) = birthday {
        updated.birthday = bd;
    }
    if let Some(n) = notes {
        updated.notes = trim_optional(n);
    }
    if let Some(loc) = location {
        updated.location = trim_optional(loc);
    }

    person_repo::update(conn, &updated)?;
    Ok(updated)
}

pub fn archive_person(conn: &Connection, person_id: Id<Person>) -> PrmResult<Person> {
    let person = person_repo::find_by_id(conn, person_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Person".into(),
            id: person_id.to_string(),
        })?;

    if person.is_self {
        return Err(PrmError::CannotArchiveSelf);
    }

    let mut updated = person;
    updated.archived = true;
    person_repo::update(conn, &updated)?;
    Ok(updated)
}

pub fn unarchive_person(conn: &Connection, person_id: Id<Person>) -> PrmResult<Person> {
    let person = person_repo::find_by_id(conn, person_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Person".into(),
            id: person_id.to_string(),
        })?;

    let mut updated = person;
    updated.archived = false;
    person_repo::update(conn, &updated)?;
    Ok(updated)
}
