use rusqlite::Connection;

use crate::db::person_repo;
use crate::error::PrmResult;
use crate::model::{Id, Person, User};

pub fn active_people(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<Person>> {
    person_repo::find_active_by_owner(conn, owner_id)
}

pub fn archived_people(conn: &Connection, owner_id: Id<User>) -> PrmResult<Vec<Person>> {
    person_repo::find_archived_by_owner(conn, owner_id)
}

pub fn get_self(conn: &Connection, owner_id: Id<User>) -> PrmResult<Option<Person>> {
    person_repo::find_self(conn, owner_id)
}

pub fn find_by_name(conn: &Connection, owner_id: Id<User>, query: &str) -> PrmResult<Vec<Person>> {
    person_repo::find_by_name(conn, owner_id, query)
}

pub fn get_person(conn: &Connection, person_id: Id<Person>) -> PrmResult<Option<Person>> {
    person_repo::find_by_id(conn, person_id)
}
