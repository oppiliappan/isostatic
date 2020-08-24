// extern
use anyhow::Result;
use log::{debug, info};
use rusqlite::{Connection, OpenFlags, NO_PARAMS};

// std
use std::fmt;
use std::path::Path;

pub fn open_connection<P>(p: P) -> Result<Connection>
where
    P: AsRef<Path> + fmt::Display,
{
    info!("Opened connection to database");
    Ok(Connection::open_with_flags(
        &p,
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?)
}

pub fn init_db<P: AsRef<Path> + fmt::Display>(p: P) -> Result<()> {
    debug!("Looking for database at `{}`", p);
    let conn = open_connection(&p)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS urls (
            link TEXT PRIMARY KEY,
            shortlink TEXT NOT NULL
        )",
        NO_PARAMS,
    )?;
    info!("SQLite3 database `{}` initialized", &p);
    Ok(())
}
