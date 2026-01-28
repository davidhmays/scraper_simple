use crate::errors::ServerError;
use crate::responses::{xlsx_response, ResultResp};
use rust_xlsxwriter::Workbook;
use crate::db::connection::Database;
use rusqlite::params;

// TODO: How to set correctly when published off localhost?
const QR_BASE_URL: &str = "https://yourdomain.com/m";

pub struct MailingExportRow {
    pub property_id: String,
    pub address_line: String,
    pub city: String,
    pub state_abbr: String,
    pub postal_code: String,
    pub county_name: Option<String>,
    pub qr_url: String,
}

pub fn export_mailings_xlsx(rows: &[MailingExportRow], filename: &str) -> ResultResp {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let headers = ["Property ID", "Address", "City", "State", "Zip", "QR URL"];
    for (col, header) in headers.iter().enumerate() {
        worksheet
            .write_string(0, col as u16, *header)
            .map_err(|e| ServerError::XlsxError(format!("Failed to write header: {}", e)))?;
    }

    for (i, row) in rows.iter().enumerate() {
        let r = (i + 1) as u32;
        worksheet.write_string(r, 0, &row.property_id)
            .map_err(|e| ServerError::XlsxError(format!("property_id: {}", e)))?;
        worksheet.write_string(r, 1, &row.address_line)
            .map_err(|e| ServerError::XlsxError(format!("address: {}", e)))?;
        worksheet.write_string(r, 2, &row.city)
            .map_err(|e| ServerError::XlsxError(format!("city: {}", e)))?;
        worksheet.write_string(r, 3, &row.state_abbr)
            .map_err(|e| ServerError::XlsxError(format!("state: {}", e)))?;
        worksheet.write_string(r, 4, &row.postal_code)
            .map_err(|e| ServerError::XlsxError(format!("zip: {}", e)))?;
        worksheet.write_string(r, 5, &row.qr_url)
            .map_err(|e| ServerError::XlsxError(format!("qr_url: {}", e)))?;
    }

    let buffer = workbook
        .save_to_buffer()
        .map_err(|e| ServerError::XlsxError(format!("Failed to save workbook: {}", e)))?;

    xlsx_response(buffer, filename)
}

pub fn get_mailings_export_rows(
    db: &Database,
    campaign: &str,
    variant: &str,
) -> Result<Vec<MailingExportRow>, ServerError> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT
                  m.property_id,
                  m.address_line,
                  m.city,
                  m.state_abbr,
                  m.postal_code,
                  l.county_name,
                  m.qr_token
                FROM mailings m
                JOIN listings l ON l.id = m.listing_id
                WHERE m.campaign = ?1 AND m.variant = ?2
                ORDER BY m.city, m.address_line
                "#,
            )
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let rows = stmt
            .query_map(params![campaign, variant], |row| {
                let token: String = row.get(6)?;
                Ok(MailingExportRow {
                    property_id: row.get(0)?,
                    address_line: row.get(1)?,
                    city: row.get(2)?,
                    state_abbr: row.get(3)?,
                    postal_code: row.get(4)?,
                    county_name: row.get(5)?,
                    qr_url: format!("{}/{}", QR_BASE_URL, token),
                })
            })
            .map_err(|e| ServerError::DbError(e.to_string()))?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| ServerError::DbError(e.to_string()))?);
        }
        Ok(out)
    })
}
