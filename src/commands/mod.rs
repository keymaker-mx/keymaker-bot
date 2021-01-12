use crate::config::Config;
use crate::errors::Error;
use mrsbfh::commands::command_generate;

mod register;

#[command_generate(bot_name = "Keymaker", description = "Control bot for keymaker")]
enum Commands {
    Register,
}
