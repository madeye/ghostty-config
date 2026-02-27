use super::model::Category;

/// Assign a category to a config key based on its prefix/name.
pub fn categorize_key(key: &str) -> Category {
    // Exact matches first
    match key {
        "theme" => return Category::Appearance,
        "keybind" => return Category::Keybindings,
        "palette" => return Category::Colors,
        "config-file" => return Category::Advanced,
        "term" => return Category::Terminal,
        "enquiry-response" => return Category::Terminal,
        "title" => return Category::Window,
        "class" | "x11-instance-name" => return Category::GTKLinux,
        "scrollback-limit" => return Category::Scrollback,
        "link" | "link-url" => return Category::Mouse,
        _ => {}
    }

    // Prefix matches
    if key.starts_with("font-") {
        return Category::Fonts;
    }

    if key.starts_with("cursor-") {
        return Category::Cursor;
    }

    if key.starts_with("mouse-") || key.starts_with("click-") {
        return Category::Mouse;
    }

    if key.starts_with("clipboard-") || key.starts_with("copy-on-select") {
        return Category::Clipboard;
    }

    if key.starts_with("shell-")
        || key == "command"
        || key == "wait-after-command"
        || key == "initial-command"
    {
        return Category::Shell;
    }

    if key.starts_with("window-")
        || key.starts_with("resize-")
        || key.starts_with("fullscreen")
        || key == "confirm-close-surface"
    {
        return Category::Window;
    }

    if key.starts_with("background") {
        return Category::Background;
    }

    if key.starts_with("foreground")
        || key.starts_with("selection-")
        || key.contains("color")
        || key == "bold-is-bright"
        || key == "minimum-contrast"
        || key == "palette"
        || key == "invert-selection-fg-bg"
        || key == "bold-color"
        || key == "faint-opacity"
    {
        return Category::Colors;
    }

    if key.starts_with("macos-")
        || key.starts_with("auto-update")
        || key == "quick-terminal-position"
        || key.starts_with("quick-terminal")
    {
        return Category::MacOS;
    }

    if key.starts_with("gtk-") || key.starts_with("adw-") || key.starts_with("linux-") {
        return Category::GTKLinux;
    }

    if key.starts_with("scrollback") || key.starts_with("scroll-") {
        return Category::Scrollback;
    }

    if key.starts_with("input-")
        || key == "vt-kam-allowed"
        || key.starts_with("desktop-notifications")
    {
        return Category::Input;
    }

    if key == "adjust-cell-width"
        || key == "adjust-cell-height"
        || key == "adjust-font-baseline"
        || key == "adjust-underline-position"
        || key == "adjust-underline-thickness"
        || key == "adjust-strikethrough-position"
        || key == "adjust-strikethrough-thickness"
        || key == "adjust-overline-position"
        || key == "adjust-overline-thickness"
        || key == "adjust-cursor-thickness"
        || key == "adjust-cursor-height"
        || key == "adjust-box-thickness"
    {
        return Category::Fonts;
    }

    if key == "unfocused-split-fill" || key == "split-color" || key.starts_with("unfocused-split") {
        return Category::Appearance;
    }

    if key == "osc-color-report-format"
        || key == "abnormal-command-exit-runtime"
        || key == "image-storage-limit"
        || key == "custom-shader"
        || key == "custom-shader-animation"
        || key.starts_with("grapheme-")
        || key.starts_with("freetype-")
        || key == "async-backend"
    {
        return Category::Advanced;
    }

    // Appearance-related
    if key == "minimum-contrast"
        || key == "bold-is-bright"
        || key.starts_with("cell-")
        || key.starts_with("focus-")
        || key.starts_with("unfocused-")
    {
        return Category::Appearance;
    }

    Category::Advanced
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_matches() {
        assert_eq!(categorize_key("theme"), Category::Appearance);
        assert_eq!(categorize_key("keybind"), Category::Keybindings);
        assert_eq!(categorize_key("palette"), Category::Colors);
        assert_eq!(categorize_key("config-file"), Category::Advanced);
        assert_eq!(categorize_key("term"), Category::Terminal);
        assert_eq!(categorize_key("enquiry-response"), Category::Terminal);
        assert_eq!(categorize_key("title"), Category::Window);
        assert_eq!(categorize_key("class"), Category::GTKLinux);
        assert_eq!(categorize_key("scrollback-limit"), Category::Scrollback);
        assert_eq!(categorize_key("link"), Category::Mouse);
    }

    #[test]
    fn test_font_prefix() {
        assert_eq!(categorize_key("font-size"), Category::Fonts);
        assert_eq!(categorize_key("font-family"), Category::Fonts);
        assert_eq!(categorize_key("font-thicken"), Category::Fonts);
        assert_eq!(categorize_key("font-family-bold"), Category::Fonts);
    }

    #[test]
    fn test_cursor_prefix() {
        assert_eq!(categorize_key("cursor-style"), Category::Cursor);
        assert_eq!(categorize_key("cursor-color"), Category::Cursor);
    }

    #[test]
    fn test_mouse_prefix() {
        assert_eq!(categorize_key("mouse-hide-while-typing"), Category::Mouse);
        assert_eq!(categorize_key("click-repeat-timeout"), Category::Mouse);
    }

    #[test]
    fn test_clipboard_prefix() {
        assert_eq!(categorize_key("clipboard-read"), Category::Clipboard);
        assert_eq!(categorize_key("copy-on-select"), Category::Clipboard);
    }

    #[test]
    fn test_shell_keys() {
        assert_eq!(categorize_key("shell-integration"), Category::Shell);
        assert_eq!(categorize_key("command"), Category::Shell);
        assert_eq!(categorize_key("wait-after-command"), Category::Shell);
    }

    #[test]
    fn test_window_prefix() {
        assert_eq!(categorize_key("window-padding-x"), Category::Window);
        assert_eq!(categorize_key("resize-overlay"), Category::Window);
        assert_eq!(categorize_key("confirm-close-surface"), Category::Window);
    }

    #[test]
    fn test_background_prefix() {
        assert_eq!(categorize_key("background"), Category::Background);
        assert_eq!(categorize_key("background-opacity"), Category::Background);
    }

    #[test]
    fn test_color_keys() {
        assert_eq!(categorize_key("foreground"), Category::Colors);
        assert_eq!(categorize_key("selection-foreground"), Category::Colors);
        assert_eq!(categorize_key("bold-color"), Category::Colors);
        assert_eq!(categorize_key("bold-is-bright"), Category::Colors);
        assert_eq!(categorize_key("minimum-contrast"), Category::Colors);
    }

    #[test]
    fn test_macos_prefix() {
        assert_eq!(categorize_key("macos-titlebar-style"), Category::MacOS);
        assert_eq!(categorize_key("auto-update"), Category::MacOS);
    }

    #[test]
    fn test_gtk_linux_prefix() {
        assert_eq!(categorize_key("gtk-titlebar"), Category::GTKLinux);
        assert_eq!(categorize_key("adw-toolbar-style"), Category::GTKLinux);
        assert_eq!(categorize_key("linux-cgroup"), Category::GTKLinux);
    }

    #[test]
    fn test_scrollback_prefix() {
        assert_eq!(categorize_key("scroll-speed"), Category::Scrollback);
    }

    #[test]
    fn test_input_prefix() {
        assert_eq!(categorize_key("input-mode"), Category::Input);
        assert_eq!(categorize_key("vt-kam-allowed"), Category::Input);
    }

    #[test]
    fn test_adjust_keys_are_fonts() {
        assert_eq!(categorize_key("adjust-cell-width"), Category::Fonts);
        assert_eq!(categorize_key("adjust-cell-height"), Category::Fonts);
        assert_eq!(categorize_key("adjust-font-baseline"), Category::Fonts);
        assert_eq!(categorize_key("adjust-underline-position"), Category::Fonts);
    }

    #[test]
    fn test_appearance_keys() {
        assert_eq!(categorize_key("unfocused-split-fill"), Category::Appearance);
        // split-color contains "color" so it matches Colors before Appearance
        assert_eq!(categorize_key("split-color"), Category::Colors);
    }

    #[test]
    fn test_advanced_keys() {
        // osc-color-report-format contains "color" so it matches Colors
        assert_eq!(categorize_key("osc-color-report-format"), Category::Colors);
        assert_eq!(categorize_key("custom-shader"), Category::Advanced);
        assert_eq!(categorize_key("image-storage-limit"), Category::Advanced);
    }

    #[test]
    fn test_unknown_key_defaults_to_advanced() {
        assert_eq!(categorize_key("totally-unknown-key"), Category::Advanced);
    }
}
