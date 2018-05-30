use std::{self, error::Error};

use ansi_term::Colour;
use log::Level;

use chrono;
use fern::Dispatch;
use log;

pub fn init() -> Result<(), Box<Error>> {
    Dispatch::new()
        .format(|out, message, record| {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string();

            let level = match record.level() {
                Level::Error => Colour::Red,
                Level::Warn => Colour::Yellow,
                Level::Info => Colour::Blue,
                Level::Debug |
                Level::Trace => Colour::White,
            };

            out.finish(format_args!(
                "{} {} {}",
                timestamp,
                level.paint(format!("{:<5}", record.level())),
                message
            ))
        })
        .level(log::LevelFilter::Off)
        .level_for("sonmhub", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
