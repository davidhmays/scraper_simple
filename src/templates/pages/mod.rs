pub mod admin;
pub mod campaigns;
pub mod lists;
pub mod mailings;

pub mod check_email;
pub mod dashboard;
pub mod home;
pub mod login;
pub mod preview;

pub use admin::admin_page;
pub use campaigns::{
    campaign_details_page, campaigns_index_page, new_campaign_page, new_media_page,
};
pub use lists::{lists_index_page, new_list_page};
pub use mailings::{mailings_index_page, new_mailing_page};

// pub use check_email::{check_email_content, check_email_page};
pub use check_email::check_email_content;
pub use dashboard::dashboard_page;
pub use home::home_page;
