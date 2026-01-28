use crate::domain::listing::ListingWithProperty;
use crate::errors::ServerError;
use crate::responses::xlsx_response;
use crate::responses::ResultResp;
use rust_xlsxwriter::Workbook;

pub fn export_listings_xlsx(listings: &[ListingWithProperty], state: &str) -> ResultResp {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Headers
    let headers = [
        "Address",
        "City",
        "State",
        "Zip / Postal",
        "County",
        "Price",
        "Beds",
        "Baths",
        "Status",
        "Coming Soon",
        "Contingent",
        "Pending",
    ];

    for (col, header) in headers.iter().enumerate() {
        worksheet
            .write_string(0, col as u16, *header)
            .map_err(|e| {
                ServerError::XlsxError(format!("Failed to write header '{}': {}", header, e))
            })?;
    }

    // Rows
    for (i, listing) in listings.iter().enumerate() {
        let r = (i + 1) as u32;

        worksheet
            .write_string(r, 0, &listing.address_line)
            .map_err(|e| ServerError::XlsxError(format!("Failed to write address: {}", e)))?;

        worksheet
            .write_string(r, 1, &listing.city)
            .map_err(|e| ServerError::XlsxError(format!("Failed to write city: {}", e)))?;

        worksheet
            .write_string(r, 2, &listing.state_abbr)
            .map_err(|e| ServerError::XlsxError(format!("Failed to write state: {}", e)))?;

        let postal_code = listing.postal_code.as_deref().unwrap_or("");
        worksheet
            .write_string(r, 3, postal_code)
            .map_err(|e| ServerError::XlsxError(format!("Failed to write postal code: {}", e)))?;

        let county_name = listing.county_name.as_deref().unwrap_or("");
        worksheet
            .write_string(r, 4, county_name)
            .map_err(|e| ServerError::XlsxError(format!("Failed to write county name: {}", e)))?;

        worksheet
            .write_number(r, 5, listing.list_price as f64)
            .map_err(|e| ServerError::XlsxError(format!("Failed to write price: {}", e)))?;

        worksheet
            .write_number(r, 6, listing.bedrooms.unwrap_or(0) as f64)
            .map_err(|e| ServerError::XlsxError(format!("Failed to write bedrooms: {}", e)))?;

        worksheet
            .write_number(r, 7, listing.bathrooms.unwrap_or(0) as f64)
            .map_err(|e| ServerError::XlsxError(format!("Failed to write bathrooms: {}", e)))?;

        worksheet
            .write_string(r, 8, &listing.status)
            .map_err(|e| ServerError::XlsxError(format!("Failed to write status: {}", e)))?;

        worksheet
            .write_string(r, 9, if listing.is_coming_soon { "Yes" } else { "No" })
            .map_err(|e| ServerError::XlsxError(format!("Failed to write coming soon: {}", e)))?;

        worksheet
            .write_string(r, 10, if listing.is_contingent { "Yes" } else { "No" })
            .map_err(|e| ServerError::XlsxError(format!("Failed to write contingent: {}", e)))?;

        worksheet
            .write_string(r, 11, if listing.is_pending { "Yes" } else { "No" })
            .map_err(|e| ServerError::XlsxError(format!("Failed to write pending: {}", e)))?;
    }

    let buffer = workbook
        .save_to_buffer()
        .map_err(|e| ServerError::XlsxError(format!("Failed to save workbook: {}", e)))?;

    xlsx_response(buffer, &format!("listings_{state}.xlsx"))
}
