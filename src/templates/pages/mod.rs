pub mod admin;
pub mod campaigns;
pub mod dashboard;
pub mod home;
pub mod login;

pub use admin::admin_page;
pub use campaigns::campaigns_page;
pub use dashboard::{dashboard_page, DashboardVm};
pub use home::home_page;
