use std::path::PathBuf;

use super::discovery::run_ghostty;
use crate::config::model::FontFamily;
use crate::error::AppError;

/// Parse the output of `ghostty +list-fonts`.
///
/// Format:
/// ```text
/// FamilyName
///   FamilyName StyleName
///   FamilyName StyleName
///
/// FamilyName2
///   ...
/// ```
pub fn load_fonts(ghostty_path: &PathBuf) -> Result<Vec<FontFamily>, AppError> {
    let output = run_ghostty(ghostty_path, &["+list-fonts"])?;
    Ok(parse_font_list(&output))
}

fn parse_font_list(output: &str) -> Vec<FontFamily> {
    let mut fonts = Vec::new();
    let mut current_family: Option<String> = None;
    let mut current_styles: Vec<String> = Vec::new();

    for line in output.lines() {
        if line.is_empty() {
            // End of a font family block
            if let Some(name) = current_family.take() {
                fonts.push(FontFamily {
                    name,
                    styles: std::mem::take(&mut current_styles),
                });
            }
            continue;
        }

        if line.starts_with("  ") || line.starts_with('\t') {
            // Style line
            current_styles.push(line.trim().to_string());
        } else {
            // Family name line
            if let Some(name) = current_family.take() {
                fonts.push(FontFamily {
                    name,
                    styles: std::mem::take(&mut current_styles),
                });
            }
            current_family = Some(line.trim().to_string());
        }
    }

    // Don't forget the last family
    if let Some(name) = current_family {
        fonts.push(FontFamily {
            name,
            styles: current_styles,
        });
    }

    fonts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    fonts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_font_list() {
        let input = "Menlo\n  Menlo Bold\n  Menlo Regular\n\nMonaco\n  Monaco\n";
        let fonts = parse_font_list(input);
        assert_eq!(fonts.len(), 2);
        assert_eq!(fonts[0].name, "Menlo");
        assert_eq!(fonts[0].styles.len(), 2);
        assert_eq!(fonts[1].name, "Monaco");
    }

    #[test]
    fn test_parse_font_list_empty() {
        let fonts = parse_font_list("");
        assert!(fonts.is_empty());
    }

    #[test]
    fn test_parse_font_list_single_family() {
        let input = "JetBrains Mono\n  JetBrains Mono Regular\n  JetBrains Mono Bold\n  JetBrains Mono Italic\n";
        let fonts = parse_font_list(input);
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].name, "JetBrains Mono");
        assert_eq!(fonts[0].styles.len(), 3);
    }

    #[test]
    fn test_parse_font_list_sorted() {
        let input = "Zapfino\n  Zapfino Regular\n\nArial\n  Arial Regular\n\nMenlo\n  Menlo Regular\n";
        let fonts = parse_font_list(input);
        assert_eq!(fonts.len(), 3);
        // Should be sorted alphabetically (case-insensitive)
        assert_eq!(fonts[0].name, "Arial");
        assert_eq!(fonts[1].name, "Menlo");
        assert_eq!(fonts[2].name, "Zapfino");
    }

    #[test]
    fn test_parse_font_list_no_trailing_newline() {
        let input = "Menlo\n  Menlo Bold\n  Menlo Regular";
        let fonts = parse_font_list(input);
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].name, "Menlo");
        assert_eq!(fonts[0].styles.len(), 2);
    }

    #[test]
    fn test_parse_font_list_family_no_styles() {
        // Family with no indented style lines, followed by blank
        let input = "SomeFont\n\nAnotherFont\n  AnotherFont Regular\n";
        let fonts = parse_font_list(input);
        assert_eq!(fonts.len(), 2);
        assert_eq!(fonts[1].name, "SomeFont");
        assert!(fonts[1].styles.is_empty());
    }
}
