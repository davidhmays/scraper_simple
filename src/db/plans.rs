// src/db/plans.rs
use crate::errors::ServerError;
use rusqlite::{params, Connection};

#[derive(Debug)]
pub struct PlanInfo {
    pub code: String,
    pub name: String,
    pub download_limit: Option<i64>,
}

pub fn get_all_plans(conn: &Connection) -> Result<Vec<PlanInfo>, ServerError> {
    let mut stmt = conn
        .prepare("SELECT code, name, download_limit FROM plans ORDER BY id")
        .map_err(|e| ServerError::DbError(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(PlanInfo {
                code: row.get(0)?,
                name: row.get(1)?,
                download_limit: row.get(2)?,
            })
        })
        .map_err(|e| ServerError::DbError(e.to_string()))?;

    let mut plans = Vec::new();
    for p in rows {
        plans.push(p.map_err(|e| ServerError::DbError(e.to_string()))?);
    }
    Ok(plans)
}

pub fn update_plan_limit(
    conn: &Connection,
    code: &str,
    limit: Option<i64>,
) -> Result<(), ServerError> {
    conn.execute(
        "UPDATE plans SET download_limit = ? WHERE code = ?",
        params![limit, code],
    )
    .map_err(|e| ServerError::DbError(e.to_string()))?;
    Ok(())
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

pub fn upgrade_user_plan(
    conn: &Connection,
    user_id: i64,
    plan_code: &str,
    now: i64,
) -> Result<(), ServerError> {
    conn.execute(
        "insert or replace into entitlements (user_id, plan_code, granted_at) values (?, ?, ?)",
        params![user_id, plan_code, now],
    )
    .map_err(|e| ServerError::DbError(format!("upgrade plan failed: {e}")))?;
    Ok(())
}
