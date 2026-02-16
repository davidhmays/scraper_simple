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
fn choose_property_scoped_id(source: &str, prop: &Property, listing_scoped_id: &str) -> String {
    // Use source.id as the property id
    let candidate = prop.source.id.as_deref();

    if let Some(id) = candidate {
        if !id.is_empty() {
            return make_scoped_id(source, id);
        }
    }

    // fallback if no source.id
    listing_scoped_id.to_string()
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
          let source = prop.source.name.as_deref().unwrap_or("unknown");
          let source_id = prop.source.id.as_deref().unwrap_or("unknown");
          let source_listing_id_raw = prop.source.listing_id.as_deref().unwrap_or("unknown");

          if source_listing_id_raw.is_empty() {
              eprintln!("Skipping record: missing source_listing_id");
              continue;
          }

          // generate IDs first
          let listing_id = make_scoped_id(source, source_listing_id_raw);
          let property_id = choose_property_scoped_id(source, prop, &listing_id);

          // now property_id is in scope, safe to use
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


          // TODO: move to separate "counties" table to reduce data duplication.

            // Pull nested structs ONCE
            let address = prop.location.address.as_ref();
            let county = prop.location.county.as_ref();
            let coordinate = prop.location.coordinate.as_ref();
            let flags = prop.flags.as_ref();
            let description = prop.description.as_ref(); // Option<&Description>

            // ----- Address -----
            let address_line = address.and_then(|a| a.line.as_deref()).unwrap_or("");
            let city = address.and_then(|a| a.city.as_deref()).unwrap_or("");
            let state_abbr = address.and_then(|a| a.state_code.as_deref()).unwrap_or("");
            let postal_code = address.and_then(|a| a.postal_code.as_deref()).unwrap_or("");
            let country = address.and_then(|a| a.country.as_deref()).unwrap_or("US");

            // ----- County -----
            let county_name = county.and_then(|c| c.name.as_deref()).unwrap_or("");
            let county_fips = county.and_then(|c| c.fips_code);

            // ----- Coordinate -----
            let latitude = coordinate.and_then(|c| c.lat);
            let longitude = coordinate.and_then(|c| c.lon);

            // ----- Description -----
            let bedrooms = description.and_then(|d| d.beds);
            let bathrooms = description.and_then(|d| d.baths);
            let lot_sqft = description.and_then(|d| d.lot_sqft);
            let property_type = description.and_then(|d| d.property_type.as_deref());

            // ----- Flags -----
            let is_coming_soon = flags.and_then(|f| f.is_coming_soon).unwrap_or(false) as i32;
            let is_contingent = flags.and_then(|f| f.is_contingent).unwrap_or(false) as i32;
            let is_foreclosure = flags.and_then(|f| f.is_foreclosure).unwrap_or(false) as i32;
            let is_new_construction = flags.and_then(|f| f.is_new_construction).unwrap_or(false) as i32;
            let is_new_listing = flags.and_then(|f| f.is_new_listing).unwrap_or(false) as i32;
            let is_pending = flags.and_then(|f| f.is_pending).unwrap_or(false) as i32;
            let is_price_reduced = flags.and_then(|f| f.is_price_reduced).unwrap_or(false) as i32;

            // ----- Other top-level fields -----
            let status = prop.status.as_deref().unwrap_or("unknown");
            let list_price = prop.list_price.unwrap_or(0);
            let price_reduced = prop.price_reduced.unwrap_or(0);
            let sold_price = prop.sold_price;
            let currency = prop.currency.as_deref().unwrap_or("USD"); // fallback to USD if missing

            // ----- Normalize state -----
            let state_abbr = state_abbr.to_uppercase();
            if state_abbr.is_empty() {
                eprintln!(
                    "Skipping listing with missing state: address='{}', city='{}', state='{}', zip='{}'",
                    address_line, city, state_abbr, postal_code
                );
                continue;
            }




            // ------------------------
            // ----- Properties (TEMP: id == source_property_id) -----
            // ------------------------
            // Here: properties.id is the internal id (TEXT) and mirrors source_property_id (TEXT)
            // We store both columns even if redundant; later you can decouple them.
            // ------------------------

            // ----- Properties table -----
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

            // ----- Listings table -----
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
                    ?22, ?23, ?24, ?25, ?26,
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
                    is_pending = excluded.is_pending,
                    currency = excluded.currency
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
                    &state_abbr, // borrow String as &str
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
                    // currency
                    currency,
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

            // ----- Observations -----
            let raw_json = serde_json::to_string(&prop)
                .map_err(|e| ServerError::DbError(e.to_string()))?;

            tx.execute(
                r#"
                INSERT INTO listing_observations (listing_id, observed_at, page_url, raw_json)
                VALUES (?1, ?2, ?3, ?4)
                "#,
                params![listing_id, now, page_url, raw_json],
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
