use std::error::Error;
use std::fs::File;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Deserializer};
use serde_yaml;

pub use github::Config as GithubConfig;

fn deserialize_addr<'de, D>(de: D) -> Result<SocketAddr, D::Error>
where
    D: Deserializer<'de>,
{
    let (addr, port) = Deserialize::deserialize(de)?;
    let addr = SocketAddr::new(addr, port);

    Ok(addr)
}

fn deserialize_duration<'de, D>(de: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = Deserialize::deserialize(de)?;
    let duration = Duration::new(secs, 0);

    Ok(duration)
}

#[derive(Clone, Debug, Deserialize)]
pub struct MergeBotConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    interval: Duration,
    github: GithubConfig,
}

impl MergeBotConfig {
    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn github(&self) -> &GithubConfig {
        &self.github
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetworkConfig {
    #[serde(deserialize_with = "deserialize_addr")]
    addr: SocketAddr,
    backlog: i32,
}

impl NetworkConfig {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn backlog(&self) -> i32 {
        self.backlog
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    network: NetworkConfig,
    merge: MergeBotConfig,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config, Box<Error>> {
        let rd = File::open(path)?;
        let cfg = serde_yaml::from_reader(&rd)?;
        Ok(cfg)
    }

    pub fn network(&self) -> &NetworkConfig {
        &self.network
    }

    pub fn merge(&self) -> &MergeBotConfig {
        &self.merge
    }
}
