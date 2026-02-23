use chrono::Local;
use rusqlite::Connection;

use crate::db::{circle_repo, contact_repo, interaction_repo, person_repo, relationship_repo};
use crate::error::PrmResult;
use crate::model::{Id, User};
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
}

pub fn stats(conn: &Connection, owner_id: Id<User>) -> PrmResult<NetworkStats> {
    let all_people = person_repo::find_by_owner(conn, owner_id)?;
    let active = all_people.iter().filter(|p| !p.archived).count();
    let archived = all_people.iter().filter(|p| p.archived).count();

    let rels = relationship_repo::find_by_owner(conn, owner_id)?;
    let total_interactions = interaction_repo::count_by_owner(conn, owner_id)?;

    let all_circles = circle_repo::find_by_owner(conn, owner_id)?;
    let active_circles = all_circles.iter().filter(|c| !c.archived).count();
    let archived_circles = all_circles.iter().filter(|c| c.archived).count();

    let today = Local::now().date_naive();
    let overdue = reminder_queries::people_needing_reminder(conn, owner_id, today)?;

    let custom_types = contact_repo::find_custom_types(conn, owner_id)?;

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
    })
}
