use crate::{errors::ParseErrors, Config};
use matrix_sdk::{
    events::{room::message::MessageEventContent, AnyMessageEventContent},
    identifiers::RoomId,
};
use tracing::*;

#[derive(Debug, Clone)]
pub struct CommandParser<'a> {
    pub config: Config<'a>,
}

impl CommandParser<'_> {
    #[instrument(skip(self))]
    async fn help_command(&self) -> Result<AnyMessageEventContent, ParseErrors> {
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

        Ok(content)
    }

    #[instrument(skip(self))]
    pub async fn parse(
        &self,
        sender: String,
        sender_display_name: String,
        room_id: RoomId,
        content: String,
    ) -> Result<AnyMessageEventContent, ParseErrors> {
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
        let content = if command == "!help" || command == "!h" {
            self.help_command().await?
        } else {
            return Err(ParseErrors::Unknown);
        };

        Ok(content)
    }
}
