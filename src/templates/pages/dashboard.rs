use crate::templates::desktop_layout;
use maud::{html, Markup};

pub struct DashboardVm {
    pub email: String,
    pub plan_code: String,
    pub plan_name: String,
    pub download_limit: Option<i64>,
    pub is_admin: bool,
}

pub fn dashboard_page(vm: &DashboardVm) -> Markup {
    desktop_layout(
        "Dashboard",
        html! {
            main class="container" {
                h1 { "Dashboard" }
                p { "Signed in as " strong { (vm.email) } }

                section class="card" {
                    h3 { "Your plan" }
                    p { strong { (vm.plan_name) } " (" (vm.plan_code) ")" }

                    @match vm.download_limit {
                        Some(n) => p { "Download limit: " strong { (n) } " / month" },
                        None => p { "Download limit: " strong { "Unlimited" } },
                    }
                }

                section class="card" {
                    h3 { "Next steps" }
                    ul {
                        li { a href="/campaigns" { "Browse campaigns" } }
                        li { a href="/export/UT" { "Export a sample state (UT)" } }
                    }
                }

                @if vm.is_admin {
                    section class="card" {
                        h3 { "Admin" }
                        p { "You have admin access." }
                        a href="/admin" { "Go to Admin" }
                    }
                }
            }
        },
    )
}
