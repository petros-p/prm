use rusqlite::Connection;

use crate::db::{contact_repo, person_repo};
use crate::error::PrmResult;
use crate::model::{ContactEntry, ContactType, CustomContactType, Id, Person, User};

pub fn phones_for(conn: &Connection, person_id: Id<Person>) -> PrmResult<Vec<ContactEntry>> {
    Ok(contact_repo::find_by_person(conn, person_id)?
        .into_iter()
        .filter(|e| matches!(e.contact_type, ContactType::Phone))
        .collect())
}

pub fn emails_for(conn: &Connection, person_id: Id<Person>) -> PrmResult<Vec<ContactEntry>> {
    Ok(contact_repo::find_by_person(conn, person_id)?
        .into_iter()
        .filter(|e| matches!(e.contact_type, ContactType::Email))
        .collect())
}

pub fn addresses_for(conn: &Connection, person_id: Id<Person>) -> PrmResult<Vec<ContactEntry>> {
    Ok(contact_repo::find_by_person(conn, person_id)?
        .into_iter()
        .filter(|e| matches!(e.contact_type, ContactType::PhysicalAddress))
        .collect())
}

pub fn custom_contacts_for(
    conn: &Connection,
    person_id: Id<Person>,
    type_id: Id<CustomContactType>,
) -> PrmResult<Vec<ContactEntry>> {
    Ok(contact_repo::find_by_person(conn, person_id)?
        .into_iter()
        .filter(|e| matches!(&e.contact_type, ContactType::Custom { type_id: tid } if *tid == type_id))
        .collect())
}

pub fn people_with_custom_contact_type(
    conn: &Connection,
    owner_id: Id<User>,
    type_id: Id<CustomContactType>,
) -> PrmResult<Vec<Person>> {
    let people = person_repo::find_active_by_owner(conn, owner_id)?;
    let mut result = Vec::new();

    for person in people {
        let contacts = contact_repo::find_by_person(conn, person.id)?;
        if contacts.iter().any(|e| matches!(&e.contact_type, ContactType::Custom { type_id: tid } if *tid == type_id)) {
            result.push(person);
        }
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

pub fn custom_contact_type_name(
    conn: &Connection,
    owner_id: Id<User>,
    type_id: Id<CustomContactType>,
) -> PrmResult<Option<String>> {
    let types = contact_repo::find_custom_types(conn, owner_id)?;
    Ok(types.into_iter().find(|t| t.id == type_id).map(|t| t.name))
}
