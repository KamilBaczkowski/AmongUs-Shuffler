use std::collections::HashMap;
use std::env;
use game::{Game, Pairs, new_game, Games};
use serenity::model::prelude::{UserId, ChannelId};
use serenity::{model::channel::Message, async_trait};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuffler::shuffle_people;
use tracing::subscriber::set_global_default;
use tracing::{info, warn, debug};

use crate::game::Players;

mod parser;
mod shuffler;
mod game;

struct Bot;

impl Bot {
    #[tracing::instrument(
        name = "Adding a new game to the store"
        skip(self, ctx, pairs),
    )]
    // Adds the game to the store so that it can be used later on.
    async fn add_game(&self, ctx: &Context, channel: ChannelId, pairs: Pairs) {
        debug!("Acquiring write lock for store.");
        let mut store = ctx.data.write().await;
        debug!("Acquiring write lock for games.");
        let mut games = store.get_mut::<Games>().unwrap().write().await;
        debug!("Locks aquired.");

        let game = new_game(pairs[0].0, channel, pairs);
        games.insert(game.get_owner(), game);
        info!("New game added.");
    }

    #[tracing::instrument(
        name = "Looking for game by owner ID."
        skip(self, ctx),
    )]
    // Retrieves a game based on an owner user id.
    async fn get_game(&self, ctx: &Context, owner: UserId) -> Option<Game> {
        debug!("Acquiring read lock for store.");
        let store = &ctx.data.read().await;
        debug!("Acquiring read lock for games.");
        let games = store.get::<Games>().unwrap().read().await;
        debug!("Locks aquired.");

        games.get(&owner).cloned()
    }

    #[tracing::instrument(
        name = "Looking for game by channel ID."
        skip(self, ctx),
    )]
    async fn get_game_by_channel_id(&self, ctx: &Context, channel: ChannelId) -> Option<Game> {
        debug!("Acquiring read lock for store.");
        let store = &ctx.data.read().await;
        debug!("Acquiring read lock for games.");
        let games = store.get::<Games>().unwrap().read().await;
        debug!("Locks aquired.");

        for (_, game) in games.iter() {
            if channel == game.get_channel() {
                info!("Game found.");
                return Some(game.clone());
            }
        }
        None
    }

    #[tracing::instrument(
        name = "Removing a game."
        skip(self, ctx),
    )]
    async fn remove_game(&self, ctx: &Context, game: Game) -> Option<Game> {
        debug!("Acquiring write lock for store.");
        let mut store = ctx.data.write().await;
        debug!("Acquiring write lock for games.");
        let mut games = store.get_mut::<Games>().unwrap().write().await;
        debug!("Locks aquired.");

        let result = games.remove(&game.get_owner());
        info!("Game deleted.");
        result
    }

    // Handles incoming guild messages.
    async fn guild_message(&self, ctx: Context, msg: Message) {
        debug!("Received a new guild message.");

        if let Err(e) = parser::parse_shuffle_message(msg.content) {
            debug!(error = debug(e), "Got an error from the parser."); // This is only a debug log,
            // because it can be a regular message that couldn't be parsed.
            return;
        };

        // Get IDs of mentioned people, but don't include bots.
        let mentioned: Players = msg.mentions.iter()
            .filter(|u| !u.bot)
            .map(|u| u.id)
            .collect();
        debug!(mentions = debug(&mentioned), "Mentions read.");

        // There are some mentions, so lets try to work on them.
        if mentioned.len() < 3 {
            debug!("Too few mentions in the message.");
            msg.channel_id.say(&ctx, "Too few real people mentioned.").await.ok();
            return;
        }

        // Try to find a game that is already associated with the current channel.
        let game = self.get_game_by_channel_id(&ctx, msg.channel_id).await;

        // Get the pairs from the previous game if there is any, so that people don't get the same
        // avatars again.
        let pairs = match game.clone() {
            Some(game) => game.get_pairs(),
            None => vec!(),
        };

        // Let try to shuffle people.
        let pairs = match shuffle_people(&mentioned, &pairs) {
            Err(e) => {
                // Something went wrong, so lets report it.
                warn!(error = debug(&e), "Got an error from the shuffler.");
                msg.channel_id.say(&ctx, format!("Error: {e:?}")).await.ok();
                return;
            }
            Ok(v) => v,
        };

        if let Some(game) = game {
            // Remove any existing games tied to this channel if there are any.
            debug!("A game already found for this channel, removing.");
            self.remove_game(&ctx, game).await;
            info!("A game for this channel removed.");
        }

        // Add the new game to the store.
        debug!("Adding a new game.");
        self.add_game(&ctx, msg.channel_id, pairs.clone()).await;
        info!("Added a new game.");

        let host = pairs[0].0;
        // Notify players about their roles.
        for (player, avatar) in &pairs {
            // Create a DM channel with the user to send them their avatar name.
            debug!(player = debug(player), "Creating a DM channel.");
            let channel = match player.create_dm_channel(&ctx).await {
                Ok(v) => v,
                Err(e) => {
                    warn!(player = debug(player), error = debug(&e), "Error while creating DM channel.");
                    // Something went wrong, lets notify about that on the channel.
                    msg.channel_id.say(
                        &ctx,
                        format!("Error while creating DM channel with <@{player}>: {e:?}"),
                    ).await.ok();
                    return;
                }
            };

            // Send a DM to the person.
            debug!(player = debug(player), "Sending DM to the user.");
            match channel.say(&ctx, format!("You play as <@{avatar}>!")).await {
                Ok(_) => (),
                Err(e) => {
                    warn!(player = debug(player), "Error while sending a DM.");
                    msg.channel_id.say(
                        &ctx,
                        format!("Error while sending DM to <@{player}>: {e:?}"),
                    ).await.ok();
                }
            }

            // Also, if the player that we currently operate on, notify them that they are the host.
            debug!(player = debug(player), "Sending DM to the host.");
            if *player == host {
                // This player was chosen as a host, so lets tell them that too.
                match channel.say(&ctx, String::from(
                    "You are also the host! Send me a message to relay it to everyone in your game."
                )).await {
                    Ok(_) => (),
                    Err(e) => {
                        warn!(player = debug(player), "Error while sending a host DM.");
                        msg.channel_id.say(
                            &ctx,
                            format!("Error while sending the host a DM: {e:?}."),
                        ).await.ok();
                    }
                }
            }
        }

        // Debug code that I use to not spam DMs, and just spam a channel.
        // let mut message = String::from("");
        // for (player, avatar) in &pairs {
        //     message = format!("{message}\n<@{player}> plays as <@{avatar}>");
        // }
        // message = format!("{message}\n<@{host}> is the owner");
        // msg.channel_id.say(&ctx, message).await.unwrap();
    }

    // Handles incoming DMs.
    async fn direct_message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            // Don't generate logs on private messages from the bot itself.
            return
        }
        debug!("Received a new private message.");

        debug!(author = debug(&msg.author), "Looking for a game by the message author.");
        let game = match self.get_game(&ctx, msg.author.id).await {
            Some(game) => {
                game
            },
            None => return,
        };

        info!(game = debug(&game), "Relaying host message to users.");
        let message = format!("The host says: \"{}\"", msg.content);
        match game.get_channel().say(&ctx, message).await {
            Ok(_) => (),
            Err(e) => {
                warn!("Error while sending a host message.");
                msg.channel_id.say(
                    &ctx,
                    format!("Error while sending the host message. {e:?}"),
                ).await.ok();
            }
        };
    }
}

#[async_trait]
impl EventHandler for Bot {
    #[tracing::instrument(
        name = "Received a new message"
        skip(self, ctx, msg),
    )]
    // Handle incoming messages
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.guild_id.is_some() {
            return self.guild_message(ctx, msg).await;
        } else {
            return self.direct_message(ctx, msg).await;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    set_global_default(subscriber).ok();

    let token = env::var("DISCORD_TOKEN").expect("Token not found in the environment.");
    let intents =
        GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Games>(RwLock::new(HashMap::default()));
    }

    match client.start().await {
        Ok(value) => {
            println!("Client ready.");
            value
        }
        Err(error) => println!("Client error: {:?}.", error),
    }
}
