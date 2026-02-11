use maud::{html, Markup};

pub fn email_cta_form() -> Markup {
    html! {
        div class="email-cta-wrapper" {
            form
                method="post"
                action="/auth/request-link"
                hx-post="/auth/request-link"
                hx-target="#auth-result"
                hx-swap="innerHTML"
                hx-disabled-elt="button"
                class="email-cta"
            {
                label class="sr-only" for="email" { "Email address" }
                input
                    type="email"
                    id="email"
                    name="email"
                    placeholder="you@domain.com"
                    autocomplete="email"
                    required;

                button type="submit" class="primary" {
                    span class="btn-text" { "Get access" }
                    span class="spinner" aria-hidden="true" {}
                }

                p class="microcopy" {
                    "We’ll email you a secure sign-in link. No password needed."
                }
            }

            div id="auth-result" {}
        }
    }
}
