use askama::Template;
use axum::extract::{Path, State};
use axum::response::Html;

use crate::app_state::SharedState;
use crate::config::model::{Category, ConfigValueType};
use crate::error::AppError;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    categories: Vec<CategoryInfo>,
    config_count: usize,
    theme_count: usize,
    font_count: usize,
    modified_count: usize,
}

struct CategoryInfo {
    slug: String,
    name: String,
    icon: String,
    count: usize,
    modified: usize,
}

pub async fn index(State(state): State<SharedState>) -> Result<Html<String>, AppError> {
    let unsaved = state.unsaved.read().await;
    let modified_count = unsaved.len();

    let categories: Vec<CategoryInfo> = Category::all()
        .into_iter()
        .map(|cat| {
            let options = state.schema.options_for_category(&cat);
            let count = options.len();
            let modified = options.iter().filter(|o| unsaved.contains(&o.key)).count();
            CategoryInfo {
                slug: cat.slug().to_string(),
                name: cat.display_name().to_string(),
                icon: cat.icon().to_string(),
                count,
                modified,
            }
        })
        .collect();

    let tmpl = IndexTemplate {
        config_count: state.schema.options.len(),
        theme_count: state.themes.len(),
        font_count: state.fonts.len(),
        modified_count,
        categories,
    };

    Ok(Html(tmpl.render().map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Template error: {}", e))
    })?))
}

#[derive(Template)]
#[template(path = "pages/category.html")]
#[allow(dead_code)]
struct CategoryTemplate {
    category_name: String,
    category_slug: String,
    categories: Vec<SidebarCategory>,
    fields: Vec<FieldData>,
}

struct SidebarCategory {
    slug: String,
    name: String,
    icon: String,
    active: bool,
}

struct FieldData {
    key: String,
    default_value: String,
    current_value: String,
    documentation: String,
    value_type: String,
    is_modified: bool,
    enum_options: Vec<String>,
    type_tag: String,
}

pub async fn category(
    State(state): State<SharedState>,
    Path(slug): Path<String>,
) -> Result<Html<String>, AppError> {
    let target_cat = Category::all()
        .into_iter()
        .find(|c| c.slug() == slug)
        .ok_or_else(|| AppError::Config(format!("Unknown category: {}", slug)))?;

    let user_config = state.user_config.read().await;
    let unsaved = state.unsaved.read().await;
    let options = state.schema.options_for_category(&target_cat);

    let fields: Vec<FieldData> = options
        .iter()
        .filter(|o| !matches!(o.value_type, ConfigValueType::Keybind))
        .map(|opt| {
            let current = user_config.get(&opt.key).unwrap_or("").to_string();
            let is_modified = unsaved.contains(&opt.key);
            let display_value = if !current.is_empty() {
                current.clone()
            } else {
                opt.default_value.clone()
            };

            let enum_options = match &opt.value_type {
                ConfigValueType::Enum(vals) => vals.clone(),
                _ => Vec::new(),
            };

            FieldData {
                key: opt.key.clone(),
                default_value: opt.default_value.clone(),
                current_value: display_value,
                documentation: opt.documentation.clone(),
                value_type: opt.value_type.to_string(),
                is_modified,
                enum_options,
                type_tag: format!("{}", opt.value_type),
            }
        })
        .collect();

    let categories: Vec<SidebarCategory> = Category::all()
        .into_iter()
        .map(|cat| SidebarCategory {
            active: cat == target_cat,
            slug: cat.slug().to_string(),
            name: cat.display_name().to_string(),
            icon: cat.icon().to_string(),
        })
        .collect();

    let tmpl = CategoryTemplate {
        category_name: target_cat.display_name().to_string(),
        category_slug: target_cat.slug().to_string(),
        categories,
        fields,
    };

    Ok(Html(tmpl.render().map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Template error: {}", e))
    })?))
}

#[derive(Template)]
#[template(path = "pages/themes.html")]
struct ThemesTemplate {
    categories: Vec<SidebarCategory>,
    themes: Vec<ThemeCardData>,
    current_theme: String,
    total_count: usize,
}

#[allow(dead_code)]
struct ThemeCardData {
    name: String,
    background: String,
    foreground: String,
    is_dark: bool,
    is_active: bool,
    palette_colors: Vec<String>,
}

pub async fn themes_page(State(state): State<SharedState>) -> Result<Html<String>, AppError> {
    let user_config = state.user_config.read().await;
    let current_theme = user_config.get("theme").unwrap_or("").to_string();

    let themes: Vec<ThemeCardData> = state
        .themes
        .iter()
        .map(|t| ThemeCardData {
            name: t.name.clone(),
            background: t.background.clone(),
            foreground: t.foreground.clone(),
            is_dark: t.is_dark,
            is_active: t.name == current_theme,
            palette_colors: t.palette[..8].to_vec(),
        })
        .collect();

    let categories: Vec<SidebarCategory> = Category::all()
        .into_iter()
        .map(|cat| SidebarCategory {
            active: false,
            slug: cat.slug().to_string(),
            name: cat.display_name().to_string(),
            icon: cat.icon().to_string(),
        })
        .collect();

    let total_count = themes.len();

    let tmpl = ThemesTemplate {
        categories,
        themes,
        current_theme,
        total_count,
    };

    Ok(Html(tmpl.render().map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Template error: {}", e))
    })?))
}

#[derive(Template)]
#[template(path = "pages/keybinds.html")]
struct KeybindsTemplate {
    categories: Vec<SidebarCategory>,
    keybinds: Vec<KeybindData>,
    actions: Vec<String>,
}

struct KeybindData {
    trigger: String,
    action: String,
    is_custom: bool,
}

pub async fn keybinds_page(State(state): State<SharedState>) -> Result<Html<String>, AppError> {
    let user_config = state.user_config.read().await;
    let custom_keybinds: Vec<&str> = user_config.get_all("keybind");

    let mut keybinds: Vec<KeybindData> = state
        .default_keybinds
        .iter()
        .map(|kb| KeybindData {
            trigger: kb.trigger.clone(),
            action: kb.action.clone(),
            is_custom: false,
        })
        .collect();

    // Add custom keybinds from user config
    for kb_str in &custom_keybinds {
        if let Some((trigger, action)) = kb_str.split_once('=') {
            keybinds.push(KeybindData {
                trigger: trigger.trim().to_string(),
                action: action.trim().to_string(),
                is_custom: true,
            });
        }
    }

    let categories: Vec<SidebarCategory> = Category::all()
        .into_iter()
        .map(|cat| SidebarCategory {
            active: false,
            slug: cat.slug().to_string(),
            name: cat.display_name().to_string(),
            icon: cat.icon().to_string(),
        })
        .collect();

    let tmpl = KeybindsTemplate {
        categories,
        keybinds,
        actions: state.actions.clone(),
    };

    Ok(Html(tmpl.render().map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Template error: {}", e))
    })?))
}

#[derive(Template)]
#[template(path = "pages/import_export.html")]
struct ImportExportTemplate {
    categories: Vec<SidebarCategory>,
    config_text: String,
}

pub async fn import_export_page(
    State(state): State<SharedState>,
) -> Result<Html<String>, AppError> {
    let user_config = state.user_config.read().await;

    let mut config_text = String::new();
    for (key, value) in user_config.all_set_values() {
        config_text.push_str(&format!("{} = {}\n", key, value));
    }

    let categories: Vec<SidebarCategory> = Category::all()
        .into_iter()
        .map(|cat| SidebarCategory {
            active: false,
            slug: cat.slug().to_string(),
            name: cat.display_name().to_string(),
            icon: cat.icon().to_string(),
        })
        .collect();

    let tmpl = ImportExportTemplate {
        categories,
        config_text,
    };

    Ok(Html(tmpl.render().map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Template error: {}", e))
    })?))
}
