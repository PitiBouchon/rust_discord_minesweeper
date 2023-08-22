#![feature(let_chains)]

mod discord_command;

use konst::primitive::parse_u64;
use konst::unwrap_ctx;
use minesweeper::MinesweeperGrid;
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::application::interaction::MessageFlags;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{GuildId, Interaction, InteractionResponseType};
use serenity::prelude::{Client, EventHandler, GatewayIntents};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use tokio::fs::{create_dir_all, remove_dir_all};
use tokio::sync::{Mutex, RwLock};
use tracing::instrument;

const TOKEN: &str = include_str!("../token.txt");
const APPLICATION_ID: u64 = unwrap_ctx!(parse_u64(include_str!("../application_id.txt")));

pub struct Handler {
    pub number_grid: AtomicUsize,
    pub grids: RwLock<HashMap<usize, Mutex<(MinesweeperGrid, PathBuf)>>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        println!("{} is connected !", data_about_bot.user.name);

        for guild in data_about_bot.guilds {
            println!("In guild : {} | {}", guild.id, guild.unavailable);
            let guild_id = guild.id;

            let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| discord_command::create_command(command))
            })
            .await;

            if let Err(why) = commands {
                dbg!("Failed creating commands: {}", why);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                if command.data.name.as_str() == "start" {
                    if let Err(error) =
                        discord_command::start::start_command(self, &ctx, &command).await
                    {
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(error).flags(MessageFlags::EPHEMERAL)
                                    })
                            })
                            .await
                        {
                            dbg!("Error start: {}", why);
                        }
                    }
                }
            }
            Interaction::MessageComponent(command) => match command.data.custom_id.as_str() {
                "play_button_id" => {
                    if let Err(error) =
                        discord_command::buttons::play::play_button(self, &ctx, &command).await
                    {
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(error).flags(MessageFlags::EPHEMERAL)
                                    })
                            })
                            .await
                        {
                            dbg!("Error play: {}", why);
                        }
                    }
                }
                "quit_button_id" => {
                    if let Err(error) =
                        discord_command::buttons::quit::quit_button(self, &ctx, &command).await
                    {
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(error).flags(MessageFlags::EPHEMERAL)
                                    })
                            })
                            .await
                        {
                            dbg!("Error quit: {}", why);
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

#[tokio::main]
#[instrument]
async fn main() {
    tracing_subscriber::fmt::init();

    let _ = remove_dir_all("./tmp/").await;
    create_dir_all("./tmp/").await.unwrap();

    let intents = GatewayIntents::empty();

    let mut client = Client::builder(TOKEN, intents)
        .event_handler(Handler {
            number_grid: AtomicUsize::new(0),
            grids: RwLock::new(HashMap::with_capacity(10)),
        })
        .application_id(APPLICATION_ID)
        .await
        .expect("Error creating the client");

    if let Err(why) = client.start().await {
        dbg!("Error: {}", why);
    }
}
