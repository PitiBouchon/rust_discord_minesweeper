use eyre::Error;
use serenity::model::prelude::message_component::MessageComponentInteraction;
use tracing::instrument;

pub mod play;
pub mod quit;

#[instrument]
fn parse_first_line_game_id(command: &MessageComponentInteraction) -> eyre::Result<usize> {
    let first_line = command
        .message
        .content
        .lines()
        .next()
        .ok_or(Error::msg("Cannot get first line"))?;
    if first_line.len() < 14 {
        return Err(Error::msg(format!(
            "First line to short ({} characters)",
            first_line.len()
        )));
    }
    Ok(first_line[14..].parse()?)
}
