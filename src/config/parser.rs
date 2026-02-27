use super::categorize::categorize_key;
use super::model::{ConfigOption, ConfigSchema};
use super::type_inference::{infer_type, is_repeatable};
use crate::error::AppError;

/// Parse the output of `ghostty +show-config --default --docs` into a ConfigSchema.
///
/// The format is:
/// ```text
/// # Documentation line 1
/// # Documentation line 2
/// key = value
/// ```
///
/// Blank lines separate blocks. Documentation lines start with `# `.
/// The key = value line follows its documentation block.
pub fn parse_show_config(output: &str) -> Result<ConfigSchema, AppError> {
    let mut options = Vec::new();
    let mut doc_lines: Vec<String> = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();

    for line in output.lines() {
        if let Some(stripped) = line.strip_prefix('#') {
            // Strip the "# " prefix for documentation
            let doc = stripped.strip_prefix(' ').unwrap_or(stripped);
            doc_lines.push(doc.to_string());
        } else if line.trim().is_empty() {
            // Blank line — separator. If we have docs but no key yet, keep accumulating.
            if doc_lines.is_empty() {
                continue;
            }
            // Check if the next block is a continuation. We'll just add the blank line to docs.
            doc_lines.push(String::new());
        } else if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();

            // Build documentation from accumulated lines, removing trailing blanks
            while doc_lines.last().is_some_and(|l| l.is_empty()) {
                doc_lines.pop();
            }
            let documentation = doc_lines.join("\n");

            // Skip duplicate keys (e.g., font-family appears once with docs, then repeated without)
            if !seen_keys.contains(&key) || !documentation.is_empty() {
                if seen_keys.contains(&key) {
                    // Replace existing option with the one that has docs
                    if !documentation.is_empty() {
                        options.retain(|o: &ConfigOption| o.key != key);
                    }
                }

                let value_type = infer_type(&key, &value, &documentation);
                let category = categorize_key(&key);
                let repeatable = is_repeatable(&key);

                options.push(ConfigOption {
                    key: key.clone(),
                    default_value: value,
                    documentation,
                    value_type,
                    category,
                    is_repeatable: repeatable,
                });
                seen_keys.insert(key);
            }

            doc_lines.clear();
        }
    }

    Ok(ConfigSchema { options })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::ConfigValueType;

    #[test]
    fn test_parse_basic() {
        let input = r#"# The font size.
font-size = 13

# Enable bold.
font-thicken = false
"#;
        let schema = parse_show_config(input).unwrap();
        assert_eq!(schema.options.len(), 2);
        assert_eq!(schema.options[0].key, "font-size");
        assert_eq!(schema.options[0].default_value, "13");
        assert_eq!(schema.options[1].key, "font-thicken");
        assert_eq!(schema.options[1].default_value, "false");
    }

    #[test]
    fn test_parse_empty_default() {
        let input = "# The font family.\nfont-family = \n";
        let schema = parse_show_config(input).unwrap();
        assert_eq!(schema.options[0].key, "font-family");
        assert_eq!(schema.options[0].default_value, "");
    }

    #[test]
    fn test_parse_multiline_docs() {
        let input = r#"# Line one.
#
# Line two with detail.
#
# Line three.
some-key = value
"#;
        let schema = parse_show_config(input).unwrap();
        assert_eq!(schema.options.len(), 1);
        assert!(schema.options[0].documentation.contains("Line one."));
        assert!(schema.options[0].documentation.contains("Line three."));
    }

    #[test]
    fn test_parse_no_docs() {
        let input = "bare-key = 42\n";
        let schema = parse_show_config(input).unwrap();
        assert_eq!(schema.options[0].key, "bare-key");
        assert_eq!(schema.options[0].documentation, "");
    }

    #[test]
    fn test_parse_boolean_type_inferred() {
        let input = "# Doc.\nfont-thicken = false\n";
        let schema = parse_show_config(input).unwrap();
        assert!(matches!(
            schema.options[0].value_type,
            ConfigValueType::Boolean
        ));
    }

    #[test]
    fn test_parse_keybind_type() {
        let input = "# Doc.\nkeybind = \n";
        let schema = parse_show_config(input).unwrap();
        assert!(matches!(
            schema.options[0].value_type,
            ConfigValueType::Keybind
        ));
        assert!(schema.options[0].is_repeatable);
    }

    #[test]
    fn test_parse_palette_type() {
        let input = "# Doc.\npalette = \n";
        let schema = parse_show_config(input).unwrap();
        assert!(matches!(
            schema.options[0].value_type,
            ConfigValueType::Palette
        ));
        assert!(schema.options[0].is_repeatable);
    }

    #[test]
    fn test_schema_find_option() {
        let input = "# A.\nfoo = 1\n# B.\nbar = 2\n";
        let schema = parse_show_config(input).unwrap();
        assert!(schema.find_option("foo").is_some());
        assert!(schema.find_option("bar").is_some());
        assert!(schema.find_option("baz").is_none());
    }

    #[test]
    fn test_schema_options_for_category() {
        let input =
            "# Doc.\nfont-size = 13\n# Doc.\nfont-thicken = false\n# Doc.\ncursor-style = block\n";
        let schema = parse_show_config(input).unwrap();
        let font_opts = schema.options_for_category(&crate::config::model::Category::Fonts);
        assert!(font_opts.len() >= 2);
        assert!(font_opts.iter().all(|o| o.key.starts_with("font-")));
    }

    #[test]
    fn test_parse_many_options() {
        // Simulate realistic output with multiple entries
        let input = r#"# The font families.
font-family =

# Bold font.
font-family-bold =

# Font size.
font-size = 13

# Background color.
background = #1e1e2e

# Theme name.
theme =

# A boolean.
font-thicken = false

# A keybind.
keybind =
"#;
        let schema = parse_show_config(input).unwrap();
        assert_eq!(schema.options.len(), 7);
    }
}
