use crate::domain::mailing::List;
use crate::templates::desktop_layout;
use maud::{html, Markup};

pub fn lists_index_page(lists: &[List]) -> Markup {
    desktop_layout(
        "Lists",
        true,
        html! {
            div class="mb-6 flex justify-between items-center" {
                div {
                    h1 class="text-3xl font-bold text-gray-800" { "Lists" }
                    p class="text-gray-500 mt-1" { "Manage your recipient lists for mailings." }
                }
                a href="/lists/new" class="px-4 py-2 bg-indigo-600 text-white font-medium rounded-md hover:bg-indigo-700 shadow-sm transition-colors" {
                    "New List"
                }
            }

            div class="bg-white shadow overflow-hidden sm:rounded-md border border-gray-200" {
                ul class="divide-y divide-gray-200" {
                    @if lists.is_empty() {
                        li class="px-6 py-12 text-center" {
                            svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01" {}
                            }
                            h3 class="mt-2 text-sm font-medium text-gray-900" { "No lists" }
                            p class="mt-1 text-sm text-gray-500" { "Create a list to start sending mailings." }
                            div class="mt-6" {
                                a href="/lists/new" class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" {
                                    "Create List"
                                }
                            }
                        }
                    } @else {
                        @for list in lists {
                            li {
                                div class="px-4 py-4 sm:px-6 hover:bg-gray-50 transition-colors duration-150" {
                                    div class="flex items-center justify-between" {
                                        p class="text-sm font-medium text-indigo-600 truncate" { (list.name) }
                                        div class="ml-2 flex-shrink-0 flex" {
                                            p class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-blue-100 text-blue-800" {
                                                (list.source_type)
                                            }
                                        }
                                    }
                                    div class="mt-2 sm:flex sm:justify-between" {
                                        div class="sm:flex" {
                                            p class="flex items-center text-sm text-gray-500" {
                                                "Created " (list.created_at.format("%Y-%m-%d"))
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

pub fn new_list_page() -> Markup {
    desktop_layout(
        "New List",
        true,
        html! {
            div class="max-w-2xl mx-auto mt-8" {
                div class="md:flex md:items-center md:justify-between mb-6" {
                    div class="flex-1 min-w-0" {
                        h2 class="text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate" { "Create New List" }
                    }
                }

                form action="/lists" method="post" class="space-y-8 divide-y divide-gray-200 bg-white p-8 rounded-lg shadow border border-gray-200" {
                    div class="space-y-6 sm:space-y-5" {
                        div {
                            label for="name" class="block text-sm font-medium text-gray-700" { "List Name" }
                            div class="mt-1" {
                                input type="text" name="name" id="name" required class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md p-2 border" placeholder="e.g. Utah Contingent Oct 2024";
                            }
                        }

                        div {
                            label for="source_type" class="block text-sm font-medium text-gray-700" { "List Source" }
                            div class="mt-1" {
                                select name="source_type" id="source_type" class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md p-2 border" {
                                    option value="manual_upload" { "Manual Upload (CSV)" }
                                    option value="system_snapshot" { "System Snapshot (Property Data)" }
                                }
                            }
                            p class="mt-2 text-sm text-gray-500" { "For now, this just creates the list container." }
                        }
                    }

                    div class="pt-5" {
                        div class="flex justify-end" {
                            a href="/lists" class="bg-white py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 mr-3" { "Cancel" }
                            button type="submit" class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" { "Create" }
                        }
                    }
                }
            }
        },
    )
}
