use rusqlite::Connection;

use crate::error::PrmResult;

/// Initialize the database schema. Creates all tables if they don't exist.
pub fn initialize(conn: &Connection) -> PrmResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            email TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS people (
            id TEXT PRIMARY KEY NOT NULL,
            network_owner_id TEXT NOT NULL REFERENCES users(id),
            name TEXT NOT NULL,
            nickname TEXT,
            how_we_met TEXT,
            birthday TEXT,
            notes TEXT,
            location TEXT,
            is_self INTEGER NOT NULL DEFAULT 0,
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS custom_contact_types (
            id TEXT PRIMARY KEY NOT NULL,
            network_owner_id TEXT NOT NULL REFERENCES users(id),
            name TEXT NOT NULL,
            UNIQUE(network_owner_id, name COLLATE NOCASE)
        );

        CREATE TABLE IF NOT EXISTS contact_entries (
            id TEXT PRIMARY KEY NOT NULL,
            person_id TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
            contact_type TEXT NOT NULL,
            custom_type_id TEXT REFERENCES custom_contact_types(id),
            string_value TEXT,
            street TEXT,
            city TEXT,
            state TEXT,
            zip TEXT,
            country TEXT,
            label TEXT
        );

        CREATE TABLE IF NOT EXISTS relationship_labels (
            id TEXT PRIMARY KEY NOT NULL,
            network_owner_id TEXT NOT NULL REFERENCES users(id),
            name TEXT NOT NULL,
            archived INTEGER NOT NULL DEFAULT 0,
            UNIQUE(network_owner_id, name COLLATE NOCASE)
        );

        CREATE TABLE IF NOT EXISTS relationships (
            person_id TEXT PRIMARY KEY NOT NULL REFERENCES people(id) ON DELETE CASCADE,
            network_owner_id TEXT NOT NULL REFERENCES users(id),
            reminder_days INTEGER
        );

        CREATE TABLE IF NOT EXISTS relationship_label_assignments (
            relationship_person_id TEXT NOT NULL REFERENCES relationships(person_id) ON DELETE CASCADE,
            label_id TEXT NOT NULL REFERENCES relationship_labels(id) ON DELETE CASCADE,
            PRIMARY KEY (relationship_person_id, label_id)
        );

        CREATE TABLE IF NOT EXISTS interactions (
            id TEXT PRIMARY KEY NOT NULL,
            relationship_person_id TEXT NOT NULL REFERENCES relationships(person_id) ON DELETE CASCADE,
            date TEXT NOT NULL,
            medium TEXT NOT NULL,
            my_location TEXT NOT NULL,
            their_location TEXT,
            note TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS interaction_topics (
            interaction_id TEXT NOT NULL REFERENCES interactions(id) ON DELETE CASCADE,
            topic TEXT NOT NULL,
            PRIMARY KEY (interaction_id, topic)
        );

        CREATE TABLE IF NOT EXISTS circles (
            id TEXT PRIMARY KEY NOT NULL,
            network_owner_id TEXT NOT NULL REFERENCES users(id),
            name TEXT NOT NULL,
            description TEXT,
            archived INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS circle_members (
            circle_id TEXT NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
            person_id TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
            PRIMARY KEY (circle_id, person_id)
        );

        CREATE TABLE IF NOT EXISTS network_metadata (
            owner_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id),
            self_id TEXT NOT NULL REFERENCES people(id)
        );

        CREATE TABLE IF NOT EXISTS ai_corrections (
            id TEXT PRIMARY KEY NOT NULL,
            owner_id TEXT NOT NULL REFERENCES users(id),
            original_text TEXT NOT NULL,
            ai_output TEXT NOT NULL,
            user_output TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        PRAGMA foreign_keys = ON;
        ",
    )?;
    Ok(())
}

/// Initialize with encryption key (for SQLCipher).
pub fn initialize_encrypted(conn: &Connection, key: &str) -> PrmResult<()> {
    conn.execute_batch(&format!("PRAGMA key = '{}';", key.replace('\'', "''")))?;
    initialize(conn)
}

/// Create an in-memory connection for testing. Available in test builds.
pub fn test_connection() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    initialize(&conn).unwrap();
    conn
}
