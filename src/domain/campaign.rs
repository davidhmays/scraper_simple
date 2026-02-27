use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct Campaign {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub status: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct NewCampaign {
    pub user_id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Media {
    pub id: i64,
    pub campaign_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub media_type: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct NewMedia {
    pub campaign_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub media_type: String,
}
