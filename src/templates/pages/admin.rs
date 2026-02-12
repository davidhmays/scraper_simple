use crate::templates::desktop_layout;
use maud::{html, Markup};

pub struct AdminVm {
    pub users: Vec<crate::db::users::UserWithStats>,
}

pub fn admin_page(vm: &AdminVm) -> Markup {
    desktop_layout(
        "Admin Dashboard",
        html! {
            main class="container" {
                h1 { "Admin Dashboard" }

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
