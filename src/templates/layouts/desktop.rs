// use crate::templates::components::navbar;
use maud::{html, Markup, DOCTYPE};

pub fn desktop_layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }
                link rel="icon" type="image/x-icon" href="/static/favicon/favicon.svg";
                link rel="alternate icon" href="/static/favicon/favicon.ico";
                link rel="stylesheet" href="/static/main.css";
            }
            body {
              header class="flex items-center justify-between px-6 py-3 shadow" {
                  // a href="/"{ "MySite" }
                  h3 { "Download Listings" }
                  nav {
                      ul {
                          li { a href="/" { "Home" } }
                          li { a href="/admin" { "Admin" }
                            //li { a href="/about" class="hover:text-blue-600" { "About" }
                          }
                          li { a href="/campaigns" { "Campaigns" } }
                      }
                  }

                  a href="/login" class="text-base font-medium hover:text-blue-600" { "Login" }
              }
                (content)
            }
        }
    }
}
// [:h3 "Download Listings"]]
// [:a {:href "/"} "Home"]
// [:a {:href "/admin"} "Admin"]
// [:div {:class "inline"}
// [:a {:href "/register"} "Create Account"]
// [:a {:href "/sign-in"}
//  [:button "Sign In"]]]]
// content
// [:footer "hi"]]))
