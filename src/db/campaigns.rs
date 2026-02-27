use crate::domain::campaign::{Campaign, Media, NewCampaign, NewMedia};
use crate::errors::ServerError;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

/// Creates a new campaign for a user.
pub fn create_campaign(conn: &Connection, new_campaign: &NewCampaign) -> Result<i64, ServerError> {
    let now = Utc::now().naive_utc();
    conn.execute(
        "INSERT INTO campaigns (user_id, name, status, created_at) VALUES (?1, ?2, 'draft', ?3)",
        params![new_campaign.user_id, new_campaign.name, now],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Retrieves all campaigns for a specific user.
pub fn get_campaigns_for_user(
    conn: &Connection,
    user_id: i64,
) -> Result<Vec<Campaign>, ServerError> {
    let mut stmt = conn.prepare(
        "SELECT id, user_id, name, status, created_at FROM campaigns WHERE user_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![user_id], |row| {
        Ok(Campaign {
            id: row.get(0)?,
            user_id: row.get(1)?,
            name: row.get(2)?,
            status: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;

    let mut campaigns = Vec::new();
    for row in rows {
        campaigns.push(row?);
    }
    Ok(campaigns)
}

/// Retrieves a single campaign by ID.
pub fn get_campaign_by_id(conn: &Connection, id: i64) -> Result<Option<Campaign>, ServerError> {
    conn.query_row(
        "SELECT id, user_id, name, status, created_at FROM campaigns WHERE id = ?1",
        params![id],
        |row| {
            Ok(Campaign {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    )
    .optional()
    .map_err(ServerError::from)
}

/// Creates a new media asset within a campaign.
pub fn create_media(conn: &Connection, new_media: &NewMedia) -> Result<i64, ServerError> {
    let now = Utc::now().naive_utc();
    conn.execute(
        "INSERT INTO media (campaign_id, name, description, media_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            new_media.campaign_id,
            new_media.name,
            new_media.description,
            new_media.media_type,
            now
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Retrieves all media associated with a specific campaign.
pub fn get_media_for_campaign(
    conn: &Connection,
    campaign_id: i64,
) -> Result<Vec<Media>, ServerError> {
    let mut stmt = conn.prepare(
        "SELECT id, campaign_id, name, description, media_type, created_at FROM media WHERE campaign_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![campaign_id], |row| {
        Ok(Media {
            id: row.get(0)?,
            campaign_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            media_type: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    let mut media_list = Vec::new();
    for row in rows {
        media_list.push(row?);
    }
    Ok(media_list)
}
