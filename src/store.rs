use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Entry {
    uid: u64,
    pat: String,
}

impl Entry {
    pub fn new(uid: u64, pat: String) -> Self {
        Entry { uid, pat }
    }
}

pub struct UserStore {
    entries: Vec<Entry>,
    path: PathBuf,
}

impl UserStore {
    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let mut file = std::fs::File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let entries = bincode::deserialize(&data)?;
        Ok(Self { entries, path })
    }

    pub fn save(&self) -> Result<()> {
        let buf = self.path.clone();
        let data = bincode::serialize(&self.entries)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .open(buf)?;
        file.write_all(&data)?;
        Ok(())
    }

    pub fn add(&mut self, e: Entry) -> Result<()> {
        self.entries.push(e);
        self.save()
    }

    pub fn get_elements_for_user(&self, user: u64) -> Vec<String> {
        self.entries
            .iter()
            .filter(|e| e.uid == user)
            .map(|e| e.pat.clone()) // cow?
            .collect()
    }

    pub fn remove_by_index(&mut self, user: u64, i: usize) -> Result<()> {
        let binding = self.get_elements_for_user(user);
        let elem = binding.get(i).ok_or(anyhow!("Out of bounds"))?;
        let searched_entry = Entry {
            uid: user,
            pat: elem.clone(),
        }; // cow!
        let global_i = self
            .entries
            .iter()
            .position(|e| *e == searched_entry)
            .ok_or(anyhow!("global pat search failed"))?;
        self.entries.remove(global_i);
        self.save()?;
        Ok(())
    }

    pub fn remove_user(&mut self, user: u64) -> Result<()> {
        let new_vec = self
            .entries
            .clone()
            .into_iter()
            .filter(|e| e.uid != user)
            .collect();
        self.entries = new_vec;
        self.save()?;
        Ok(())
    }

    pub fn get_users_matching(&self, hay: &str) -> Vec<u64> {
        self.entries
            .iter()
            .filter(|e| hay.contains(&e.pat))
            .map(|e| e.uid)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_match() {
        let us = UserStore {
            entries: vec![
                Entry {
                    uid: 1,
                    pat: "".to_string(),
                },
                Entry {
                    uid: 6,
                    pat: "One ".to_string(),
                },
                Entry {
                    uid: 6,
                    pat: "Naru".to_string(),
                },
            ],
            path: Default::default(),
        };
        let res = us.get_users_matching("One Piece");
        assert_eq!(res, vec![1, 6])
    }
}
