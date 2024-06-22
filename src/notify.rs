use crate::rss::RssEntry;
use crate::setup::get_user_store;
use anyhow::{anyhow, Result};
use serenity::all::{CreateEmbed, CreateMessage, Http, UserId};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinSet;

pub async fn eval_entry(mut receiver: Receiver<Vec<RssEntry>>) -> Result<()> {
    loop {
        let entries = receiver.recv().await.ok_or(anyhow!("channel died"))?;
        for entry in entries {
            // we dont want to get rate limited when scraping
            tokio::time::sleep(Duration::from_secs(1)).await;
            notify_users(entry).await?;
        }
    }
}

async fn notify_users(entry: RssEntry) -> Result<()> {
    let user_store = get_user_store().read().await;
    let users_to_notify = user_store.get_users_matching(&entry.title);
    drop(user_store);
    if !users_to_notify.is_empty() {
        let magnet = match entry.get_magnet_for_entry().await {
            Ok(m) => {m}
            Err(e) => {
                println!("{}", &e);
                format!("{e}")
            }
        };
        let http: Http = Http::new(&env::var("DISCORD_TOKEN").unwrap());
        let notify_data = Arc::new((magnet, http, entry));
        let mut jset = JoinSet::new();
        for user in users_to_notify.into_iter() {
            jset.spawn(notify_user(user, Arc::clone(&notify_data)));
        }
        while let Some(Err(e)) = jset.join_next().await {
            log::error!("error while dming user: {e}");
        }
    }
    Ok(())
}

async fn notify_user(user: u64, data: Arc<(String, Http, RssEntry)>) -> Result<()> {
    let user = UserId::from(user).to_user(&data.1).await?;
    let embed = CreateEmbed::new()
        .title(&data.2.title)
        .description(&data.2.guid)
        .field(
            "Download",
            format!("[🧲](https://yukino.onrender.com/?r={})", data.0),
            true,
        );
    let msg = CreateMessage::new().content("").embed(embed);
    user.direct_message(&data.1, msg).await?;
    Ok(())
}
