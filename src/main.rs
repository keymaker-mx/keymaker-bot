use crate::config::Config;
use matrix_sdk::{
    self,
    events::{
        room::message::{MessageEventContent, TextMessageEventContent},
        AnyMessageEventContent, SyncMessageEvent,
    },
    Client, ClientConfig, EventEmitter, JsonStore, SyncRoom, SyncSettings,
};
use matrix_sdk_common_macros::async_trait;
use tracing::*;
use url::Url;

mod commandparser;
mod config;
mod errors;

struct KeybaseBot {
    /// This clone of the `Client` will send requests to the server,
    /// while the other keeps us in sync with the server using `sync`.
    client: Client,
}

impl KeybaseBot {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl EventEmitter for KeybaseBot {
    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        if let SyncRoom::Joined(room) = room {
            let msg_body = if let SyncMessageEvent {
                content: MessageEventContent::Text(TextMessageEventContent { body: msg_body, .. }),
                ..
            } = event
            {
                msg_body.clone()
            } else {
                String::new()
            };

            if msg_body.contains("!party") {
                let content = AnyMessageEventContent::RoomMessage(MessageEventContent::Text(
                    TextMessageEventContent {
                        body: "🎉🎊🥳 let's PARTY!! 🥳🎊🎉".to_string(),
                        formatted: None,
                        relates_to: None,
                    },
                ));
                // we clone here to hold the lock for as little time as possible.
                let room_id = room.read().await.room_id.clone();

                println!("sending");

                self.client
                    // send our message to the room we found the "!party" command in
                    // the last parameter is an optional Uuid which we don't care about.
                    .room_send(&room_id, content, None)
                    .await
                    .unwrap();

                println!("message sent");
            }
        }
    }
}

async fn login_and_sync(config: Config<'_>) -> Result<(), matrix_sdk::Error> {
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
        .add_event_emitter(Box::new(KeybaseBot::new(client.clone())))
        .await;

    // since we called `sync_once` before we entered our sync loop we must pass
    // that sync token to `sync`
    let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
    // this keeps state from the server streaming in to CommandBot via the EventEmitter trait
    client.sync(settings).await;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Loading Configs...");
    let config = Config::load("config.yml")?;
    login_and_sync(config).await?;

    Ok(())
}
