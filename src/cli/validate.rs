use std::path::PathBuf;

use super::discovery::run_ghostty;
use crate::error::AppError;

/// Run `ghostty +validate-config` and return the output.
pub fn validate_config(ghostty_path: &PathBuf) -> Result<String, AppError> {
    match run_ghostty(ghostty_path, &["+validate-config"]) {
        Ok(output) => {
            if output.trim().is_empty() {
                Ok("Configuration is valid!".to_string())
            } else {
                Ok(output)
            }
        }
        Err(e) => Ok(format!("Validation error: {}", e)),
    }
}
