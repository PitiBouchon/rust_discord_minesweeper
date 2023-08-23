use crate::discord_command::buttons::parse_first_line_game_id;
use crate::Handler;
use eyre::Error;
use minesweeper::MinesweeperGrid;
use serenity::client::Context;
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::model::prelude::InteractionResponseType;
use std::io;
use std::io::{ErrorKind, Write};
use std::ops::DerefMut;
use std::path::Path;
use std::process::{Command, Stdio};
use tracing::{debug, instrument};

pub async fn play_button(
    handler: &Handler,
    ctx: &Context,
    command: &MessageComponentInteraction,
) -> eyre::Result<()> {
    debug!("User {} pressed Play", command.user.name);

    let game_id = parse_first_line_game_id(command)?;

    let grids = handler.grids.read().await;
    let game_lock = grids
        .get(&game_id)
        .ok_or(Error::msg(format!("Game {} does not exists", game_id)))?;
    let mut game_data = game_lock.lock().await;
    let (grid, file_path) = game_data.deref_mut();
    let (xpos, ypos) = run_file(grid, file_path.as_path())?;
    let res = grid.discover(xpos, ypos);
    if res.is_some() {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|message| {
                        message.content(format!(
                            "# Minesweeper {}\n{}",
                            game_id,
                            grid.to_discord_string()
                        ))
                    })
            })
            .await?
    } else {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|message| {
                        message
                            .content(format!("# Minesweeper ENDED\n{}", grid.to_discord_string()))
                            .components(|c| c)
                    })
            })
            .await?
    }
    Ok(())
}

#[instrument]
fn run_file(grid: &MinesweeperGrid, file_path: &Path) -> eyre::Result<(usize, usize)> {
    let mut child = Command::new("wasmtime")
        .arg("run")
        .arg(file_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    child
        .stdin
        .as_mut()
        .ok_or(io::Error::new(ErrorKind::NotFound, "No stdin"))?
        .write_all(grid.to_console_string().as_bytes())?;

    let output = child.wait_with_output()?;
    let decoded_output = String::from_utf8_lossy(&output.stdout);
    let output_parts = decoded_output.split(',').collect::<Vec<&str>>();

    let xpos = output_parts
        .first()
        .ok_or(Error::msg(format!("Cannot parse xpos: {}", decoded_output)))?
        .trim()[1..]
        .parse()?;

    let tmp_ypos_str = output_parts
        .get(1)
        .ok_or(Error::msg(format!("Cannot parse ypos: {}", decoded_output)))?
        .trim();

    if tmp_ypos_str.is_empty() {
        return Err(Error::msg(format!("Empty ypos: {}", decoded_output)));
    }

    let ypos = tmp_ypos_str[..tmp_ypos_str.len() - 1].parse()?;

    Ok((xpos, ypos))
}
