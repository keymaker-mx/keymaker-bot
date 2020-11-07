use crate::config::Config;
use crate::{
    errors::ParseErrors,
    extensions::{AnyMessageEventContentExt, RoomExt},
};
use matrix_sdk::{
    self,
    events::{
        room::message::{MessageEventContent, TextMessageEventContent},
        AnyMessageEventContent, SyncMessageEvent,
    },
    Client, ClientConfig, EventEmitter, JsonStore, SyncRoom, SyncSettings,
};
use matrix_sdk_common_macros::async_trait;
use tokio::sync::mpsc;
use tracing::*;
use url::Url;

mod command_parser;
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
            if msg_body == "" {
                return;
            }

            let sender = event.sender.clone().to_string();

            let (tx, mut rx) = mpsc::channel(100);
            let mut parser = crate::command_parser::CommandParser {
                config: self.config.clone(),
                tx,
            };
            let room_id = room.read().await.clone().room_id;

            let display_name = room
                .read()
                .await
                .clone()
                .get_sender_displayname(event)
                .to_string();

            let cloned_client = self.client.clone();
            let event_id = event.event_id.clone();
            let cloned_room_id = room_id.clone();

            tokio::spawn(async move {
                if let Err(e) = parser.parse(sender, display_name, msg_body).await {
                    if e.to_string() == ParseErrors::NotACommand.to_string()
                        || e.to_string() == ParseErrors::NotAllowed.to_string()
                    {
                        // Ignore
                        return;
                    }
                    if let Err(e) = cloned_client.clone()
                        .room_send(
                            &cloned_room_id.clone(),
                            AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
                                format!("Error happened: {}", e.to_string()),
                            )),
                            None,
                        )
                        .await
                    {
                        error!("{}", e);
                    }
                }
            });

            while let Some(mut v) = rx.recv().await {
                v.add_relates_to(event_id.clone());
                if let Err(e) = self.client.clone().room_send(&room_id.clone(), v, None).await {
                    error!("{}", e);
                }
            }
        }
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
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Loading Configs...");
    let config = Config::load("config.yml")?;
    login_and_sync(config).await?;

    Ok(())
}
