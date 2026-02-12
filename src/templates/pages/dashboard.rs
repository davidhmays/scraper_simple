use crate::templates::desktop_layout;
use maud::{html, Markup};

pub struct DashboardVm {
    pub email: String,
    pub plan_code: String,
    pub plan_name: String,
    pub download_limit: Option<i64>,
    pub usage: i64,
    pub is_admin: bool,
    pub last_state: Option<String>,
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

                (export_card(vm))

                section class="card" {
                    h3 { "Campaigns" }
                    ul {
                        li { a href="/campaigns" { "Browse campaigns" } }
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

pub fn export_card(vm: &DashboardVm) -> Markup {
    let limit_reached = vm
        .download_limit
        .map(|limit| vm.usage >= limit)
        .unwrap_or(false);

    html! {
        section
            class="card"
            id="export-card"
            hx-get="/dashboard/export-card"
            hx-trigger="refresh"
            hx-swap="outerHTML"
        {
            h3 { "Export Listings" }

            @if let Some(limit) = vm.download_limit {
                div style="margin-bottom: 1rem;" {
                    p { "Used this month: " strong { (vm.usage) } " / " (limit) }
                    @if limit_reached {
                        p style="color: #dc2626; font-weight: bold;" { "Limit reached." }
                    }
                }
            }

            form
                action="/export"
                method="get"
                onsubmit="setTimeout(() => htmx.trigger('#export-card', 'refresh'), 2000)"
                style="display: flex; gap: 10px; align-items: center; margin-top: 10px;"
            {
                label for="state" class="sr-only" { "Select State" }
                select name="state" id="state" required style="padding: 8px; font-size: 16px;" {
                    option value="" disabled selected[vm.last_state.is_none()] { "Select a State..." }
                    @for (abbr, name) in crate::geos::US_STATES {
                        option value=(abbr) selected[vm.last_state.as_deref() == Some(*abbr)] { (name) }
                    }
                }
                @if limit_reached {
                    button type="button" disabled style="padding: 8px 16px; font-size: 16px; cursor: not-allowed; opacity: 0.6; background-color: #9ca3af; color: white; border: none; border-radius: 4px;" { "Limit Reached" }
                } @else {
                    button type="submit" style="padding: 8px 16px; font-size: 16px; cursor: pointer;" { "Download" }
                }
            }
        }
    }
}
