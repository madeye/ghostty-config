use std::sync::Arc;

use tokio::sync::RwLock;
use tracing_subscriber::EnvFilter;

mod app_state;
mod cli;
mod config;
mod error;
mod routes;

use app_state::AppState;
use cli::actions::load_actions;
use cli::discovery::{find_ghostty, run_ghostty};
use cli::fonts::load_fonts;
use cli::keybinds::load_keybinds;
use cli::themes::load_themes;
use config::file_io::{default_config_path, read_config};
use config::parser::parse_show_config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    tracing::info!("Starting Ghostty Config UI...");

    // Find ghostty binary
    let ghostty_path = find_ghostty()?;
    tracing::info!("Found ghostty at: {}", ghostty_path.display());

    // Load config schema from ghostty
    tracing::info!("Discovering config options...");
    let config_output = run_ghostty(&ghostty_path, &["+show-config", "--default", "--docs"])?;
    let schema = parse_show_config(&config_output)?;
    tracing::info!("Discovered {} config options", schema.options.len());

    // Load themes
    tracing::info!("Loading themes...");
    let themes = load_themes()?;
    tracing::info!("Loaded {} themes", themes.len());

    // Load fonts
    tracing::info!("Loading fonts...");
    let fonts = load_fonts(&ghostty_path).unwrap_or_else(|e| {
        tracing::warn!("Failed to load fonts: {}", e);
        Vec::new()
    });
    tracing::info!("Loaded {} font families", fonts.len());

    // Load actions
    let actions = load_actions(&ghostty_path).unwrap_or_default();
    tracing::info!("Loaded {} actions", actions.len());

    // Load default keybinds
    let default_keybinds = load_keybinds(&ghostty_path).unwrap_or_default();
    tracing::info!("Loaded {} default keybinds", default_keybinds.len());

    // Read user config
    let config_path = default_config_path();
    tracing::info!("Config file: {}", config_path.display());
    let user_config = read_config(&config_path)?;

    // Build shared state
    let state = Arc::new(AppState {
        schema,
        user_config: RwLock::new(user_config),
        themes,
        fonts,
        actions,
        default_keybinds,
        ghostty_path,
        unsaved: RwLock::new(std::collections::HashSet::new()),
    });

    // Build router
    let app = routes::build_router(state);

    let addr = "127.0.0.1:3456";
    tracing::info!("Server starting at http://{}", addr);

    // Open browser
    let url = format!("http://{}", addr);
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Err(e) = open::that(&url) {
            tracing::warn!("Failed to open browser: {}", e);
            eprintln!("Open http://{} in your browser", addr);
        }
    });

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
