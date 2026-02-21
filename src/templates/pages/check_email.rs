use crate::templates::desktop_layout;
use maud::{html, Markup};

/// Returns the partial HTML content for the success message.
/// Used for HTMX swaps to replace the login form.
pub fn check_email_content(email: &str) -> Markup {
    html! {
        div class="text-center py-8 px-4 fade-in" {
            div class="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-green-100 mb-4" {
                svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-green-600" {
                    polyline points="20 6 9 17 4 12" {}
                }
            }

            h3 class="text-lg leading-6 font-medium text-gray-900" { "Check your email" }

            div class="mt-2" {
                p class="text-sm text-gray-500" {
                    "We sent a sign-in link to "
                    strong class="text-gray-900" { (email) }
                    "."
                }
                p class="text-sm text-gray-500 mt-2" {
                    "Click the link in the email to sign in."
                }
            }

            div class="mt-6" {
                a href="/login" class="text-sm font-medium text-blue-600 hover:text-blue-500" {
                    "Try with a different email"
                }
            }
        }
    }
}

// Returns the full page layout with the check email message.
// Used for direct navigation or redirects.
// pub fn check_email_page(email: &str, is_admin: bool) -> Markup {
//     desktop_layout(
//         "Check your email",
//         is_admin,
//         html! {
//             main class="container mx-auto mt-12 p-4 max-w-lg" {
//                 div class="bg-white p-8 rounded-lg shadow-sm border border-gray-200" {
//                     (check_email_content(email))
//                 }
//             }
//         },
//     )
// }
