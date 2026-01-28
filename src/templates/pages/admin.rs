use crate::templates::{components::button, desktop_layout};
use maud::{html, Markup};

pub fn admin_page() -> Markup {
    desktop_layout(
        "Admin",
        html! {
            h1 { "Admin" }

            form method="post" action="/update-db" {
                label for="state" { "Select State" }

                select id="state" name="state" {

                    option value="AL" { "Alabama" }
                    option value="AK" { "Alaska" }
                    option value="AZ" { "Arizona" }
                    option value="UT" { "Utah" }
                }

                button type="submit" { "Update Database" }
            }

            // (card("About this site", html! {
            //     p { "This is an example page built with Maud templates." }
            // }))
        },
    )
}
