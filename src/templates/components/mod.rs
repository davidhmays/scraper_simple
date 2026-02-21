pub mod card;
pub mod email_cta;
pub mod error;

use maud::{html, Markup};

pub use card::card;
pub use email_cta::email_cta_form;
pub use error::html_error_response;

// pub fn button(label: &str) -> Markup {
//     html! {
//         button class="btn" { (label) }
//     }
// }
