pub mod admin;

pub mod check_email;
pub mod dashboard;
pub mod home;
pub mod login;
pub mod preview;

pub use admin::admin_page;

// pub use check_email::{check_email_content, check_email_page};
pub use check_email::check_email_content;
pub use dashboard::dashboard_page;
pub use home::home_page;
