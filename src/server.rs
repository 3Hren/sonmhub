use std::borrow::Cow;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::str;

use actix_web::{http::StatusCode, Error, HttpMessage, HttpRequest, HttpResponse, ResponseError};
use futures::prelude::*;
use rustc_hex::{FromHex};
use serde_json::{from_slice, Value};

use secure::{self, Forbidden};

const X_HUB_SIGNATURE: &str = "X-Hub-Signature";

enum ErrorKind {
    GitHubEventHeaderRequired,
    GitHubSignatureHeaderRequired,
    InvalidSignatureHeader,
    UnsupportedHMACMethod,
}

impl Into<HttpResponse> for ErrorKind {
    fn into(self) -> HttpResponse {
        let (code, err) = match self {
            ErrorKind::GitHubEventHeaderRequired => {
                (StatusCode::NOT_FOUND, "header `X-GitHub-Event` is required")
            }
            ErrorKind::GitHubSignatureHeaderRequired => {
                (StatusCode::UNAUTHORIZED, "header `X-Hub-Signature` is required")
            }
            ErrorKind::InvalidSignatureHeader => {
                (StatusCode::UNAUTHORIZED, "invalid `X-Hub-Signature` header")
            }
            ErrorKind::UnsupportedHMACMethod => {
                (StatusCode::UNAUTHORIZED, "unsupported HMAC method")
            }
        };

        HttpResponse::build(code).json(json!({"error": err}))
    }
}

/// An error that is raised when the main application is shutting down.
#[derive(Debug)]
struct ShuttingDown;

impl Display for ShuttingDown {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.write_str("shitting down")
    }
}

impl error::Error for ShuttingDown {}

impl ResponseError for ShuttingDown {}

#[derive(Clone, Debug)]
pub enum Event {
    Log(Value),
}

#[derive(Clone, Debug)]
pub struct Server<W> {
    tx: W,
    secret: Cow<'static, [u8]>,
}

impl<W> Server<W>
where
    W: Sink<SinkItem = Event> + Clone + 'static,
{
    pub fn new(tx: W) -> Self {
        Self {
            tx,
            secret: b"<secret>"[..].into(),
        }
    }

    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), Forbidden> {
        secure::verify(data, &self.secret, signature)
    }

    pub fn index(request: HttpRequest<Server<W>>) -> impl Future<Item = HttpResponse, Error = Error> {
        request.state().clone().execute(request)
    }

    #[async]
    fn execute(self, request: HttpRequest<Server<W>>) -> Result<HttpResponse, Error> {
        match request.headers().get("X-GitHub-Event").map(|v| v.as_bytes()) {
            Some(b"ping") => {
                return Ok(HttpResponse::Ok().finish())
            }
            Some(event) => {
                event.to_owned()
            }
            None => {
                return Ok(ErrorKind::GitHubEventHeaderRequired.into())
            }
        };

        let signature = match request.headers().get(X_HUB_SIGNATURE).map(|v| v.as_bytes()).filter(|v| v.len() == 45).map(|v| v.split_at(5)) {
            Some((b"sha1=", signature)) => {
                match str::from_utf8(signature).ok().and_then(|v| v.from_hex().ok()) {
                    Some(signature) => {
                        signature.to_owned()
                    }
                    None => {
                        return Ok(ErrorKind::InvalidSignatureHeader.into())
                    }
                }
            },
            Some(..) => {
                return Ok(ErrorKind::UnsupportedHMACMethod.into())
            }
            None => {
                return Ok(ErrorKind::GitHubSignatureHeaderRequired.into())
            }
        };

        let body = await!(request.body())?;

        if let Err(err) = self.verify(&body, &signature) {
            return Ok(err.into())
        }

        let tx = self.tx.clone();

        let body: Value = from_slice(&body)?;
        await!(tx.send(Event::Log(body))).or(Err(ShuttingDown))?;

        Ok(HttpResponse::Ok().finish())
    }
}
