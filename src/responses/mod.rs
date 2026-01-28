pub mod errors;
pub mod html;
pub mod xlsx;

// These two *are* in responses/errors.rs
pub use errors::{html_error_response, ResultResp};

// Normal HTML response
pub use html::html_response;
pub use xlsx::xlsx_response;
