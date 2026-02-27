use axum::extract::State;
use axum::response::Html;
use serde::Deserialize;

use crate::app_state::SharedState;
use crate::config::model::ConfigEntry;
use crate::error::AppError;
use super::config_api::{toast_html, unsaved_badge_oob};

/// GET /api/export — export config as plain text.
pub async fn export_config(State(state): State<SharedState>) -> Result<String, AppError> {
    let user_config = state.user_config.read().await;

    let mut output = String::new();
    for entry in &user_config.entries {
        match entry {
            ConfigEntry::Comment(text) => {
                output.push_str(text);
                output.push('\n');
            }
            ConfigEntry::BlankLine => {
                output.push('\n');
            }
            ConfigEntry::KeyValue { key, value } => {
                output.push_str(&format!("{} = {}\n", key, value));
            }
        }
    }

    Ok(output)
}

#[derive(Deserialize)]
pub struct ImportForm {
    pub config_text: String,
}

/// POST /api/import — import config from plain text (in memory, unsaved).
pub async fn import_config(
    State(state): State<SharedState>,
    axum::Form(form): axum::Form<ImportForm>,
) -> Result<Html<String>, AppError> {
    let mut user_config = state.user_config.write().await;
    let file_path = user_config.file_path.clone();

    let mut new_entries = Vec::new();
    for line in form.config_text.lines() {
        if line.trim().is_empty() {
            new_entries.push(ConfigEntry::BlankLine);
        } else if line.starts_with('#') {
            new_entries.push(ConfigEntry::Comment(line.to_string()));
        } else if let Some((key, value)) = line.split_once('=') {
            new_entries.push(ConfigEntry::KeyValue {
                key: key.trim().to_string(),
                value: value.trim().to_string(),
            });
        } else {
            new_entries.push(ConfigEntry::Comment(line.to_string()));
        }
    }

    user_config.entries = new_entries;
    user_config.file_path = file_path;
    drop(user_config);
    state.mark_unsaved("import").await;
    let count = state.unsaved_count().await;

    let mut html = toast_html("Configuration imported (unsaved). Use Save or Apply.", false);
    html.push_str(&unsaved_badge_oob(count));
    Ok(Html(html))
}
