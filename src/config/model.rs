use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// The type of value a config option accepts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValueType {
    Boolean,
    Integer,
    Float,
    Color,
    Enum(Vec<String>),
    Text,
    Font,
    Path,
    Keybind,
    Palette,
    CommaSeparated(Box<ConfigValueType>),
}

impl fmt::Display for ConfigValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValueType::Boolean => write!(f, "boolean"),
            ConfigValueType::Integer => write!(f, "integer"),
            ConfigValueType::Float => write!(f, "float"),
            ConfigValueType::Color => write!(f, "color"),
            ConfigValueType::Enum(_) => write!(f, "enum"),
            ConfigValueType::Text => write!(f, "text"),
            ConfigValueType::Font => write!(f, "font"),
            ConfigValueType::Path => write!(f, "path"),
            ConfigValueType::Keybind => write!(f, "keybind"),
            ConfigValueType::Palette => write!(f, "palette"),
            ConfigValueType::CommaSeparated(_) => write!(f, "comma-separated"),
        }
    }
}

/// UI category for grouping config options.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Category {
    Fonts,
    Colors,
    Window,
    Cursor,
    Mouse,
    Clipboard,
    Keybindings,
    Shell,
    Appearance,
    Background,
    MacOS,
    GTKLinux,
    Scrollback,
    Input,
    Terminal,
    Advanced,
}

impl Category {
    pub fn all() -> Vec<Category> {
        vec![
            Category::Fonts,
            Category::Colors,
            Category::Window,
            Category::Cursor,
            Category::Mouse,
            Category::Clipboard,
            Category::Keybindings,
            Category::Shell,
            Category::Appearance,
            Category::Background,
            Category::MacOS,
            Category::GTKLinux,
            Category::Scrollback,
            Category::Input,
            Category::Terminal,
            Category::Advanced,
        ]
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Category::Fonts => "fonts",
            Category::Colors => "colors",
            Category::Window => "window",
            Category::Cursor => "cursor",
            Category::Mouse => "mouse",
            Category::Clipboard => "clipboard",
            Category::Keybindings => "keybindings",
            Category::Shell => "shell",
            Category::Appearance => "appearance",
            Category::Background => "background",
            Category::MacOS => "macos",
            Category::GTKLinux => "gtk-linux",
            Category::Scrollback => "scrollback",
            Category::Input => "input",
            Category::Terminal => "terminal",
            Category::Advanced => "advanced",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Category::Fonts => "Fonts",
            Category::Colors => "Colors",
            Category::Window => "Window",
            Category::Cursor => "Cursor",
            Category::Mouse => "Mouse",
            Category::Clipboard => "Clipboard",
            Category::Keybindings => "Keybindings",
            Category::Shell => "Shell",
            Category::Appearance => "Appearance",
            Category::Background => "Background",
            Category::MacOS => "macOS",
            Category::GTKLinux => "GTK / Linux",
            Category::Scrollback => "Scrollback",
            Category::Input => "Input",
            Category::Terminal => "Terminal",
            Category::Advanced => "Advanced",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Category::Fonts => "Aa",
            Category::Colors => "\u{1f3a8}",
            Category::Window => "\u{25a1}",
            Category::Cursor => "\u{258d}",
            Category::Mouse => "\u{1f5b1}",
            Category::Clipboard => "\u{1f4cb}",
            Category::Keybindings => "\u{2328}",
            Category::Shell => ">_",
            Category::Appearance => "\u{2728}",
            Category::Background => "\u{1f5bc}",
            Category::MacOS => "\u{1f34e}",
            Category::GTKLinux => "\u{1f427}",
            Category::Scrollback => "\u{2195}",
            Category::Input => "\u{270f}",
            Category::Terminal => "\u{1f4df}",
            Category::Advanced => "\u{2699}",
        }
    }
}

/// A single config option parsed from ghostty +show-config --default --docs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOption {
    pub key: String,
    pub default_value: String,
    pub documentation: String,
    pub value_type: ConfigValueType,
    pub category: Category,
    pub is_repeatable: bool,
}

/// The full schema of all discovered config options.
#[derive(Debug, Clone)]
pub struct ConfigSchema {
    pub options: Vec<ConfigOption>,
}

impl ConfigSchema {
    pub fn options_for_category(&self, cat: &Category) -> Vec<&ConfigOption> {
        self.options.iter().filter(|o| &o.category == cat).collect()
    }

    pub fn find_option(&self, key: &str) -> Option<&ConfigOption> {
        self.options.iter().find(|o| o.key == key)
    }
}

/// An entry in the user's config file — preserves structure.
#[derive(Debug, Clone)]
pub enum ConfigEntry {
    Comment(String),
    BlankLine,
    KeyValue { key: String, value: String },
}

/// The user's config file, preserving comments and ordering.
#[derive(Debug, Clone)]
pub struct UserConfig {
    pub entries: Vec<ConfigEntry>,
    pub file_path: PathBuf,
}

impl UserConfig {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            entries: Vec::new(),
            file_path,
        }
    }

    /// Get the value for a key (returns the last occurrence for repeatable keys).
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find_map(|e| match e {
                ConfigEntry::KeyValue { key: k, value } if k == key => Some(value.as_str()),
                _ => None,
            })
    }

    /// Get all values for a repeatable key.
    pub fn get_all(&self, key: &str) -> Vec<&str> {
        self.entries
            .iter()
            .filter_map(|e| match e {
                ConfigEntry::KeyValue { key: k, value } if k == key => Some(value.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Set a value. Updates existing key in-place or appends.
    pub fn set(&mut self, key: &str, value: &str) {
        // Find existing key and update in-place
        for entry in &mut self.entries {
            if let ConfigEntry::KeyValue { key: k, value: v } = entry {
                if k == key {
                    *v = value.to_string();
                    return;
                }
            }
        }
        // Key not found — append
        self.entries.push(ConfigEntry::KeyValue {
            key: key.to_string(),
            value: value.to_string(),
        });
    }

    /// Remove a key (reset to default). Removes the line entirely.
    pub fn remove(&mut self, key: &str) {
        self.entries.retain(|e| match e {
            ConfigEntry::KeyValue { key: k, .. } => k != key,
            _ => true,
        });
    }

    /// Get all set key-value pairs.
    pub fn all_set_values(&self) -> Vec<(&str, &str)> {
        self.entries
            .iter()
            .filter_map(|e| match e {
                ConfigEntry::KeyValue { key, value } => Some((key.as_str(), value.as_str())),
                _ => None,
            })
            .collect()
    }
}

/// Info about an installed theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub palette: Vec<String>,
    pub is_dark: bool,
    pub cursor_color: Option<String>,
    pub selection_background: Option<String>,
}

/// Info about a font family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamily {
    pub name: String,
    pub styles: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ConfigValueType ---

    #[test]
    fn test_config_value_type_display() {
        assert_eq!(ConfigValueType::Boolean.to_string(), "boolean");
        assert_eq!(ConfigValueType::Integer.to_string(), "integer");
        assert_eq!(ConfigValueType::Float.to_string(), "float");
        assert_eq!(ConfigValueType::Color.to_string(), "color");
        assert_eq!(ConfigValueType::Enum(vec![]).to_string(), "enum");
        assert_eq!(ConfigValueType::Text.to_string(), "text");
        assert_eq!(ConfigValueType::Font.to_string(), "font");
        assert_eq!(ConfigValueType::Path.to_string(), "path");
        assert_eq!(ConfigValueType::Keybind.to_string(), "keybind");
        assert_eq!(ConfigValueType::Palette.to_string(), "palette");
        assert_eq!(
            ConfigValueType::CommaSeparated(Box::new(ConfigValueType::Text)).to_string(),
            "comma-separated"
        );
    }

    // --- Category ---

    #[test]
    fn test_category_all_returns_all_variants() {
        let all = Category::all();
        assert_eq!(all.len(), 16);
    }

    #[test]
    fn test_category_slug_roundtrip() {
        for cat in Category::all() {
            let slug = cat.slug();
            assert!(!slug.is_empty());
            // Slug should be lowercase and contain only alphanumeric chars or hyphens
            assert!(slug.chars().all(|c| c.is_alphanumeric() || c == '-'));
        }
    }

    #[test]
    fn test_category_display_name_nonempty() {
        for cat in Category::all() {
            assert!(!cat.display_name().is_empty());
        }
    }

    #[test]
    fn test_category_icon_nonempty() {
        for cat in Category::all() {
            assert!(!cat.icon().is_empty());
        }
    }

    #[test]
    fn test_category_equality() {
        assert_eq!(Category::Fonts, Category::Fonts);
        assert_ne!(Category::Fonts, Category::Colors);
    }

    // --- ConfigSchema ---

    #[test]
    fn test_schema_find_option() {
        let schema = ConfigSchema {
            options: vec![
                ConfigOption {
                    key: "font-size".to_string(),
                    default_value: "13".to_string(),
                    documentation: "Font size".to_string(),
                    value_type: ConfigValueType::Float,
                    category: Category::Fonts,
                    is_repeatable: false,
                },
                ConfigOption {
                    key: "theme".to_string(),
                    default_value: "".to_string(),
                    documentation: "Theme".to_string(),
                    value_type: ConfigValueType::Text,
                    category: Category::Appearance,
                    is_repeatable: false,
                },
            ],
        };
        assert!(schema.find_option("font-size").is_some());
        assert!(schema.find_option("theme").is_some());
        assert!(schema.find_option("nonexistent").is_none());
    }

    #[test]
    fn test_schema_options_for_category() {
        let schema = ConfigSchema {
            options: vec![
                ConfigOption {
                    key: "font-size".to_string(),
                    default_value: "13".to_string(),
                    documentation: "".to_string(),
                    value_type: ConfigValueType::Float,
                    category: Category::Fonts,
                    is_repeatable: false,
                },
                ConfigOption {
                    key: "font-thicken".to_string(),
                    default_value: "false".to_string(),
                    documentation: "".to_string(),
                    value_type: ConfigValueType::Boolean,
                    category: Category::Fonts,
                    is_repeatable: false,
                },
                ConfigOption {
                    key: "theme".to_string(),
                    default_value: "".to_string(),
                    documentation: "".to_string(),
                    value_type: ConfigValueType::Text,
                    category: Category::Appearance,
                    is_repeatable: false,
                },
            ],
        };
        let font_opts = schema.options_for_category(&Category::Fonts);
        assert_eq!(font_opts.len(), 2);
        let appearance_opts = schema.options_for_category(&Category::Appearance);
        assert_eq!(appearance_opts.len(), 1);
        let cursor_opts = schema.options_for_category(&Category::Cursor);
        assert_eq!(cursor_opts.len(), 0);
    }

    // --- UserConfig ---

    #[test]
    fn test_user_config_new_is_empty() {
        let config = UserConfig::new(PathBuf::from("/tmp/test"));
        assert!(config.entries.is_empty());
        assert_eq!(config.get("anything"), None);
    }

    #[test]
    fn test_user_config_set_and_get() {
        let mut config = UserConfig::new(PathBuf::from("/tmp/test"));
        config.set("font-size", "14");
        assert_eq!(config.get("font-size"), Some("14"));
    }

    #[test]
    fn test_user_config_set_overwrites() {
        let mut config = UserConfig::new(PathBuf::from("/tmp/test"));
        config.set("font-size", "14");
        config.set("font-size", "16");
        assert_eq!(config.get("font-size"), Some("16"));
        // Should still have only 1 entry
        assert_eq!(config.all_set_values().len(), 1);
    }

    #[test]
    fn test_user_config_remove() {
        let mut config = UserConfig::new(PathBuf::from("/tmp/test"));
        config.set("font-size", "14");
        config.set("theme", "Dracula");
        config.remove("font-size");
        assert_eq!(config.get("font-size"), None);
        assert_eq!(config.get("theme"), Some("Dracula"));
    }

    #[test]
    fn test_user_config_get_all() {
        let mut config = UserConfig::new(PathBuf::from("/tmp/test"));
        config.entries.push(ConfigEntry::KeyValue {
            key: "keybind".to_string(),
            value: "ctrl+a=select_all".to_string(),
        });
        config.entries.push(ConfigEntry::KeyValue {
            key: "keybind".to_string(),
            value: "ctrl+c=copy".to_string(),
        });
        let all = config.get_all("keybind");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], "ctrl+a=select_all");
        assert_eq!(all[1], "ctrl+c=copy");
    }

    #[test]
    fn test_user_config_get_returns_last_occurrence() {
        let mut config = UserConfig::new(PathBuf::from("/tmp/test"));
        config.entries.push(ConfigEntry::KeyValue {
            key: "keybind".to_string(),
            value: "first".to_string(),
        });
        config.entries.push(ConfigEntry::KeyValue {
            key: "keybind".to_string(),
            value: "second".to_string(),
        });
        assert_eq!(config.get("keybind"), Some("second"));
    }

    #[test]
    fn test_user_config_all_set_values() {
        let mut config = UserConfig::new(PathBuf::from("/tmp/test"));
        config.entries.push(ConfigEntry::Comment("# comment".to_string()));
        config.set("font-size", "14");
        config.entries.push(ConfigEntry::BlankLine);
        config.set("theme", "Dracula");

        let values = config.all_set_values();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&("font-size", "14")));
        assert!(values.contains(&("theme", "Dracula")));
    }

    #[test]
    fn test_user_config_remove_preserves_comments() {
        let mut config = UserConfig::new(PathBuf::from("/tmp/test"));
        config.entries.push(ConfigEntry::Comment("# header".to_string()));
        config.set("font-size", "14");
        config.entries.push(ConfigEntry::BlankLine);
        config.set("theme", "Dracula");

        config.remove("font-size");
        // Comment and blank line should still be there
        assert!(matches!(&config.entries[0], ConfigEntry::Comment(_)));
        assert_eq!(config.get("font-size"), None);
        assert_eq!(config.get("theme"), Some("Dracula"));
    }
}
