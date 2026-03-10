use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub auto_purge_after: String,
    pub max_trash_size_mb: u64,
    pub sarcasm_level: String,
    pub personality: bool,
    pub trash_dir: Option<PathBuf>,
    pub confirm_purge: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_purge_after: "7d".to_owned(),
            max_trash_size_mb: 0,
            sarcasm_level: "normal".to_owned(),
            personality: true,
            trash_dir: None,
            confirm_purge: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut config = Self::load_from_file().unwrap_or_default();
        Self::apply_env_overrides(&mut config);
        config
    }

    fn load_from_file() -> Option<Self> {
        let path = dirs::config_dir()?.join("zut").join("config.toml");
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    fn apply_env_overrides(config: &mut Self) {
        if let Ok(val) = std::env::var("ZUT_SARCASM") {
            config.sarcasm_level = val;
        }
        if let Ok(val) = std::env::var("ZUT_TRASH_DIR") {
            config.trash_dir = Some(PathBuf::from(val));
        }
        if let Ok(val) = std::env::var("ZUT_PERSONALITY") {
            config.personality = matches!(val.as_str(), "1" | "true" | "yes");
        }
    }
}
