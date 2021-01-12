use crate::config::Config;
use crate::errors::Error;
use mrsbfh::commands::command_generate;
#[command_generate(bot_name = "Keymaker", description = "Control bot for keymaker")]
enum Commands {
}
