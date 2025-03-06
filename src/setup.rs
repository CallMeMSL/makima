use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use tokio::sync::RwLock;

use crate::store::{Entry, UserStore};

static USER_STORE: OnceLock<RwLock<UserStore>> = OnceLock::new();

pub fn get_user_store() -> &'static RwLock<UserStore> {
    USER_STORE.get_or_init(|| panic!("user store accessed before setup"))
}

pub fn setup_resources(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let mut user_store_path = path.to_path_buf();
    user_store_path.push("user.bin");
    let mut last_seen = path.to_path_buf();
    last_seen.push("last.txt");

    if !path.exists() {
        std::fs::create_dir(path)?;
    }

    if !user_store_path.exists() {
        let empty: Vec<Entry> = Vec::new();
        let data = bincode::serialize(&empty)?;
        let mut file = File::create(&user_store_path)?;
        file.write_all(&data)?;
    }

    let us = UserStore::from_path(user_store_path)?;
    USER_STORE.get_or_init(|| RwLock::new(us));

    if !last_seen.exists() {
        let mut file = File::create(&last_seen)?;
        let near_past = chrono::Utc::now() - chrono::Duration::hours(4);
        file.write_all(near_past.to_rfc2822().as_bytes())?;
    }

    Ok(())
}

pub fn load_last_seen(p: impl Into<PathBuf>) -> Result<DateTime<FixedOffset>> {
    let mut p = p.into();
    p.push("last.txt");
    let mut file = File::open(&p)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let pub_date = chrono::DateTime::parse_from_rfc2822(&contents)?;
    Ok(pub_date)
}
