use std::time::Instant;

use actix;
use actix_web::{
    client::{ClientRequest, SendRequestError}, error::JsonPayloadError, http, http::StatusCode,
    server, App, Error, HttpMessage, HttpRequest, HttpResponse,
};
use futures::prelude::*;
use serde::Deserialize;
use serde_json::{self, Value};
use tokio::timer::{self, Interval};

use github::{self, Client as Github, MergeInput, MergeRequest};
use config::{Config, MergeBotConfig};

struct AppState {}

// curl -L api.github.com/repos/sonm-io/core/pulls/806/files
#[async]
fn index(req: HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    match req.headers().get("X-GitHub-Event").map(|v| v.as_bytes()) {
        Some(b"ping") => {
            return Ok(HttpResponse::Ok().finish());
        }
        Some(..) => {}
        None => {}
    }

    println!("{:?}", req);
    let body: Value = await!(req.json())?;
    println!("{:?}", body);

    Ok(HttpResponse::new(StatusCode::OK))
}

#[derive(Debug)]
enum BotError {
    Timer(timer::Error),
    SendRequest(SendRequestError),
    Web(Error),
    Json(JsonPayloadError),
    Deserialize(serde_json::Error),
    Github(github::Error),
}

impl From<timer::Error> for BotError {
    fn from(err: timer::Error) -> Self {
        BotError::Timer(err)
    }
}

impl From<Error> for BotError {
    fn from(err: Error) -> Self {
        BotError::Web(err)
    }
}

impl From<SendRequestError> for BotError {
    fn from(err: SendRequestError) -> Self {
        BotError::SendRequest(err)
    }
}

impl From<JsonPayloadError> for BotError {
    fn from(err: JsonPayloadError) -> Self {
        BotError::Json(err)
    }
}

impl From<serde_json::Error> for BotError {
    fn from(err: serde_json::Error) -> Self {
        BotError::Deserialize(err)
    }
}

impl From<github::Error> for BotError {
    fn from(err: github::Error) -> Self {
        BotError::Github(err)
    }
}

#[derive(Debug, Deserialize)]
struct PullRequest {
    title: String,
    number: u32,
    mergeable: String,
    #[serde(rename = "baseRefName")]
    base_ref_name: String,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
}

struct AutoMergeBot {
    cfg: MergeBotConfig,
}

impl AutoMergeBot {
    fn new(cfg: MergeBotConfig) -> Self {
        Self { cfg }
    }

    #[async]
    fn run(self) -> Result<(), BotError> {
        let owner = "sonm-io";
        let repo = "core";
        let uri = format!(
            "{}/repos/{}/{}/pulls",
            "https://api.github.com", owner, repo
        );
        let uri_graphql = "https://api.github.com/graphql";
        let body = include_str!("../graphql/pull_requests.graphql");
        let body = json!({
            "query": body,
        });

        let github = Github::new(self.cfg.github().clone());

        let authorization = format!("token {}", self.cfg.github().oauth_token);

        #[async]
        for timestamp in Interval::new(Instant::now(), self.cfg.interval()) {
            let request = ClientRequest::post(&uri_graphql)
                .header("Accept", "application/vnd.github.jean-grey-preview+json")
                .header("Authorization", authorization.clone())
                .header("User-Agent", self.cfg.github().user_agent.clone())
                .json(body.clone())?;
            debug!("-> {} {}", request.method(), request.uri());
            let resp = await!(request.send())?;

            debug!("<- {}", resp.status());
            let body: Value = await!(resp.json())?;
            debug!("<- {}", body);
            let pull_requests: Vec<PullRequest> =
                Deserialize::deserialize(&body["data"]["repository"]["pullRequests"]["nodes"])?;
            debug!("<- {:?}", body);

            // Update outdated PRs.
            for pull_request in pull_requests {
                let status = await!(github.pull_request(owner, repo).state(pull_request.head_ref_name.clone()))?;

                debug!("<- {:?}", status);

                if pull_request.mergeable == "!!MERGEABLE" && pull_request.base_ref_name == "master" {
                    let request = MergeRequest {
                        owner: owner.to_string(),
                        repo: repo.to_string(),
                        body: MergeInput {
                            base: pull_request.head_ref_name.clone(),
                            head: "master".to_string(),
                            commit_message: format!(
                                "Merge branch 'master' into {}",
                                pull_request.head_ref_name
                            ),
                        },
                    };

                    await!(github.merge(request))?;
                }
            }
        }

        // TODO: On PR created - select 3 random person and assign as a reviewers.
        //      Chose depending on files they own. If less people - take random.
        // TODO:    If author == owner of ALL file changed -> decrease by 1.
        // TODO: Auto-merge if: 1) all checks passed; 2) at least N required MEMBER
        // reviewers approved with write access.

        // TODO: Update ONLY the earliest APPROVED PR to decrease load on travis.

        Ok(())
    }
}

pub fn run(cfg: Config) -> i32 {
    let sys = actix::System::new("sonm-github");

    let server = server::new(|| {
        let state = AppState {};

        App::with_state(state).resource("/hook", |r| r.method(http::Method::POST).a(index))
    });

    server
        .bind(cfg.network().addr().to_string())
        .unwrap()
        .shutdown_timeout(1)
        .start();

    let bot = AutoMergeBot::new(cfg.merge().clone());
    sys.handle().spawn(bot.run().then(|result| {
        match result {
            Ok(()) => info!("bot finished"),
            Err(err) => error!("finished bot with {:?}", err),
        }
        Ok(())
    }));

    info!("Started http server: {}", cfg.network().addr().to_string());
    sys.run()
}
