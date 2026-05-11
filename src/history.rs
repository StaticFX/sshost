use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRecord {
    pub host: String,
    pub timestamp: DateTime<Utc>,
}

fn history_path() -> PathBuf {
    let home = std::env::home_dir().unwrap();
    let dir = home.join(".config/sshost");
    let _ = fs::create_dir_all(&dir);
    dir.join("history.json")
}

pub fn log_connection(host: &str) {
    let mut records = get_history();
    records.push(ConnectionRecord {
        host: host.to_string(),
        timestamp: Utc::now(),
    });
    // Keep last 500 entries
    if records.len() > 500 {
        records = records.split_off(records.len() - 500);
    }
    let _ = fs::write(history_path(), serde_json::to_string_pretty(&records).unwrap_or_default());
}

pub fn get_history() -> Vec<ConnectionRecord> {
    let path = history_path();
    if !path.exists() {
        return vec![];
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn get_last_connection(host: &str) -> Option<DateTime<Utc>> {
    get_history()
        .iter()
        .filter(|r| r.host == host)
        .map(|r| r.timestamp)
        .max()
}
