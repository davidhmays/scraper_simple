use crate::templates::{components::button, desktop_layout};
use maud::{html, Markup};

pub fn campaigns_page(selected_state: &str, counties: &[(String, i64)], is_admin: bool) -> Markup {
    desktop_layout(
        "Campaigns",
        is_admin,
        html! {
            h1 { "Campaigns & QR Codes" }

            form method="post" action="/campaigns" {
                // --- State ---
                label for="state" { "State" }
                select
                    id="state"
                    name="state"
                    required
                    onchange="window.location='/campaigns?state=' + this.value"
                {
                    option value="AL" selected[selected_state == "AL"] { "Alabama" }
                    option value="AK" selected[selected_state == "AK"] { "Alaska" }
                    option value="AZ" selected[selected_state == "AZ"] { "Arizona" }
                    option value="UT" selected[selected_state == "UT"] { "Utah" }
                }

                // --- County ---
                label for="counties" { "Counties (optional; multi-select)" }
                select id="counties" name="counties" multiple size="10" {
                    @for (county_name, n) in counties {
                        option value=(county_name) { (county_name) " (" (n) ")" }
                    }
                }

                // --- Listing flags (OR) ---
                fieldset {
                    legend { "Flags (any-of / OR)" }
                    label { input type="checkbox" checked name="flags" value="pending"; " Pending" }
                    label { input type="checkbox" checked name="flags" value="contingent"; " Contingent" }
                    label { input type="checkbox" checked name="flags" value="coming_soon"; " Coming Soon" }
                    label { input type="checkbox" checked name="flags" value="new_listing"; " New Listing" }
                    label { input type="checkbox" checked name="flags" value="new_construction"; " New Construction" }
                }

                // --- Property types (OR) ---
                fieldset {
                    legend { "Property Types (any-of / OR)" }
                    label { input type="checkbox" checked name="types" value="single_family"; " Single Family" }
                    label { input type="checkbox" checked name="types" value="condos"; " Condos" }
                    label { input type="checkbox" checked name="types" value="townhomes"; " Townhomes" }
                    label { input type="checkbox" checked name="types" value="multi_family"; " Multi Family" }
                    label { input type="checkbox" checked name="types" value="land"; " Land" }
                    label { input type="checkbox" checked name="types" value="farm"; " Farm" }
                }

                button type="submit" { "Create Campaign" }
            }
        },
    )
}

// Key point: use repeated keys (flags=...&flags=... and types=...&types=...)
//   that’s the easiest way to represent vectors in x-www-form-urlencoded.
