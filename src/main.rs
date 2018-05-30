#[macro_use]
extern crate clap;
extern crate sonmhub;

use std::error::Error;

use clap::{App, Arg};
use sonmhub::{bot, config::Config, logging};

fn main() -> Result<(), Box<Error>> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .required(true)
                .value_name("FILE")
                .help("Path to the configuration file")
                .takes_value(true),
        )
        .get_matches();

    let path = matches
        .value_of("config")
        .expect("failed to extract configuration path");

    let cfg = Config::load(path)?;

    logging::init()?;

    match bot::run(cfg) {
        0 => Ok(()),
        v => Err(format!("application exited with {} code", v).into()),
    }
}
