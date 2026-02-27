use std::path::PathBuf;

use super::discovery::run_ghostty;
use crate::error::AppError;

/// Load all available actions from `ghostty +list-actions`.
pub fn load_actions(ghostty_path: &PathBuf) -> Result<Vec<String>, AppError> {
    let output = run_ghostty(ghostty_path, &["+list-actions"])?;
    Ok(parse_action_list(&output))
}

/// Parse actions output text into a list of action names.
pub(crate) fn parse_action_list(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_action_list_basic() {
        let input = "copy\npaste\nnew_window\nclose_surface\n";
        let actions = parse_action_list(input);
        assert_eq!(actions.len(), 4);
        assert_eq!(actions[0], "copy");
        assert_eq!(actions[3], "close_surface");
    }

    #[test]
    fn test_parse_action_list_empty() {
        let actions = parse_action_list("");
        assert!(actions.is_empty());
    }

    #[test]
    fn test_parse_action_list_skips_blank_lines() {
        let input = "copy\n\npaste\n\n";
        let actions = parse_action_list(input);
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn test_parse_action_list_trims_whitespace() {
        let input = "  copy  \n  paste  \n";
        let actions = parse_action_list(input);
        assert_eq!(actions[0], "copy");
        assert_eq!(actions[1], "paste");
    }
}
