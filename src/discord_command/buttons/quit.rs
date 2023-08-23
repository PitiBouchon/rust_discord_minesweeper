use std::collections::HashMap;
use crate::discord_command::buttons::parse_first_line_game_id;
use crate::Handler;
use serenity::client::Context;
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::model::prelude::InteractionResponseType;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use tokio::sync::Mutex;
use tracing::log::debug;
use minesweeper::MinesweeperGrid;

pub async fn quit_button(
    handler: &Handler,
    ctx: &Context,
    command: &MessageComponentInteraction,
) -> eyre::Result<()> {
    debug!("User {} pressed Quit", command.user.name);

    let game_id = parse_first_line_game_id(command)?;

    let mut grids = handler.grids.write().await;
    remove_grid(grids.deref_mut(), game_id).await?;

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|message| message.components(|c| c))
        })
        .await?;

    Ok(())
}

pub async fn remove_grid(grids: &mut HashMap<usize, Mutex<(MinesweeperGrid, PathBuf)>>, game_id: usize) -> eyre::Result<()> {
    if let Some(game_lock) = grids.remove(&game_id) {
        let game_lock = game_lock.lock().await;
        let (_, file_path) = game_lock.deref();
        tokio::fs::remove_file(file_path).await?;
    }
    Ok(())
}
