use std::path::Path;

use rusqlite::Connection;
use serde_json::Value;

use crate::db::{
    circle_repo, contact_repo, interaction_repo, network_repo, person_repo, relationship_repo,
    schema,
};
use crate::error::{PrmError, PrmResult};
use crate::model::*;

/// Imports a Scala PRM JSON network file into a SQLite database.
/// Returns the number of people imported.
pub fn import_json(json_path: &Path, db_path: &Path) -> PrmResult<ImportStats> {
    let json_str = std::fs::read_to_string(json_path)?;
    let json: Value = serde_json::from_str(&json_str)?;

    let conn = Connection::open(db_path)?;
    schema::initialize(&conn)?;

    import_network(&conn, &json)
}

#[derive(Debug)]
pub struct ImportStats {
    pub people: usize,
    pub relationships: usize,
    pub interactions: usize,
    pub circles: usize,
    pub labels: usize,
    pub custom_contact_types: usize,
}

fn import_network(conn: &Connection, json: &Value) -> PrmResult<ImportStats> {
    let owner_id_str = json["ownerId"].as_str()
        .ok_or_else(|| PrmError::Other("Missing ownerId".into()))?;
    let self_id_str = json["selfId"].as_str()
        .ok_or_else(|| PrmError::Other("Missing selfId".into()))?;

    // Create user
    let user = User {
        id: parse_id(owner_id_str)?,
        name: String::new(), // Will be updated from self person
        email: String::new(),
    };
    network_repo::insert_user(conn, &user)?;

    let self_id: Id<Person> = parse_id(self_id_str)?;

    // Import custom contact types first (needed by contact entries)
    let mut custom_type_count = 0;
    if let Some(types) = json["customContactTypes"].as_object() {
        for (_, ct_val) in types {
            let ct = CustomContactType {
                id: parse_id(ct_val["id"].as_str().unwrap_or(""))?,
                name: ct_val["name"].as_str().unwrap_or("").to_string(),
            };
            contact_repo::insert_custom_type(conn, user.id, &ct)?;
            custom_type_count += 1;
        }
    }

    // Import relationship labels
    let mut label_count = 0;
    if let Some(labels) = json["relationshipLabels"].as_object() {
        for (_, label_val) in labels {
            let label = RelationshipLabel {
                id: parse_id(label_val["id"].as_str().unwrap_or(""))?,
                name: label_val["name"].as_str().unwrap_or("").to_string(),
                archived: label_val["archived"].as_bool().unwrap_or(false),
            };
            relationship_repo::insert_label(conn, user.id, &label)?;
            label_count += 1;
        }
    }

    // Import people
    let mut people_count = 0;
    let mut user_name = String::new();
    if let Some(people) = json["people"].as_object() {
        for (_, person_val) in people {
            let person = parse_person(person_val)?;
            if person.id == self_id {
                user_name = person.name.clone();
            }
            person_repo::insert(conn, user.id, &person)?;

            // Import contact entries embedded in person
            if let Some(contacts) = person_val["contactInfo"].as_array() {
                for contact_val in contacts {
                    let entry = parse_contact_entry(contact_val)?;
                    contact_repo::insert(conn, person.id, &entry)?;
                }
            }

            people_count += 1;
        }
    }

    // Update user name from self person
    if !user_name.is_empty() {
        conn.execute(
            "UPDATE users SET name = ?1 WHERE id = ?2",
            rusqlite::params![user_name, owner_id_str],
        )?;
    }

    // Set network metadata
    network_repo::set_network_metadata(conn, user.id, self_id)?;

    // Import relationships
    let mut rel_count = 0;
    let mut interaction_count = 0;
    if let Some(rels) = json["relationships"].as_object() {
        for (_, rel_val) in rels {
            let person_id: Id<Person> = parse_id(rel_val["personId"].as_str().unwrap_or(""))?;

            let label_ids: Vec<Id<RelationshipLabel>> = match &rel_val["labels"] {
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| parse_id(s).ok())
                    .collect(),
                // Handle Set serialized as object keys
                Value::Object(obj) => obj
                    .keys()
                    .filter_map(|k| parse_id(k).ok())
                    .collect(),
                _ => Vec::new(),
            };

            let reminder_days = rel_val["reminderDays"]
                .as_i64()
                .map(|d| d as i32);

            let rel = Relationship {
                person_id,
                labels: label_ids,
                reminder_days,
            };
            relationship_repo::upsert(conn, user.id, &rel)?;
            rel_count += 1;

            // Import interactions
            if let Some(interactions) = rel_val["interactionHistory"].as_array() {
                for int_val in interactions {
                    let interaction = parse_interaction(int_val)?;
                    interaction_repo::insert(conn, person_id, &interaction)?;
                    interaction_count += 1;
                }
            }
        }
    }

    // Import circles
    let mut circle_count = 0;
    if let Some(circles) = json["circles"].as_object() {
        for (_, circle_val) in circles {
            let circle = parse_circle(circle_val)?;
            circle_repo::insert(conn, user.id, &circle)?;

            // Add members
            let member_ids: Vec<Id<Person>> = match &circle_val["memberIds"] {
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| parse_id(s).ok())
                    .collect(),
                Value::Object(obj) => obj
                    .keys()
                    .filter_map(|k| parse_id(k).ok())
                    .collect(),
                _ => Vec::new(),
            };
            if !member_ids.is_empty() {
                let _ = circle_repo::add_members(conn, circle.id, &member_ids);
            }
            circle_count += 1;
        }
    }

    Ok(ImportStats {
        people: people_count,
        relationships: rel_count,
        interactions: interaction_count,
        circles: circle_count,
        labels: label_count,
        custom_contact_types: custom_type_count,
    })
}

fn parse_id<T>(s: &str) -> PrmResult<Id<T>> {
    let uuid = uuid::Uuid::parse_str(s)
        .map_err(|e| PrmError::Other(format!("Invalid UUID '{}': {}", s, e)))?;
    Ok(Id::new(uuid))
}

fn parse_person(val: &Value) -> PrmResult<Person> {
    let id = parse_id(val["id"].as_str().unwrap_or(""))?;
    let birthday = val["birthday"]
        .as_str()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    // Support both "location" and legacy "defaultLocation"
    let location = val["location"]
        .as_str()
        .or_else(|| val["defaultLocation"].as_str())
        .map(|s| s.to_string());

    Ok(Person {
        id,
        name: val["name"].as_str().unwrap_or("").to_string(),
        nickname: val["nickname"].as_str().map(|s| s.to_string()),
        how_we_met: val["howWeMet"].as_str().map(|s| s.to_string()),
        birthday,
        notes: val["notes"].as_str().map(|s| s.to_string()),
        location,
        is_self: val["isSelf"].as_bool().unwrap_or(false),
        archived: val["archived"].as_bool().unwrap_or(false),
    })
}

fn parse_contact_entry(val: &Value) -> PrmResult<ContactEntry> {
    let id = parse_id(val["id"].as_str().unwrap_or(""))?;
    let label = val["label"].as_str().map(|s| s.to_string());

    let contact_type_val = &val["contactType"];
    let contact_type = match contact_type_val["type"].as_str().unwrap_or("") {
        "Phone" => ContactType::Phone,
        "Email" => ContactType::Email,
        "PhysicalAddress" => ContactType::PhysicalAddress,
        "Custom" => {
            let type_id = parse_id(contact_type_val["typeId"].as_str().unwrap_or(""))?;
            ContactType::Custom { type_id }
        }
        other => return Err(PrmError::Other(format!("Unknown contact type: {}", other))),
    };

    let value_val = &val["value"];
    let value = match value_val["type"].as_str().unwrap_or("") {
        "String" => ContactValue::StringValue {
            value: value_val["value"].as_str().unwrap_or("").to_string(),
        },
        "Address" => ContactValue::AddressValue {
            value: Address {
                street: value_val["street"].as_str().unwrap_or("").to_string(),
                city: value_val["city"].as_str().unwrap_or("").to_string(),
                state: value_val["state"].as_str().unwrap_or("").to_string(),
                zip: value_val["zip"].as_str().unwrap_or("").to_string(),
                country: value_val["country"].as_str().unwrap_or("").to_string(),
            },
        },
        other => return Err(PrmError::Other(format!("Unknown contact value type: {}", other))),
    };

    Ok(ContactEntry {
        id,
        contact_type,
        value,
        label,
    })
}

fn parse_interaction(val: &Value) -> PrmResult<Interaction> {
    let id = parse_id(val["id"].as_str().unwrap_or(""))?;

    let date = val["date"]
        .as_str()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Local::now().date_naive());

    let medium = match val["medium"].as_str().unwrap_or("InPerson") {
        "InPerson" => InteractionMedium::InPerson,
        "Text" => InteractionMedium::Text,
        "PhoneCall" => InteractionMedium::PhoneCall,
        "VideoCall" => InteractionMedium::VideoCall,
        "SocialMedia" => InteractionMedium::SocialMedia,
        _ => InteractionMedium::InPerson,
    };

    let topics: Vec<String> = match &val["topics"] {
        Value::Array(arr) => arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
        // Handle Set serialized as object keys
        Value::Object(obj) => obj.keys().cloned().collect(),
        _ => Vec::new(),
    };

    Ok(Interaction {
        id,
        date,
        medium,
        my_location: val["myLocation"].as_str().unwrap_or("").to_string(),
        their_location: val["theirLocation"].as_str().map(|s| s.to_string()),
        topics,
        note: val["note"].as_str().map(|s| s.to_string()),
    })
}

fn parse_circle(val: &Value) -> PrmResult<Circle> {
    Ok(Circle {
        id: parse_id(val["id"].as_str().unwrap_or(""))?,
        name: val["name"].as_str().unwrap_or("").to_string(),
        description: val["description"].as_str().map(|s| s.to_string()),
        member_ids: Vec::new(), // Members added separately
        archived: val["archived"].as_bool().unwrap_or(false),
    })
}
