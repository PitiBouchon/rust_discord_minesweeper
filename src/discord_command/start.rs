use crate::Handler;
use eyre::Error;
use minesweeper::MinesweeperGrid;
use serenity::model::prelude::application_command::{
    ApplicationCommandInteraction, CommandDataOptionValue,
};
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::{Attachment, InteractionResponseType, ReactionType};
use serenity::prelude::Context;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;
use tracing::instrument;
use tracing::log::debug;

struct MinesweeperSettings {
    width: usize,
    height: usize,
    bomb_probability: f64,
}

pub async fn start_command(
    handler: &Handler,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> eyre::Result<()> {
    debug!("Start a new game from user {}", command.user.name);

    let attachment = get_attachment(command).ok_or(Error::msg("No attachment given"))?;
    if attachment.size > 100_000_000 {
        return Err(Error::msg(format!(
            "File too big ({}>100MB)",
            attachment.size
        )));
    }
    let content_type = attachment
        .content_type
        .as_ref()
        .ok_or(Error::msg("Attachment has no content type"))?;
    let extension = match content_type.as_str() {
        "application/wasm" => "wasm",
        "text/x-python" => "py",
        "text/x-python; charset=utf-8" => "py",
        _ => {
            return Err(Error::msg(format!(
                "Attachment has bad content type: {}",
                content_type
            )))
        }
    };
    let settings = get_settings(command);
    let file_bytes = attachment.download().await?;
    let game_id = handler.number_grid.fetch_add(1, Ordering::AcqRel); // TODO: Do better
    let file_path = store_wasm_to_file(file_bytes.as_slice(), game_id, extension).await?;
    let grid = MinesweeperGrid::new(settings.width, settings.height, settings.bomb_probability);

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content(format!(
                            "# Minesweeper {}\n{}",
                            game_id,
                            grid.to_discord_string()
                        ))
                        .components(|c| {
                            c.create_action_row(|row| {
                                row.create_button(|button| {
                                    button
                                        .custom_id("play_button_id")
                                        .label("Play")
                                        .emoji(ReactionType::Unicode("▶️".to_string()))
                                        .style(ButtonStyle::Success)
                                });
                                row.create_button(|button| {
                                    button
                                        .custom_id("quit_button_id")
                                        .label("Quit")
                                        .emoji(ReactionType::Unicode("🛑".to_string()))
                                        .style(ButtonStyle::Danger)
                                })
                            })
                        })
                })
        })
        .await?;

    let mut grids = handler.grids.write().await;
    grids.insert(game_id, Mutex::new((grid, file_path)));

    Ok(())
}

#[instrument]
fn get_attachment(command: &ApplicationCommandInteraction) -> Option<&Attachment> {
    if let CommandDataOptionValue::Attachment(attachment) =
        command.data.options.get(0)?.resolved.as_ref()?
    {
        Some(attachment)
    } else {
        None
    }
}

#[instrument]
fn get_settings(command: &ApplicationCommandInteraction) -> MinesweeperSettings {
    let mut width = 8;
    let mut height = 8;
    let mut bomb_probability = 0.2;
    for option in command.data.options.iter() {
        match option.name.as_str() {
            "width" => {
                if let Some(CommandDataOptionValue::Integer(width_desired)) = option.resolved {
                    width = width_desired.clamp(2, 100) as usize;
                }
            }
            "height" => {
                if let Some(CommandDataOptionValue::Integer(height_desired)) = option.resolved {
                    height = height_desired.clamp(2, 100) as usize;
                }
            }
            "probability" => {
                if let Some(CommandDataOptionValue::Number(bomb_probability_desired)) =
                    option.resolved
                {
                    bomb_probability = bomb_probability_desired.clamp(0.1, 0.9)
                }
            }
            _ => (),
        }
    }
    MinesweeperSettings {
        width,
        height,
        bomb_probability,
    }
}

#[instrument]
async fn store_wasm_to_file(
    bytes: &[u8],
    game_id: usize,
    extension: &str,
) -> eyre::Result<PathBuf> {
    let path = PathBuf::from(format!("./tmp/{}.{}", game_id, extension));
    tokio::fs::write(path.as_path(), bytes).await?;
    Ok(path)
}
