use crate::metadata;
use chrono::Utc;
use humansize::{DECIMAL, format_size};
use serde::{Deserialize, Serialize};
use std::{fs, io};

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistentStats {
    restored_count: u64,
}

pub struct TrashStats {
    pub files_count: usize,
    pub total_size: u64,
    pub oldest: Option<String>,
    pub largest: Option<String>,
    pub files_this_week: usize,
    pub restored_count: u64,
}

fn stats_path() -> std::path::PathBuf {
    metadata::zut_dir().join("stats.json")
}

fn load_persistent() -> PersistentStats {
    let Ok(content) = fs::read_to_string(stats_path()) else {
        return PersistentStats::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_persistent(stats: &PersistentStats) -> io::Result<()> {
    let content = serde_json::to_string_pretty(stats).map_err(io::Error::other)?;
    fs::write(stats_path(), content)
}

pub fn increment_restored() {
    let mut p = load_persistent();
    p.restored_count += 1;
    let _ = save_persistent(&p);
}

pub fn compute_stats() -> io::Result<TrashStats> {
    let entries = metadata::load_entries()?;
    let persistent = load_persistent();
    let now = Utc::now();
    let week_ago = now - chrono::Duration::days(7);

    let oldest = entries.iter().min_by_key(|e| e.deleted_at).map(|e| {
        e.original_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "?".to_owned())
    });

    let largest = entries.iter().max_by_key(|e| e.size_bytes).map(|e| {
        let name = e
            .original_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "?".to_owned());
        format!("{} ({})", name, format_size(e.size_bytes, DECIMAL))
    });

    Ok(TrashStats {
        files_count: entries.len(),
        total_size: entries.iter().map(|e| e.size_bytes).sum(),
        oldest,
        largest,
        files_this_week: entries.iter().filter(|e| e.deleted_at > week_ago).count(),
        restored_count: persistent.restored_count,
    })
}
