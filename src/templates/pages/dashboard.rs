// src/templates/pages/dashboard.rs

use crate::domain::changes::ChangeViewModel;
use crate::templates::desktop_layout;
use maud::{html, Markup};

/// Renders the main "Changes Dashboard" page.
pub fn dashboard_page(changes: &[ChangeViewModel], years: &[String]) -> Markup {
    desktop_layout(
        "Dashboard",
        true, // is_admin flag for layout
        html! {
            // Page Header
            div class="mb-6" {
                h1 class="text-3xl font-bold text-gray-800" { "Changes Dashboard" }
                p class="text-gray-500 mt-1" { "Download change events or preview the most recent updates." }
            }

            // --- Export Form Card ---
            div class="bg-white border rounded-lg shadow-sm p-6 mb-8" {
                h2 class="text-xl font-semibold text-gray-800 mb-4" { "Download Change Log" }
                p class="text-sm text-gray-600 mb-6" {
                    "Select a state and year to download a full spreadsheet (.xlsx) of all recorded change events. This is ideal for detailed sorting and filtering."
                }

                form action="/export/changes" method="get" class="flex items-end space-x-4" {
                    // State Selector
                    div {
                        label for="state" class="block text-sm font-medium text-gray-700 mb-1" { "State" }
                        select name="state" id="state" required class="w-48 p-2 border border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500" {
                            option value="" disabled selected { "Select a State..." }
                            @for (abbr, name) in crate::geos::US_STATES {
                                option value=(abbr) { (name) }
                            }
                        }
                    }
                    // Year Selector
                    div {
                        label for="year" class="block text-sm font-medium text-gray-700 mb-1" { "Year" }
                        select name="year" id="year" required class="w-32 p-2 border border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500" {
                            @if years.is_empty() {
                                option value="" disabled selected { "No Data" }
                            } @else {
                                @for (i, year) in years.iter().enumerate() {
                                    @if i == 0 {
                                        option value=(year) selected { (year) }
                                    } @else {
                                        option value=(year) { (year) }
                                    }
                                }
                            }
                        }
                    }
                    // Submit Button
                    div {
                        button type="submit" class="px-5 py-2 bg-indigo-600 text-white font-semibold rounded-md shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" {
                            "Download"
                        }
                    }
                }
            }


            // --- Recent Changes Preview ---
            div {
                h2 class="text-xl font-semibold text-gray-800 mb-4" { "Recent Changes Preview" }
            }

            // Conditional rendering: show the table if there are changes, otherwise show an empty state.
            @if changes.is_empty() {
                div class="text-center py-12 px-6 bg-white border rounded-lg shadow-sm" {
                    svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01" {}
                    }
                    h3 class="mt-2 text-sm font-medium text-gray-900" { "No recent changes found" }
                    p class="mt-1 text-sm text-gray-500" { "Try running a new scrape from the admin panel to see the latest updates." }
                }
            } @else {
                // Data Table for displaying changes
                div class="overflow-x-auto bg-white border rounded-lg shadow-sm" {
                    table class="min-w-full divide-y divide-gray-200" {
                        thead class="bg-gray-50" {
                            tr {
                                th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Property" }
                                th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Last Changed" }
                                th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Status" }
                                th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Price" }
                                th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Flags" }
                            }
                        }
                        tbody class="bg-white divide-y divide-gray-200" {
                            @for change in changes.iter().take(15) { // <<< LIMIT THE PREVIEW HERE
                                tr class="hover:bg-gray-50 transition-colors duration-150" {
                                    // Property Column
                                    td class="px-6 py-4 whitespace-nowrap" {
                                        div class="text-sm font-medium text-gray-900" { (change.address_line) }
                                        div class="text-sm text-gray-500" { (change.city) ", " (change.postal_code) }
                                    }
                                    // Last Changed Column
                                    td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500" {
                                        div { (change.change_date.format("%Y-%m-%d")) }
                                        div class="text-xs text-gray-400" { (change.change_date.format("%-I:%M %p")) }
                                        div class="text-xs text-indigo-600 font-medium" { (change.change_type) }
                                    }
                                    // Status Column
                                    td class="px-6 py-4 whitespace-nowrap text-sm" {
                                        @if change.change_type == "Status Change" {
                                            (format_status(&change.previous_value))
                                            span class="mx-1 text-gray-400" { "→" }
                                            (format_status(&change.current_value))
                                        } @else {
                                            (format_status(&change.canonical_status))
                                        }
                                    }
                                    // Price Column
                                    td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500" {
                                        @if change.change_type == "Price Change" {
                                            div {
                                                span class="line-through" { "$" (change.previous_value) }
                                                span { " → $" (change.current_value) }
                                            }
                                            @if let Some(amt) = change.price_reduction {
                                                span class="text-xs font-semibold inline-block py-1 px-2 rounded-full text-red-600 bg-red-200" {
                                                    (format!("-{}", format_price(amt)))
                                                }
                                            }
                                        } @else if let Some(curr) = change.price {
                                            (format_price(curr))
                                        } @else {
                                            "N/A"
                                        }
                                    }
                                    // Flags Column
                                    td class="px-6 py-4 whitespace-nowrap text-sm" {
                                        @if change.is_new_listing {
                                            div class="font-semibold text-green-800" { "New Listing" }
                                        }
                                        @if change.is_foreclosure {
                                            div class="font-semibold text-red-800" { "Foreclosure" }
                                        }
                                        @if change.is_price_reduced {
                                            div class="font-semibold text-red-600" { "Price Reduced" }
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

/// Helper function to format a price for display with a dollar sign and commas.
fn format_price(price: i64) -> String {
    if price == 0 {
        return "N/A".to_string();
    }
    // This is a simple but effective formatter for US currency.
    let price_str = price.to_string();
    let mut result = String::new();
    let len = price_str.len();
    let first = len % 3;
    if first > 0 {
        result.push_str(&price_str[..first]);
    }
    for (i, chunk) in price_str[first..].as_bytes().chunks(3).enumerate() {
        if first > 0 || i > 0 {
            result.push(',');
        }
        result.push_str(std::str::from_utf8(chunk).unwrap());
    }
    format!("${}", result)
}

/// Helper function to render a status string as a colored badge.
fn format_status(status: &str) -> Markup {
    let (text, color_classes) = match status {
        "for_sale" => ("For Sale", "bg-blue-100 text-blue-800"),
        "contingent" => ("Contingent", "bg-yellow-100 text-yellow-800"),
        "pending" => ("Pending", "bg-orange-100 text-orange-800"),
        "sold" => ("Sold", "bg-green-100 text-green-800"),
        s if s.is_empty() => ("(unknown)", "bg-gray-100 text-gray-800"),
        s => (s, "bg-gray-100 text-gray-800"), // Default case for any other status
    };
    html! {
        span class={"px-2 inline-flex text-xs leading-5 font-semibold rounded-full " (color_classes)} {
            (text)
        }
    }
}
