use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashEntry {
    pub id: Uuid,
    pub original_path: PathBuf,
    pub trash_path: PathBuf,
    pub deleted_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub is_dir: bool,
    pub is_symlink: bool,
}

pub fn zut_dir() -> PathBuf {
    dirs::home_dir()
        .expect("impossible de trouver le répertoire home")
        .join(".zut")
}

pub fn metadata_path() -> PathBuf {
    zut_dir().join("metadata.json")
}

pub fn trash_dir() -> PathBuf {
    zut_dir().join("trash")
}

pub fn init_dirs() -> io::Result<()> {
    fs::create_dir_all(trash_dir())
}

pub fn load_entries() -> io::Result<Vec<TrashEntry>> {
    let path = metadata_path();

    let content = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e),
    };

    if content.trim().is_empty() {
        return Ok(vec![]);
    }

    serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn save_entries(entries: &[TrashEntry]) -> io::Result<()> {
    let dir = zut_dir();
    let tmp_path = dir.join("metadata.tmp.json");
    let final_path = metadata_path();

    let content = serde_json::to_string_pretty(entries).map_err(io::Error::other)?;

    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, &final_path)
}

pub fn append_entry(entry: &TrashEntry) -> io::Result<()> {
    let mut entries = load_entries()?;
    entries.push(entry.clone());
    save_entries(&entries)
}

pub fn remove_entry(id: &Uuid) -> io::Result<()> {
    let entries: Vec<TrashEntry> = load_entries()?
        .into_iter()
        .filter(|e| &e.id != id)
        .collect();
    save_entries(&entries)
}
