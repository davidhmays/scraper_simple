use crate::db::connection::Database;
use crate::errors::ServerError;

use rusqlite::{params, params_from_iter};

use super::mailing::{create_mailing, MediaType, NewMailing};

#[derive(Debug, Clone, Copy)]
pub enum ListingFlag {
    ComingSoon,
    Contingent,
    Foreclosure,
    NewConstruction,
    NewListing,
    Pending,
}

#[derive(Debug, Clone, Copy)]
pub enum PropertyType {
    SingleFamily,
    Townhomes,
    Land,
    MultiFamily,
    Farm,
    Condos,
}

impl ListingFlag {
    pub fn column(self) -> &'static str {
        match self {
            ListingFlag::ComingSoon => "is_coming_soon",
            ListingFlag::Contingent => "is_contingent",
            ListingFlag::Foreclosure => "is_foreclosure",
            ListingFlag::NewConstruction => "is_new_construction",
            ListingFlag::NewListing => "is_new_listing",
            ListingFlag::Pending => "is_pending",
        }
    }
}

impl PropertyType {
    pub fn as_str(self) -> &'static str {
        match self {
            PropertyType::SingleFamily => "single_family",
            PropertyType::Townhomes => "townhomes",
            PropertyType::Land => "land",
            PropertyType::MultiFamily => "multi_family",
            PropertyType::Farm => "farm",
            PropertyType::Condos => "condos",
        }
    }
}

fn placeholders(n: usize) -> String {
    std::iter::repeat("?")
        .take(n)
        .collect::<Vec<_>>()
        .join(", ")
}

pub struct NewCampaign {
    pub name: String,
    pub variant: String,
    pub description: Option<String>,

    pub media_type: MediaType,
    pub media_size: String,

    /// OR semantics: match if ANY of these flags are true.
    pub any_of_flags: Vec<ListingFlag>,

    /// OR semantics: match if ANY of these types match `listings.property_type`
    pub any_of_types: Vec<PropertyType>,

    pub any_of_counties: Vec<String>,

    /// AND semantics guardrail.
    pub state_abbr: String,

    /// ZIP targeting (required, non-empty)
    pub zip_codes: Vec<String>,
}

/// Generate one mailing per *property* that matches:
///   state_abbr
///   AND (flag OR flag OR ...)
///   AND multiiple property_types IN (types...)
///   AND optional postal_code IN (zips...)
pub fn generate_mailings_for_campaign(
    db: &Database,
    campaign: &NewCampaign,
) -> Result<Vec<(i64, String)>, ServerError> {
    if campaign.state_abbr.trim().is_empty() {
        return Err(ServerError::DbError(
            "campaign.state_abbr must not be empty".into(),
        ));
    }
    // if campaign.zip_codes.is_empty() {
    //     return Err(ServerError::DbError("campaign.zip_codes must not be empty".into()));
    // }
    if campaign.any_of_flags.is_empty() {
        return Err(ServerError::DbError(
            "campaign.any_of_flags must not be empty".into(),
        ));
    }
    if campaign.any_of_types.is_empty() {
        return Err(ServerError::DbError(
            "campaign.any_of_types must not be empty".into(),
        ));
    }

    // (l.is_pending = 1 OR l.is_contingent = 1 OR ...)
    let flags_or_clause = campaign
        .any_of_flags
        .iter()
        .map(|f| format!("l.{} = 1", f.column()))
        .collect::<Vec<_>>()
        .join(" OR ");

    // Build optional clauses + bind vector in the exact same order.
    // Build optional clauses + bind vector in the exact same order.
    let mut bind: Vec<String> = Vec::new();
    bind.push(campaign.state_abbr.clone());

    let mut where_extra = String::new();

    // Optional: counties (multi-select)
    if !campaign.any_of_counties.is_empty() {
        where_extra.push_str(&format!(
            " AND l.county_name IN ({})",
            placeholders(campaign.any_of_counties.len())
        ));
        bind.extend(campaign.any_of_counties.iter().cloned());
    }

    // Optional: ZIPs
    if !campaign.zip_codes.is_empty() {
        where_extra.push_str(&format!(
            " AND l.postal_code IN ({})",
            placeholders(campaign.zip_codes.len())
        ));
        bind.extend(campaign.zip_codes.iter().cloned());
    }

    // Required: types
    where_extra.push_str(&format!(
        " AND l.property_type IN ({})",
        placeholders(campaign.any_of_types.len())
    ));
    bind.extend(campaign.any_of_types.iter().map(|t| t.as_str().to_string()));

    let sql = format!(
        r#"
        SELECT DISTINCT l.property_id
        FROM listings l
        WHERE l.state_abbr = ?
          AND ({flags_or_clause})
          {where_extra}
        ORDER BY l.property_id
        "#
    );

    let property_ids: Vec<String> = db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let rows = stmt
            .query_map(params_from_iter(bind.iter()), |row| row.get::<_, String>(0))
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| ServerError::DbError(e.to_string()))?);
        }
        Ok(out)
    })?;

    let mut created = Vec::new();
    for property_id in property_ids {
        let input = NewMailing {
            property_id,
            campaign: campaign.name.clone(),
            variant: campaign.variant.clone(),
            description: campaign.description.clone(),
            media_type: campaign.media_type,
            media_size: campaign.media_size.clone(),
        };

        created.push(create_mailing(db, &input)?);
    }

    Ok(created)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use std::time::{SystemTime, UNIX_EPOCH};

    const SCHEMA_SQL: &str = include_str!("../../sql/schema.sql");

    fn unique_temp_db_path() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("campaign_test_{nanos}.sqlite"));
        p.to_string_lossy().to_string()
    }

    fn make_test_db() -> Database {
        Database::new(unique_temp_db_path())
    }

    fn init_schema(db: &Database) {
        db.with_conn(|conn| {
            conn.execute_batch(SCHEMA_SQL)
                .map_err(|e| ServerError::DbError(e.to_string()))?;
            Ok::<(), ServerError>(())
        })
        .expect("schema init failed");
    }

    fn seed_listing(
        db: &Database,
        property_id: &str,
        listing_id: &str,
        source_listing_id: &str,
        status: &str,
        last_seen_at: &str,
        postal_code: &str,
        is_pending: i32,
        is_contingent: i32,
    ) {
        db.with_conn(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO properties (id, source_property_id) VALUES (?1, ?2)",
                params![property_id, property_id],
            )
            .unwrap();

            conn.execute(
                r#"
                INSERT INTO listings (
                  id, property_id,
                  source, source_id, source_listing_id,
                  address_line, city, state_abbr, postal_code,
                  first_seen_at, last_seen_at, status,
                  list_price,
                  is_pending, is_contingent
                ) VALUES (
                  ?1, ?2,
                  'realtor', 'R1', ?3,
                  '123 Main St', 'Townsville', 'UT', ?4,
                  '2026-01-01 00:00:00', ?5, ?6,
                  300000,
                  ?7, ?8
                )
                "#,
                params![
                    listing_id,
                    property_id,
                    source_listing_id,
                    postal_code,
                    last_seen_at,
                    status,
                    is_pending,
                    is_contingent
                ],
            )
            .unwrap();

            Ok::<(), ServerError>(())
        })
        .unwrap();
    }

    #[test]
    fn generates_mailings_filtered_by_zip_and_pending_or_contingent() {
        let db = make_test_db();
        init_schema(&db);

        // ZIP allowlist for campaign:
        // - include 84000 and 84001
        // - exclude 99999
        let allowed_zips = vec!["84000".to_string(), "84001".to_string()];

        // A: pending, zip 84000 -> included
        seed_listing(
            &db,
            "prop:A",
            "lst:A1",
            "A1",
            "for_sale",
            "2026-01-10 00:00:00",
            "84000",
            1,
            0,
        );

        // B: contingent, zip 84001 -> included
        seed_listing(
            &db,
            "prop:B",
            "lst:B1",
            "B1",
            "for_sale",
            "2026-01-10 00:00:00",
            "84001",
            0,
            1,
        );

        // C: pending, but zip 99999 -> excluded by zip filter
        seed_listing(
            &db,
            "prop:C",
            "lst:C1",
            "C1",
            "for_sale",
            "2026-01-10 00:00:00",
            "99999",
            1,
            0,
        );

        // D: TWO listings in allowed zips, both match flags -> still only one mailing
        seed_listing(
            &db,
            "prop:D",
            "lst:D1",
            "D1",
            "for_sale",
            "2026-01-05 00:00:00",
            "84000",
            1,
            0,
        );
        seed_listing(
            &db,
            "prop:D",
            "lst:D2",
            "D2",
            "for_sale",
            "2026-01-12 00:00:00",
            "84001",
            0,
            1,
        );

        pub struct NewCampaign {
            pub name: String,
            pub variant: String,
            pub description: Option<String>,
            pub media_type: MediaType,
            pub media_size: String,
            pub state_abbr: String,
            pub any_of_flags: Vec<ListingFlag>,
            pub any_of_counties: Vec<String>,
            pub zip_codes: Vec<String>, // required, non-empty
        }

        let created = generate_mailings_for_campaign(&db, &campaign).expect("campaign failed");

        // Should include A, B, D = 3
        assert_eq!(created.len(), 3);

        db.with_conn(|conn| {
            // exactly 3 rows for this campaign+variant
            let count: i64 = conn
                .query_row(
                    r#"
                    SELECT COUNT(*)
                    FROM mailings
                    WHERE campaign = ?1 AND variant = ?2
                    "#,
                    params![campaign.name.as_str(), campaign.variant.as_str()],
                    |r| r.get(0),
                )
                .map_err(|e| ServerError::DbError(e.to_string()))?;
            assert_eq!(count, 3);

            // prop:C should not exist
            let c_exists: i64 = conn
                .query_row(
                    r#"
                    SELECT COUNT(*)
                    FROM mailings
                    WHERE property_id = 'prop:C'
                      AND campaign = ?1
                      AND variant = ?2
                    "#,
                    params![campaign.name.as_str(), campaign.variant.as_str()],
                    |r| r.get(0),
                )
                .map_err(|e| ServerError::DbError(e.to_string()))?;
            assert_eq!(c_exists, 0);

            // prop:D only one
            let d_count: i64 = conn
                .query_row(
                    r#"
                    SELECT COUNT(*)
                    FROM mailings
                    WHERE property_id = 'prop:D'
                      AND campaign = ?1
                      AND variant = ?2
                    "#,
                    params![campaign.name.as_str(), campaign.variant.as_str()],
                    |r| r.get(0),
                )
                .map_err(|e| ServerError::DbError(e.to_string()))?;
            assert_eq!(d_count, 1);

            Ok::<(), ServerError>(())
        })
        .expect("verification failed");
    }
}
