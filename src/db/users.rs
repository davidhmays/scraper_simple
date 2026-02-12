use crate::errors::ServerError;
use rusqlite::{params, Connection};
use time::OffsetDateTime;

#[derive(Debug)]
pub struct UserWithStats {
    pub id: i64,
    pub email: String,
    pub last_login_at: Option<i64>,
    pub plan_name: Option<String>,
    pub usage_this_month: i64,
    pub is_admin: bool,
}

pub fn is_user_admin(conn: &Connection, user_id: i64) -> Result<bool, ServerError> {
    let count: i64 = conn
        .query_row(
            "select count(*) from users where id = ? and is_admin = 1",
            params![user_id],
            |r| r.get(0),
        )
        .map_err(|e| ServerError::DbError(format!("check admin failed: {e}")))?;

    Ok(count > 0)
}

pub fn get_all_users_with_stats(
    conn: &Connection,
    now: i64,
) -> Result<Vec<UserWithStats>, ServerError> {
    // Determine start of current month (UTC) for usage calculation
    let dt = OffsetDateTime::from_unix_timestamp(now).unwrap_or_else(|_| OffsetDateTime::now_utc());

    let start_of_month = dt
        .replace_day(1)
        .unwrap_or(dt)
        .replace_time(time::Time::MIDNIGHT)
        .unix_timestamp();

    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                u.id,
                u.email,
                u.last_login_at,
                p.name,
                COUNT(d.id),
                u.is_admin
            FROM users u
            LEFT JOIN entitlements e ON e.user_id = u.id
            LEFT JOIN plans p ON p.code = e.plan_code
            LEFT JOIN download_events d ON d.user_id = u.id AND d.created_at >= ?
            GROUP BY u.id
            ORDER BY u.id DESC
            "#,
        )
        .map_err(|e| ServerError::DbError(format!("prepare user stats query failed: {e}")))?;

    let rows = stmt
        .query_map(params![start_of_month], |row| {
            Ok(UserWithStats {
                id: row.get(0)?,
                email: row.get(1)?,
                last_login_at: row.get(2)?,
                plan_name: row.get(3)?,
                usage_this_month: row.get(4)?,
                is_admin: row.get::<_, i64>(5)? != 0,
            })
        })
        .map_err(|e| ServerError::DbError(format!("query user stats failed: {e}")))?;

    let mut users = Vec::new();
    for user in rows {
        users.push(user.map_err(|e| ServerError::DbError(format!("read user row failed: {e}")))?);
    }

    Ok(users)
}
