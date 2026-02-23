use rusqlite::Connection;

use crate::db::{contact_repo, person_repo};
use crate::error::{PrmError, PrmResult};
use crate::model::{Address, ContactEntry, CustomContactType, Id, Person, User};
use crate::validation::{self, trim_optional};

pub fn add_phone(
    conn: &Connection,
    person_id: Id<Person>,
    number: &str,
    label: Option<&str>,
) -> PrmResult<ContactEntry> {
    ensure_person_exists(conn, person_id)?;
    let valid_number = validation::non_blank(number, "phone number")?;
    let entry = ContactEntry::phone(valid_number, trim_optional(label));
    contact_repo::insert(conn, person_id, &entry)?;
    Ok(entry)
}

pub fn add_email(
    conn: &Connection,
    person_id: Id<Person>,
    email: &str,
    label: Option<&str>,
) -> PrmResult<ContactEntry> {
    ensure_person_exists(conn, person_id)?;
    let valid_email = validation::non_blank(email, "email")?;
    let entry = ContactEntry::email(valid_email, trim_optional(label));
    contact_repo::insert(conn, person_id, &entry)?;
    Ok(entry)
}

pub fn add_address(
    conn: &Connection,
    person_id: Id<Person>,
    street: &str,
    city: &str,
    state: &str,
    zip: &str,
    country: &str,
    label: Option<&str>,
) -> PrmResult<ContactEntry> {
    ensure_person_exists(conn, person_id)?;
    let address = Address {
        street: validation::non_blank(street, "street")?,
        city: validation::non_blank(city, "city")?,
        state: validation::non_blank(state, "state")?,
        zip: validation::non_blank(zip, "zip")?,
        country: validation::non_blank(country, "country")?,
    };
    let entry = ContactEntry::address(address, trim_optional(label));
    contact_repo::insert(conn, person_id, &entry)?;
    Ok(entry)
}

pub fn add_custom_contact(
    conn: &Connection,
    person_id: Id<Person>,
    type_id: Id<CustomContactType>,
    value: &str,
    label: Option<&str>,
) -> PrmResult<ContactEntry> {
    ensure_person_exists(conn, person_id)?;
    let valid_value = validation::non_blank(value, "value")?;
    let entry = ContactEntry::custom(type_id, valid_value, trim_optional(label));
    contact_repo::insert(conn, person_id, &entry)?;
    Ok(entry)
}

pub fn remove_contact(conn: &Connection, entry_id: Id<ContactEntry>) -> PrmResult<()> {
    contact_repo::delete(conn, entry_id)
}

pub fn update_contact_label(
    conn: &Connection,
    entry_id: Id<ContactEntry>,
    label: Option<&str>,
) -> PrmResult<()> {
    let trimmed = trim_optional(label);
    contact_repo::update_label(conn, entry_id, trimmed.as_deref())
}

pub fn create_custom_contact_type(
    conn: &Connection,
    owner_id: Id<User>,
    name: &str,
) -> PrmResult<CustomContactType> {
    let valid_name = validation::non_blank(name, "name")?;

    // Check if already exists
    if contact_repo::find_custom_type_by_name(conn, owner_id, &valid_name)?.is_some() {
        return Err(PrmError::AlreadyExists {
            entity_type: "CustomContactType".into(),
            identifier: valid_name,
        });
    }

    let ct = CustomContactType::create(valid_name);
    contact_repo::insert_custom_type(conn, owner_id, &ct)?;
    Ok(ct)
}

fn ensure_person_exists(conn: &Connection, person_id: Id<Person>) -> PrmResult<()> {
    person_repo::find_by_id(conn, person_id)?
        .ok_or_else(|| PrmError::NotFound {
            entity_type: "Person".into(),
            id: person_id.to_string(),
        })?;
    Ok(())
}
