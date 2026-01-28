mod mailing;
mod campaign;

pub use mailing::{create_mailing, MediaType, NewMailing};
pub use campaign::{generate_mailings_for_campaign, NewCampaign, ListingFlag, PropertyType};
 //TODO Move property type and listing flag to correct places.
