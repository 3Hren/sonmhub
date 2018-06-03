//! How does it work?
//!
//! A PR can be merged if all of the following conditions are met:
//! - It has neither `wip` label nor `wip` word in the title.
//! - All status checks have passed.
//! - There is no non `APPROVED` reviews from `MEMBER` or `OWNER` users with
//! push access to this   repository. At least N approves is required.
//!   https://api.github.com/repos/<owner>/<repo>/pulls/<num>/reviews
//!   https://api.github.com/repos/<owner>/<repo>/collaborators
//!
//! All these checks are being executed every 60 sec.

#![feature(proc_macro, proc_macro_non_items, generators, try_from)]

extern crate actix;
extern crate actix_web;
extern crate ansi_term;
extern crate chrono;
extern crate crypto;
extern crate fern;
extern crate futures_await as futures;
#[macro_use]
extern crate log;
extern crate rustc_hex;
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
pub mod secure;
pub mod server;
