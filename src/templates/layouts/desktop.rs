// use crate::templates::components::navbar;
use maud::{html, Markup, DOCTYPE};

pub fn desktop_layout(title: &str, is_admin: bool, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }
                link rel="icon" href="/static/favicon/favicon.ico";
                link rel="icon" type="image/svg+xml" href="/static/favicon/favicon.svg";
                link rel="shortcut icon" href="/static/favicon/favicon.ico";
                link rel="stylesheet" href="/static/main.css";
                script src="/static/htmx.js" defer {};
            }
            body {
              header class="flex items-center justify-between px-6 py-3 shadow" {
                  // a href="/"{ "MySite" }
                  svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="24"
                      height="24"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="#524ed2"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      class="icon icon-tabler icon-tabler-home"
                  {
                      path stroke="none" d="M0 0h24v24H0z" fill="none" {}
                      path d="M5 12l-2 0l9 -9l9 9l-2 0" {}
                      path d="M5 12v7a2 2 0 0 0 2 2h10a2 2 0 0 0 2 -2v-7" {}
                      path d="M9 21v-6a2 2 0 0 1 2 -2h2a2 2 0 0 1 2 2v6" {}
                  }
                  h3 { "Download Listings" }
                  nav {
                      ul {
                          li { a href="/" { "Home" } }
                          @if is_admin {
                              li { a href="/admin" { "Admin" } }
                          }
                          // li { a href="/campaigns" { "Campaigns" } }
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
