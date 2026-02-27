use axum::extract::State;
use axum::response::Html;

use crate::app_state::SharedState;
use crate::error::AppError;

/// GET /api/preview — return a terminal preview HTML partial.
pub async fn preview_data(State(state): State<SharedState>) -> Result<Html<String>, AppError> {
    let user_config = state.user_config.read().await;

    let bg = user_config.get("background").unwrap_or("#1e1e2e");
    let fg = user_config.get("foreground").unwrap_or("#cdd6f4");
    let cursor_color = user_config.get("cursor-color").unwrap_or("#f5e0dc");
    let font_family = user_config.get("font-family").unwrap_or("monospace");
    let font_size = user_config.get("font-size").unwrap_or("13");

    // Get palette colors for ANSI preview
    let palette_colors: Vec<String> = (0..16)
        .map(|i| {
            let key = "palette".to_string();
            // Check for palette = i=#color entries
            for val in user_config.get_all(&key) {
                if let Some((idx_str, color)) = val.split_once('=') {
                    if idx_str.trim().parse::<usize>().ok() == Some(i) {
                        return color.trim().to_string();
                    }
                }
            }
            default_palette_color(i)
        })
        .collect();

    Ok(Html(format!(
        r#"<div class="rounded-xl overflow-hidden shadow-lg border border-gray-700" id="terminal-preview">
            <div class="flex items-center gap-2 px-4 py-2 bg-gray-800 border-b border-gray-700">
                <span class="w-3 h-3 rounded-full bg-red-500"></span>
                <span class="w-3 h-3 rounded-full bg-yellow-500"></span>
                <span class="w-3 h-3 rounded-full bg-green-500"></span>
                <span class="ml-2 text-gray-400 text-xs">ghostty</span>
            </div>
            <div class="p-4" style="background-color: {bg}; color: {fg}; font-family: '{font_family}', monospace; font-size: {font_size}px; line-height: 1.5;">
                <div><span style="color: {c2}">user</span><span style="color: {fg}">@</span><span style="color: {c4}">ghostty</span> <span style="color: {c6}">~</span> <span style="color: {fg}">$</span> ls -la</div>
                <div style="color: {c4}">drwxr-xr-x</span>  <span>5 user staff  160 Jan  1 12:00 .</div>
                <div style="color: {c2}">-rw-r--r--</span>  <span>1 user staff  842 Jan  1 12:00 config</div>
                <div style="color: {c1}">-rwxr-xr-x</span>  <span>1 user staff 2048 Jan  1 12:00 script.sh</div>
                <div style="color: {c3}">-rw-r--r--</span>  <span>1 user staff  256 Jan  1 12:00 notes.txt</div>
                <div><span style="color: {c2}">user</span><span style="color: {fg}">@</span><span style="color: {c4}">ghostty</span> <span style="color: {c6}">~</span> <span style="color: {fg}">$</span> <span class="inline-block w-2 h-4 animate-pulse" style="background-color: {cursor_color}"></span></div>
            </div>
        </div>"#,
        bg = bg,
        fg = fg,
        cursor_color = cursor_color,
        font_family = font_family,
        font_size = font_size,
        c1 = palette_colors
            .get(1)
            .map(|s| s.as_str())
            .unwrap_or("#ff5555"),
        c2 = palette_colors
            .get(2)
            .map(|s| s.as_str())
            .unwrap_or("#50fa7b"),
        c3 = palette_colors
            .get(3)
            .map(|s| s.as_str())
            .unwrap_or("#f1fa8c"),
        c4 = palette_colors
            .get(4)
            .map(|s| s.as_str())
            .unwrap_or("#bd93f9"),
        c6 = palette_colors
            .get(6)
            .map(|s| s.as_str())
            .unwrap_or("#8be9fd"),
    )))
}

fn default_palette_color(index: usize) -> String {
    match index {
        0 => "#21222c",
        1 => "#ff5555",
        2 => "#50fa7b",
        3 => "#f1fa8c",
        4 => "#bd93f9",
        5 => "#ff79c6",
        6 => "#8be9fd",
        7 => "#f8f8f2",
        8 => "#6272a4",
        9 => "#ff6e6e",
        10 => "#69ff94",
        11 => "#ffffa5",
        12 => "#d6acff",
        13 => "#ff92df",
        14 => "#a4ffff",
        15 => "#ffffff",
        _ => "#ffffff",
    }
    .to_string()
}
