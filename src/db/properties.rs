// Force recompile to ensure schema changes are picked up
use crate::db::connection::Database;
use crate::domain::changes::ChangeViewModel;
use crate::domain::logic::derive_canonical_status;
use crate::domain::property::{PropertyChange, ScrapedProperty, TrackedProperty};
use crate::errors::ServerError;
use crate::scraper::models::Property as ScraperProperty;
use chrono::{NaiveDateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result as RusqliteResult};

/// Main entry point for saving scraped data.
///
/// This function orchestrates the entire change-tracking process. It takes raw
/// scraped data, converts it into a clean domain model, and then, within a single
/// database transaction, processes each property to identify, log, and store any
/// changes from its previously known state.
pub fn save_scraped_properties(
    db: &Database,
    scraper_properties: &[ScraperProperty],
) -> Result<(), ServerError> {
    // First, convert the raw, nested scraper models into our clean, flattened
    // domain models. This validates that we have the necessary data to proceed.
    let properties: Vec<ScrapedProperty> = scraper_properties
        .iter()
        .filter_map(|p| match ScrapedProperty::from_scraper_property(p) {
            Ok(sp) => Some(sp),
            Err(e) => {
                eprintln!("Skipping property due to validation error: {}", e);
                None
            }
        })
        .collect();

    // Perform the entire operation within a single database transaction to ensure
    // that our main table and history log remain perfectly consistent.
    db.with_conn(|conn| {
        let tx = conn
            .transaction()
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        for prop in &properties {
            process_one_property(&tx, prop)?;
        }

        tx.commit().map_err(|e| ServerError::DbError(e.to_string()))
    })
}

/// Processes a single scraped property within a database transaction.
fn process_one_property(
    tx: &Connection,
    scraped_prop: &ScrapedProperty,
) -> Result<(), ServerError> {
    let now = Utc::now().naive_utc();

    // Attempt to find an existing property in our database using its unique address.
    let maybe_tracked_prop = find_property_by_address(tx, scraped_prop)?;

    match maybe_tracked_prop {
        // If the property already exists, we check for changes.
        Some(tracked_prop) => {
            let changes = tracked_prop.diff(scraped_prop);
            if !changes.is_empty() {
                log_changes(tx, &changes)?;
                update_property(tx, tracked_prop.id, scraped_prop, now)?;
            }
            // Always update the source's `last_seen_at` timestamp.
            update_source(tx, scraped_prop, now)?;
        }
        // If it's a new property, we create it and log its initial state.
        None => {
            let property_id = insert_property(tx, scraped_prop, now)?;
            log_initial_state(tx, property_id, scraped_prop, now)?;
            insert_or_update_source(tx, property_id, scraped_prop, now)?;
        }
    }
    Ok(())
}

/// Finds a property by its unique address components.
fn find_property_by_address(
    conn: &Connection,
    prop: &ScrapedProperty,
) -> Result<Option<TrackedProperty>, ServerError> {
    conn.query_row(
        r#"
        SELECT
            id, status, list_price, sold_price, sold_date, is_pending, is_contingent,
            is_new_listing, is_foreclosure, is_price_reduced, is_coming_soon
        FROM properties
        WHERE address_line = ?1 AND city = ?2 AND postal_code = ?3
        "#,
        params![&prop.address_line, &prop.city, &prop.postal_code],
        |row| {
            Ok(TrackedProperty {
                id: row.get(0)?,
                status: row.get(1)?,
                list_price: row.get(2)?,
                sold_price: row.get(3)?,
                sold_date: row.get(4)?,
                is_pending: row.get(5)?,
                is_contingent: row.get(6)?,
                is_new_listing: row.get(7)?,
                is_foreclosure: row.get(8)?,
                is_price_reduced: row.get(9)?,
                is_coming_soon: row.get(10)?,
            })
        },
    )
    .optional()
    .map_err(|e| ServerError::DbError(e.to_string()))
}

/// Inserts a new record into the main `properties` table.
fn insert_property(
    tx: &Connection,
    prop: &ScrapedProperty,
    now: NaiveDateTime,
) -> Result<i64, ServerError> {
    let mut stmt = tx.prepare(
        r#"
        INSERT INTO properties (
            address_line, city, postal_code, state_abbr, county_name,
            status, list_price, sold_price, sold_date, is_pending, is_contingent,
            is_new_listing, is_foreclosure, is_price_reduced, is_coming_soon,
            first_seen_at, last_seen_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
        "#,
    )?;
    stmt.execute(params![
        &prop.address_line,
        &prop.city,
        &prop.postal_code,
        &prop.state_abbr,
        &prop.county_name,
        &prop.status,
        &prop.list_price,
        &prop.sold_price,
        &prop.sold_date,
        &prop.is_pending,
        &prop.is_contingent,
        &prop.is_new_listing,
        &prop.is_foreclosure,
        &prop.is_price_reduced,
        &prop.is_coming_soon,
        now,
        now,
    ])?;
    Ok(tx.last_insert_rowid())
}

/// Updates the current state fields for an existing property.
fn update_property(
    tx: &Connection,
    property_id: i64,
    prop: &ScrapedProperty,
    now: NaiveDateTime,
) -> Result<(), ServerError> {
    tx.execute(
        r#"
        UPDATE properties SET
            status = ?1, list_price = ?2, sold_price = ?3, sold_date = ?4,
            is_pending = ?5, is_contingent = ?6, is_new_listing = ?7, is_foreclosure = ?8,
            is_price_reduced = ?9, is_coming_soon = ?10, last_seen_at = ?11
        WHERE id = ?12
        "#,
        params![
            &prop.status,
            &prop.list_price,
            &prop.sold_price,
            &prop.sold_date,
            &prop.is_pending,
            &prop.is_contingent,
            &prop.is_new_listing,
            &prop.is_foreclosure,
            &prop.is_price_reduced,
            &prop.is_coming_soon,
            now,
            property_id,
        ],
    )?;
    Ok(())
}

/// Inserts a batch of changes into the `property_history` table.
fn log_changes(tx: &Connection, changes: &[PropertyChange]) -> RusqliteResult<()> {
    let mut stmt = tx.prepare(
        r#"
        INSERT INTO property_history (property_id, observed_at, field_name, previous_value, current_value)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )?;
    let now = Utc::now().naive_utc();
    for change in changes {
        stmt.execute(params![
            change.property_id,
            now,
            &change.field_name,
            &change.previous_value,
            &change.current_value,
        ])?;
    }
    Ok(())
}

/// For a newly discovered property, logs the initial state of all its tracked fields.
fn log_initial_state(
    tx: &Connection,
    property_id: i64,
    prop: &ScrapedProperty,
    now: NaiveDateTime,
) -> RusqliteResult<()> {
    let mut stmt = tx.prepare(
        r#"
        INSERT INTO property_history (property_id, observed_at, field_name, previous_value, current_value)
        VALUES (?1, ?2, ?3, NULL, ?4)
        "#,
    )?;

    macro_rules! log_field {
        ($field:ident, $field_name:expr) => {
            if let Some(value) = &prop.$field {
                stmt.execute(params![property_id, now, $field_name, value.to_string(),])?;
            }
        };
    }

    log_field!(status, "status");
    log_field!(list_price, "list_price");
    log_field!(sold_price, "sold_price");
    log_field!(sold_date, "sold_date");
    log_field!(is_pending, "is_pending");
    log_field!(is_contingent, "is_contingent");
    log_field!(is_new_listing, "is_new_listing");
    log_field!(is_foreclosure, "is_foreclosure");
    log_field!(is_price_reduced, "is_price_reduced");
    log_field!(is_coming_soon, "is_coming_soon");

    Ok(())
}

/// Creates or updates the link between our internal property and the source listing.
fn insert_or_update_source(
    tx: &Connection,
    property_id: i64,
    prop: &ScrapedProperty,
    now: NaiveDateTime,
) -> RusqliteResult<()> {
    tx.execute(
        r#"
        INSERT INTO property_sources (property_id, source_name, source_listing_id, first_seen_at, last_seen_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(source_name, source_listing_id) DO UPDATE SET
            property_id = excluded.property_id,
            last_seen_at = excluded.last_seen_at
        "#,
        params![property_id, &prop.source_name, &prop.source_listing_id, now, now],
    )?;
    Ok(())
}

/// Fetches a detailed log of all change events for a given state and year.
/// This is designed to be exported to a spreadsheet for filtering and sorting.
pub fn get_change_events_for_export(
    conn: &Connection,
    state: &str,
    year: i32,
) -> Result<Vec<ChangeViewModel>, ServerError> {
    let mut stmt = conn.prepare(
        r#"
        -- This complex query is designed to construct our "Change Event" log.
        -- We select not just the history event itself, but also the full context
        -- of the property's state *at the time of the change*. To do this, we
        -- have to join the history table with the properties table.
        SELECT
            h.observed_at,
            h.field_name,
            h.previous_value,
            h.current_value,
            p.address_line,
            p.city,
            p.state_abbr,
            p.postal_code,
            p.county_name,
            p.list_price,
            p.sold_date,
            p.status AS raw_status, -- The status from the scraper
            p.is_pending,
            p.is_contingent,
            p.is_coming_soon,
            p.is_new_listing,
            p.is_price_reduced,
            p.is_foreclosure
        FROM property_history h
        JOIN properties p ON h.property_id = p.id
        WHERE
            p.state_abbr = ?1
            AND strftime('%Y', h.observed_at) = ?2
            -- We only want to create primary spreadsheet rows for these two change types
            AND h.field_name IN ('status', 'list_price')
        ORDER BY h.observed_at DESC
        "#,
    )?;

    let year_str = year.to_string();
    let rows = stmt.query_map(params![state, year_str], |row| {
        let field_name: String = row.get("field_name")?;

        // --- Business Logic for Canonical Status ---
        // We derive the canonical status for both the previous and current state
        // based on our business rules, creating a much cleaner output.
        let previous_value_str: Option<String> = row.get("previous_value")?;
        let current_value_str: String = row.get("current_value")?;

        // Extract fields explicitly to ensure type safety and handle NULLs
        let sold_date: Option<NaiveDateTime> = row.get("sold_date")?;
        let raw_status: Option<String> = row.get("raw_status")?;
        let is_pending: bool = row.get::<_, Option<bool>>("is_pending")?.unwrap_or(false);
        let is_contingent: bool = row
            .get::<_, Option<bool>>("is_contingent")?
            .unwrap_or(false);
        let is_coming_soon: bool = row
            .get::<_, Option<bool>>("is_coming_soon")?
            .unwrap_or(false);

        let current_status = derive_canonical_status(
            &sold_date,
            is_pending,
            is_contingent,
            is_coming_soon,
            &raw_status,
        );

        let (change_type, previous_value, current_value) = if field_name == "status" {
            // For status changes, the previous and current values are our derived statuses.
            let prev_status = derive_canonical_status(
                &sold_date,
                // For previous state, we default flags to false as we don't have history for them in this row.
                false,
                false,
                false,
                &previous_value_str,
            );
            (
                "Status Change".to_string(),
                prev_status.to_string(),
                current_status.to_string(),
            )
        } else {
            // list_price
            (
                "Price Change".to_string(),
                previous_value_str.unwrap_or_default(),
                current_value_str,
            )
        };

        // --- Populate the rest of the ViewModel ---
        let address_line: String = row.get("address_line")?;
        let city: String = row.get("city")?;
        let state_abbr: Option<String> = row.get("state_abbr")?;
        let postal_code: String = row.get("postal_code")?;

        let price_reduction = if change_type == "Price Change" {
            let prev = previous_value.parse::<i64>().ok();
            let curr = current_value.parse::<i64>().ok();
            if let (Some(p), Some(c)) = (prev, curr) {
                Some(p - c)
            } else {
                None
            }
        } else {
            None
        };

        let address_full = format!(
            "{}, {}, {} {}",
            address_line,
            city,
            state_abbr.as_deref().unwrap_or(""),
            postal_code
        );

        Ok(ChangeViewModel {
            change_date: row.get("observed_at")?,
            change_type,
            previous_value,
            current_value,
            address_full,
            address_line,
            city,
            state_abbr,
            postal_code,
            county_name: row.get("county_name")?,
            price: row.get("list_price")?,
            canonical_status: current_status.to_string(),
            is_new_listing: row
                .get::<_, Option<bool>>("is_new_listing")?
                .unwrap_or(false),
            is_price_reduced: row
                .get::<_, Option<bool>>("is_price_reduced")?
                .unwrap_or(false),
            is_foreclosure: row
                .get::<_, Option<bool>>("is_foreclosure")?
                .unwrap_or(false),
            is_ready_to_build: raw_status.as_deref() == Some("ready_to_build"),
            price_reduction,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Updates the `last_seen_at` timestamp for an existing source listing.
fn update_source(
    tx: &Connection,
    prop: &ScrapedProperty,
    now: NaiveDateTime,
) -> RusqliteResult<()> {
    tx.execute(
        r#"
        UPDATE property_sources SET last_seen_at = ?1
        WHERE source_name = ?2 AND source_listing_id = ?3
        "#,
        params![now, &prop.source_name, &prop.source_listing_id],
    )?;
    Ok(())
}

/// Gets a list of distinct years from the property history for the filter dropdown.
pub fn get_distinct_change_years(conn: &Connection) -> Result<Vec<String>, ServerError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT DISTINCT strftime('%Y', observed_at) AS year
        FROM property_history
        ORDER BY year DESC
        "#,
    )?;

    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut years = Vec::new();
    for year_result in rows {
        years.push(year_result?);
    }
    Ok(years)
}

/// Fetches a list of properties with their most recent changes for the dashboard.
/// This query is the heart of the "Changes Dashboard" UI.
pub fn get_recent_changes(
    conn: &Connection,
    days: i64,
) -> Result<Vec<ChangeViewModel>, ServerError> {
    // For the dashboard preview, we can reuse the more detailed export query.
    // In a production app with heavy traffic, we might create a more lightweight
    // query specifically for the dashboard, but this is perfectly fine.
    let now = Utc::now();
    let year = now.format("%Y").to_string().parse::<i32>().unwrap_or(2024);

    // We get all changes for the current year and then limit in the application.
    // This is simpler than adding more complex date logic to the SQL query for now.
    let all_changes = get_change_events_for_export(conn, "UT", year)?; // Assuming a default state for preview

    // Filter to the last `days` and take the most recent 15 for the preview
    let recent_changes: Vec<ChangeViewModel> = all_changes
        .into_iter()
        .filter(|c| (now.naive_utc() - c.change_date).num_days() <= days)
        .collect();

    Ok(recent_changes)
}
