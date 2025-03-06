extern crate url;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha1::Digest;
use std::fmt::{Debug, Formatter};
use url::form_urlencoded;

#[derive(Debug, Serialize, Deserialize)]
struct Node(String, i64);

#[derive(Debug, Serialize, Deserialize)]
struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct Info {
    // Field order matters for info hash
    #[serde(default)]
    pub files: Option<Vec<File>>,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(default)]
    pub md5sum: Option<String>,
    pub name: String,
    #[serde(default)]
    pub path: Option<Vec<String>>,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    pub pieces: ByteBuf,
    #[serde(default)]
    pub private: Option<u8>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    pub root_hash: Option<String>,
}

impl Info {
    pub fn get_into_hash_hex(&self) -> Result<String> {
        let value = serde_bencode::to_bytes(&self)?;
        let mut hasher = sha1::Sha1::new();
        hasher.update(&value);
        let info_hash = hasher.finalize();
        let info_hash = hex::encode(info_hash);
        Ok(info_hash)
    }
}

#[derive(Deserialize)]
pub(crate) struct Torrent {
    info: Info,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
}

impl Torrent {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let torrent: Torrent = serde_bencode::from_bytes(bytes)?;
        Ok(torrent)
    }

    pub fn create_magnet_link(&self) -> Result<String> {
        let mut link = String::from("magnet:?");
        let mut params = vec![];

        let xt = format!("urn:btih:{}", self.info.get_into_hash_hex()?);
        params.push(("xt", xt));
        params.push(("dn", self.info.name.clone()));
        if let Some(announce) = &self.announce {
            params.push(("tr", announce.to_string()));
        }
        if let Some(announce_list) = &self.announce_list {
            for trackers in announce_list {
                for tracker in trackers {
                    params.push(("tr", tracker.to_string()));
                }
            }
        }
        if let Some(httpseeds) = &self.httpseeds {
            for seed in httpseeds {
                params.push(("ws", seed.to_string()));
            }
        }
        link.push_str(
            &params
                .into_iter()
                .enumerate()
                .map(|(i, (k, v))| format!("{}{}={}", if i == 0 { "" } else { "&" }, k, v))
                .collect::<String>(),
        );
        let url_encoded = form_urlencoded::byte_serialize(link.as_bytes()).collect();
        Ok(url_encoded)
    }
}

impl Debug for Torrent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "name:\t\t{}", self.info.name)?;
        writeln!(f, "announce:\t{:?}", self.announce)?;
        writeln!(f, "nodes:\t\t{:?}", self.nodes)?;
        match &self.announce_list {
            Some(al) => {
                writeln!(f, "announce-list:\t{:?}", al)?;
            }
            None => {
                writeln!(f, "announce list:\tNone")?;
            }
        }
        writeln!(f, "httpseeds:\t{:?}", self.httpseeds)?;
        writeln!(f, "creation date:\t{:?}", self.creation_date)?;
        writeln!(f, "comment:\t{:?}", self.comment)?;
        writeln!(f, "created by:\t{:?}", self.created_by)?;
        writeln!(f, "encoding:\t{:?}", self.encoding)?;
        writeln!(f, "piece length:\t{:?}", self.info.piece_length)?;
        writeln!(f, "private:\t{:?}", self.info.private)?;
        writeln!(f, "root hash:\t{:?}", self.info.root_hash)?;
        writeln!(f, "md5sum:\t\t{:?}", self.info.md5sum)?;
        writeln!(f, "path:\t\t{:?}", self.info.path)?;
        match &self.info.files {
            Some(files) => {
                for file in files {
                    writeln!(f, "file path:\t{:?}", file.path)?;
                    writeln!(f, "file length:\t{}", file.length)?;
                    writeln!(f, "file md5sum:\t{:?}", file.md5sum)?;
                }
            }
            None => {
                writeln!(f, "files:\t\tNone")?;
            }
        }
        Ok(())
    }
}
