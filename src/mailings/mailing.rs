use crate::db::connection::Database;
use crate::errors::ServerError;

use base64::Engine;
use rand::RngCore;
use rusqlite::{params, OptionalExtension};

#[derive(Debug, Clone, Copy)]
pub enum MediaType {
    Postcard,
    Letter,
    Flyer,
}


//TODO: convert SQL used in test to stable SQL files used in functions.

impl MediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaType::Postcard => "postcard",
            MediaType::Letter => "letter",
            MediaType::Flyer => "flyer",
        }
    }
}

pub struct NewMailing {
    pub property_id: String,
    pub campaign: String,
    pub variant: String,
    pub description: Option<String>,
    pub media_type: MediaType,
    pub media_size: String,
}

/// TODO: move to config/env later
const QR_BASE_URL: &str = "https://yourdomain.com/m";

fn generate_qr_token() -> String {
    // 16 bytes = 128-bit token, url-safe base64 (no padding)
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub fn create_mailing(db: &Database, input: &NewMailing) -> Result<(i64, String), ServerError> {
    db.with_conn(|conn| {
        let tx = conn
            .transaction()
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        // Pick one "best" listing to snapshot address
        let listing: Option<(String, String, String, String, String)> = tx
            .query_row(
                r#"
                SELECT
                  id,
                  address_line,
                  city,
                  state_abbr,
                  postal_code
                FROM listings
                WHERE property_id = ?1
                ORDER BY
                  CASE
                    WHEN status IN ('for_sale','active','coming_soon','pending','contingent') THEN 0
                    ELSE 1
                  END,
                  last_seen_at DESC
                LIMIT 1
                "#,
                params![input.property_id.as_str()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?, // listing_id
                        row.get::<_, String>(1)?, // address_line
                        row.get::<_, String>(2)?, // city
                        row.get::<_, String>(3)?, // state_abbr
                        row.get::<_, String>(4)?, // postal_code
                    ))
                },
            )
            .optional()
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let (listing_id, address_line, city, state_abbr, postal_code) = match listing {
            Some(v) => v,
            None => {
                return Err(ServerError::DbError(format!(
                    "create_mailing: no listings found for property_id={}",
                    input.property_id
                )));
            }
        };

        // Insert mailing with unique qr_token (retry on extremely unlikely collision)
        // Try to insert a new mailing (idempotent per property+campaign+variant).
        // If it already exists, DO NOTHING and then fetch existing id/token.
        let mut qr_token = generate_qr_token();

        for attempt in 1..=5 {
            let changed = tx
                .execute(
                    r#"
                    INSERT INTO mailings (
                      property_id,
                      listing_id,

                      campaign,
                      variant,
                      description,

                      media_type,
                      media_size,

                      address_line,
                      city,
                      state_abbr,
                      postal_code,

                      qr_token
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                    ON CONFLICT(property_id, campaign, variant) DO NOTHING
                    "#,
                    params![
                        input.property_id.as_str(),
                        listing_id.as_str(),
                        input.campaign.as_str(),
                        input.variant.as_str(),
                        input.description.as_deref(),
                        input.media_type.as_str(),
                        input.media_size.as_str(),
                        address_line.as_str(),
                        city.as_str(),
                        state_abbr.as_str(),
                        postal_code.as_str(),
                        qr_token.as_str(),
                    ],
                )
                .map_err(|e| ServerError::DbError(e.to_string()))?;

            if changed == 1 {
                // Inserted new row.
                let mailing_id = tx.last_insert_rowid();
                tx.commit()
                    .map_err(|e| ServerError::DbError(e.to_string()))?;

                let qr_url = format!("{}/{}", QR_BASE_URL, qr_token);
                return Ok((mailing_id, qr_url));
            }

            // No row inserted: it already exists for (property_id,campaign,variant).
            // Fetch existing id + token and return it.
            let existing: Option<(i64, String)> = tx
                .query_row(
                    r#"
                    SELECT id, qr_token
                    FROM mailings
                    WHERE property_id = ?1 AND campaign = ?2 AND variant = ?3
                    LIMIT 1
                    "#,
                    params![
                        input.property_id.as_str(),
                        input.campaign.as_str(),
                        input.variant.as_str()
                    ],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
                .map_err(|e| ServerError::DbError(e.to_string()))?;

            if let Some((mailing_id, existing_token)) = existing {
                tx.commit()
                    .map_err(|e| ServerError::DbError(e.to_string()))?;

                let qr_url = format!("{}/{}", QR_BASE_URL, existing_token);
                return Ok((mailing_id, qr_url));
            }

            // Extremely unlikely edge case: DO NOTHING happened but row not found.
            // Retry with a new token a few times, then fail.
            if attempt < 5 {
                qr_token = generate_qr_token();
                continue;
            }

            return Err(ServerError::DbError(
                "create_mailing: conflict/do-nothing but couldn't load existing mailing".into(),
            ));
        }
        unreachable!();


        let mailing_id = tx.last_insert_rowid();
        tx.commit()
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let qr_url = format!("{}/{}", QR_BASE_URL, qr_token);
        Ok((mailing_id, qr_url))
    })
}



#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{params, OptionalExtension};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Embed your real schema into the test binary.
    // Adjust the path to wherever you placed schema.sql:
    const SCHEMA_SQL: &str = include_str!("../../sql/schema.sql");

    fn unique_temp_db_path() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("mailings_test_{nanos}.sqlite"));
        p.to_string_lossy().to_string()
    }

    /// IMPORTANT: change this to match your Database constructor.
    /// The goal is: open a Database that points at a temp file path.
    fn make_test_db() -> Database {
        let path = unique_temp_db_path();

        // Examples you might have:
        // Database::new(&path).unwrap()
        // Database::open(&path).unwrap()
        // Database::from_path(path)
        Database::new(&path)
    }

    #[test]
    fn create_mailing_uses_real_schema_and_inserts_row() {
        let db = make_test_db();

        // Initialize the DB using your real schema.sql
        db.with_conn(|conn| {
            conn.execute_batch(SCHEMA_SQL)
                .map_err(|e| ServerError::DbError(e.to_string()))?;
            Ok::<(), ServerError>(())
        })
        .expect("schema init failed");

        // Seed minimal data required by create_mailing
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO properties (id, source_property_id) VALUES (?1, ?2)",
                params!["prop:1", "prop:1"],
            )
            .unwrap();

            // Insert two listings so we can verify "best listing" selection.
            // Your schema has many nullable columns, but requires:
            // - id, property_id, source, source_listing_id, address_line
            // - first_seen_at, last_seen_at, status, list_price
            conn.execute(
                r#"
                INSERT INTO listings (
                  id, property_id,
                  source, source_id, source_listing_id,
                  address_line, city, state_abbr, postal_code,
                  first_seen_at, last_seen_at, status,
                  list_price
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                "#,
                params![
                    "lst:old", "prop:1",
                    "realtor", "R1", "111",
                    "100 Old St", "Townsville", "UT", "84000",
                    "2026-01-01 00:00:00", "2026-01-02 00:00:00", "for_sale",
                    300000
                ],
            )
            .unwrap();

            conn.execute(
                r#"
                INSERT INTO listings (
                  id, property_id,
                  source, source_id, source_listing_id,
                  address_line, city, state_abbr, postal_code,
                  first_seen_at, last_seen_at, status,
                  list_price
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                "#,
                params![
                    "lst:new", "prop:1",
                    "realtor", "R1", "112",
                    "200 New St", "Townsville", "UT", "84001",
                    "2026-01-01 00:00:00", "2026-01-10 00:00:00", "for_sale",
                    350000
                ],
            )
            .unwrap();

            Ok::<(), ServerError>(())
        })
        .expect("seed failed");

        let input = NewMailing {
            property_id: "prop:1".to_string(),
            campaign: "utah_jan_2026".to_string(),
            variant: "A".to_string(),
            description: Some("test postcard".to_string()),
            media_type: MediaType::Postcard,
            media_size: "6x9".to_string(),
        };

        let (mailing_id, qr_url) = create_mailing(&db, &input).expect("create_mailing failed");

        // Call again — should NOT create a second row
        let (mailing_id2, qr_url2) = create_mailing(&db, &input).expect("create_mailing failed again");

        assert_eq!(mailing_id, mailing_id2);
        assert_eq!(qr_url, qr_url2);

        assert!(mailing_id > 0);
        assert!(qr_url.starts_with(QR_BASE_URL));


        // Verify it chose the newest listing + stored expected fields
        db.with_conn(|conn| {
            let row = conn
                .query_row(
                    r#"
                    SELECT listing_id, media_type, address_line, postal_code, qr_token
                    FROM mailings
                    WHERE id = ?1
                    "#,
                    params![mailing_id],
                    |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, String>(2)?,
                            r.get::<_, String>(3)?,
                            r.get::<_, String>(4)?,
                        ))
                    },
                )
                .optional()
                .unwrap();

            let (listing_id, media_type, address_line, postal_code, qr_token) =
                row.expect("mailings row missing");

            assert_eq!(listing_id, "lst:new");      // newest last_seen_at
            assert_eq!(media_type, "postcard");     // enum stored as lowercase text
            assert_eq!(address_line, "200 New St"); // snapshot from chosen listing
            assert_eq!(postal_code, "84001");
            assert!(!qr_token.is_empty());
            assert!(qr_url.ends_with(&qr_token));

            Ok::<(), ServerError>(())
        })
        .expect("verify failed");
    }
}
