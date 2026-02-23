use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::PrmResult;
use crate::model::{Id, User};

pub struct CorrectionRecord {
    pub original_text: String,
    pub ai_output: String,
    pub user_output: String,
}

pub fn insert(
    conn: &Connection,
    owner_id: Id<User>,
    original_text: &str,
    ai_output: &str,
    user_output: &str,
) -> PrmResult<()> {
    conn.execute(
        "INSERT INTO ai_corrections (id, owner_id, original_text, ai_output, user_output, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
        params![
            Uuid::new_v4().to_string(),
            owner_id.value.to_string(),
            original_text,
            ai_output,
            user_output,
        ],
    )?;
    Ok(())
}

pub fn recent(
    conn: &Connection,
    owner_id: Id<User>,
    limit: usize,
) -> PrmResult<Vec<CorrectionRecord>> {
    let mut stmt = conn.prepare(
        "SELECT original_text, ai_output, user_output
         FROM ai_corrections
         WHERE owner_id = ?1
         ORDER BY created_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![owner_id.value.to_string(), limit as i64], |row| {
        Ok(CorrectionRecord {
            original_text: row.get(0)?,
            ai_output: row.get(1)?,
            user_output: row.get(2)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
