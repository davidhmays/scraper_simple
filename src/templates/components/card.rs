use maud::{html, Markup};

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
