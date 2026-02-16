use chrono::NaiveDate;
use rusqlite::Connection;

use crate::db::{interaction_repo, person_repo, relationship_repo};
use crate::error::PrmResult;
use crate::model::{Id, Person, Relationship, User};

/// Whether someone is overdue for contact and by how much.
#[derive(Debug, Clone)]
pub enum OverdueStatus {
    /// Person has been contacted before; days is positive if overdue, negative if not yet due.
    DaysOverdue(i64),
    /// Person has never been contacted.
    NeverContacted,
}

/// Information about a relationship's reminder status.
#[derive(Debug, Clone)]
pub struct ReminderStatus {
    pub person: Person,
    pub relationship: Relationship,
    pub reminder_days: i32,
    pub days_since_last_interaction: Option<i64>,
    pub overdue_status: OverdueStatus,
}

pub fn reminder_status(
    conn: &Connection,
    person_id: Id<Person>,
    as_of: NaiveDate,
) -> PrmResult<Option<ReminderStatus>> {
    let person = match person_repo::find_by_id(conn, person_id)? {
        Some(p) => p,
        None => return Ok(None),
    };

    let rel = match relationship_repo::find_by_person(conn, person_id)? {
        Some(r) => r,
        None => return Ok(None),
    };

    let reminder_days = match rel.reminder_days {
        Some(d) => d,
        None => return Ok(None),
    };

    let last_date = interaction_repo::find_last_interaction_date(conn, person_id)?;
    let days_since = last_date.map(|d| (as_of - d).num_days());

    let overdue_status = match days_since {
        Some(d) => OverdueStatus::DaysOverdue(d - reminder_days as i64),
        None => OverdueStatus::NeverContacted,
    };

    Ok(Some(ReminderStatus {
        person,
        relationship: rel,
        reminder_days,
        days_since_last_interaction: days_since,
        overdue_status,
    }))
}

pub fn people_needing_reminder(
    conn: &Connection,
    owner_id: Id<User>,
    as_of: NaiveDate,
) -> PrmResult<Vec<ReminderStatus>> {
    let rels = relationship_repo::find_by_owner(conn, owner_id)?;
    let mut results = Vec::new();

    for rel in rels {
        if let Some(status) = reminder_status(conn, rel.person_id, as_of)? {
            let is_overdue = match &status.overdue_status {
                OverdueStatus::NeverContacted => true,
                OverdueStatus::DaysOverdue(days) => *days > 0,
            };

            if is_overdue && !status.person.archived {
                results.push(status);
            }
        }
    }

    results.sort_by(|a, b| {
        let a_key = match &a.overdue_status {
            OverdueStatus::NeverContacted => i64::MIN,
            OverdueStatus::DaysOverdue(d) => -d,
        };
        let b_key = match &b.overdue_status {
            OverdueStatus::NeverContacted => i64::MIN,
            OverdueStatus::DaysOverdue(d) => -d,
        };
        a_key.cmp(&b_key)
    });

    Ok(results)
}

pub fn all_reminders(
    conn: &Connection,
    owner_id: Id<User>,
    as_of: NaiveDate,
) -> PrmResult<Vec<ReminderStatus>> {
    let rels = relationship_repo::find_by_owner(conn, owner_id)?;
    let mut results = Vec::new();

    for rel in rels {
        if let Some(status) = reminder_status(conn, rel.person_id, as_of)? {
            if !status.person.archived {
                results.push(status);
            }
        }
    }

    results.sort_by(|a, b| {
        let a_key = match &a.overdue_status {
            OverdueStatus::NeverContacted => i64::MIN,
            OverdueStatus::DaysOverdue(d) => -d,
        };
        let b_key = match &b.overdue_status {
            OverdueStatus::NeverContacted => i64::MIN,
            OverdueStatus::DaysOverdue(d) => -d,
        };
        a_key.cmp(&b_key)
    });

    Ok(results)
}
