use crate::domain::campaign::{Campaign, Media};
use crate::templates::desktop_layout;
use maud::{html, Markup};

pub fn campaigns_index_page(campaigns: &[Campaign]) -> Markup {
    desktop_layout(
        "Campaigns",
        true,
        html! {
            div class="mb-6 flex justify-between items-center" {
                div {
                    h1 class="text-3xl font-bold text-gray-800" { "Campaigns" }
                    p class="text-gray-500 mt-1" { "Manage your strategic marketing initiatives." }
                }
                a href="/campaigns/new" class="px-4 py-2 bg-indigo-600 text-white font-medium rounded-md hover:bg-indigo-700 shadow-sm transition-colors" {
                    "New Campaign"
                }
            }

            div class="bg-white shadow overflow-hidden sm:rounded-md border border-gray-200" {
                ul class="divide-y divide-gray-200" {
                    @if campaigns.is_empty() {
                        li class="px-6 py-12 text-center" {
                            svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" {}
                            }
                            h3 class="mt-2 text-sm font-medium text-gray-900" { "No campaigns" }
                            p class="mt-1 text-sm text-gray-500" { "Get started by creating a new campaign." }
                            div class="mt-6" {
                                a href="/campaigns/new" class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" {
                                    "Create Campaign"
                                }
                            }
                        }
                    } @else {
                        @for campaign in campaigns {
                            li {
                                a href=(format!("/campaigns/{}", campaign.id)) class="block hover:bg-gray-50 transition-colors duration-150" {
                                    div class="px-4 py-4 sm:px-6" {
                                        div class="flex items-center justify-between" {
                                            p class="text-sm font-medium text-indigo-600 truncate" { (campaign.name) }
                                            div class="ml-2 flex-shrink-0 flex" {
                                                p class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800" {
                                                    (campaign.status)
                                                }
                                            }
                                        }
                                        div class="mt-2 sm:flex sm:justify-between" {
                                            div class="sm:flex" {
                                                p class="flex items-center text-sm text-gray-500" {
                                                    "Created " (campaign.created_at.format("%Y-%m-%d"))
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

pub fn new_campaign_page() -> Markup {
    desktop_layout(
        "New Campaign",
        true,
        html! {
            div class="max-w-2xl mx-auto mt-8" {
                div class="md:flex md:items-center md:justify-between mb-6" {
                    div class="flex-1 min-w-0" {
                        h2 class="text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate" { "Create New Campaign" }
                    }
                }

                form action="/campaigns" method="post" class="space-y-8 divide-y divide-gray-200 bg-white p-8 rounded-lg shadow border border-gray-200" {
                    div class="space-y-6 sm:space-y-5" {
                        div {
                            label for="name" class="block text-sm font-medium text-gray-700" { "Campaign Name" }
                            div class="mt-1" {
                                input type="text" name="name" id="name" required class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md p-2 border" placeholder="e.g. Summer 2024 Promo";
                            }
                            p class="mt-2 text-sm text-gray-500" { "This is internal-facing. Use something descriptive." }
                        }
                    }

                    div class="pt-5" {
                        div class="flex justify-end" {
                            a href="/campaigns" class="bg-white py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 mr-3" { "Cancel" }
                            button type="submit" class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" { "Create" }
                        }
                    }
                }
            }
        },
    )
}

pub fn campaign_details_page(campaign: &Campaign, media: &[Media]) -> Markup {
    desktop_layout(
        &format!("Campaign: {}", campaign.name),
        true,
        html! {
            div class="mb-6 flex justify-between items-center" {
                div {
                    h1 class="text-3xl font-bold text-gray-800" { (campaign.name) }
                    p class="text-gray-500 mt-1" { "Status: " (campaign.status) }
                }
                a href=(format!("/campaigns/{}/media/new", campaign.id)) class="px-4 py-2 bg-indigo-600 text-white font-medium rounded-md hover:bg-indigo-700 shadow-sm transition-colors" {
                    "Add Media"
                }
            }

            div class="bg-white shadow overflow-hidden sm:rounded-md border border-gray-200 mb-8" {
                div class="px-4 py-5 sm:px-6" {
                    h3 class="text-lg leading-6 font-medium text-gray-900" { "Media Assets" }
                    p class="mt-1 max-w-2xl text-sm text-gray-500" { "Creative variants associated with this campaign." }
                }
                ul class="divide-y divide-gray-200" {
                    @if media.is_empty() {
                        li class="px-6 py-12 text-center" {
                            p class="text-sm text-gray-500" { "No media assets yet." }
                        }
                    } @else {
                        @for m in media {
                            li class="px-4 py-4 sm:px-6" {
                                div class="flex items-center justify-between" {
                                    div {
                                        p class="text-sm font-medium text-indigo-600 truncate" { (m.name) }
                                        p class="text-sm text-gray-500" { (m.media_type) }
                                    }
                                    @if let Some(desc) = &m.description {
                                        p class="text-sm text-gray-500" { (desc) }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div {
                a href="/campaigns" class="text-indigo-600 hover:text-indigo-900" { "← Back to Campaigns" }
            }
        },
    )
}

pub fn new_media_page(campaign_id: i64) -> Markup {
    desktop_layout(
        "Add Media",
        true,
        html! {
            div class="max-w-2xl mx-auto mt-8" {
                h2 class="text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate mb-6" { "Add Media Asset" }

                form action=(format!("/campaigns/{}/media", campaign_id)) method="post" class="space-y-8 divide-y divide-gray-200 bg-white p-8 rounded-lg shadow border border-gray-200" {
                    div class="space-y-6 sm:space-y-5" {
                        div {
                            label for="name" class="block text-sm font-medium text-gray-700" { "Media Name" }
                            div class="mt-1" {
                                input type="text" name="name" id="name" required class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md p-2 border" placeholder="e.g. Postcard V1";
                            }
                        }
                        div {
                            label for="media_type" class="block text-sm font-medium text-gray-700" { "Type" }
                            div class="mt-1" {
                                select name="media_type" id="media_type" class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md p-2 border" {
                                    option value="postcard_4x6" { "Postcard (4x6)" }
                                    option value="postcard_6x9" { "Postcard (6x9)" }
                                    option value="letter_8.5x11" { "Letter (8.5x11)" }
                                }
                            }
                        }
                        div {
                            label for="description" class="block text-sm font-medium text-gray-700" { "Description" }
                            div class="mt-1" {
                                textarea name="description" id="description" rows="3" class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md p-2 border" {}
                            }
                        }
                    }

                    div class="pt-5" {
                        div class="flex justify-end" {
                            a href=(format!("/campaigns/{}", campaign_id)) class="bg-white py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 mr-3" { "Cancel" }
                            button type="submit" class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" { "Create" }
                        }
                    }
                }
            }
        },
    )
}
