use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

use actix_web::http::header::{HeaderValue, IntoHeaderValue, InvalidHeaderValue};

pub use self::client::*;
pub use self::status::*;

mod client;
mod status;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BaseUrl(Cow<'static, str>);

impl Default for BaseUrl {
    fn default() -> Self {
        BaseUrl(Cow::from("https://api.github.com"))
    }
}

impl Display for BaseUrl {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            BaseUrl(v) => fmt.write_str(&v),
        }
    }
}

impl Deref for BaseUrl {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoHeaderValue for BaseUrl {
    type Error = InvalidHeaderValue;

    fn try_into(self) -> Result<HeaderValue, Self::Error> {
        match self {
            BaseUrl(v) => v.try_into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserAgent(Cow<'static, str>);

impl Default for UserAgent {
    fn default() -> Self {
        UserAgent(Cow::from("sonmhub"))
    }
}

impl IntoHeaderValue for UserAgent {
    type Error = InvalidHeaderValue;

    fn try_into(self) -> Result<HeaderValue, Self::Error> {
        match self {
            UserAgent(v) => v.try_into(),
        }
    }
}
