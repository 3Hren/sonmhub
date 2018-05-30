#![feature(proc_macro, proc_macro_non_items, generators, try_from)]

extern crate actix;
extern crate actix_web;
extern crate ansi_term;
extern crate chrono;
extern crate fern;
extern crate futures_await as futures;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde_yaml;
extern crate tokio;
extern crate url;

pub mod bot;
pub mod config;
pub mod github;
pub mod logging;
