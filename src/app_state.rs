use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::keybinds::Keybinding;
use crate::config::model::{ConfigSchema, FontFamily, ThemeInfo, UserConfig};

pub struct AppState {
    pub schema: ConfigSchema,
    pub user_config: RwLock<UserConfig>,
    pub themes: Vec<ThemeInfo>,
    pub fonts: Vec<FontFamily>,
    pub actions: Vec<String>,
    pub default_keybinds: Vec<Keybinding>,
    pub ghostty_path: PathBuf,
    /// Set of keys with unsaved changes.
    pub unsaved: RwLock<HashSet<String>>,
}

impl AppState {
    pub async fn mark_unsaved(&self, key: &str) {
        self.unsaved.write().await.insert(key.to_string());
    }

    pub async fn clear_unsaved(&self) {
        self.unsaved.write().await.clear();
    }

    pub async fn unsaved_count(&self) -> usize {
        self.unsaved.read().await.len()
    }
}

pub type SharedState = Arc<AppState>;
