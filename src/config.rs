use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub base_url: Option<String>,
    pub account: Option<String>,
    pub token: Option<String>,
    pub board: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;

        let path = Self::config_path();
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, &content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    pub fn config_dir() -> PathBuf {
        if let Some(proj) = ProjectDirs::from("", "", "fizzy") {
            proj.config_dir().to_path_buf()
        } else {
            PathBuf::from(std::env::var("HOME").unwrap_or_default())
                .join(".config")
                .join("fizzy")
        }
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn base_url(&self) -> String {
        std::env::var("FIZZY_URL")
            .ok()
            .or_else(|| self.base_url.clone())
            .unwrap_or_else(|| "https://app.fizzy.do".to_string())
    }

    pub fn token(&self) -> Option<String> {
        std::env::var("FIZZY_TOKEN").ok().or_else(|| self.token.clone())
    }

    pub fn require_token(&self) -> Result<String> {
        self.token()
            .ok_or_else(|| anyhow!("Not logged in. Run `fizzyctl login` first."))
    }

    pub fn account(&self) -> Option<String> {
        std::env::var("FIZZY_ACCOUNT")
            .ok()
            .or_else(|| self.account.clone())
    }

    pub fn require_account(&self) -> Result<String> {
        self.account().ok_or_else(|| {
            anyhow!("No account set. Run `fizzyctl accounts` then `fizzyctl set account <slug>`.")
        })
    }
}
