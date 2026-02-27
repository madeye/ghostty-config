use regex::Regex;
use std::sync::LazyLock;

use super::model::ConfigValueType;

static ENUM_BULLET_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s+\*\s+`([^`]+)`").unwrap());

/// Infer the value type of a config option from its key, default value, and documentation.
pub fn infer_type(key: &str, default: &str, docs: &str) -> ConfigValueType {
    // Manual overrides for known keys
    if let Some(t) = manual_override(key) {
        return t;
    }

    // Keybind
    if key == "keybind" {
        return ConfigValueType::Keybind;
    }

    // Palette
    if key == "palette" {
        return ConfigValueType::Palette;
    }

    // Font keys
    if key == "font-family"
        || key == "font-family-bold"
        || key == "font-family-italic"
        || key == "font-family-bold-italic"
    {
        return ConfigValueType::Font;
    }

    // Boolean: default is "true" or "false"
    if default == "true" || default == "false" {
        return ConfigValueType::Boolean;
    }

    // Color: key contains "color" or "background" or "foreground" and default looks like hex
    if (key.contains("color")
        || key == "background"
        || key == "foreground"
        || key.starts_with("selection-")
        || key == "cursor-text"
        || key == "bold-color"
        || key.starts_with("split-"))
        && (default.is_empty() || default.starts_with('#'))
    {
        return ConfigValueType::Color;
    }

    // Path keys
    if key == "config-file"
        || key == "working-directory"
        || key.starts_with("custom-shader")
    {
        return ConfigValueType::Path;
    }

    // Try to extract enum values from docs
    let enum_values = extract_enum_values(docs);
    if enum_values.len() >= 2 {
        return ConfigValueType::Enum(enum_values);
    }

    // Float: default contains a decimal point
    if default.contains('.') && default.parse::<f64>().is_ok() {
        return ConfigValueType::Float;
    }

    // Integer: default parses as integer
    if !default.is_empty() && default.parse::<i64>().is_ok() {
        return ConfigValueType::Integer;
    }

    // Comma-separated
    if key == "font-synthetic-style" || key == "font-feature" {
        return ConfigValueType::CommaSeparated(Box::new(ConfigValueType::Text));
    }

    ConfigValueType::Text
}

/// Extract enum values from documentation bullet lists like:
///   * `value` - Description
fn extract_enum_values(docs: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut in_list = false;

    for line in docs.lines() {
        if let Some(caps) = ENUM_BULLET_RE.captures(line) {
            let val = caps[1].to_string();
            // Skip values that look like examples or non-enum items
            if !val.contains(' ') && !val.contains('=') && !val.starts_with("e.g") {
                values.push(val);
                in_list = true;
            }
        } else if in_list && !line.trim().is_empty() && !line.starts_with("  ") && !line.starts_with('#') {
            // We've left the bullet list
            // Actually, keep collecting — docs may have multiple paragraphs between bullets
        }
    }

    values
}

/// Check if a key is known to be repeatable.
pub fn is_repeatable(key: &str) -> bool {
    matches!(
        key,
        "keybind"
            | "palette"
            | "font-family"
            | "font-family-bold"
            | "font-family-italic"
            | "font-family-bold-italic"
            | "font-feature"
            | "font-variation"
            | "font-variation-bold"
            | "font-variation-italic"
            | "font-variation-bold-italic"
            | "font-codepoint-map"
            | "config-file"
            | "custom-shader"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_inference() {
        assert!(matches!(infer_type("font-thicken", "false", ""), ConfigValueType::Boolean));
        assert!(matches!(infer_type("bold-is-bright", "true", ""), ConfigValueType::Boolean));
    }

    #[test]
    fn test_color_inference() {
        assert!(matches!(infer_type("background", "", ""), ConfigValueType::Color));
        assert!(matches!(infer_type("foreground", "", ""), ConfigValueType::Color));
        assert!(matches!(infer_type("cursor-color", "#f0f0f0", ""), ConfigValueType::Color));
        assert!(matches!(infer_type("selection-foreground", "", ""), ConfigValueType::Color));
        assert!(matches!(infer_type("bold-color", "", ""), ConfigValueType::Color));
    }

    #[test]
    fn test_font_inference() {
        assert!(matches!(infer_type("font-family", "", ""), ConfigValueType::Font));
        assert!(matches!(infer_type("font-family-bold", "", ""), ConfigValueType::Font));
        assert!(matches!(infer_type("font-family-italic", "", ""), ConfigValueType::Font));
        assert!(matches!(infer_type("font-family-bold-italic", "", ""), ConfigValueType::Font));
    }

    #[test]
    fn test_keybind_inference() {
        assert!(matches!(infer_type("keybind", "", ""), ConfigValueType::Keybind));
    }

    #[test]
    fn test_palette_inference() {
        assert!(matches!(infer_type("palette", "", ""), ConfigValueType::Palette));
    }

    #[test]
    fn test_integer_inference() {
        assert!(matches!(infer_type("scrollback-limit", "10000", ""), ConfigValueType::Integer));
        assert!(matches!(infer_type("font-thicken-strength", "255", ""), ConfigValueType::Integer));
    }

    #[test]
    fn test_float_inference() {
        assert!(matches!(infer_type("font-size", "13", ""), ConfigValueType::Float)); // manual override
        assert!(matches!(infer_type("faint-opacity", "0.5", ""), ConfigValueType::Float));
        assert!(matches!(infer_type("unknown-float", "1.5", ""), ConfigValueType::Float));
    }

    #[test]
    fn test_path_inference() {
        assert!(matches!(infer_type("config-file", "", ""), ConfigValueType::Path));
        assert!(matches!(infer_type("working-directory", "", ""), ConfigValueType::Path));
        assert!(matches!(infer_type("custom-shader", "", ""), ConfigValueType::Path));
    }

    #[test]
    fn test_enum_extraction_from_docs() {
        let docs = r#"Valid values:

  * `block` - A block cursor
  * `bar` - A bar cursor
  * `underline` - An underline cursor
"#;
        assert!(matches!(infer_type("cursor-style", "block", docs), ConfigValueType::Enum(v) if v.len() == 3));
    }

    #[test]
    fn test_enum_skips_examples() {
        let docs = "  * `e.g. something` - skip\n  * `value one` - skip spaces\n";
        // These should not be extracted as enum values
        let result = infer_type("some-key", "", docs);
        assert!(matches!(result, ConfigValueType::Text));
    }

    #[test]
    fn test_text_fallback() {
        assert!(matches!(infer_type("unknown-key", "", ""), ConfigValueType::Text));
        assert!(matches!(infer_type("title", "my title", ""), ConfigValueType::Text));
    }

    #[test]
    fn test_repeatable_keys() {
        assert!(is_repeatable("keybind"));
        assert!(is_repeatable("palette"));
        assert!(is_repeatable("font-family"));
        assert!(is_repeatable("font-feature"));
        assert!(is_repeatable("config-file"));
        assert!(!is_repeatable("font-size"));
        assert!(!is_repeatable("theme"));
        assert!(!is_repeatable("background"));
    }

    #[test]
    fn test_manual_overrides() {
        assert!(matches!(infer_type("font-size", "13", ""), ConfigValueType::Float));
        assert!(matches!(infer_type("window-padding-balance", "false", ""), ConfigValueType::Boolean));
        assert!(matches!(infer_type("adjust-cell-width", "", ""), ConfigValueType::Text));
    }
}

fn manual_override(key: &str) -> Option<ConfigValueType> {
    match key {
        "font-size" => Some(ConfigValueType::Float),
        "adjust-cell-width" | "adjust-cell-height" => Some(ConfigValueType::Text),
        "adjust-font-baseline"
        | "adjust-underline-position"
        | "adjust-underline-thickness"
        | "adjust-strikethrough-position"
        | "adjust-strikethrough-thickness"
        | "adjust-overline-position"
        | "adjust-overline-thickness"
        | "adjust-cursor-thickness"
        | "adjust-cursor-height"
        | "adjust-box-thickness" => Some(ConfigValueType::Text),
        "window-padding-x" | "window-padding-y" => Some(ConfigValueType::Text),
        "window-padding-balance" => Some(ConfigValueType::Boolean),
        "scrollback-limit" => Some(ConfigValueType::Integer),
        "image-storage-limit" => Some(ConfigValueType::Integer),
        "font-thicken-strength" => Some(ConfigValueType::Integer),
        "faint-opacity" => Some(ConfigValueType::Float),
        _ => None,
    }
}
