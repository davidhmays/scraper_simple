use crate::db::connection::Database;
use crate::domain::listing::ListingWithProperty;
use crate::errors::ServerError;
use crate::scraper::models::Property;
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde_json::Value;
use std::fs::File;
use std::io::BufWriter;

const SQL_COUNTIES_BY_STATE: &str = include_str!("../../sql/counties_by_state_alpha.sql");

pub fn save_properties_debug(properties: &[Property], filename: &str) -> std::io::Result<()> {
    let file = File::create(filename)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, properties)?;
    Ok(())
}

pub fn get_counties_by_state(
    db: &Database,
    state_abbr: &str,
) -> Result<Vec<(String, i64)>, ServerError> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(SQL_COUNTIES_BY_STATE)
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let rows = stmt
            .query_map(params![state_abbr], |row| {
                Ok((
                    row.get::<_, String>(0)?, // county_name
                    row.get::<_, i64>(1)?,    // n
                ))
            })
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| ServerError::DbError(e.to_string()))?);
        }
        Ok(out)
    })
}

/// Prefix IDs with the source so ids can't collide across sources.
/// Example: "realtor:12345678"
fn make_scoped_id(source: &str, raw_id: &str) -> String {
    format!("{}:{}", source.trim().to_lowercase(), raw_id.trim())
}

/// TEMP strategy:
/// - internal property_id = scoped(source, source_property_id)
/// - internal listing_id  = scoped(source, source_listing_id)
///
/// If you truly want `property_id == listing_id` for now, you can set
/// `property_id = listing_id`, but you'll lose the ability to model multiple
/// listings per property later. Recommended is to use a real source property id
/// if the payload has it; otherwise fall back to listing id.
fn choose_property_scoped_id(source: &str, prop: &Value, listing_scoped_id: &str) -> String {
    // Try common places a property id might live. Adjust to match your payload.
    // If none exists, fall back to listing id.
    let candidate = prop["source"]["property_id"]
        .as_str()
        .or_else(|| prop["property_id"].as_str())
        .unwrap_or("");

    if !candidate.is_empty() {
        make_scoped_id(source, candidate)
    } else {
        listing_scoped_id.to_string()
    }
}

pub fn save_properties(
    db: &Database,
    properties: &[Property],
    page_url: &str,
) -> Result<(), ServerError> {
    let now = Utc::now().naive_utc();

    db.with_conn(|conn: &mut rusqlite::Connection| -> Result<(), ServerError> {
        save_properties_debug(properties, "properties_debug.json")
            .expect("Failed to save properties debug file");

        let tx = conn
            .transaction()
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        for prop in properties {
            // ------------------------
            // ----- Source IDs -----
            // ------------------------
            let source = prop["source"]["name"].as_str().unwrap_or("unknown");
            let source_id = prop["source"]["id"].as_str().unwrap_or("unknown");
            let source_listing_id_raw = prop["source"]["listing_id"].as_str().unwrap_or("");

            if source_listing_id_raw.is_empty() {
                eprintln!("Skipping record: missing source_listing_id");
                continue;
            }

            let listing_id = make_scoped_id(source, source_listing_id_raw);
            let property_id = choose_property_scoped_id(source, prop, &listing_id);

            // ------------------------
            // ----- Address / Facts (now on listings) -----
            // ------------------------
            let address_line = prop["location"]["address"]["line"].as_str().unwrap_or("");
            let city = prop["location"]["address"]["city"].as_str().unwrap_or("");
            let state_abbr = prop["location"]["address"]["state_code"].as_str().unwrap_or("");
            let postal_code = prop["location"]["address"]["postal_code"].as_str().unwrap_or("");
            let county_name = prop["location"]["county"]["name"].as_str();
            let county_fips = prop["location"]["county"]["fips_code"].as_i64();
            let country = prop["location"]["address"]["country"].as_str().unwrap_or("US");
            let latitude = prop["location"]["coordinate"]["lat"].as_f64();
            let longitude = prop["location"]["coordinate"]["lon"].as_f64();

            let bedrooms = prop["description"]["beds"].as_i64();
            let bathrooms = prop["description"]["baths"].as_i64();
            let lot_sqft = prop["description"]["lot_sqft"].as_i64();
            let property_type = prop["description"]["type"].as_str();

            // Guard against missing address basics (still a good idea for downstream UX)

            // TODO: allow missing addresses, just NOT in the mailings.
            if address_line.is_empty() || city.is_empty() || state_abbr.is_empty() || postal_code.is_empty() {
                eprintln!(
                    "Skipping listing with missing address: address='{}', city='{}', state='{}', zip='{}'",
                    address_line, city, state_abbr, postal_code
                );
                continue;
            }

            // ------------------------
            // ----- Properties (TEMP: id == source_property_id) -----
            // ------------------------
            // Here: properties.id is the internal id (TEXT) and mirrors source_property_id (TEXT)
            // We store both columns even if redundant; later you can decouple them.
            tx.execute(
                r#"
                INSERT INTO properties (id, source_property_id, created_at)
                VALUES (?1, ?2, ?3)
                ON CONFLICT(id) DO UPDATE SET
                    source_property_id = excluded.source_property_id
                "#,
                params![property_id, property_id, now],
            )
            .map_err(|e| ServerError::DbError(e.to_string()))?;

            // ------------------------
            // ----- Listings -----
            // ------------------------
            let status = prop["status"].as_str().unwrap_or("unknown");
            let list_price = prop["list_price"].as_i64().unwrap_or(0);
            let price_reduced = prop["price_reduced"].as_i64().unwrap_or(0);
            let sold_price = prop["sold_price"].as_i64();

            let is_coming_soon = prop["flags"]["is_coming_soon"].as_bool().unwrap_or(false) as i32;
            let is_contingent = prop["flags"]["is_contingent"].as_bool().unwrap_or(false) as i32;
            let is_foreclosure = prop["flags"]["is_foreclosure"].as_bool().unwrap_or(false) as i32;
            let is_new_construction = prop["flags"]["is_new_construction"].as_bool().unwrap_or(false) as i32;
            let is_new_listing = prop["flags"]["is_new_listing"].as_bool().unwrap_or(false) as i32;
            let is_pending = prop["flags"]["is_pending"].as_bool().unwrap_or(false) as i32;
            let is_price_reduced = prop["is_price_reduced"].as_bool().unwrap_or(false) as i32;

            tx.execute(
                r#"
                INSERT INTO listings (
                    id, property_id,
                    source, source_id, source_listing_id,

                    address_line, city, state_abbr, postal_code, county_name, county_fips, country,
                    latitude, longitude,
                    bedrooms, bathrooms, lot_sqft, property_type,

                    first_seen_at, last_seen_at, status,
                    list_price, price_reduced, is_price_reduced, sold_price, currency,

                    is_coming_soon, is_contingent, is_foreclosure, is_new_construction,
                    is_new_listing, is_pending
                ) VALUES (
                    ?1, ?2,
                    ?3, ?4, ?5,

                    ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                    ?13, ?14,
                    ?15, ?16, ?17, ?18,

                    ?19, ?20, ?21,
                    ?22, ?23, ?24, ?25, COALESCE(?26, 'USD'),

                    ?27, ?28, ?29, ?30,
                    ?31, ?32
                )
                ON CONFLICT(source, source_listing_id) DO UPDATE SET
                    property_id = excluded.property_id,

                    address_line = excluded.address_line,
                    city = excluded.city,
                    state_abbr = excluded.state_abbr,
                    postal_code = excluded.postal_code,
                    county_name = excluded.county_name,
                    county_fips = excluded.county_fips,
                    country = excluded.country,
                    latitude = excluded.latitude,
                    longitude = excluded.longitude,

                    bedrooms = excluded.bedrooms,
                    bathrooms = excluded.bathrooms,
                    lot_sqft = excluded.lot_sqft,
                    property_type = excluded.property_type,

                    last_seen_at = excluded.last_seen_at,
                    status = excluded.status,

                    list_price = excluded.list_price,
                    price_reduced = excluded.price_reduced,
                    is_price_reduced = excluded.is_price_reduced,
                    sold_price = excluded.sold_price,

                    is_coming_soon = excluded.is_coming_soon,
                    is_contingent = excluded.is_contingent,
                    is_foreclosure = excluded.is_foreclosure,
                    is_new_construction = excluded.is_new_construction,
                    is_new_listing = excluded.is_new_listing,
                    is_pending = excluded.is_pending
                "#,
                params![
                    // ids
                    listing_id,
                    property_id,
                    // source
                    source,
                    source_id,
                    source_listing_id_raw,
                    // address/geo
                    address_line,
                    city,
                    state_abbr,
                    postal_code,
                    county_name,
                    county_fips,
                    country,
                    latitude,
                    longitude,
                    // facts
                    bedrooms,
                    bathrooms,
                    lot_sqft,
                    property_type,
                    // lifecycle
                    now,
                    now,
                    status,
                    // pricing
                    list_price,
                    price_reduced,
                    is_price_reduced,
                    sold_price,
                    // currency (if present)
                    prop["currency"].as_str(),
                    // flags
                    is_coming_soon,
                    is_contingent,
                    is_foreclosure,
                    is_new_construction,
                    is_new_listing,
                    is_pending
                ],
            )
            .map_err(|e| ServerError::DbError(e.to_string()))?;

            // ------------------------
            // ----- Observations -----
            // ------------------------
            // With Option A, we already know listing_id (TEXT PK), no need to re-query.
            tx.execute(
                r#"
                INSERT INTO listing_observations (listing_id, observed_at, page_url, raw_json)
                VALUES (?1, ?2, ?3, ?4)
                "#,
                params![listing_id, now, page_url, prop.to_string()],
            )
            .map_err(|e| ServerError::DbError(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        Ok(())
    })
}

pub fn get_listings_by_state(
    db: &Database,
    state_abbr: &str,
) -> Result<Vec<ListingWithProperty>, ServerError> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT
                    l.property_id,           -- 0 (TEXT)
                    l.id,                    -- 1 (TEXT)

                    l.source,                -- 2
                    l.source_id,             -- 3
                    l.source_listing_id,     -- 4
                    p.source_property_id,    -- 5

                    l.address_line,          -- 6
                    l.city,                  -- 7
                    l.state_abbr,            -- 8
                    l.postal_code,           -- 9
                    l.county_name,           -- 10

                    l.bedrooms,              -- 11
                    l.bathrooms,             -- 12

                    l.list_price,            -- 13
                    l.status,                -- 14

                    l.is_coming_soon,        -- 15
                    l.is_contingent,         -- 16
                    l.is_pending             -- 17
                FROM listings l
                JOIN properties p ON p.id = l.property_id
                WHERE l.state_abbr = ?
                ORDER BY l.city, l.address_line
                "#,
            )
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let rows = stmt
            .query_map([state_abbr], |row| {
                Ok(ListingWithProperty {
                    property_id: row.get(0)?,
                    listing_id: row.get(1)?,

                    source: row.get(2)?,
                    source_id: row.get(3)?,
                    source_listing_id: row.get(4)?,
                    source_property_id: row.get(5)?,

                    address_line: row.get(6)?,
                    city: row.get(7)?,
                    state_abbr: row.get(8)?,
                    postal_code: row.get(9)?,
                    county_name: row.get(10)?,

                    bedrooms: row.get(11)?,
                    bathrooms: row.get(12)?,

                    list_price: row.get(13)?,
                    status: row.get(14)?,

                    is_coming_soon: row.get(15)?,
                    is_contingent: row.get(16)?,
                    is_pending: row.get(17)?,
                })
            })
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| ServerError::DbError(e.to_string()))?);
        }

        Ok(results)
    })
}

// HARD CODED FLAGS for testing.
pub fn get_target_zips_for_state_pending_or_contingent(
    db: &Database,
    state_abbr: &str,
) -> Result<Vec<String>, ServerError> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT DISTINCT postal_code
                FROM listings
                WHERE state_abbr = ?1
                  AND (is_pending = 1 OR is_contingent = 1)
                  AND postal_code IS NOT NULL
                  AND postal_code <> ''
                ORDER BY postal_code
                "#,
            )
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let rows = stmt
            .query_map(params![state_abbr], |r| r.get::<_, String>(0))
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| ServerError::DbError(e.to_string()))?);
        }
        Ok(out)
    })
}
