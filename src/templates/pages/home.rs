use crate::templates::{
    components::{button, card},
    desktop_layout,
};
use maud::{html, Markup};

pub fn home_page() -> Markup {
    desktop_layout(
        "Home",
        html! {
            h1 { "Sort Real Estate Listings Your Way" }
            h2 { "Download current property data as a spreadsheet." }
            h2 { "Updated daily, any state, completely free." }
            (button("Click me!"))

            (card("About this site", html! {
                p { "This is an example page built with Maud templates." }
            }))
        },
    )
}

// [:main
//  [:h1 "Sort Real Estate Listings Your Way"]
//  [:h2 "Download current property data as a spreadsheet."]
//  [:h2 "Updated daily, any state, completely free."]
//  [:div {:class "inline selection"}
//   (state-dropdown data/us-states)
//   [:button {:id "download"} "download"]]
//  [:script {:src "/home-page.js" :defer true}]]))
