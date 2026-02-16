use chrono::NaiveDate;
use rusqlite::Connection;

use crate::db::{interaction_repo, person_repo, relationship_repo};
use crate::error::PrmResult;
use crate::model::{Id, Interaction, Person, User};

pub fn interactions_with(
    conn: &Connection,
    person_id: Id<Person>,
) -> PrmResult<Vec<Interaction>> {
    interaction_repo::find_by_person(conn, person_id)
}

pub fn last_interaction_with(
    conn: &Connection,
    person_id: Id<Person>,
) -> PrmResult<Option<Interaction>> {
    let interactions = interaction_repo::find_by_person(conn, person_id)?;
    Ok(interactions.into_iter().next())
}

pub fn last_interaction_date(
    conn: &Connection,
    person_id: Id<Person>,
) -> PrmResult<Option<NaiveDate>> {
    interaction_repo::find_last_interaction_date(conn, person_id)
}

pub fn days_since_interaction(
    conn: &Connection,
    person_id: Id<Person>,
    as_of: NaiveDate,
) -> PrmResult<Option<i64>> {
    let last_date = interaction_repo::find_last_interaction_date(conn, person_id)?;
    Ok(last_date.map(|d| (as_of - d).num_days()))
}

pub fn interactions_in_range(
    conn: &Connection,
    owner_id: Id<User>,
    from: NaiveDate,
    to: NaiveDate,
) -> PrmResult<Vec<(Person, Interaction)>> {
    let rows = interaction_repo::find_in_date_range(conn, owner_id, from, to)?;
    let mut results = Vec::new();

    for (person_id, interaction) in rows {
        if let Some(person) = person_repo::find_by_id(conn, person_id)? {
            results.push((person, interaction));
        }
    }

    Ok(results)
}

/// Gets people you haven't interacted with in a given number of days.
/// Unlike reminders, this checks ALL relationships, not just those with reminders set.
pub fn not_contacted_in(
    conn: &Connection,
    owner_id: Id<User>,
    days: i64,
    as_of: NaiveDate,
) -> PrmResult<Vec<(Person, Option<i64>)>> {
    let rels = relationship_repo::find_by_owner(conn, owner_id)?;
    let mut results = Vec::new();

    for rel in rels {
        if let Some(person) = person_repo::find_by_id(conn, rel.person_id)? {
            if person.archived {
                continue;
            }
            let days_since = days_since_interaction(conn, rel.person_id, as_of)?;
            if days_since.map_or(true, |d| d >= days) {
                results.push((person, days_since));
            }
        }
    }

    results.sort_by(|a, b| {
        let a_val = a.1.unwrap_or(i64::MAX);
        let b_val = b.1.unwrap_or(i64::MAX);
        b_val.cmp(&a_val)
    });

    Ok(results)
}
