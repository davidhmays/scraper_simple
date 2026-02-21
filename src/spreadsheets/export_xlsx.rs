// src/spreadsheets/export_xlsx.rs

use crate::domain::changes::ChangeViewModel;
use crate::domain::property::TrackedProperty;
use crate::errors::ServerError;
use crate::responses::{xlsx_response, ResultResp};
use rust_xlsxwriter::{Workbook, XlsxError};

/// This is a placeholder for the old export function. It is no longer used by the
/// primary application flow but is kept to prevent compilation errors in any
/// remaining references. It should be removed in a future cleanup.
pub fn export_listings_xlsx(_listings: &[TrackedProperty], _state: &str) -> ResultResp {
    let mut workbook = Workbook::new();
    let _worksheet = workbook.add_worksheet();
    let buffer = workbook.save_to_buffer().unwrap();
    xlsx_response(buffer, "deprecated_export.xlsx")
}

/// Creates a spreadsheet from a list of property change events.
/// This is the primary export function for the application, designed to be
/// easily filterable and sortable by users in Excel.
pub fn export_changes_xlsx(events: &[ChangeViewModel], state: &str, year: i32) -> ResultResp {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Define the headers for our new event-log spreadsheet, as requested.
    let headers = [
        "Change Date",
        "Change Time",
        "Change Type",
        "Previous Value",
        "Current Value",
        "Full Address",
        "Address Line",
        "City",
        "State",
        "Zip",
        "County",
        "Current Price",
        "Price Reduction",
        "Canonical Status",
        "New Listing?",
        "Price Reduced Flag?",
        "Foreclosure?",
        "Ready to Build?",
        // Note: Beds and SqFt are no longer tracked in the simplified schema.
    ];

    // Write headers to the first row.
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string(0, col as u16, *header)?;
    }

    // Write the data rows, one row per change event.
    for (i, event) in events.iter().enumerate() {
        let row = (i + 1) as u32;

        worksheet.write_string(row, 0, &event.change_date.format("%Y-%m-%d").to_string())?;
        worksheet.write_string(row, 1, &event.change_date.format("%H:%M:%S").to_string())?;
        worksheet.write_string(row, 2, &event.change_type)?;
        worksheet.write_string(row, 3, &event.previous_value)?;
        worksheet.write_string(row, 4, &event.current_value)?;
        worksheet.write_string(row, 5, &event.address_full)?;
        worksheet.write_string(row, 6, &event.address_line)?;
        worksheet.write_string(row, 7, &event.city)?;
        worksheet.write_string(row, 8, event.state_abbr.as_deref().unwrap_or(""))?;
        worksheet.write_string(row, 9, &event.postal_code)?;
        worksheet.write_string(row, 10, event.county_name.as_deref().unwrap_or(""))?;

        if let Some(price) = event.price {
            worksheet.write_number(row, 11, price as f64)?;
        }
        if let Some(reduction) = event.price_reduction {
            worksheet.write_number(row, 12, reduction as f64)?;
        }

        worksheet.write_string(row, 13, &event.canonical_status)?;

        worksheet.write_string(row, 14, if event.is_new_listing { "Yes" } else { "No" })?;
        worksheet.write_string(row, 15, if event.is_price_reduced { "Yes" } else { "No" })?;
        worksheet.write_string(row, 16, if event.is_foreclosure { "Yes" } else { "No" })?;
        worksheet.write_string(row, 17, if event.is_ready_to_build { "Yes" } else { "No" })?;
    }

    let buffer = workbook.save_to_buffer()?;

    let filename = format!("changes_{}_{}.xlsx", state, year);
    Ok(xlsx_response(buffer, &filename)?)
}
