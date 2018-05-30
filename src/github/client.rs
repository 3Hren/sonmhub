use std::convert::TryFrom;

use actix_web::client::ClientRequest;
use actix_web::client::SendRequestError;
use actix_web::error::JsonPayloadError;
use actix_web::http::{Method, StatusCode};
use actix_web::{self, HttpMessage};
use futures::prelude::*;
use serde_json::{self, Value};
use url::{self, Url};

use github::{BaseUrl, CombinedStatus, UserAgent};

#[derive(Debug)]
pub enum Error {
    InvalidUrl(url::ParseError),
    Json(serde_json::Error),
    Web(actix_web::Error),
    SendRequest(SendRequestError),
    JsonPayload(JsonPayloadError),
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::InvalidUrl(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<actix_web::Error> for Error {
    fn from(err: actix_web::Error) -> Self {
        Error::Web(err)
    }
}

impl From<SendRequestError> for Error {
    fn from(err: SendRequestError) -> Self {
        Error::SendRequest(err)
    }
}

impl From<JsonPayloadError> for Error {
    fn from(err: JsonPayloadError) -> Self {
        Error::JsonPayload(err)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// GitHub base URL.
    #[serde(default)]
    pub base_url: BaseUrl,
    #[serde(default)]
    pub user_agent: UserAgent,
    pub oauth_token: String,
}

impl Config {
    #[inline]
    pub fn accept(&self) -> &str {
        "application/vnd.github.v3+json"
    }

    #[inline]
    pub fn authorization(&self) -> String {
        format!("token {}", self.oauth_token)
    }
}

trait Request {
    fn method() -> Method;
    fn path(&self) -> String;
    fn body(&self) -> Option<Result<String, Error>>;
}

pub struct MergeRequest {
    pub owner: String,
    pub repo: String,
    pub body: MergeInput,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MergeInput {
    pub base: String,
    pub head: String,
    pub commit_message: String,
}

impl Request for MergeRequest {
    fn method() -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        format!("repos/{}/{}/merges", self.owner, self.repo)
    }

    fn body(&self) -> Option<Result<String, Error>> {
        Some(serde_json::to_string(&self.body).map_err(Error::Json))
    }
}

#[derive(Clone)]
pub struct PullRequestApi {
    cfg: Config,
    owner: String,
    repo: String,
}

impl PullRequestApi {
    pub fn state(&self, reference: String) -> impl Future<Item =CombinedStatus, Error = Error> {
        PullRequestApi::execute_state(self.clone(), reference)
    }

    #[async]
    pub fn execute_state(self, reference: String) -> Result<CombinedStatus, Error> {
        let mut url = Url::parse(&self.cfg.base_url)?;
        url.path_segments_mut().unwrap().extend(&[
            "repos",
            &self.owner,
            &self.repo,
            "commits",
            &reference,
            "status",
        ]);

        let mut request = ClientRequest::get(url)
            .header("Accept", self.cfg.accept())
            .header("Authorization", self.cfg.authorization())
            .header("User-Agent", self.cfg.user_agent.clone())
            .finish()?;

        debug!("-> {} {}", request.method(), request.uri());
        let response = await!(request.send())?;
        debug!("<- {}", response.status());
        let value: Value = await!(response.json())?;
        debug!("<- {}", value);

        let state = value["state"].as_str().unwrap();

        let state = CombinedStatus::try_from(state).unwrap();

        Ok(state)
    }
}

pub struct Client {
    cfg: Config,
}

impl Client {
    pub fn new(cfg: Config) -> Self {
        Self { cfg }
    }

    pub fn pull_requests(&self, owner: &str, repo: &str) {
        unimplemented!()
    }

    pub fn pull_request(&self, owner: &str, repo: &str) -> PullRequestApi {
        PullRequestApi {
            cfg: self.cfg.clone(),
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn merge(&self, request: MergeRequest) -> impl Future<Item = Value, Error = Error> {
        Client::execute(self.cfg.clone(), request)
    }

    #[async]
    fn execute<R: Request + 'static>(cfg: Config, request: R) -> Result<Value, Error> {
        let uri = format!("{}/{}", cfg.base_url, request.path());
        let authorization = format!("token {}", cfg.oauth_token);

        let mut req = ClientRequest::build()
            .method(R::method())
            .uri(uri)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", authorization.clone())
            .header("User-Agent", cfg.user_agent.clone())
            .finish()?;

        if let Some(body) = request.body() {
            req.set_body(body?);
        }

        debug!("-> {:?}", req);
        let resp = await!(req.send())?;
        debug!("<- {:?}", resp);
        match resp.status() {
            StatusCode::CREATED => {
                let body = await!(resp.json())?;
                debug!("<- {:?}", body);
                Ok(body)
            }
            status => Ok(Value::Null),
        }
    }
}
