use chrono::Local;
use rusqlite::Connection;

use crate::db::{circle_repo, contact_repo, interaction_repo, person_repo, relationship_repo};
use crate::error::PrmResult;
use crate::model::{Id, Person, User};
use crate::queries::reminder_queries;

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_people: usize,
    pub active_people: usize,
    pub archived_people: usize,
    pub total_relationships: usize,
    pub total_interactions: i64,
    pub total_circles: usize,
    pub active_circles: usize,
    pub archived_circles: usize,
    pub reminders_overdue: usize,
    pub custom_contact_types: usize,
    /// Active people (excluding self) with no recorded interactions.
    pub never_contacted: usize,
    /// Active people (excluding self) with no reminder set.
    pub no_reminder_set: usize,
    /// The active person with the longest gap since last contact, and how many days ago.
    pub longest_gap: Option<(String, i64)>,
}

pub fn stats(conn: &Connection, owner_id: Id<User>, self_id: Id<Person>) -> PrmResult<NetworkStats> {
    let all_people = person_repo::find_by_owner(conn, owner_id)?;
    let active = all_people.iter().filter(|p| !p.archived && p.id != self_id).count();
    let archived = all_people.iter().filter(|p| p.archived && p.id != self_id).count();

    let rels = relationship_repo::find_by_owner(conn, owner_id)?;
    let total_interactions = interaction_repo::count_by_owner(conn, owner_id)?;

    let all_circles = circle_repo::find_by_owner(conn, owner_id)?;
    let active_circles = all_circles.iter().filter(|c| !c.archived).count();
    let archived_circles = all_circles.iter().filter(|c| c.archived).count();

    let today = Local::now().date_naive();
    let overdue = reminder_queries::people_needing_reminder(conn, owner_id, today)?;

    let custom_types = contact_repo::find_custom_types(conn, owner_id)?;

    // Compute actionable metrics over active non-self people.
    let active_non_self: Vec<_> = all_people.iter()
        .filter(|p| !p.archived && p.id != self_id)
        .collect();

    let mut never_contacted = 0usize;
    let mut no_reminder_set = 0usize;
    let mut longest_gap: Option<(String, i64)> = None;

    for person in &active_non_self {
        let last_date = interaction_repo::find_last_interaction_date(conn, person.id)?;
        match last_date {
            None => never_contacted += 1,
            Some(d) => {
                let days = (today - d).num_days();
                if days > 0 && longest_gap.as_ref().map_or(true, |(_, g)| days > *g) {
                    longest_gap = Some((person.name.clone(), days));
                }
            }
        }

        let has_reminder = rels.iter()
            .find(|r| r.person_id == person.id)
            .and_then(|r| r.reminder_days)
            .is_some();
        if !has_reminder {
            no_reminder_set += 1;
        }
    }

    Ok(NetworkStats {
        total_people: all_people.len(),
        active_people: active,
        archived_people: archived,
        total_relationships: rels.len(),
        total_interactions,
        total_circles: all_circles.len(),
        active_circles,
        archived_circles,
        reminders_overdue: overdue.len(),
        custom_contact_types: custom_types.len(),
        never_contacted,
        no_reminder_set,
        longest_gap,
    })
}
