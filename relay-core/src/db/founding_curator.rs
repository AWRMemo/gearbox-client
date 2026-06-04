use crate::db::open_db;

pub fn ensure_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS founding_curator_counter (
            key TEXT PRIMARY KEY,
            value INTEGER NOT NULL DEFAULT 0
        );"
    )
    .map_err(|e| format!("founding curator table: {e}"))
}

pub fn total_signups() -> Result<i64, String> {
    let conn = open_db()?;
    ensure_table(&conn)?;
    let count: i64 = conn
        .query_row(
            "SELECT value FROM founding_curator_counter WHERE key = 'total_signups'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(count)
}

pub fn remaining_spots() -> i64 {
    let cap: i64 = 5000;
    let taken = total_signups().unwrap_or(0);
    (cap - taken).max(0)
}

pub fn increment_signups() -> Result<(), String> {
    let conn = open_db()?;
    ensure_table(&conn)?;
    conn.execute(
        "INSERT INTO founding_curator_counter (key, value) VALUES ('total_signups', 1)
         ON CONFLICT(key) DO UPDATE SET value = value + 1",
        [],
    )
    .map(|_| ())
    .map_err(|e| format!("increment signups: {e}"))
}
