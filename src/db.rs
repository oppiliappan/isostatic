use anyhow::Result;
use rusqlite::{Connection, OpenFlags, NO_PARAMS};

use std::path::Path;

pub fn init_db<P: AsRef<Path>>(p: P) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        p,
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS urls (
            link TEXT PRIMARY KEY,
            shortlink TEXT NOT NULL
        )",
        NO_PARAMS,
    )?;
    return Ok(conn);
}
