use crate::commands::match_command;
use crate::config::Config;
use matrix_sdk::{
    self, async_trait,
    events::{
        room::member::MemberEventContent, room::message::MessageEventContent, StrippedStateEvent,
        SyncMessageEvent,
    },
    Client, ClientConfig, EventEmitter, JsonStore, SyncRoom, SyncSettings,
};
use mrsbfh::config::Loader;
use tokio::sync::mpsc;
use tracing::*;
use url::Url;

mod commands;
mod config;
mod errors;
mod extensions;

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

async fn login_and_sync(config: Config<'static>) -> Result<(), matrix_sdk::Error> {
    let store = JsonStore::open(&config.store_path.to_string())?;
    let client_config = ClientConfig::new().state_store(Box::new(store));

    let homeserver_url =
        Url::parse(&config.homeserver_url).expect("Couldn't parse the homeserver URL");
    // create a new Client with the given homeserver url and config
    let mut client = Client::new_with_config(homeserver_url, client_config).unwrap();

    client
        .login(&config.mxid, &config.password, None, Some("keybase bot"))
        .await?;

    println!("logged in as {}", config.mxid);

    // An initial sync to set up state and so our bot doesn't respond to old messages.
    // If the `StateStore` finds saved state in the location given the initial sync will
    // be skipped in favor of loading state from the store
    client.sync_once(SyncSettings::default()).await.unwrap();
    // add our CommandBot to be notified of incoming messages, we do this after the initial
    // sync to avoid responding to messages before the bot was running.
    client
        .add_event_emitter(Box::new(KeybaseBot::new(client.clone(), config)))
        .await;

    // since we called `sync_once` before we entered our sync loop we must pass
    // that sync token to `sync`
    let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
    // this keeps state from the server streaming in to CommandBot via the EventEmitter trait
    client.sync(settings).await;

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
