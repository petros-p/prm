use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{PrmError, PrmResult};
use crate::model::{Address, ContactEntry, ContactType, ContactValue, CustomContactType, Id, Person, User};

pub fn insert(conn: &Connection, person_id: Id<Person>, entry: &ContactEntry) -> PrmResult<()> {
    let (contact_type_str, custom_type_id) = match &entry.contact_type {
        ContactType::Phone => ("Phone", None),
        ContactType::Email => ("Email", None),
        ContactType::PhysicalAddress => ("PhysicalAddress", None),
        ContactType::Custom { type_id } => ("Custom", Some(type_id.value.to_string())),
    };

    let (string_value, street, city, state, zip, country) = match &entry.value {
        ContactValue::StringValue { value } => (Some(value.as_str()), None, None, None, None, None),
        ContactValue::AddressValue { value } => (
            None,
            Some(value.street.as_str()),
            Some(value.city.as_str()),
            Some(value.state.as_str()),
            Some(value.zip.as_str()),
            Some(value.country.as_str()),
        ),
    };

    conn.execute(
        "INSERT INTO contact_entries (id, person_id, contact_type, custom_type_id, string_value, street, city, state, zip, country, label)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            entry.id.value.to_string(),
            person_id.value.to_string(),
            contact_type_str,
            custom_type_id,
            string_value,
            street,
            city,
            state,
            zip,
            country,
            entry.label,
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, entry_id: Id<ContactEntry>) -> PrmResult<()> {
    conn.execute(
        "DELETE FROM contact_entries WHERE id = ?1",
        params![entry_id.value.to_string()],
    )?;
    Ok(())
}

pub fn update_label(
    conn: &Connection,
    entry_id: Id<ContactEntry>,
    label: Option<&str>,
) -> PrmResult<()> {
    conn.execute(
        "UPDATE contact_entries SET label = ?1 WHERE id = ?2",
        params![label, entry_id.value.to_string()],
    )?;
    Ok(())
}

pub fn find_by_person(conn: &Connection, person_id: Id<Person>) -> PrmResult<Vec<ContactEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, contact_type, custom_type_id, string_value, street, city, state, zip, country, label
         FROM contact_entries WHERE person_id = ?1",
    )?;

    let entries = stmt
        .query_map(params![person_id.value.to_string()], |row| {
            Ok(row_to_contact_entry(row))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
}

pub fn insert_custom_type(
    conn: &Connection,
    owner_id: Id<User>,
    ct: &CustomContactType,
) -> PrmResult<()> {
    conn.execute(
        "INSERT INTO custom_contact_types (id, network_owner_id, name) VALUES (?1, ?2, ?3)",
        params![ct.id.value.to_string(), owner_id.value.to_string(), ct.name],
    )?;
    Ok(())
}

pub fn find_custom_types(
    conn: &Connection,
    owner_id: Id<User>,
) -> PrmResult<Vec<CustomContactType>> {
    let mut stmt = conn.prepare(
        "SELECT id, name FROM custom_contact_types WHERE network_owner_id = ?1 ORDER BY name",
    )?;

    let types = stmt
        .query_map(params![owner_id.value.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok((id_str, name))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|(id_str, name)| {
            Ok(CustomContactType {
                id: Id::new(
                    Uuid::parse_str(&id_str)
                        .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
                ),
                name,
            })
        })
        .collect::<PrmResult<Vec<_>>>()?;

    Ok(types)
}

pub fn find_custom_type_by_name(
    conn: &Connection,
    owner_id: Id<User>,
    name: &str,
) -> PrmResult<Option<CustomContactType>> {
    let mut stmt = conn.prepare(
        "SELECT id, name FROM custom_contact_types WHERE network_owner_id = ?1 AND name = ?2 COLLATE NOCASE",
    )?;

    let result = stmt.query_row(
        params![owner_id.value.to_string(), name],
        |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok((id_str, name))
        },
    );

    match result {
        Ok((id_str, name)) => Ok(Some(CustomContactType {
            id: Id::new(
                Uuid::parse_str(&id_str)
                    .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
            ),
            name,
        })),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn row_to_contact_entry(row: &rusqlite::Row) -> PrmResult<ContactEntry> {
    let id_str: String = row.get(0).map_err(rusqlite::Error::from)?;
    let contact_type_str: String = row.get(1).map_err(rusqlite::Error::from)?;
    let custom_type_id_str: Option<String> = row.get(2).map_err(rusqlite::Error::from)?;
    let string_value: Option<String> = row.get(3).map_err(rusqlite::Error::from)?;
    let street: Option<String> = row.get(4).map_err(rusqlite::Error::from)?;
    let city: Option<String> = row.get(5).map_err(rusqlite::Error::from)?;
    let state: Option<String> = row.get(6).map_err(rusqlite::Error::from)?;
    let zip: Option<String> = row.get(7).map_err(rusqlite::Error::from)?;
    let country: Option<String> = row.get(8).map_err(rusqlite::Error::from)?;
    let label: Option<String> = row.get(9).map_err(rusqlite::Error::from)?;

    let contact_type = match contact_type_str.as_str() {
        "Phone" => ContactType::Phone,
        "Email" => ContactType::Email,
        "PhysicalAddress" => ContactType::PhysicalAddress,
        "Custom" => {
            let type_id_str = custom_type_id_str
                .ok_or_else(|| PrmError::Other("Custom contact type missing type_id".into()))?;
            ContactType::Custom {
                type_id: Id::new(
                    Uuid::parse_str(&type_id_str)
                        .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
                ),
            }
        }
        other => return Err(PrmError::Other(format!("Unknown contact type: {}", other))),
    };

    let value = if contact_type_str == "PhysicalAddress" {
        ContactValue::AddressValue {
            value: Address {
                street: street.unwrap_or_default(),
                city: city.unwrap_or_default(),
                state: state.unwrap_or_default(),
                zip: zip.unwrap_or_default(),
                country: country.unwrap_or_default(),
            },
        }
    } else {
        ContactValue::StringValue {
            value: string_value.unwrap_or_default(),
        }
    };

    Ok(ContactEntry {
        id: Id::new(
            Uuid::parse_str(&id_str)
                .map_err(|e| PrmError::Other(format!("Invalid UUID: {}", e)))?,
        ),
        contact_type,
        value,
        label,
    })
}
