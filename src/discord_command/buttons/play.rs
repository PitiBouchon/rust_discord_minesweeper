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
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tracing::instrument;
use tracing::log::debug;
use crate::discord_command::buttons::quit::remove_grid;

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
    let (xpos, ypos) = run_file(grid, file_path.as_path()).await?;
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
        let grid_string = grid.to_discord_string();
        drop(game_data); // Why do I need to drop it manually ?
        drop(grids);
        let mut grids = handler.grids.write().await;
        remove_grid(grids.deref_mut(), game_id).await?;
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|message| {
                        message
                            .content(format!("# Minesweeper ENDED\n{}", grid_string))
                            .components(|c| c)
                    })
            })
            .await?
    }
    Ok(())
}

#[instrument]
async fn run_wasm(grid_console_string: String, file_path: PathBuf) -> eyre::Result<(usize, usize)> {
    let file_extension = file_path
        .extension()
        .ok_or(Error::msg("Internal error with file extension"))?;
    let mut child = if file_extension == "wasm" {
        Command::new("wasmtime")
            .arg("run")
            .arg(file_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?
    } else {
        let content = tokio::fs::read_to_string(file_path).await?;

        Command::new("wasmtime")
            .arg("run")
            .arg("--mapdir=./lib/python3.10::./python/lib/python3.10")
            .arg("--")
            .arg("./python/python3.10.wasm")
            .arg("-c")
            .arg(content)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?
    };

    child
        .stdin
        .as_mut()
        .ok_or(io::Error::new(ErrorKind::NotFound, "No stdin"))?
        .write_all(grid_console_string.as_bytes())?;

    let output = child.wait_with_output()?;
    let decoded_output = String::from_utf8_lossy(&output.stdout);
    let output_parts = decoded_output.split(',').collect::<Vec<&str>>();

    let tmp_xpos_str = output_parts
        .first()
        .ok_or(Error::msg(format!("Cannot parse xpos: {}", decoded_output)))?
        .trim();

    if tmp_xpos_str.is_empty() {
        return Err(Error::msg(format!("Empty xpos: {}", decoded_output)));
    }

    let xpos = tmp_xpos_str[1..].parse()?;

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

#[instrument]
async fn run_file(grid: &MinesweeperGrid, file_path: &Path) -> eyre::Result<(usize, usize)> {
    let handle = tokio::task::spawn(run_wasm(grid.to_console_string(), file_path.to_path_buf()));
    tokio::time::timeout(Duration::from_secs(2), handle).await??
}
