use axum::extract::{Query, State};
use axum::response::Html;
use serde::Deserialize;

use crate::app_state::SharedState;
use crate::error::AppError;

#[derive(Deserialize)]
pub struct FontQuery {
    pub search: Option<String>,
}

/// GET /api/fonts — return all font families.
pub async fn list_fonts(State(state): State<SharedState>) -> Result<Html<String>, AppError> {
    let mut html = String::new();
    for font in &state.fonts {
        html.push_str(&format!(
            r#"<option value="{}">{}</option>"#,
            font.name, font.name
        ));
    }
    Ok(Html(html))
}

/// GET /api/fonts/search — search fonts.
pub async fn search_fonts(
    State(state): State<SharedState>,
    Query(query): Query<FontQuery>,
) -> Result<Html<String>, AppError> {
    let search = query.search.unwrap_or_default().to_lowercase();
    let mut html = String::new();

    html.push_str(r#"<option value="">System Default</option>"#);

    for font in &state.fonts {
        if !search.is_empty() && !font.name.to_lowercase().contains(&search) {
            continue;
        }
        html.push_str(&format!(
            r#"<option value="{name}" style="font-family: '{name}'">{name}</option>"#,
            name = font.name
        ));
    }
    Ok(Html(html))
}
