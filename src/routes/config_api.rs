use axum::extract::{Path, State};
use axum::response::Html;
use serde::Deserialize;

use crate::app_state::SharedState;
use crate::config::file_io::{read_config, write_config};
use crate::error::AppError;

#[derive(Deserialize)]
pub struct SetValueForm {
    pub value: String,
}

/// GET /api/config/:key — return the current value.
pub async fn get_value(
    State(state): State<SharedState>,
    Path(key): Path<String>,
) -> Result<Html<String>, AppError> {
    let user_config = state.user_config.read().await;
    let value = user_config.get(&key).unwrap_or("").to_string();

    let default = state
        .schema
        .find_option(&key)
        .map(|o| o.default_value.as_str())
        .unwrap_or("");

    let display = if value.is_empty() {
        default.to_string()
    } else {
        value
    };

    Ok(Html(display))
}

/// PUT /api/config/:key — update a config value in memory (no disk write).
pub async fn set_value(
    State(state): State<SharedState>,
    Path(key): Path<String>,
    axum::Form(form): axum::Form<SetValueForm>,
) -> Result<Html<String>, AppError> {
    let value = form.value.trim().to_string();

    let mut user_config = state.user_config.write().await;

    let is_default = state
        .schema
        .find_option(&key)
        .map(|o| o.default_value == value)
        .unwrap_or(false);

    if is_default || value.is_empty() {
        user_config.remove(&key);
    } else {
        user_config.set(&key, &value);
    }

    state.mark_unsaved(&key).await;
    let count = state.unsaved_count().await;

    Ok(Html(toast_with_badge("Updated (unsaved)", false, count)))
}

/// DELETE /api/config/:key — remove a config value in memory (no disk write).
pub async fn delete_value(
    State(state): State<SharedState>,
    Path(key): Path<String>,
) -> Result<Html<String>, AppError> {
    let mut user_config = state.user_config.write().await;
    user_config.remove(&key);
    state.mark_unsaved(&key).await;
    let count = state.unsaved_count().await;

    Ok(Html(toast_with_badge("Reset to default (unsaved)", false, count)))
}

/// POST /api/save — write in-memory config to disk, then reload.
pub async fn save_config(
    State(state): State<SharedState>,
) -> Result<Html<String>, AppError> {
    let path = {
        let user_config = state.user_config.read().await;
        write_config(&user_config)?;
        user_config.file_path.clone()
    };

    // Reload from disk so in-memory state matches the file.
    let reloaded = read_config(&path)?;
    *state.user_config.write().await = reloaded;
    state.clear_unsaved().await;

    Ok(Html(toast_with_badge("Config saved to disk", false, 0)))
}

/// POST /api/apply — save config to disk and tell Ghostty to reload.
pub async fn apply_config(
    State(state): State<SharedState>,
) -> Result<Html<String>, AppError> {
    let path = {
        let user_config = state.user_config.read().await;
        write_config(&user_config)?;
        user_config.file_path.clone()
    };

    // Reload from disk so in-memory state matches the file.
    let reloaded = read_config(&path)?;
    *state.user_config.write().await = reloaded;
    state.clear_unsaved().await;

    let reload_result = trigger_ghostty_reload();

    let (message, is_warn) = match &reload_result {
        Ok(_) => ("Config saved and Ghostty reloaded", false),
        Err(e) => {
            tracing::warn!("Failed to trigger Ghostty reload: {}", e);
            ("Config saved (reload Ghostty manually with Cmd+Shift+,)", true)
        }
    };

    Ok(Html(toast_with_badge(message, is_warn, 0)))
}

/// Trigger Ghostty to reload its config.
fn trigger_ghostty_reload() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(r#"tell application "System Events"
    if (name of processes) contains "ghostty" then
        tell process "ghostty"
            keystroke "," using {command down, shift down}
        end tell
    end if
end tell"#)
            .output()
            .map_err(|e| format!("osascript failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("osascript error: {}", stderr));
        }
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Auto-reload not supported on this platform".to_string())
    }
}

/// Build a toast HTML + an OOB swap to update the unsaved badge.
fn toast_with_badge(message: &str, is_error: bool, unsaved_count: usize) -> String {
    let mut html = toast_html(message, is_error);
    html.push_str(&unsaved_badge_oob(unsaved_count));
    html
}

pub fn toast_html(message: &str, is_error: bool) -> String {
    let color_class = if is_error {
        "bg-amber-500"
    } else {
        "bg-emerald-500"
    };
    let mut html = String::new();
    html.push_str("<div class=\"");
    html.push_str(color_class);
    html.push_str(" text-white px-4 py-2 rounded-lg shadow-lg text-sm font-medium animate-fade-in\" style=\"animation: fadeIn 0.2s ease-out, fadeOut 0.3s ease-in 1.7s forwards;\">");
    html.push_str(message);
    html.push_str("</div>");
    html
}

/// OOB swap to update the unsaved badge in the header.
pub fn unsaved_badge_oob(count: usize) -> String {
    let mut html = String::new();
    html.push_str("<span id=\"unsaved-badge\" hx-swap-oob=\"innerHTML\">");
    if count > 0 {
        html.push_str("<span class=\"inline-flex items-center justify-center w-5 h-5 text-xs font-bold text-white bg-red-500 rounded-full\">");
        html.push_str(&count.to_string());
        html.push_str("</span>");
    }
    html.push_str("</span>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_html_success() {
        let html = toast_html("Saved!", false);
        assert!(html.contains("bg-emerald-500"));
        assert!(html.contains("Saved!"));
        assert!(!html.contains("bg-amber-500"));
    }

    #[test]
    fn test_toast_html_error() {
        let html = toast_html("Error occurred", true);
        assert!(html.contains("bg-amber-500"));
        assert!(html.contains("Error occurred"));
        assert!(!html.contains("bg-emerald-500"));
    }

    #[test]
    fn test_unsaved_badge_oob_zero() {
        let html = unsaved_badge_oob(0);
        assert!(html.contains("unsaved-badge"));
        assert!(html.contains("hx-swap-oob"));
        // Should not contain the count badge
        assert!(!html.contains("bg-red-500"));
    }

    #[test]
    fn test_unsaved_badge_oob_nonzero() {
        let html = unsaved_badge_oob(3);
        assert!(html.contains("unsaved-badge"));
        assert!(html.contains("hx-swap-oob"));
        assert!(html.contains("bg-red-500"));
        assert!(html.contains("3"));
    }

    #[test]
    fn test_toast_with_badge() {
        let html = toast_with_badge("Updated", false, 2);
        // Should contain both the toast and the badge
        assert!(html.contains("Updated"));
        assert!(html.contains("bg-emerald-500"));
        assert!(html.contains("unsaved-badge"));
        assert!(html.contains("2"));
    }
}
