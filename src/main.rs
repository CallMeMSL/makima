use std::env;
use std::path::PathBuf;
use std::time::Duration;

use crate::message_handler::message_handler;
use crate::notify::eval_entry;
use crate::rss::poll_rss;
#[allow(deprecated)]
use serenity::all::standard::Configuration;
#[allow(deprecated)]
use serenity::all::{Message, StandardFramework};
use serenity::async_trait;
use serenity::prelude::*;

mod message_handler;
mod notify;
mod rss;
mod setup;
mod store;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.is_private() && !msg.author.bot {
            if let Err(e) = message_handler(ctx, msg).await {
                log::error!("Error occurred: {e}")
            };
        }
    }
}

#[allow(deprecated)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Warn).unwrap();
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN");
    let rss = env::var("RSS_URL").expect("RSS_URL");
    let check_val = env::var("CHECK_VAL").unwrap_or("60".into());
    let failure_val = env::var("FAILURE_VAL").unwrap_or("180".into());
    let binding = PathBuf::from(env::var("STORE_FOLDER_PATH").unwrap_or("~/.makima".into()));
    let store_path = plain_path::plain(
        &binding
    )?;



    setup::setup_resources(&store_path).unwrap();

    let (send, rec) = tokio::sync::mpsc::channel(3);
    tokio::spawn(poll_rss(
        rss,
        store_path.to_str().unwrap().to_string(),
        Duration::from_secs(check_val.parse()?),
        Duration::from_secs(failure_val.parse()?),
        send,
    ));
    tokio::spawn(eval_entry(rec));

    let framework = StandardFramework::new();
    framework.configure(Configuration::new().no_dm_prefix(true));
    let mut client = Client::builder(token, GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
    Ok(())
}
