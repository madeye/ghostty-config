use std::path::PathBuf;
use std::process::Command;

use crate::error::AppError;

/// Find the ghostty binary path.
pub fn find_ghostty() -> Result<PathBuf, AppError> {
    // Try common locations
    let candidates = [
        "/Applications/Ghostty.app/Contents/MacOS/ghostty",
        "/usr/local/bin/ghostty",
        "/usr/bin/ghostty",
    ];

    for path in &candidates {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // Try `which ghostty`
    if let Ok(output) = Command::new("which").arg("ghostty").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    Err(AppError::Cli(
        "Could not find ghostty binary. Is Ghostty installed?".to_string(),
    ))
}

/// Run a ghostty CLI command and return stdout.
pub fn run_ghostty(ghostty_path: &PathBuf, args: &[&str]) -> Result<String, AppError> {
    let output = Command::new(ghostty_path)
        .args(args)
        .output()
        .map_err(|e| AppError::Cli(format!("Failed to run ghostty: {}", e)))?;

    // Ghostty may output to stderr for some commands
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() && stdout.is_empty() {
        // Some ghostty commands write to stderr even on success
        if !stderr.is_empty() {
            return Ok(stderr);
        }
        return Err(AppError::Cli(format!(
            "ghostty command failed: {}",
            stderr
        )));
    }

    if stdout.is_empty() && !stderr.is_empty() {
        return Ok(stderr);
    }

    Ok(stdout)
}
