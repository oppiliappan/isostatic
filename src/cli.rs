use anyhow::Result;
use lazy_static::lazy_static;

use std::default::Default;
use std::path::PathBuf;

pub struct Config {
    pub help: bool,
    pub port: u16,
    pub db_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            help: false,
            port: 3000,
            db_path: "./urls.db_3".into(),
        }
    }
}

lazy_static! {
    pub static ref CONFIG: Config = parse_args().unwrap_or(Default::default());
}

pub static HELP_TEXT: &'static str = "
Usage
-----

hedge [-h | --help] [--port <number>] [--database <path>]

Options
-------

    -h, --help       Prints help information
        --port       Port to start the server on (default: 3000)
        --database   Path to database (default: urls.db_3)
";

fn parse_args() -> Result<Config> {
    let mut _a = pico_args::Arguments::from_env();
    return Ok(Config {
        help: _a.contains(["-h", "--help"]),
        port: _a
            .opt_value_from_fn("--port", str::parse::<u16>)?
            .unwrap_or(7878),
        db_path: _a
            .opt_value_from_str("--database")?
            .unwrap_or(PathBuf::from("./urls.db_3")),
    });
}
