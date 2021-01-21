use crate::commands::match_command;
use crate::config::Config;
use matrix_sdk::{
    self, async_trait,
    events::{
        room::member::MemberEventContent, room::message::MessageEventContent, StrippedStateEvent,
        SyncMessageEvent,
    },
    Client, ClientConfig, EventEmitter, Session as SDKSession, SyncRoom, SyncSettings,
};
use mrsbfh::config::Loader;
use mrsbfh::utils::Session;
use std::convert::TryFrom;
use std::fs;
use std::path::Path;
use tokio::sync::mpsc;
use tracing::*;
use url::Url;

mod commands;
mod config;
mod errors;
mod extensions;
mod models;

struct KeybaseBot {
    /// This clone of the `Client` will send requests to the server,
    /// while the other keeps us in sync with the server using `sync`.
    client: Client,
    config: Config<'static>,
}

impl KeybaseBot {
    #[instrument]
    pub fn new(client: Client, config: Config<'static>) -> Self {
        Self {
            client,
            config: config.clone(),
        }
    }
}

#[mrsbfh::commands::commands]
#[mrsbfh::utils::autojoin]
#[async_trait]
impl EventEmitter for KeybaseBot {
    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
    }
    async fn on_stripped_state_member(
        &self,
        room: SyncRoom,
        room_member: &StrippedStateEvent<MemberEventContent>,
        _: Option<MemberEventContent>,
    ) {
    }
}

async fn login_and_sync(config: Config<'static>) -> color_eyre::Result<()> {
    let store_path_string = config.store_path.to_string();
    let store_path = Path::new(&store_path_string);
    if !store_path.exists() {
        fs::create_dir_all(store_path)?;
    }

    let client_config = ClientConfig::new().store_path(fs::canonicalize(&store_path)?);

    let homeserver_url =
        Url::parse(&config.homeserver_url).expect("Couldn't parse the homeserver URL");

    let mut client = Client::new_with_config(homeserver_url, client_config).unwrap();

    if let Some(session) = Session::load(config.session_path.parse().unwrap()) {
        info!("Starting relogin");

        let session = SDKSession {
            access_token: session.access_token,
            device_id: session.device_id.into(),
            user_id: matrix_sdk::identifiers::UserId::try_from(session.user_id.as_str()).unwrap(),
        };

        if let Err(e) = client.restore_login(session).await {
            error!("{}", e);
        };
        info!("Finished relogin");
    } else {
        info!("Starting login");
        let login_response = client
            .login(
                &config.mxid,
                &config.password,
                None,
                Some(&"timetracking-bot".to_string()),
            )
            .await;
        match login_response {
            Ok(login_response) => {
                info!("Session: {:#?}", login_response);
                let session = Session {
                    homeserver: client.homeserver().to_string(),
                    user_id: login_response.user_id.to_string(),
                    access_token: login_response.access_token,
                    device_id: login_response.device_id.into(),
                };
                session.save(config.session_path.parse().unwrap())?;
            }
            Err(e) => error!("Error while login: {}", e),
        }
        info!("Finished login");
    }

    println!("logged in as {}", config.mxid);

    client
        .add_event_emitter(Box::new(KeybaseBot::new(client.clone(), config)))
        .await;

    client.sync(SyncSettings::default()).await;

    Ok(())
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_thread_names(true)
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Loading Configs...");
    let config = Config::load("config.yml")?;
    login_and_sync(config).await?;

    Ok(())
}
