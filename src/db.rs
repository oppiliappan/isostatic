use anyhow::Result;
use log::{debug, info};
use rusqlite::{Connection, OpenFlags, NO_PARAMS};

use std::fmt;
use std::path::Path;

pub fn init_db<P: AsRef<Path> + fmt::Display>(p: P) -> Result<Connection> {
    debug!("Looking for database at `{}`", p);
    let conn = Connection::open_with_flags(
        &p,
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS urls (
            link TEXT PRIMARY KEY,
            shortlink TEXT NOT NULL
        )",
        NO_PARAMS,
    )?;
    info!("SQLite3 database `{}` initialized", &p);
    return Ok(conn);
}
