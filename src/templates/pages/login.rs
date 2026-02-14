use crate::templates::{components::email_cta_form, desktop_layout};
use maud::{html, Markup};

pub fn login_page(is_admin: bool) -> Markup {
    desktop_layout(
        "Sign in",
        is_admin,
        html! {
            main class="container narrow" {
                h1 { "Sign in" }
                p class="lead" {
                    "Enter your email and we’ll send you a secure sign-in link."
                }

                (email_cta_form())
            }
        },
    )
}
