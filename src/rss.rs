use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use regex::Regex;
use reqwest::IntoUrl;
use rss::Channel;
use tokio::sync::mpsc::Sender;

use crate::setup::load_last_seen;

pub async fn poll_rss(
    link: String,
    store_folder: String,
    timeout: Duration,
    fail_timeout: Duration,
    notify_sender: Sender<Vec<RssEntry>>,
) {
    let mut latest_element = load_last_seen(&store_folder).unwrap();
    loop {
        let feed = match load_rss_feed(&link).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("error occurred while loading rss: {e}.");
                tokio::time::sleep(fail_timeout).await;
                continue;
            }
        };
        if feed[0].title == latest_element {
            tokio::time::sleep(timeout).await;
            continue;
        }
        let new_latest = feed[0].title.clone();
        let mut new_shows = Vec::new();
        for item in feed.into_iter() {
            if item.title == latest_element {
                break;
            }
            new_shows.push(item);
        }
        notify_sender.send(new_shows).await.unwrap();
        write_latest(&store_folder, &new_latest).unwrap();
        latest_element = new_latest;
        tokio::time::sleep(timeout).await;
    }
}

fn write_latest(store_folder: impl Into<PathBuf>, latest: &str) -> Result<()> {
    let mut buf = store_folder.into();
    buf.push("last.txt");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(false)
        .truncate(true)
        .open(buf)?;
    file.write_all(latest.as_bytes())?;
    Ok(())
}

pub struct RssEntry {
    pub title: String,
    pub guid: String,
}

pub async fn load_rss_feed(link: impl IntoUrl) -> Result<Vec<RssEntry>> {
    let content = reqwest::get(link).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    let mut entries = Vec::with_capacity(channel.items.len());
    for (i, item) in channel.items.into_iter().enumerate() {
        if item.title.is_none() {
            log::error!("item {i} in rss feed has no title.");
            continue;
        }
        let title = item.title.unwrap();
        if item.guid.is_none() {
            log::error!("item {i}:{title} in rss feed does not have a link");
            continue;
        }
        let guid = item.guid.unwrap().value;
        entries.push(RssEntry { title, guid })
    }
    if entries.is_empty() {
        bail!("RSS feed returned no valid items");
    }
    Ok(entries)
}

static MAGNET_EXTRACTOR: OnceLock<Regex> = OnceLock::new();

fn get_magnet_extractor() -> &'static Regex {
    MAGNET_EXTRACTOR.get_or_init(|| Regex::new(r#""(magnet.*)" class="card-footer-item">"#).unwrap())
}



impl RssEntry {
    pub async fn get_magnet_for_entry(&self) -> Result<String> {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let response = reqwest::get(&self.guid).await?;
        let data = response.text().await?;
        let ex = get_magnet_extractor();
        let (_, [link]) = ex
            .captures(&data)
            .ok_or(anyhow!("no magnet link found in webpage"))?
            .extract();
        Ok(link.to_string())
    }
}

