use crate::templates::desktop_layout;
use maud::{html, Markup};

pub struct AdminVm {
    pub users: Vec<crate::db::users::UserWithStats>,
    pub plans: Vec<crate::db::plans::PlanInfo>,
    pub scrapes: Vec<crate::db::scrapes::ScrapeRun>,
}

pub fn admin_page(vm: &AdminVm) -> Markup {
    desktop_layout(
        "Admin Dashboard",
        true,
        html! {
            main class="container" {
                h1 { "Admin Dashboard" }

                div class="card" style="margin-bottom: 2rem;" {
                    h3 { "Plans Configuration" }
                    div style="overflow-x: auto;" {
                        table style="width: 100%; border-collapse: collapse; margin-top: 1rem;" {
                            thead {
                                tr {
                                    th style="padding: 12px 8px; border-bottom: 2px solid #e5e7eb; text-align: left;" { "Code" }
                                    th style="padding: 12px 8px; border-bottom: 2px solid #e5e7eb; text-align: left;" { "Name" }
                                    th style="padding: 12px 8px; border-bottom: 2px solid #e5e7eb; text-align: left;" { "Limit" }
                                    th style="padding: 12px 8px; border-bottom: 2px solid #e5e7eb; text-align: left;" { "Update" }
                                }
                            }
                            tbody {
                                @for plan in &vm.plans {
                                    tr {
                                        td style="padding: 8px; border-bottom: 1px solid #f3f4f6;" { (plan.code) }
                                        td style="padding: 8px; border-bottom: 1px solid #f3f4f6;" { (plan.name) }
                                        td style="padding: 8px; border-bottom: 1px solid #f3f4f6;" {
                                            @match plan.download_limit {
                                                Some(n) => (n),
                                                None => "Unlimited",
                                            }
                                        }
                                        td style="padding: 8px; border-bottom: 1px solid #f3f4f6;" {
                                            @if plan.code == "lifetime" {
                                                span style="color: #6b7280; font-style: italic;" { "Fixed" }
                                            } @else {
                                                form action=(format!("/admin/plans/{}/limit", plan.code)) method="post" style="display: flex; gap: 8px; align-items: center; margin: 0;" {
                                                    input type="number" name="limit" value=(plan.download_limit.unwrap_or(0)) style="padding: 4px; width: 80px; border: 1px solid #ccc; border-radius: 4px;" required;
                                                    button type="submit" style="padding: 4px 8px; background: #3b82f6; color: white; border: none; border-radius: 4px; cursor: pointer;" { "Set" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div class="card" style="margin-bottom: 2rem;" {
                    h3 { "Scraper Control" }
                    form action="/admin/scrape" method="post" style="display: flex; gap: 10px; align-items: center; margin-bottom: 1rem;" {
                        select name="state" required style="padding: 8px; border-radius: 4px; border: 1px solid #ccc;" {
                            option value="" disabled selected { "Select State to Scrape..." }
                            @for (abbr, name) in crate::geos::US_STATES {
                                option value=(abbr) { (name) }
                            }
                        }
                        button type="submit" style="padding: 8px 16px; background: #10b981; color: white; border: none; border-radius: 4px; cursor: pointer;" { "Start Scrape Job" }
                    }

                    h4 { "Recent Runs" }
                    div style="overflow-x: auto;" {
                        table style="width: 100%; border-collapse: collapse; font-size: 0.9em;" {
                            thead {
                                tr {
                                    th style="padding: 8px; text-align: left; border-bottom: 2px solid #eee;" { "ID" }
                                    th style="padding: 8px; text-align: left; border-bottom: 2px solid #eee;" { "State" }
                                    th style="padding: 8px; text-align: left; border-bottom: 2px solid #eee;" { "Started" }
                                    th style="padding: 8px; text-align: left; border-bottom: 2px solid #eee;" { "Status" }
                                    th style="padding: 8px; text-align: left; border-bottom: 2px solid #eee;" { "Pages" }
                                    th style="padding: 8px; text-align: left; border-bottom: 2px solid #eee;" { "Found" }
                                }
                            }
                            tbody {
                                @for run in &vm.scrapes {
                                    tr {
                                        td style="padding: 8px; border-bottom: 1px solid #f9f9f9;" { (run.id) }
                                        td style="padding: 8px; border-bottom: 1px solid #f9f9f9;" { (run.state) }
                                        td style="padding: 8px; border-bottom: 1px solid #f9f9f9;" { (run.started_at) }
                                        td style="padding: 8px; border-bottom: 1px solid #f9f9f9;" {
                                            @if run.finished_at.is_none() {
                                                span style="color: blue;" { "Running..." }
                                            } @else if run.success == Some(true) {
                                                span style="color: green;" { "Success" }
                                            } @else {
                                                span style="color: red;" { "Failed" }
                                                @if let Some(err) = &run.error_message {
                                                    br; span style="font-size: 0.8em; color: #666;" { (err) }
                                                }
                                            }
                                        }
                                        td style="padding: 8px; border-bottom: 1px solid #f9f9f9;" { (run.pages_fetched.unwrap_or(0)) }
                                        td style="padding: 8px; border-bottom: 1px solid #f9f9f9;" { (run.properties_seen.unwrap_or(0)) }
                                    }
                                }
                            }
                        }
                    }
                }

                div class="card" {
                    h3 { "Users Management" }
                    div style="overflow-x: auto;" {
                        table style="width: 100%; border-collapse: collapse; margin-top: 1rem;" {
                            thead {
                                tr {
                                    th style="padding: 12px 8px; border-bottom: 2px solid #e5e7eb; text-align: left;" { "ID" }
                                    th style="padding: 12px 8px; border-bottom: 2px solid #e5e7eb; text-align: left;" { "Email" }
                                    th style="padding: 12px 8px; border-bottom: 2px solid #e5e7eb; text-align: left;" { "Plan" }
                                    th style="padding: 12px 8px; border-bottom: 2px solid #e5e7eb; text-align: left;" { "Usage (Mo)" }
                                    th style="padding: 12px 8px; border-bottom: 2px solid #e5e7eb; text-align: left;" { "Role" }
                                    th style="padding: 12px 8px; border-bottom: 2px solid #e5e7eb; text-align: left;" { "Last Login" }
                                    th style="padding: 12px 8px; border-bottom: 2px solid #e5e7eb; text-align: left;" { "Actions" }
                                }
                            }
                            tbody {
                                @for user in &vm.users {
                                    tr {
                                        td style="padding: 8px; border-bottom: 1px solid #f3f4f6;" { (user.id) }
                                        td style="padding: 8px; border-bottom: 1px solid #f3f4f6;" { (user.email) }
                                        td style="padding: 8px; border-bottom: 1px solid #f3f4f6;" {
                                            span style="background: #e5e7eb; padding: 2px 6px; border-radius: 4px; font-size: 0.85em;" {
                                                (user.plan_name.as_deref().unwrap_or("None"))
                                            }
                                        }
                                        td style="padding: 8px; border-bottom: 1px solid #f3f4f6;" { (user.usage_this_month) }
                                        td style="padding: 8px; border-bottom: 1px solid #f3f4f6;" {
                                            @if user.is_admin {
                                                span style="background: #dbeafe; color: #1e40af; padding: 2px 6px; border-radius: 4px; font-size: 0.85em; font-weight: 500;" { "Admin" }
                                            } @else {
                                                "User"
                                            }
                                        }
                                        td style="padding: 8px; border-bottom: 1px solid #f3f4f6; color: #6b7280; font-size: 0.9em;" {
                                            @match user.last_login_at {
                                                Some(ts) => (ts), // Todo: Format date if needed
                                                None => "Never",
                                            }
                                        }
                                        td style="padding: 8px; border-bottom: 1px solid #f3f4f6;" {
                                            form action=(format!("/admin/users/{}/reset-usage", user.id)) method="post" onsubmit="return confirm('Reset download usage for this user?');" style="margin: 0;" {
                                                button type="submit" style="color: #dc2626; background: none; border: none; cursor: pointer; font-size: 0.9em; font-weight: 500; padding: 0;" {
                                                    "Reset Limit"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}
