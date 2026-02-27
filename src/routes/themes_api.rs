use axum::extract::{Query, State};
use axum::response::Html;
use serde::Deserialize;

use crate::app_state::SharedState;
use crate::error::AppError;

#[derive(Deserialize)]
pub struct ThemeQuery {
    pub search: Option<String>,
    pub filter: Option<String>, // "all", "dark", "light"
}

/// GET /api/themes — list themes with optional search/filter.
pub async fn list_themes(
    State(state): State<SharedState>,
    Query(query): Query<ThemeQuery>,
) -> Result<Html<String>, AppError> {
    let user_config = state.user_config.read().await;
    let current_theme = user_config.get("theme").unwrap_or("").to_string();

    let search = query.search.unwrap_or_default().to_lowercase();
    let filter = query.filter.unwrap_or_else(|| "all".to_string());

    let mut html = String::new();

    for theme in &state.themes {
        if !search.is_empty() && !theme.name.to_lowercase().contains(&search) {
            continue;
        }

        match filter.as_str() {
            "dark" if !theme.is_dark => continue,
            "light" if theme.is_dark => continue,
            _ => {}
        }

        let is_active = theme.name == current_theme;
        let active_class = if is_active {
            "ring-2 ring-indigo-500"
        } else {
            "hover:ring-2 hover:ring-gray-400"
        };

        let palette_swatches: String = theme.palette[..8]
            .iter()
            .filter(|c| !c.is_empty())
            .map(|c| {
                let mut s = String::new();
                s.push_str(
                    "<span class=\"w-4 h-4 rounded-full inline-block\" style=\"background-color: ",
                );
                s.push_str(c);
                s.push_str("\"></span>");
                s
            })
            .collect::<Vec<_>>()
            .join("");

        let active_badge = if is_active {
            "<span class=\"text-xs bg-indigo-100 text-indigo-700 px-2 py-0.5 rounded-full\">Active</span>"
        } else {
            ""
        };

        html.push_str(
            "<div class=\"rounded-xl border border-gray-200 p-3 cursor-pointer transition-all ",
        );
        html.push_str(active_class);
        html.push_str("\" hx-post=\"/api/themes/apply?name=");
        html.push_str(&theme.name);
        html.push_str("\" hx-target=\"#toast-container\" hx-swap=\"innerHTML\" onclick=\"setTimeout(function(){location.reload()},500)\">");
        html.push_str(
            "<div class=\"rounded-lg h-20 mb-2 flex items-end p-2\" style=\"background-color: ",
        );
        html.push_str(&theme.background);
        html.push_str("; color: ");
        html.push_str(&theme.foreground);
        html.push_str("\"><span class=\"text-xs font-mono opacity-80\">$ ghostty</span></div>");
        html.push_str("<div class=\"flex items-center justify-between mb-1\"><span class=\"font-medium text-sm truncate\">");
        html.push_str(&theme.name);
        html.push_str("</span>");
        html.push_str(active_badge);
        html.push_str("</div><div class=\"flex gap-1 mt-1\">");
        html.push_str(&palette_swatches);
        html.push_str("</div></div>");
    }

    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct ApplyThemeQuery {
    pub name: String,
}

/// POST /api/themes/apply — set the theme in config.
pub async fn apply_theme(
    State(state): State<SharedState>,
    Query(query): Query<ApplyThemeQuery>,
) -> Result<Html<String>, AppError> {
    let mut user_config = state.user_config.write().await;
    user_config.set("theme", &query.name);
    drop(user_config);
    state.mark_unsaved("theme").await;
    let count = state.unsaved_count().await;

    let mut html = String::from("<div class=\"bg-emerald-500 text-white px-4 py-2 rounded-lg shadow-lg text-sm font-medium\">Theme set to: ");
    html.push_str(&query.name);
    html.push_str(" (unsaved)</div>");
    html.push_str(&super::config_api::unsaved_badge_oob(count));

    Ok(Html(html))
}
