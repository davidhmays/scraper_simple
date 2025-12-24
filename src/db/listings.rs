use crate::db::connection::Database;
use crate::errors::ServerError;
use chrono::Utc;
use rusqlite::params;
use serde_json::Value;

pub fn save_properties(
    db: &Database,
    properties: &[Value],
    page_url: &str,
) -> Result<(), ServerError> {
    let now = Utc::now().naive_utc();

    db.with_conn(|conn: &mut rusqlite::Connection| -> Result<(), ServerError> {
        let tx = conn
            .transaction()
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        for prop in properties {
            // ----- Properties -----
            let source_id = prop["property_id"].as_str().unwrap_or_default();
            let address_line = prop["address"]["line"].as_str().unwrap_or_default();
            let city = prop["address"]["city"].as_str().unwrap_or_default();
            let state = prop["address"]["state"].as_str().unwrap_or_default();
            let state_abbr = prop["address"]["state_code"].as_str().unwrap_or_default();
            let postal_code = prop["address"]["postal_code"].as_str().unwrap_or_default();
            let county_name = prop["address"]["county_name"].as_str().unwrap_or_default();
            let county_fips = prop["address"]["county_fips"].as_i64();
            let country = prop["address"]["country"].as_str().unwrap_or("US");
            let latitude = prop["geo"]["lat"].as_f64();
            let longitude = prop["geo"]["lng"].as_f64();
            let bedrooms = prop["beds"].as_i64();
            let bathrooms = prop["baths"].as_i64();
            let lot_sqft = prop["lot_sqft"].as_i64();
            let property_type = prop["prop_type"].as_str().unwrap_or_default();

            tx.execute(
                "INSERT INTO properties (source_id, source, address_line, city, state, state_abbr, postal_code, county_name, county_fips, country, latitude, longitude, bedrooms, bathrooms, lot_sqft, property_type, created_at)
                 VALUES (?1, 'realtor', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                 ON CONFLICT(source_id) DO UPDATE SET
                     address_line=excluded.address_line,
                     city=excluded.city,
                     state=excluded.state,
                     state_abbr=excluded.state_abbr,
                     postal_code=excluded.postal_code,
                     county_name=excluded.county_name,
                     county_fips=excluded.county_fips,
                     country=excluded.country,
                     latitude=excluded.latitude,
                     longitude=excluded.longitude,
                     bedrooms=excluded.bedrooms,
                     bathrooms=excluded.bathrooms,
                     lot_sqft=excluded.lot_sqft,
                     property_type=excluded.property_type",
                params![
                    source_id,
                    address_line,
                    city,
                    state,
                    state_abbr,
                    postal_code,
                    county_name,
                    county_fips,
                    country,
                    latitude,
                    longitude,
                    bedrooms,
                    bathrooms,
                    lot_sqft,
                    property_type,
                    now
                ],
            )
            .map_err(|e| ServerError::DbError(e.to_string()))?;

            let property_id: i64 = tx
                .query_row(
                    "SELECT id FROM properties WHERE source_id = ?1",
                    [source_id],
                    |row| row.get(0),
                )
                .map_err(|e| ServerError::DbError(e.to_string()))?;

            // ----- Listings -----
            let realtor_listing_id = prop["listing_id"].as_str().unwrap_or_default();
            let status = prop["status"].as_str().unwrap_or("unknown");
            let list_price = prop["list_price"].as_i64().unwrap_or(0);
            let price_reduced = prop["price_reduced"].as_i64().unwrap_or(0);
            let is_price_reduced = prop["is_price_reduced"].as_bool().unwrap_or(false);
            let sold_price = prop["sold_price"].as_i64();

            tx.execute(
                "INSERT INTO listings (property_id, realtor_listing_id, first_seen_at, last_seen_at, status, list_price, price_reduced, is_price_reduced, sold_price)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(realtor_listing_id) DO UPDATE SET
                     last_seen_at=excluded.last_seen_at,
                     status=excluded.status,
                     list_price=excluded.list_price,
                     price_reduced=excluded.price_reduced,
                     is_price_reduced=excluded.is_price_reduced,
                     sold_price=excluded.sold_price",
                params![
                    property_id,
                    realtor_listing_id,
                    now,
                    now,
                    status,
                    list_price,
                    price_reduced,
                    is_price_reduced,
                    sold_price
                ],
            )
            .map_err(|e| ServerError::DbError(e.to_string()))?;

            let listing_id: i64 = tx
                .query_row(
                    "SELECT id FROM listings WHERE realtor_listing_id = ?1",
                    [realtor_listing_id],
                    |row| row.get(0),
                )
                .map_err(|e| ServerError::DbError(e.to_string()))?;

            // ----- Observations -----
            tx.execute(
                "INSERT INTO listing_observations (listing_id, observed_at, page_url, raw_json)
                 VALUES (?1, ?2, ?3, ?4)",
                params![listing_id, now, page_url, prop.to_string()],
            )
            .map_err(|e| ServerError::DbError(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        Ok(())
    })
}
