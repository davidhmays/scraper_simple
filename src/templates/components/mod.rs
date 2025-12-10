use maud::{html, Markup};

pub mod error;

pub use error::html_error_response;

pub fn button(label: &str) -> Markup {
    html! {
        button class="btn" { (label) }
    }
}

pub fn card(title: &str, body: Markup) -> Markup {
    html! {
        div class="card" {
            h2 { (title) }
            div class="card-body" {
                (body)
            }
        }
    }
}
