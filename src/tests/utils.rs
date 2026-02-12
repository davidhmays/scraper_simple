use crate::db::connection::{init_db, Database};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn init_test_db() -> Database {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("test_db_{}.sqlite", nanos));

    let db = Database::new(path.to_string_lossy().to_string());

    init_db(&db, "sql/schema.sql").expect("Failed to initialize test DB");

    db
}
