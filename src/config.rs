use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::{fs, path::Path};
use tracing::*;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config<'a> {
    pub homeserver_url: Cow<'a, str>,
    pub mxid: Cow<'a, str>,
    pub password: Cow<'a, str>,
    pub store_path: Cow<'a, str>,
    pub admins: Vec<Cow<'a, str>>,
}

impl Config<'_> {
    #[instrument]
    pub fn load<P: AsRef<Path> + std::fmt::Debug>(path: P) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
        let config: Self = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}
