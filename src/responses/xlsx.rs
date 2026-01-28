// responses/xlsx.rs
use crate::errors::ServerError;
use crate::responses::ResultResp;
use astra::{Body, ResponseBuilder};

/// Return XLSX file as HTTP response
pub fn xlsx_response(buffer: Vec<u8>, filename: &str) -> ResultResp {
    let resp = ResponseBuilder::new()
        .status(200)
        .header(
            "Content-Type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(buffer))
        .map_err(|_| ServerError::InternalError)?; // Convert any builder error

    Ok(resp)
}
