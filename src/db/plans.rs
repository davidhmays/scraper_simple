// src/db/plans.rs
use crate::errors::ServerError;
use rusqlite::{params, Connection};

pub struct PlanInfo {
    pub code: String,
    pub name: String,
    pub download_limit: Option<i64>,
}

pub fn get_user_plan(conn: &Connection, user_id: i64) -> Result<PlanInfo, ServerError> {
    conn.query_row(
        r#"
        select
            p.code,
            p.name,
            p.download_limit
        from entitlements e
        join plans p on p.code = e.plan_code
        where e.user_id = ?
        "#,
        params![user_id],
        |row| {
            Ok(PlanInfo {
                code: row.get(0)?,
                name: row.get(1)?,
                download_limit: row.get(2)?,
            })
        },
    )
    .map_err(|e| ServerError::DbError(format!("failed to load user plan: {e}")))
}
