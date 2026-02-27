use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct List {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub source_type: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct NewList {
    pub user_id: i64,
    pub name: String,
    pub source_type: String,
}

#[derive(Debug, Clone)]
pub struct Mailing {
    pub id: i64,
    pub campaign_id: i64,
    pub list_id: i64,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub scheduled_at: Option<NaiveDateTime>,
}

#[derive(Debug)]
pub struct NewMailing {
    pub campaign_id: i64,
    pub list_id: i64,
    pub status: String,
    pub scheduled_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct RecipientInstance {
    pub id: i64,
    pub mailing_id: i64,
    pub list_row_id: i64,
    pub media_id: i64,
    pub qr_token: String,
}
