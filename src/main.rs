use std::env;
use serenity::{model::channel::Message, async_trait};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuffler::shuffle_people;
use tracing::subscriber::set_global_default;
use tracing::{info, warn, debug};

mod parser;
mod shuffler;

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    #[tracing::instrument(
        name = "Received a new message"
        skip(self, ctx, msg),
    )]
    async fn message(&self, ctx: Context, msg: Message) {
        let mentioned = match parser::parse_shuffle_message(msg.content) {
            Err(e) => {
                debug!(error = debug(e), "Got an error from the parser.");
                return;
            }
            Ok(v) => v,
        };

        if mentioned.len() < 2 {
            return;
        }
        let pairs = match shuffle_people(&mentioned) {
            Err(e) => {
                warn!(error = debug(&e), "Got an error from the shuffler.");
                msg.channel_id.say(&ctx, format!("Error: {:?}", e)).await.ok();
                return;
            }
            Ok(v) => v,
        };

        for (player, avatar) in &pairs {
            let channel = match player.create_dm_channel(&ctx).await {
                Ok(v) => v,
                Err(e) => {
                    warn!(player = debug(player), error = debug(&e), "Error while creating DM channel.");
                    msg.channel_id.say(
                        &ctx,
                        format!("Error while creating DM channel with <@{}>: {:?}", player, e),
                    ).await.ok();
                    return;
                }
            };

            match channel.say(&ctx, format!("You play as <@{}>!", avatar)).await {
                Ok(_) => (),
                Err(e) => {
                    warn!(player = debug(player), "Error while sending a DM.");
                    msg.channel_id.say(
                        &ctx,
                        format!("Error while sending DM to <@{}>: {:?}", player, e),
                    ).await.ok();
                }
            }
        }

        // let mut message = String::from("");
        // for (player, avatar) in pairs {
        //     message = format!("{}\n<@{}> plays as <@{}>", message, player, avatar);
        // }
        // msg.channel_id.say(ctx, message).await.unwrap();
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
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    match client.start().await {
        Ok(client) => {
            println!("Client ready.");
            client
        }
        Err(error) => println!("Client error: {:?}.", error),
    }
}
