use crate::config::Config;
use crate::errors::Error;
use matrix_sdk::events::{room::message::MessageEventContent, AnyMessageEventContent};
use mrsbfh::commands::command;

#[command(
    help = "`!register` - Register your server to the keymaker project. You need to be server admin for this. For further information checkout [PLACEHOLDER]"
)]
pub async fn register<'a>(
    mut tx: mrsbfh::Sender,
    _config: Config<'a>,
    _sender: String,
    mut _args: Vec<&str>,
) -> Result<(), Error>
where
    Config<'a>: mrsbfh::config::Loader + Clone,
{
    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain("TODO"));

    tx.send(content).await?;

    Ok(())
}
