use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const PROJECT_FILE: &str = ".fizzyctl.toml";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    pub board_id: Option<String>,
    pub account: Option<String>,
}

impl ProjectConfig {
    /// Walk up from cwd to find .fizzyctl.toml
    pub fn find() -> Option<PathBuf> {
        let mut dir = std::env::current_dir().ok()?;
        loop {
            let candidate = dir.join(PROJECT_FILE);
            if candidate.exists() {
                return Some(candidate);
            }
            if !dir.pop() {
                return None;
            }
        }
    }

    pub fn load() -> Result<Option<Self>> {
        if let Some(path) = Self::find() {
            let content = fs::read_to_string(&path)?;
            let config: Self = toml::from_str(&content)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    pub fn load_or_default() -> Self {
        Self::load().ok().flatten().unwrap_or_default()
    }

    pub fn save(path: &Path, config: &Self) -> Result<()> {
        let content = toml::to_string_pretty(config)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get the board ID from project config, falling back to global config.
    pub fn board_id(&self) -> Option<&str> {
        self.board_id.as_deref()
    }

    /// Project root is the directory containing .fizzyctl.toml
    pub fn project_root() -> Option<PathBuf> {
        Self::find().and_then(|p| p.parent().map(|p| p.to_path_buf()))
    }

    /// Resolve board ID: flag > project config > global config
    pub fn resolve_board(
        flag: Option<&str>,
        project: &ProjectConfig,
        global: &crate::config::Config,
    ) -> Result<String> {
        flag.map(|s| s.to_string())
            .or_else(|| project.board_id.clone())
            .or_else(|| global.board.clone())
            .ok_or_else(|| {
                anyhow!("No board specified. Run `fizzyctl init` or use --board <id>.")
            })
    }
}
