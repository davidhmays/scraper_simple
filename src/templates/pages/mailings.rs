use crate::domain::campaign::Campaign;
use crate::domain::mailing::{List, Mailing};
use crate::templates::desktop_layout;
use maud::{html, Markup};

pub fn mailings_index_page(mailings: &[Mailing]) -> Markup {
    desktop_layout(
        "Mailings",
        true,
        html! {
            div class="mb-6 flex justify-between items-center" {
                div {
                    h1 class="text-3xl font-bold text-gray-800" { "Mailings" }
                    p class="text-gray-500 mt-1" { "Manage your operational mail batches." }
                }
                a href="/mailings/new" class="px-4 py-2 bg-indigo-600 text-white font-medium rounded-md hover:bg-indigo-700 shadow-sm transition-colors" {
                    "New Mailing"
                }
            }

            div class="bg-white shadow overflow-hidden sm:rounded-md border border-gray-200" {
                ul class="divide-y divide-gray-200" {
                    @if mailings.is_empty() {
                        li class="px-6 py-12 text-center" {
                            svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" {}
                            }
                            h3 class="mt-2 text-sm font-medium text-gray-900" { "No mailings" }
                            p class="mt-1 text-sm text-gray-500" { "Create a mailing to tie a campaign to a list." }
                            div class="mt-6" {
                                a href="/mailings/new" class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" {
                                    "Create Mailing"
                                }
                            }
                        }
                    } @else {
                        @for mailing in mailings {
                            li {
                                div class="px-4 py-4 sm:px-6 hover:bg-gray-50 transition-colors duration-150" {
                                    div class="flex items-center justify-between" {
                                        p class="text-sm font-medium text-indigo-600 truncate" { "Mailing #" (mailing.id) }
                                        div class="ml-2 flex-shrink-0 flex" {
                                            p class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800" {
                                                (mailing.status)
                                            }
                                        }
                                    }
                                    div class="mt-2 sm:flex sm:justify-between" {
                                        div class="sm:flex" {
                                            p class="flex items-center text-sm text-gray-500" {
                                                "Created " (mailing.created_at.format("%Y-%m-%d"))
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

pub fn new_mailing_page(campaigns: &[Campaign], lists: &[List]) -> Markup {
    desktop_layout(
        "New Mailing",
        true,
        html! {
            div class="max-w-2xl mx-auto mt-8" {
                div class="md:flex md:items-center md:justify-between mb-6" {
                    div class="flex-1 min-w-0" {
                        h2 class="text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate" { "Create New Mailing" }
                    }
                }

                form action="/mailings" method="post" class="space-y-8 divide-y divide-gray-200 bg-white p-8 rounded-lg shadow border border-gray-200" {
                    div class="space-y-6 sm:space-y-5" {
                        div {
                            label for="campaign_id" class="block text-sm font-medium text-gray-700" { "Select Campaign" }
                            div class="mt-1" {
                                select name="campaign_id" id="campaign_id" required class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md p-2 border" {
                                    option value="" disabled selected { "Select a Campaign..." }
                                    @for campaign in campaigns {
                                        option value=(campaign.id) { (campaign.name) }
                                    }
                                }
                            }
                        }

                        div {
                            label for="list_id" class="block text-sm font-medium text-gray-700" { "Select List" }
                            div class="mt-1" {
                                select name="list_id" id="list_id" required class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md p-2 border" {
                                    option value="" disabled selected { "Select a List..." }
                                    @for list in lists {
                                        option value=(list.id) { (list.name) }
                                    }
                                }
                            }
                        }

                        div {
                            label for="scheduled_at" class="block text-sm font-medium text-gray-700" { "Scheduled Date (Optional)" }
                            div class="mt-1" {
                                input type="datetime-local" name="scheduled_at" id="scheduled_at" class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md p-2 border";
                            }
                        }
                    }

                    div class="pt-5" {
                        div class="flex justify-end" {
                            a href="/mailings" class="bg-white py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 mr-3" { "Cancel" }
                            button type="submit" class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" { "Create" }
                        }
                    }
                }
            }
        },
    )
}
