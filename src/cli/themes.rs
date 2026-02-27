use std::fs;
use std::path::{Path, PathBuf};

use crate::config::model::ThemeInfo;
use crate::error::AppError;

/// Get the theme directory path.
pub fn theme_dir() -> Option<PathBuf> {
    let candidates = [
        "/Applications/Ghostty.app/Contents/Resources/ghostty/themes",
        "/usr/share/ghostty/themes",
        "/usr/local/share/ghostty/themes",
    ];

    for path in &candidates {
        let p = PathBuf::from(path);
        if p.is_dir() {
            return Some(p);
        }
    }

    // Try XDG data dirs
    if let Ok(data_dir) = std::env::var("XDG_DATA_DIRS") {
        for dir in data_dir.split(':') {
            let p = PathBuf::from(dir).join("ghostty/themes");
            if p.is_dir() {
                return Some(p);
            }
        }
    }

    None
}

/// Load all themes with color extraction.
pub fn load_themes() -> Result<Vec<ThemeInfo>, AppError> {
    let dir = match theme_dir() {
        Some(d) => d,
        None => {
            tracing::warn!("Could not find ghostty themes directory");
            return Ok(Vec::new());
        }
    };

    let mut themes = Vec::new();

    let entries = fs::read_dir(&dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(theme) = parse_theme_file(&path) {
                themes.push(theme);
            }
        }
    }

    themes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(themes)
}

/// Parse a single theme file and extract colors.
pub(crate) fn parse_theme_file(path: &Path) -> Option<ThemeInfo> {
    let name = path.file_name()?.to_str()?.to_string();
    let content = fs::read_to_string(path).ok()?;

    let mut background = String::from("#000000");
    let mut foreground = String::from("#ffffff");
    let mut palette = vec![String::new(); 16];
    let mut cursor_color = None;
    let mut selection_background = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "background" => background = value.to_string(),
                "foreground" => foreground = value.to_string(),
                "cursor-color" => cursor_color = Some(value.to_string()),
                "selection-background" => selection_background = Some(value.to_string()),
                "palette" => {
                    if let Some((idx_str, color)) = value.split_once('=') {
                        if let Ok(idx) = idx_str.trim().parse::<usize>() {
                            if idx < 16 {
                                palette[idx] = color.trim().to_string();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let is_dark = is_dark_color(&background);

    Some(ThemeInfo {
        name,
        background,
        foreground,
        palette,
        is_dark,
        cursor_color,
        selection_background,
    })
}

/// Determine if a hex color is dark based on luminance.
pub(crate) fn is_dark_color(hex: &str) -> bool {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return true;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64;

    // Relative luminance
    let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
    luminance < 128.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_is_dark_color_black() {
        assert!(is_dark_color("#000000"));
    }

    #[test]
    fn test_is_dark_color_white() {
        assert!(!is_dark_color("#ffffff"));
    }

    #[test]
    fn test_is_dark_color_dark_blue() {
        assert!(is_dark_color("#1e1e2e"));
    }

    #[test]
    fn test_is_dark_color_light_gray() {
        assert!(!is_dark_color("#f0f0f0"));
    }

    #[test]
    fn test_is_dark_color_without_hash() {
        assert!(is_dark_color("000000"));
        assert!(!is_dark_color("ffffff"));
    }

    #[test]
    fn test_is_dark_color_short_hex() {
        // Short hex should default to dark
        assert!(is_dark_color("#abc"));
    }

    #[test]
    fn test_parse_theme_file_basic() {
        let content = "background = #1e1e2e\nforeground = #cdd6f4\npalette = 0=#45475a\npalette = 1=#f38ba8\n";
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();

        let theme = parse_theme_file(tmp.path()).unwrap();
        assert_eq!(theme.background, "#1e1e2e");
        assert_eq!(theme.foreground, "#cdd6f4");
        assert!(theme.is_dark);
        assert_eq!(theme.palette[0], "#45475a");
        assert_eq!(theme.palette[1], "#f38ba8");
    }

    #[test]
    fn test_parse_theme_file_with_cursor_and_selection() {
        let content = "background = #ffffff\nforeground = #000000\ncursor-color = #ff0000\nselection-background = #aaaaaa\n";
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();

        let theme = parse_theme_file(tmp.path()).unwrap();
        assert_eq!(theme.background, "#ffffff");
        assert!(!theme.is_dark);
        assert_eq!(theme.cursor_color, Some("#ff0000".to_string()));
        assert_eq!(theme.selection_background, Some("#aaaaaa".to_string()));
    }

    #[test]
    fn test_parse_theme_file_skips_comments() {
        let content = "# This is a comment\nbackground = #000000\n# Another comment\nforeground = #ffffff\n";
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();

        let theme = parse_theme_file(tmp.path()).unwrap();
        assert_eq!(theme.background, "#000000");
        assert_eq!(theme.foreground, "#ffffff");
    }

    #[test]
    fn test_parse_theme_file_defaults() {
        // Empty file should use defaults
        let content = "";
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();

        let theme = parse_theme_file(tmp.path()).unwrap();
        assert_eq!(theme.background, "#000000"); // default
        assert_eq!(theme.foreground, "#ffffff"); // default
        assert!(theme.is_dark);
        assert_eq!(theme.cursor_color, None);
        assert_eq!(theme.selection_background, None);
    }

    #[test]
    fn test_parse_theme_file_palette_bounds() {
        let content = "palette = 15=#abcdef\npalette = 16=#ignore\n";
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();

        let theme = parse_theme_file(tmp.path()).unwrap();
        assert_eq!(theme.palette[15], "#abcdef");
        // palette[16] doesn't exist (only 16 entries)
        assert_eq!(theme.palette.len(), 16);
    }
}
