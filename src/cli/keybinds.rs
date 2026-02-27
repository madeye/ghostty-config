use std::path::PathBuf;

use super::discovery::run_ghostty;
use crate::error::AppError;
use serde::{Deserialize, Serialize};

/// A parsed keybinding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    pub trigger: String,
    pub action: String,
}

/// Load default keybindings from `ghostty +list-keybinds`.
///
/// Format: `keybind = trigger=action`
pub fn load_keybinds(ghostty_path: &PathBuf) -> Result<Vec<Keybinding>, AppError> {
    let output = run_ghostty(ghostty_path, &["+list-keybinds"])?;
    Ok(parse_keybind_list(&output))
}

/// Parse keybind output text into a list of keybindings.
pub(crate) fn parse_keybind_list(output: &str) -> Vec<Keybinding> {
    let mut keybinds = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Format: keybind = trigger=action
        let content = line.strip_prefix("keybind").unwrap_or(line);
        let content = content.trim().strip_prefix('=').unwrap_or(content).trim();

        // Split on first = to get trigger=action
        if let Some((trigger, action)) = content.split_once('=') {
            keybinds.push(Keybinding {
                trigger: trigger.trim().to_string(),
                action: action.trim().to_string(),
            });
        }
    }

    keybinds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keybind_list_basic() {
        let input = "keybind = ctrl+c=copy\nkeybind = ctrl+v=paste\n";
        let keybinds = parse_keybind_list(input);
        assert_eq!(keybinds.len(), 2);
        assert_eq!(keybinds[0].trigger, "ctrl+c");
        assert_eq!(keybinds[0].action, "copy");
        assert_eq!(keybinds[1].trigger, "ctrl+v");
        assert_eq!(keybinds[1].action, "paste");
    }

    #[test]
    fn test_parse_keybind_list_empty() {
        let keybinds = parse_keybind_list("");
        assert!(keybinds.is_empty());
    }

    #[test]
    fn test_parse_keybind_list_skips_blank_lines() {
        let input = "keybind = ctrl+c=copy\n\n\nkeybind = ctrl+v=paste\n";
        let keybinds = parse_keybind_list(input);
        assert_eq!(keybinds.len(), 2);
    }

    #[test]
    fn test_parse_keybind_list_complex_trigger() {
        let input = "keybind = ctrl+shift+n=new_window\n";
        let keybinds = parse_keybind_list(input);
        assert_eq!(keybinds.len(), 1);
        assert_eq!(keybinds[0].trigger, "ctrl+shift+n");
        assert_eq!(keybinds[0].action, "new_window");
    }

    #[test]
    fn test_parse_keybind_list_action_with_params() {
        // Some actions have parameters after a colon
        let input = "keybind = ctrl+1=goto_tab:1\n";
        let keybinds = parse_keybind_list(input);
        assert_eq!(keybinds.len(), 1);
        assert_eq!(keybinds[0].trigger, "ctrl+1");
        assert_eq!(keybinds[0].action, "goto_tab:1");
    }
}
