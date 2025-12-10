use crate::db::Database;
use crate::errors::ServerError;
use crate::responses::html_response;
use crate::responses::ResultResp;
use crate::templates;
use astra::Request;

pub fn handle(req: Request, db: &Database) -> ResultResp {
    let method = req.method().as_str();
    let path = req.uri().path();

    match (method, path) {
        ("GET", "/") => html_response(templates::pages::home_page()),
        // ("GET", "/about") => templates::html("<h1>About</h1>"),
        // ("GET", "/hello") => templates::html("<h1>Hello!</h1>"),

        // SQLite test route
        // ("GET", "/count") => {
        //     let count = db.with_conn(|conn| {
        //         // 1. Prepare the SQL
        //         let mut stmt = conn
        //             .prepare("SELECT COUNT(*) FROM items")
        //             .map_err(|e| ServerError::DbError(format!("Prepare failed: {e}")))?;

        //         // 2. Run the query (empty params)
        //         let mut rows = stmt
        //             .query([])
        //             .map_err(|e| ServerError::DbError(format!("Query failed: {e}")))?;

        //         // 3. Fetch first row
        //         let row = rows
        //             .next()
        //             .map_err(|e| ServerError::DbError(format!("Rows.next failed: {e}")))?
        //             .ok_or_else(|| ServerError::DbError("No rows".into()))?;

        //         // 4. Extract COUNT(*) as i64
        //         let val: i64 = row
        //             .get(0)
        //             .map_err(|e| ServerError::DbError(format!("Column read failed: {e}")))?;

        //         Ok(val)
        //     })?;

        //     templates::html(&format!("<h1>DB says: {count}</h1>"))
        // }
        // ("GET", "/add") => {
        //     let params = parse_query(&req);
        //     let name = params.get("name").map(String::as_str).unwrap_or("unnamed");

        //     db.with_conn(|conn| {
        //         conn.execute("INSERT INTO items (name) VALUES (?)", [name])
        //             .map_err(|e| ServerError::DbError(format!("Insert failed: {e}")))?;

        //         Ok(())
        //     })?;

        //     templates::html(&format!("<h1>Added {name}</h1>"))
        // }
        _ => Err(ServerError::NotFound),
    }
}

fn parse_query(req: &astra::Request) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();

    if let Some(q) = req.uri().query() {
        for pair in q.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                map.insert(k.to_string(), v.to_string());
            }
        }
    }

    map
}
