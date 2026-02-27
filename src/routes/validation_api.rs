use axum::extract::State;
use axum::response::Html;

use crate::app_state::SharedState;
use crate::cli::validate::validate_config;
use crate::error::AppError;

/// GET /api/validate — run ghostty +validate-config and return the result.
pub async fn validate(State(state): State<SharedState>) -> Result<Html<String>, AppError> {
    let result = validate_config(&state.ghostty_path)?;

    let (icon, color_class) = if result.contains("valid")
        || result.contains("Valid")
        || result.trim().is_empty()
        || result == "Configuration is valid!"
    {
        (
            "&#x2705;",
            "bg-emerald-50 border-emerald-300 text-emerald-800",
        )
    } else {
        ("&#x26a0;", "bg-amber-50 border-amber-300 text-amber-800")
    };

    Ok(Html(format!(
        r#"<div class="border rounded-lg p-4 {color_class}" id="validation-result">
            <div class="flex items-center gap-2 font-medium mb-1">
                <span>{icon}</span>
                <span>Validation Result</span>
            </div>
            <pre class="text-sm font-mono whitespace-pre-wrap mt-2">{result}</pre>
        </div>"#,
        color_class = color_class,
        icon = icon,
        result = result,
    )))
}
