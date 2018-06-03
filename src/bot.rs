use std::fs::{File, OpenOptions};
use std::io::Write;

use actix;
use actix_web::{http, server, App};

use config::Config;
use futures::prelude::*;
use futures::sync::mpsc;
use server::Event;
use server::Server;

//#[derive(Debug)]
//enum BotError {
//    Timer(timer::Error),
//    SendRequest(SendRequestError),
//    Web(Error),
//    Json(JsonPayloadError),
//    Deserialize(serde_json::Error),
//    Github(github::Error),
//}
//
//impl From<timer::Error> for BotError {
//    fn from(err: timer::Error) -> Self {
//        BotError::Timer(err)
//    }
//}
//
//impl From<Error> for BotError {
//    fn from(err: Error) -> Self {
//        BotError::Web(err)
//    }
//}
//
//impl From<SendRequestError> for BotError {
//    fn from(err: SendRequestError) -> Self {
//        BotError::SendRequest(err)
//    }
//}
//
//impl From<JsonPayloadError> for BotError {
//    fn from(err: JsonPayloadError) -> Self {
//        BotError::Json(err)
//    }
//}
//
//impl From<serde_json::Error> for BotError {
//    fn from(err: serde_json::Error) -> Self {
//        BotError::Deserialize(err)
//    }
//}
//
//impl From<github::Error> for BotError {
//    fn from(err: github::Error) -> Self {
//        BotError::Github(err)
//    }
//}
//
//#[derive(Debug, Deserialize)]
//struct PullRequest {
//    title: String,
//    number: u32,
//    mergeable: String,
//    #[serde(rename = "baseRefName")]
//    base_ref_name: String,
//    #[serde(rename = "headRefName")]
//    head_ref_name: String,
//}
//
//#[derive(Clone, Debug)]
//pub enum Event {
//    TryMerge,
//    /// Event loop tick.
//    Turn,
//}
//
//struct MergeBot<R> {
//    rx: R,
//}
//
//impl<R> MergeBot<R>
//where
//    R: Stream<Item = Event, Error = ()>
//{
//
//}
//
//struct AutoMergeBot {
//    cfg: MergeBotConfig,
//}
//
//impl AutoMergeBot {
//    fn new(cfg: MergeBotConfig) -> Self {
//        Self { cfg }
//    }
//
//    #[async]
//    fn run(self) -> Result<(), BotError> {
//        let owner = "sonm-io";
//        let repo = "core";
//        let uri = format!(
//            "{}/repos/{}/{}/pulls",
//            "https://api.github.com", owner, repo
//        );
//        let uri_graphql = "https://api.github.com/graphql";
//        let body = include_str!("../graphql/pull_requests.graphql");
//        let body = json!({
//            "query": body,
//        });
//
//        let github = Github::new(self.cfg.github().clone());
//
// let authorization = format!("token {}",
// self.cfg.github().oauth_token);
//
//        #[async]
//        for timestamp in Interval::new(Instant::now(), self.cfg.interval()) {
//            let request = ClientRequest::post(&uri_graphql)
// .header("Accept",
// "application/vnd.github.jean-grey-preview+json")
// .header("Authorization", authorization.clone())
// .header("User-Agent", self.cfg.github().user_agent.clone())
// .json(body.clone())?; debug!("-> {} {}", request.method(),
// request.uri());            let resp = await!(request.send())?;
//
//            debug!("<- {}", resp.status());
//            let body: Value = await!(resp.json())?;
//            debug!("<- {}", body);
//            let pull_requests: Vec<PullRequest> =
//
// Deserialize::
// deserialize(&body["data"]["repository"]["pullRequests"]["nodes"])?;
// debug!("<- {:?}", body);
//
//            // Update outdated PRs.
//            for pull_request in pull_requests {
// let status = await!(github.pull_request(owner,
// repo).state(&pull_request.head_ref_name))?;
//
//                debug!("<- {:?}", status);
//
// if pull_request.mergeable == "!!MERGEABLE" &&
// pull_request.base_ref_name == "master" { let request =
// MergeRequest {                        owner: owner.to_string(),
//                        repo: repo.to_string(),
//                        body: MergeInput {
//                            base: pull_request.head_ref_name.clone(),
//                            head: "master".to_string(),
//                            commit_message: format!(
//                                "Merge branch 'master' into {}",
//                                pull_request.head_ref_name
//                            ),
//                        },
//                    };
//
//                    await!(github.merge(request))?;
//                }
//            }
//        }
//
// // TODO: Update ONLY the earliest APPROVED PR to decrease load on
// travis. // TODO: On ANY PR event - try to merge (under bot account
// without admin access, only push). // TODO: Make actor: listen for
// events, put in the ordered set. Every 1 sec tick - flush the queue.
//
//        Ok(())
//    }
//}

struct Runtime<S> {
    stream: S,
    log: File,
}

impl<S> Runtime<S>
where
    S: Stream<Item = Event, Error = ()> + 'static,
{
    fn new(stream: S) -> Self {
        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .open("event.log").unwrap();

        Self { stream, log }
    }

    #[async]
    fn run(mut self) -> Result<(), ()> {
        #[async]
        for event in self.stream {
            match event {
                Event::Log(event) => {
                    self.log.write_all(event.to_string().as_bytes()).unwrap();
                    self.log.write_all(b"\n").unwrap();
                    self.log.flush().unwrap();
                }
            }
        }

        Ok(())
    }
}

pub fn run(cfg: Config) -> i32 {
    let sys = actix::System::new("sonmbot");

    let (tx, rx) = mpsc::channel(1024);

    let server = server::new(move || {
        let tx = tx.clone();
        let state = Server::new(tx);

        App::with_state(state).resource("/hook", |r| r.method(http::Method::POST).a(Server::index))
    });

    server
        .bind(cfg.network().addr().to_string())
        .unwrap()
        .shutdown_timeout(1)
        .start();

    sys.handle().spawn(Runtime::new(rx).run().then(|result| {
        match result {
            Ok(()) => info!("runtime finished"),
            Err(err) => error!("finished runtime with {:?}", err),
        }
        Ok(())
    }));

    info!("Started http server: {}", cfg.network().addr().to_string());
    sys.run()
}
