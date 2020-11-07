use crate::{errors::ParseErrors, Config};
use matrix_sdk::events::{room::message::MessageEventContent, AnyMessageEventContent};
use tokio::sync::mpsc::Sender;
use tracing::*;

#[derive(Debug, Clone)]
pub struct CommandParser<'a> {
    pub config: Config<'a>,
    pub tx: Sender<AnyMessageEventContent>,
}

impl CommandParser<'_> {
    #[instrument(skip(self))]
    async fn help_command(&mut self) -> Result<(), ParseErrors> {
        let content = AnyMessageEventContent::RoomMessage(MessageEventContent::notice_html(
            "
                 # Help for the Keybase Matrix Bot\n\n
                 ## Commands\n\n
                 * `!help` - This output\n
                 "
            .to_string(),
            "
                 <h1>Help for the Keybase Matrix Bot</h1>\n
                 <h2>Commands</h2>\n
                 <ul>\n
                     <li><code>!help</code> - This output</li>\n
                 </ul>"
                .to_string(),
        ));

        self.tx.send(content).await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn parse(
        &mut self,
        sender: String,
        sender_display_name: String,
        content: String,
    ) -> Result<(), ParseErrors> {
        // Todo maybe make this macro based to be able to autogen a help later. Not that important for now

        // TODO replace with a check to base this on the well known data
        // TODO move into register command
        /*if !self.config.allowed_users.iter().any(|x| *x == sender) {
            return Err(ParseErrors::NotAllowed);
        }*/

        // Ignore non commands
        if !content.starts_with('!') {
            return Err(ParseErrors::NotACommand);
        }

        let mut split = content.split_whitespace();

        // This is save
        // ? doesnt work as it returns std::option::NoneError :(
        let command = split.next().unwrap();

        // Make sure this is immutable
        let args: Vec<&str> = split.collect();

        #[allow(clippy::if_same_then_else)]
        if command == "!help" || command == "!h" {
            self.help_command().await?
        } else {
            return Err(ParseErrors::Unknown);
        };

        Ok(())
    }
}
