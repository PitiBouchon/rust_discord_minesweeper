pub mod buttons;
pub mod start;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::command::CommandOptionType;

pub fn create_command(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("start")
        .description("Play Minesweeper with a .wasm file")
        .create_option(|option| {
            option
                .name("attachment")
                .description("A wasm file")
                .required(true)
                .kind(CommandOptionType::Attachment)
        })
        .create_option(|option| {
            option
                .name("width")
                .description("Width of the Minesweeper grid")
                .required(false)
                .kind(CommandOptionType::Integer)
                .add_int_choice("Small", 5)
                .add_int_choice("Medium", 8)
                .add_int_choice("Big", 12)
        })
        .create_option(|option| {
            option
                .name("height")
                .description("Height of the Minesweeper grid")
                .required(false)
                .kind(CommandOptionType::Integer)
                .add_int_choice("Small", 5)
                .add_int_choice("Medium", 8)
                .add_int_choice("Big", 12)
        })
        .create_option(|option| {
            option
                .name("probability")
                .description("Bomb probability of the Minesweeper grid")
                .required(false)
                .kind(CommandOptionType::Number)
                .add_number_choice("Easy", 0.2)
                .add_number_choice("Normal", 0.4)
                .add_number_choice("Hard", 0.6)
        })
}
