use crate::domain::mailing::{List, Mailing, NewList, NewMailing};
use crate::errors::ServerError;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

/// Creates a new list (container for recipients).
pub fn create_list(conn: &Connection, new_list: &NewList) -> Result<i64, ServerError> {
    let now = Utc::now().naive_utc();
    conn.execute(
        "INSERT INTO lists (user_id, name, source_type, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![new_list.user_id, new_list.name, new_list.source_type, now],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Retrieves all lists for a specific user.
pub fn get_lists_for_user(conn: &Connection, user_id: i64) -> Result<Vec<List>, ServerError> {
    let mut stmt = conn.prepare(
        "SELECT id, user_id, name, source_type, created_at FROM lists WHERE user_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![user_id], |row| {
        Ok(List {
            id: row.get(0)?,
            user_id: row.get(1)?,
            name: row.get(2)?,
            source_type: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;

    let mut lists = Vec::new();
    for row in rows {
        lists.push(row?);
    }
    Ok(lists)
}

/// Creates a new mailing (operational batch).
pub fn create_mailing(conn: &Connection, new_mailing: &NewMailing) -> Result<i64, ServerError> {
    let now = Utc::now().naive_utc();
    conn.execute(
        "INSERT INTO mailings (campaign_id, list_id, status, created_at, scheduled_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            new_mailing.campaign_id,
            new_mailing.list_id,
            new_mailing.status,
            now,
            new_mailing.scheduled_at
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Retrieves all mailings for a specific campaign.
pub fn get_mailings_for_campaign(
    conn: &Connection,
    campaign_id: i64,
) -> Result<Vec<Mailing>, ServerError> {
    let mut stmt = conn.prepare(
        "SELECT id, campaign_id, list_id, status, created_at, scheduled_at FROM mailings WHERE campaign_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![campaign_id], |row| {
        Ok(Mailing {
            id: row.get(0)?,
            campaign_id: row.get(1)?,
            list_id: row.get(2)?,
            status: row.get(3)?,
            created_at: row.get(4)?,
            scheduled_at: row.get(5)?,
        })
    })?;

    let mut mailings = Vec::new();
    for row in rows {
        mailings.push(row?);
    }
    Ok(mailings)
}

/// Retrieves a single mailing by ID.
pub fn get_mailing_by_id(conn: &Connection, id: i64) -> Result<Option<Mailing>, ServerError> {
    conn.query_row(
        "SELECT id, campaign_id, list_id, status, created_at, scheduled_at FROM mailings WHERE id = ?1",
        params![id],
        |row| {
            Ok(Mailing {
                id: row.get(0)?,
                campaign_id: row.get(1)?,
                list_id: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
                scheduled_at: row.get(5)?,
            })
        },
    )
    .optional()
    .map_err(ServerError::from)
}
