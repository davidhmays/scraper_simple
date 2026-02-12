pub mod admin;
pub mod campaigns;
pub mod check_email;
pub mod dashboard;
pub mod home;
pub mod login;

pub use admin::admin_page;
pub use campaigns::campaigns_page;
pub use check_email::{check_email_content, check_email_page};
pub use dashboard::{dashboard_page, DashboardVm};
pub use home::home_page;
