mod brevo;
mod campaign;
mod mailing;

pub use brevo::BrevoMailer;
pub use campaign::{generate_mailings_for_campaign, ListingFlag, NewCampaign, PropertyType};
pub use mailing::{create_mailing, MediaType, NewMailing};
//TODO Move property type and listing flag to correct places.
