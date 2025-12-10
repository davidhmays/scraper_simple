// templates/pages/home.rs

use crate::templates::{
    components::{button, card},
    desktop_layout,
};
use maud::{html, Markup};

pub fn home_page() -> Markup {
    desktop_layout(
        "Home",
        html! {
            h1 { "Welcome to the Home Page" }

            (button("Click me!"))

            (card("About this site", html! {
                p { "This is an example page built with Maud templates." }
            }))
        },
    )
}
