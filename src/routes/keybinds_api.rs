use axum::extract::State;
use axum::response::Html;
use serde::Deserialize;

use crate::app_state::SharedState;
use crate::config::model::ConfigEntry;
use crate::error::AppError;
use super::config_api::{toast_html, unsaved_badge_oob};

#[derive(Deserialize)]
pub struct AddKeybindForm {
    pub trigger: String,
    pub action: String,
}

/// POST /api/keybinds — add a new keybinding (in memory).
pub async fn add_keybind(
    State(state): State<SharedState>,
    axum::Form(form): axum::Form<AddKeybindForm>,
) -> Result<Html<String>, AppError> {
    let trigger = form.trigger.trim();
    let action = form.action.trim();

    if trigger.is_empty() || action.is_empty() {
        return Ok(Html(toast_html("Both trigger and action are required", true)));
    }

    let keybind_value = format!("{}={}", trigger, action);

    let mut user_config = state.user_config.write().await;
    user_config.entries.push(ConfigEntry::KeyValue {
        key: "keybind".to_string(),
        value: keybind_value,
    });
    drop(user_config);
    state.mark_unsaved("keybind").await;
    let count = state.unsaved_count().await;

    let mut html = toast_html("Keybinding added (unsaved)", false);
    html.push_str(&unsaved_badge_oob(count));
    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct DeleteKeybindForm {
    pub trigger: String,
    pub action: String,
}

/// POST /api/keybinds/delete — remove a custom keybinding (in memory).
pub async fn delete_keybind(
    State(state): State<SharedState>,
    axum::Form(form): axum::Form<DeleteKeybindForm>,
) -> Result<Html<String>, AppError> {
    let target = format!("{}={}", form.trigger.trim(), form.action.trim());

    let mut user_config = state.user_config.write().await;
    user_config.entries.retain(|e| match e {
        ConfigEntry::KeyValue { key, value } => !(key == "keybind" && value == &target),
        _ => true,
    });
    drop(user_config);
    state.mark_unsaved("keybind-delete").await;
    let count = state.unsaved_count().await;

    let mut html = toast_html("Keybinding removed (unsaved)", false);
    html.push_str(&unsaved_badge_oob(count));
    Ok(Html(html))
}
