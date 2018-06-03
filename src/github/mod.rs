use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

use actix_web::http::header::{HeaderValue, IntoHeaderValue, InvalidHeaderValue};
use serde::{de, Deserialize, Deserializer};
use url::Url;

pub use self::client::*;
pub use self::status::*;

mod client;
mod status;

/// Represents an entry point to the github API.
///
/// This must only specify URL without path/query, i.e. be the first part of the URL.
#[derive(Clone, Debug, PartialEq)]
pub struct BaseUrl(Url);

impl BaseUrl {
    pub fn with_path(self, path: &str) -> Url {
        match self {
            BaseUrl(mut v) => {
                v.set_path(path);
                v
            }
        }
    }
}

impl Default for BaseUrl {
    fn default() -> Self {
        BaseUrl(Url::parse("https://api.github.com").unwrap())
    }
}

impl Display for BaseUrl {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            BaseUrl(v) => v.fmt(fmt),
        }
    }
}

impl<'de> Deserialize<'de> for BaseUrl {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let v = Deserialize::deserialize(de)?;
        let url = Url::parse(v).map_err(|err| {
            de::Error::custom(format!("{}", err))
        })?;

        Ok(BaseUrl(url))
    }
}

impl IntoHeaderValue for BaseUrl {
    type Error = InvalidHeaderValue;

    fn try_into(self) -> Result<HeaderValue, Self::Error> {
        match self {
            BaseUrl(v) => v.as_str().try_into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserAgent(Cow<'static, str>);

impl Default for UserAgent {
    fn default() -> Self {
        UserAgent(Cow::from("sonmbot"))
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
