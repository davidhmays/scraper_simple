pub mod errors;
pub mod html;

// These two *are* in responses/errors.rs
pub use errors::{html_error_response, ResultResp};

// ServerError is NOT in responses/errors.rs anymore.
// It lives in the top-level `errors.rs`
pub use crate::errors::ServerError;

// Normal HTML response
pub use html::html_response;
