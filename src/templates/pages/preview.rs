use crate::domain::listing::ListingWithProperty;
use maud::{html, Markup};

pub fn preview_table(
    listings: &[ListingWithProperty],
    total_count: usize,
    is_paid: bool,
) -> Markup {
    html! {
        div class="mt-6 fade-in" {
            div class="flex items-center justify-between mb-3" {
                p class="text-gray-700" {
                    "Found " strong { (total_count) } " records."
                }
                @if !is_paid {
                    span class="text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded-full font-medium" { "Preview Mode" }
                }
            }

            div class="overflow-hidden border border-gray-200 rounded-lg shadow-sm" {
                table class="min-w-full divide-y divide-gray-200" {
                    thead class="bg-gray-50" {
                        tr {
                            th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Address" }
                            th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "City" }
                            th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Price" }
                            th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Status" }
                        }
                    }
                    tbody class="bg-white divide-y divide-gray-200" {
                        @for listing in listings.iter().take(5) {
                            tr {
                                td class="px-4 py-3 whitespace-nowrap text-sm text-gray-900" {
                                    @if is_paid {
                                        (listing.address_line)
                                    } @else {
                                        span class="blur-sm select-none text-transparent bg-gray-200 rounded px-1" style="filter: blur(4px); user-select: none;" { (listing.address_line) }
                                        // Also show partial text for context if blur is too aggressive or fails to load CSS
                                        span class="ml-2 text-gray-400 font-mono text-xs" { (listing.redacted_address()) }
                                    }
                                }
                                td class="px-4 py-3 whitespace-nowrap text-sm text-gray-500" { (listing.city) }
                                td class="px-4 py-3 whitespace-nowrap text-sm text-gray-500" { "$" (listing.list_price) }
                                td class="px-4 py-3 whitespace-nowrap text-sm text-gray-500" {
                                    span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800" {
                                        (listing.status)
                                    }
                                }
                            }
                        }
                    }
                }
            }

            @if !is_paid && total_count > 0 {
                div class="mt-6 p-6 bg-gradient-to-r from-blue-50 to-indigo-50 border border-blue-100 rounded-lg text-center shadow-sm" {
                    h4 class="text-lg font-bold text-gray-900 mb-2" { "Want the full list?" }
                    p class="text-gray-600 mb-4" {
                        "Unlock " strong { (total_count) } " full records with complete addresses and details."
                    }

                    form action="/checkout" method="post" {
                        button type="submit" class="inline-flex items-center justify-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-green-600 hover:bg-green-700 shadow-md transition-colors duration-200" {
                            // Lock icon
                            svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor" {
                                path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd" {}
                            }
                            "Unlock All Data for $29.99"
                        }
                    }
                    p class="mt-3 text-xs text-gray-500" { "One-time payment. Lifetime access." }
                }
            }
        }
    }
}
