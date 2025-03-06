use std::cmp::Reverse;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Result};
use chrono::{DateTime, FixedOffset};
use reqwest::IntoUrl;
use rss::Channel;
use tokio::sync::mpsc::Sender;

use crate::setup::load_last_seen;
use crate::torrent::Torrent;

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
        let mut new_entries: Vec<_> = feed
            .into_iter()
            .filter(|item| item.pub_date > latest_element)
            .collect();
        if new_entries.is_empty() {
            tokio::time::sleep(timeout).await;
            continue;
        }
        new_entries.sort_by_key(|item| Reverse(item.pub_date));

        let new_latest = new_entries[0].pub_date.clone();
        notify_sender.send(new_entries).await.unwrap();
        let latest_string = new_latest.to_rfc2822();
        write_latest(&store_folder, &latest_string).unwrap();
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
    pub link: String,
    pub pub_date: DateTime<FixedOffset>,
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
        if item.link.is_none() {
            log::error!("item {i}:{title} in rss feed does not have a link");
            continue;
        }
        let link = item.link.unwrap();
        let pub_date = chrono::DateTime::parse_from_rfc2822(&item.pub_date.unwrap());
        if pub_date.is_err() {
            log::error!("item {i}:{title} in rss feed has an invalid publication date");
            continue;
        }
        let pub_date = pub_date?;
        entries.push(RssEntry {
            title,
            link,
            pub_date,
        })
    }
    if entries.is_empty() {
        bail!("RSS feed returned no valid items");
    }
    Ok(entries)
}

impl RssEntry {
    pub async fn get_magnet_for_entry(&self) -> Result<String> {
        let response = reqwest::get(&self.link).await?;
        let data = response.bytes().await?;
        let torrent = Torrent::from_bytes(&data)?;
        torrent.create_magnet_link()
    }
}
