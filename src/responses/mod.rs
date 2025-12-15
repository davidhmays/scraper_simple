pub mod errors;
pub mod html;

// These two *are* in responses/errors.rs
pub use errors::{html_error_response, ResultResp};

// Normal HTML response
pub use html::html_response;
