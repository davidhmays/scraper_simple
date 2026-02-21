use crate::templates::{
    components::{card, email_cta_form},
    desktop_layout,
};
use maud::{html, Markup};

pub fn home_page(is_admin: bool) -> Markup {
    desktop_layout(
        "Property Status Changes",
        is_admin,
        html! {
            main class="container" {
                // HERO
                section class="hero" {
                    // Keep your existing headings exactly
                    h1 { "Sort Real Estate Listings Your Way" }
                    h2 { "Download current property data as a spreadsheet." }
                    h2 { "Updated daily, any state, completely free." }

                    // Replace the placeholder CTA with an email-first CTA
                    (email_cta_form())

                    // Trust row + sample link
                    div class="hero-meta" {
                        ul class="trust-row" {
                            li { "✓ Email only" }
                            li { "✓ No password" }
                            li { "✓ CSV / Excel" }
                        }
                        a href="/sample" class="sample-link" { "See sample file" }
                    }
                }

                // HOW IT WORKS
                section id="how-it-works" class="section" {
                    h3 { "How it works" }
                    div class="grid-3" {
                        (card("1) Enter email", html! {
                            p { "We’ll email you a secure sign-in link. No password needed." }
                        }))
                        (card("2) Pick your state", html! {
                            p { "Choose the state you want, then select your format." }
                        }))
                        (card("3) Download", html! {
                            p { "Get a spreadsheet in seconds—ready for Excel or Google Sheets." }
                        }))
                    }
                }

                // ABOUT (your placeholder card is fine here)
                (card("About this site", html! {
                    p { "This is an example page built with Maud templates." }
                }))

                // FAQ (optional but very helpful)
                section id="faq" class="section" {
                    h3 { "FAQ" }
                    div class="faq" {
                        details {
                            summary { "Do I need a password?" }
                            p { "No—sign in via emailed link." }
                        }
                        details {
                            summary { "How often is data updated?" }
                            p { "Daily." }
                        }
                        details {
                            summary { "What format is the download?" }
                            p { "CSV (and optionally Excel)." }
                        }
                        details {
                            summary { "Can I download multiple states?" }
                            p { "Yes." }
                        }
                    }
                }

                // FINAL CTA (for scrollers)
                section class="final-cta" {
                    h3 { "Ready to download listings?" }
                    (email_cta_form())
                    p class="fine-print" { "No spam. Unsubscribe anytime." }
                }
            }
        },
    )
}
