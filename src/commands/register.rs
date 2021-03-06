use crate::models::well_known::WellKnown;
use crate::{config::Config, database::get_database_pool};
use crate::{errors::Error, models::well_known::ServerRegistrationStatus};
use matrix_sdk::{
    events::{room::message::MessageEventContent, AnyMessageEventContent},
    identifiers::{user_id::UserId, RoomId},
};
use mrsbfh::commands::command;
use reqwest::{Client, StatusCode};
use std::convert::TryFrom;

#[command(
    help = "`!register` - Register your server to the keymaker project. You need to be server admin for this. For further information checkout [PLACEHOLDER]"
)]
pub async fn register<'a>(
    matrix_client: matrix_sdk::Client,
    mut tx: mrsbfh::Sender,
    config: Config<'a>,
    sender: String,
    mut _args: Vec<&str>,
) -> Result<(), Error>
where
    Config<'a>: mrsbfh::config::Loader + Clone,
{
    // Signal verification start
    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
        "Starting verification process...",
    ));
    tx.send(content).await?;

    let sender_id_typed = UserId::try_from(sender.clone()).unwrap();
    let server = sender_id_typed.server_name().as_str();

    // Signal step 1
    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
        "[Step 1/8] Getting well-known file...",
    ));
    tx.send(content).await?;

    let well_known_url =
        "https://".to_string() + server + "/.well-known/matrix/mx.homeservers.metadata";

    let client = Client::new();

    // TODO mention tutorial/fixes in errors
    // TODO Add checkmark if step was fine
    if let Ok(resp) = client.get(&well_known_url).send().await {
        // Signal step 2
        let content = AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
            "[Step 2/8] Ensuring .well-known file is reachable...",
        ));
        tx.send(content).await?;

        // Check for status 200
        if resp.status() != StatusCode::OK {
            let content =
                AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(format!(
                    "[ERROR] .well-known file at: '{}' returned incorrect status code {}. We expect Status Code 200.",
                    well_known_url,
                    resp.status()
                )));
            tx.send(content).await?;
            return Ok(());
        }

        // Signal step 3
        let content = AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
            "[Step 3/8] Ensuring .well-known file is valid...",
        ));
        tx.send(content).await?;

        // Verify Json
        match resp.json::<WellKnown>().await {
            Ok(well_known) => {
                // Signal step 4
                let content = AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
                    "[Step 4/8] Ensuring .well-known file has you listed as an admin of the server...",
                ));
                tx.send(content).await?;

                // Ensure sender is an admin of the server
                if !well_known.admins.iter().any(|x| x == &sender) {
                    let content =
                        AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(format!(
                            "[ERROR] According to the .well-known file at: '{}' you are not any of the admins of this homeserver. Your mxid: {}, Admins in the .well-known config: {:?}",
                            well_known_url,
                            sender,
                            well_known.admins
                        )));
                    tx.send(content).await?;
                }

                // Signal step 5
                let content =
                    AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
                        "[Step 5/8] Ensuring .well-known file server_name field is reachable...",
                    ));
                tx.send(content).await?;

                // Ensure server_name is reachable
                let server_name_address = format!("https://{}", well_known.server_name);
                if client.head(&server_name_address).send().await.is_err() {
                    let content =
                        AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(format!(
                            "[ERROR] The server_name field from the .well-known file ('{}') cannot be reached.",
                            server_name_address,
                        )));
                    tx.send(content).await?;
                    return Ok(());
                }

                // Signal step 6
                let content =
                    AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
                        "[Step 6/8] Ensuring .well-known file url field is reachable...",
                    ));
                tx.send(content).await?;

                // Ensure url is reachable
                let url_address = format!("https://{}", well_known.url);
                if client.head(&url_address).send().await.is_err() {
                    let content =
                        AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(format!(
                            "[ERROR] The url field from the .well-known file ('{}') cannot be reached.",
                            url_address,
                        )));
                    tx.send(content).await?;
                    return Ok(());
                }

                // Signal step 7
                let content =
                    AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
                        "[Step 7/8] Ensuring .well-known file logo_url field is reachable...",
                    ));
                tx.send(content).await?;

                // Ensure logo_url is reachable
                if let Some(ref logo_url) = well_known.logo_url {
                    if client.head(logo_url).send().await.is_err() {
                        let content =
                            AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(format!(
                                "[ERROR] The logo_url field from the .well-known file ('{}') cannot be reached.",
                                logo_url,
                            )));
                        tx.send(content).await?;
                        return Ok(());
                    }
                } else {
                    let content =
                        AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
                            "[Step 7/8] Skipping check as no logo_url was defined.",
                        ));
                    tx.send(content).await?;
                }

                let database = get_database_pool(config.clone()).await?;

                sqlx::query!(
                    r#"
                        INSERT INTO servers ( name, url, server_name, logo_url, admins, categories, rules, description, registration_status, verified )
                        VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10 )
                    "#,
                    well_known.name,
                    well_known.url,
                    well_known.server_name,
                    well_known.logo_url,
                    &well_known.admins,
                    &well_known.categories,
                    well_known.rules,
                    well_known.description,
                    well_known.registration_status as ServerRegistrationStatus,
                    false
                )
                .execute(&database)
                    .await?;

                for category in well_known.categories {
                    sqlx::query!(r#"INSERT INTO servers_categories (server_url, category_name) VALUES ( $1, $2 )"#,well_known.url,category).execute(&database).await?;
                }

                let content =
                    AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
                        format!("@room New Server needs verification: {}", server),
                    ));

                if let Ok(ref room_id) = RoomId::try_from(config.admin_room_id.as_ref()) {
                    if matrix_client
                        .room_send(room_id, content, None)
                        .await
                        .is_ok()
                    {
                        let content =
                            AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
                                "[Step 8/8] Server fulfilled automated tests. The server was sent to manual verification. This can take up to some days. The bot will notify you about any update.",
                            ));
                        tx.send(content).await?;
                    } else {
                        let content =
                            AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
                                "[ERROR][Step 8/8] Server fulfilled automated tests. But the bot wasn't able to inform the project admins. Please try again another day or report this at #serverlist:nordgedanken.dev .",
                            ));
                        tx.send(content).await?;
                    }
                } else {
                    let content =
                        AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(
                            "[ERROR][Step 8/8] Server fulfilled automated tests. But the bot wasn't able to inform the project admins. Please try again another day or report this at #serverlist:nordgedanken.dev .",
                        ));
                    tx.send(content).await?;
                }

                // TODO check if already ran to not rerun if being verified
                // TODO add admin commands to press !verify which write verified == 1 to the db (causes it to show up) and tell user it successfully got added
                // TODO add admin command !reject <reason> to reject a server (deletes it from database)
                // TODO add user command !cancel which stops manual verification and deletes it from db
                // TODO add admin command !dm which asks for a dm with the server admin
            }
            Err(e) => {
                tracing::error!("Error parsing json: {:?}", e);
                let content = AnyMessageEventContent::RoomMessage(
                    MessageEventContent::notice_plain(format!(
                        "[ERROR] .well-known file at: '{}' has invalid format.",
                        well_known_url
                    )),
                );
                tx.send(content).await?;
                return Ok(());
            }
        }
    } else {
        let content =
            AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(format!(
                "[ERROR] Unable to find well_known file at: '{}'. This is most likely due to a connectivity issue.",
                well_known_url
            )));
        tx.send(content).await?;
        return Ok(());
    }

    Ok(())
}
