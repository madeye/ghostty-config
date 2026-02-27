use std::fs;
use std::path::Path;

use super::model::{ConfigEntry, UserConfig};
use crate::error::AppError;

/// Read a config file, preserving comments and blank lines.
pub fn read_config(path: &Path) -> Result<UserConfig, AppError> {
    let mut config = UserConfig::new(path.to_path_buf());

    if !path.exists() {
        return Ok(config);
    }

    let content = fs::read_to_string(path)?;

    for line in content.lines() {
        if line.trim().is_empty() {
            config.entries.push(ConfigEntry::BlankLine);
        } else if line.starts_with('#') {
            config.entries.push(ConfigEntry::Comment(line.to_string()));
        } else if let Some((key, value)) = line.split_once('=') {
            config.entries.push(ConfigEntry::KeyValue {
                key: key.trim().to_string(),
                value: value.trim().to_string(),
            });
        } else {
            // Treat unparseable lines as comments to preserve them
            config.entries.push(ConfigEntry::Comment(line.to_string()));
        }
    }

    Ok(config)
}

/// Write the config file, preserving structure.
pub fn write_config(config: &UserConfig) -> Result<(), AppError> {
    // Ensure parent directory exists
    if let Some(parent) = config.file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut output = String::new();
    for entry in &config.entries {
        match entry {
            ConfigEntry::Comment(text) => {
                output.push_str(text);
                output.push('\n');
            }
            ConfigEntry::BlankLine => {
                output.push('\n');
            }
            ConfigEntry::KeyValue { key, value } => {
                output.push_str(key);
                output.push_str(" = ");
                output.push_str(value);
                output.push('\n');
            }
        }
    }

    fs::write(&config.file_path, output)?;
    Ok(())
}

/// Get the default config file path.
pub fn default_config_path() -> std::path::PathBuf {
    if let Some(config_dir) = dirs_config_dir() {
        config_dir.join("ghostty").join("config")
    } else {
        // Fallback
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Path::new(&home)
            .join(".config")
            .join("ghostty")
            .join("config")
    }
}

fn dirs_config_dir() -> Option<std::path::PathBuf> {
    directories::BaseDirs::new().map(|d| d.config_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_roundtrip() {
        let content = "# My config\nfont-size = 14\n\n# Theme\ntheme = Dracula\n";
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();

        let config = read_config(tmp.path()).unwrap();
        assert_eq!(config.entries.len(), 5);
        assert_eq!(config.get("font-size"), Some("14"));
        assert_eq!(config.get("theme"), Some("Dracula"));
    }

    #[test]
    fn test_read_nonexistent_file() {
        let config = read_config(Path::new("/tmp/nonexistent_ghostty_test_config")).unwrap();
        assert!(config.entries.is_empty());
    }

    #[test]
    fn test_read_preserves_comments() {
        let content = "# Comment line\nfont-size = 14\n";
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();

        let config = read_config(tmp.path()).unwrap();
        assert_eq!(config.entries.len(), 2);
        assert!(matches!(&config.entries[0], ConfigEntry::Comment(c) if c == "# Comment line"));
    }

    #[test]
    fn test_read_preserves_blank_lines() {
        let content = "font-size = 14\n\n\ntheme = Dracula\n";
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();

        let config = read_config(tmp.path()).unwrap();
        // 1 kv + 2 blanks + 1 kv = 4
        assert_eq!(config.entries.len(), 4);
        assert!(matches!(&config.entries[1], ConfigEntry::BlankLine));
        assert!(matches!(&config.entries[2], ConfigEntry::BlankLine));
    }

    #[test]
    fn test_read_unparseable_line_becomes_comment() {
        let content = "some random text\nfont-size = 14\n";
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();

        let config = read_config(tmp.path()).unwrap();
        assert!(matches!(&config.entries[0], ConfigEntry::Comment(c) if c == "some random text"));
    }

    #[test]
    fn test_write_and_read_back() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let mut config = UserConfig::new(path.clone());
        config.entries.push(ConfigEntry::Comment("# Generated config".to_string()));
        config.entries.push(ConfigEntry::BlankLine);
        config.entries.push(ConfigEntry::KeyValue {
            key: "font-size".to_string(),
            value: "16".to_string(),
        });
        config.entries.push(ConfigEntry::KeyValue {
            key: "theme".to_string(),
            value: "Dracula".to_string(),
        });

        write_config(&config).unwrap();

        let read_back = read_config(&path).unwrap();
        assert_eq!(read_back.get("font-size"), Some("16"));
        assert_eq!(read_back.get("theme"), Some("Dracula"));
        assert_eq!(read_back.entries.len(), 4);
    }

    #[test]
    fn test_write_preserves_formatting() {
        let content = "# My settings\nfont-size = 14\n\n# Colors\nbackground = #1e1e2e\n";
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();
        let path = tmp.path().to_path_buf();

        let config = read_config(&path).unwrap();
        write_config(&config).unwrap();

        let written = fs::read_to_string(&path).unwrap();
        assert_eq!(written, content);
    }

    #[test]
    fn test_set_then_write_roundtrip() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let mut config = UserConfig::new(path.clone());
        config.set("font-size", "14");
        config.set("theme", "Dracula");
        write_config(&config).unwrap();

        let read_back = read_config(&path).unwrap();
        assert_eq!(read_back.get("font-size"), Some("14"));
        assert_eq!(read_back.get("theme"), Some("Dracula"));
    }

    #[test]
    fn test_default_config_path_exists() {
        let path = default_config_path();
        // Just verify it returns a path that includes "ghostty" and "config"
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("ghostty"));
        assert!(path_str.ends_with("config"));
    }
}
