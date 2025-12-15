pub mod card;
pub mod error;

use maud::{html, Markup};

pub use card::card;
pub use error::html_error_response;

pub fn button(label: &str) -> Markup {
    html! {
        button class="btn" { (label) }
    }
}
