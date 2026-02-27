use axum::Router;
use tower_http::services::ServeDir;

use crate::app_state::SharedState;

pub mod config_api;
pub mod fonts_api;
pub mod import_export_api;
pub mod keybinds_api;
pub mod pages;
pub mod preview_api;
pub mod themes_api;
pub mod validation_api;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        // Pages
        .route("/", axum::routing::get(pages::index))
        .route("/category/{slug}", axum::routing::get(pages::category))
        .route("/themes", axum::routing::get(pages::themes_page))
        .route("/keybinds", axum::routing::get(pages::keybinds_page))
        .route(
            "/import-export",
            axum::routing::get(pages::import_export_page),
        )
        // Config API (HTMX)
        .route(
            "/api/config/{key}",
            axum::routing::get(config_api::get_value)
                .put(config_api::set_value)
                .delete(config_api::delete_value),
        )
        // Themes API
        .route("/api/themes", axum::routing::get(themes_api::list_themes))
        .route(
            "/api/themes/apply",
            axum::routing::post(themes_api::apply_theme),
        )
        // Fonts API
        .route("/api/fonts", axum::routing::get(fonts_api::list_fonts))
        .route(
            "/api/fonts/search",
            axum::routing::get(fonts_api::search_fonts),
        )
        // Keybinds API
        .route(
            "/api/keybinds",
            axum::routing::post(keybinds_api::add_keybind),
        )
        .route(
            "/api/keybinds/delete",
            axum::routing::post(keybinds_api::delete_keybind),
        )
        // Save / Apply
        .route("/api/save", axum::routing::post(config_api::save_config))
        .route("/api/apply", axum::routing::post(config_api::apply_config))
        // Validation
        .route(
            "/api/validate",
            axum::routing::get(validation_api::validate),
        )
        // Import/Export
        .route(
            "/api/export",
            axum::routing::get(import_export_api::export_config),
        )
        .route(
            "/api/import",
            axum::routing::post(import_export_api::import_config),
        )
        // Preview
        .route(
            "/api/preview",
            axum::routing::get(preview_api::preview_data),
        )
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
