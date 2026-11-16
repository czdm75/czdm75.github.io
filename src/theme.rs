use anyhow::{Context, Result, ensure};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ThemeOptions {
    pub name: String,
    pub directory: Option<PathBuf>,
    pub options: toml::Table,
}

impl Default for ThemeOptions {
    fn default() -> Self {
        Self {
            name: "default".into(),
            directory: None,
            options: toml::Table::new(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ThemeView {
    options: toml::Table,
    stylesheets: Vec<String>,
    scripts: Vec<String>,
}

impl ThemeOptions {
    /// Set a dynamic option using a dotted path (also used by CLI compatibility aliases).
    pub fn set_option(&mut self, path: &str, value: toml::Value) -> Result<()> {
        let parts: Vec<_> = path.split('.').collect();
        ensure!(
            parts.iter().all(|part| !part.is_empty()),
            "theme option path cannot have empty segments: {path}"
        );
        let mut table = &mut self.options;
        for key in &parts[..parts.len() - 1] {
            let entry = table
                .entry((*key).to_owned())
                .or_insert_with(|| toml::Value::Table(toml::Table::new()));
            table = entry
                .as_table_mut()
                .with_context(|| format!("theme option parent must be a table: {path}"))?;
        }
        table.insert(parts[parts.len() - 1].to_owned(), value);
        Ok(())
    }
}

pub struct Asset {
    pub path: String,
    pub bytes: Vec<u8>,
}

/// A runtime theme package. Templates and asset bytes are read once per build.
pub struct Theme {
    pub templates: Handlebars<'static>,
    pub assets: Vec<Asset>,
    pub revision: String,
    stylesheets: Vec<String>,
    scripts: Vec<String>,
    options: toml::Table,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct Manifest {
    template_dirs: Vec<PathBuf>,
    #[serde(default)]
    stylesheets: Vec<String>,
    #[serde(default)]
    scripts: Vec<String>,
    #[serde(default)]
    assets: BTreeMap<String, PathBuf>,
    #[serde(default)]
    defaults: toml::Table,
    #[serde(default)]
    option_schema: BTreeMap<String, OptionRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OptionRule {
    #[serde(rename = "type")]
    kind: OptionType,
    #[serde(default, rename = "enum")]
    allowed: Option<Vec<toml::Value>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum OptionType {
    String,
    Boolean,
    Integer,
    Float,
    Array,
    Table,
    Datetime,
}

fn merge_options(base: &mut toml::Table, overrides: &toml::Table) {
    for (key, value) in overrides {
        if let (Some(toml::Value::Table(existing)), toml::Value::Table(incoming)) =
            (base.get_mut(key), value)
        {
            merge_options(existing, incoming);
        } else {
            base.insert(key.clone(), value.clone());
        }
    }
}

fn validate_options(options: &toml::Table, schema: &BTreeMap<String, OptionRule>) -> Result<()> {
    for (path, rule) in schema {
        let parts: Vec<_> = path.split('.').collect();
        ensure!(
            parts.iter().all(|part| !part.is_empty()),
            "invalid option-schema path: {path}"
        );
        let mut table = options;
        let mut found = None;
        for (index, key) in parts.iter().enumerate() {
            let Some(value) = table.get(*key) else {
                break;
            };
            if index == parts.len() - 1 {
                found = Some(value);
            } else {
                table = value.as_table().with_context(|| {
                    format!(
                        "theme.options.{} must be a table",
                        parts[..=index].join(".")
                    )
                })?;
            }
        }
        let Some(value) = found else {
            continue;
        };
        let matches = match rule.kind {
            OptionType::String => value.is_str(),
            OptionType::Boolean => value.is_bool(),
            OptionType::Integer => value.is_integer(),
            OptionType::Float => value.is_float(),
            OptionType::Array => value.is_array(),
            OptionType::Table => value.is_table(),
            OptionType::Datetime => value.is_datetime(),
        };
        ensure!(matches, "invalid type for theme.options.{path}");
        if let Some(allowed) = &rule.allowed {
            ensure!(
                allowed.contains(value),
                "invalid value for theme.options.{path}: {value}; allowed values: {allowed:?}"
            );
        }
    }
    Ok(())
}

impl Theme {
    pub fn view(&self, prefix: &str, revision: &str) -> ThemeView {
        let urls = |paths: &[String]| {
            paths
                .iter()
                .map(|path| format!("{prefix}{}?v={revision}", crate::assemble::escape_url(path)))
                .collect()
        };
        ThemeView {
            options: self.options.clone(),
            stylesheets: urls(&self.stylesheets),
            scripts: urls(&self.scripts),
        }
    }

    pub fn load(options: &ThemeOptions) -> Result<Self> {
        if let Some(directory) = &options.directory {
            return Self::from_directory(directory, &options.options);
        }
        let mut components = Path::new(&options.name).components();
        ensure!(
            matches!(components.next(), Some(Component::Normal(_)))
                && components.next().is_none()
                && !options.name.contains('\\'),
            "theme name must be a single directory name; use theme.directory for a path"
        );
        let mut candidates = vec![std::env::current_dir()?.join("theme").join(&options.name)];
        if let Some(parent) = std::env::current_exe()?.parent() {
            candidates.push(parent.join("theme").join(&options.name));
        }
        // Development fallback: binaries run from another project can use this checkout's themes.
        candidates.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("theme")
                .join(&options.name),
        );
        let directory = candidates
            .iter()
            .find(|path| path.is_dir())
            .with_context(|| {
                format!(
                    "theme '{}' not found; set [theme].directory or --theme-dir",
                    options.name
                )
            })?;
        Self::from_directory(directory, &options.options)
    }

    fn from_directory(directory: &Path, overrides: &toml::Table) -> Result<Self> {
        let path = directory.join("theme.toml");
        let source = fs::read_to_string(&path)
            .with_context(|| format!("read theme manifest {}", path.display()))?;
        let manifest: Manifest = toml::from_str(&source)
            .with_context(|| format!("parse theme manifest {}", path.display()))?;
        let mut options = manifest.defaults;
        merge_options(&mut options, overrides);
        validate_options(&options, &manifest.option_schema)
            .with_context(|| format!("validate theme options for {}", path.display()))?;
        let mut templates = Handlebars::new();
        templates.set_strict_mode(true);
        for relative in &manifest.template_dirs {
            let root = directory.join(relative);
            register_templates(&mut templates, &root, &root)?;
        }
        ensure!(
            templates.get_template("page").is_some(),
            "theme {} must provide page.hbs",
            path.display()
        );
        let mut assets = Vec::new();
        for (output, source) in &manifest.assets {
            ensure!(
                output.starts_with("_theme/")
                    && !output.contains('\\')
                    && Path::new(output)
                        .components()
                        .all(|c| matches!(c, Component::Normal(_))),
                "theme asset output must be a normal relative path under _theme/: {output}"
            );
            let source = directory.join(source);
            assets.push(Asset {
                path: output.clone(),
                bytes: fs::read(&source)
                    .with_context(|| format!("read theme asset {}", source.display()))?,
            });
        }
        for resource in manifest.stylesheets.iter().chain(&manifest.scripts) {
            ensure!(
                manifest.assets.contains_key(resource),
                "theme resource is not declared in assets: {resource}"
            );
        }
        let revision = format!(
            "{:x}",
            assets
                .iter()
                .flat_map(|asset| asset.path.bytes().chain(asset.bytes.iter().copied()))
                .fold(0xcbf29ce484222325u64, |hash, byte| (hash ^ u64::from(byte))
                    .wrapping_mul(0x100000001b3))
        );
        Ok(Self {
            templates,
            assets,
            revision,
            stylesheets: manifest.stylesheets,
            scripts: manifest.scripts,
            options,
        })
    }
}

fn register_templates(
    registry: &mut Handlebars<'static>,
    root: &Path,
    directory: &Path,
) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("read template directory {}", directory.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() {
            register_templates(registry, root, &path)?;
        } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "hbs") {
            let relative = path.strip_prefix(root)?.with_extension("");
            let name = relative
                .to_str()
                .context("non-UTF-8 template path")?
                .replace('\\', "/");
            let source = fs::read_to_string(&path)
                .with_context(|| format!("read template {}", path.display()))?;
            registry
                .register_template_string(&name, source)
                .with_context(|| format!("register template {}", path.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_options_merge_recursively_validate_and_render_as_context() -> Result<()> {
        let root = std::env::temp_dir().join(format!("typst-theme-context-{}", std::process::id()));
        fs::create_dir_all(root.join("templates"))?;
        fs::write(
            root.join("templates/page.hbs"),
            "{{#if (and theme.options.toolbar.enabled (not theme.options.toc.enabled))}}{{theme.options.toolbar.title}}{{/if}} {{theme.options.toc.mode}} {{theme.options.toc.depth}} {{#each theme.options.tags}}[{{this}}]{{/each}}",
        )?;
        fs::write(
            root.join("theme.toml"),
            r#"
template-dirs = ["templates"]
[defaults]
tags = ["one", "two"]
[defaults.toolbar]
enabled = true
title = "Default"
[defaults.toc]
enabled = true
mode = "split"
depth = 3
[option-schema."toc.enabled"]
type = "boolean"
[option-schema."toc.mode"]
type = "string"
enum = ["split", "embedded"]
"#,
        )?;
        let mut options = ThemeOptions {
            directory: Some(root.clone()),
            ..Default::default()
        };
        options.set_option("toc.enabled", toml::Value::Boolean(false))?;
        options.set_option("toolbar.title", toml::Value::String("<Custom>".into()))?;
        options.set_option(
            "tags",
            toml::Value::Array(vec![toml::Value::String("new".into())]),
        )?;
        let theme = Theme::load(&options)?;
        let context = serde_json::json!({"theme": theme.view("", "revision")});
        assert_eq!(
            theme.templates.render("page", &context)?,
            "&lt;Custom&gt; split 3 [new]"
        );
        for (path, value) in [
            ("toc.enabled", toml::Value::String("yes".into())),
            ("toc.mode", toml::Value::String("unknown".into())),
            ("toc", toml::Value::Boolean(false)),
        ] {
            let mut invalid = options.clone();
            invalid.set_option(path, value)?;
            let error = Theme::load(&invalid)
                .err()
                .expect("schema must reject invalid options");
            assert!(format!("{error:#}").contains("theme.options.toc"));
        }
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn runtime_loading_supports_overrides_binary_assets_and_live_edits() -> Result<()> {
        let root = std::env::temp_dir().join(format!("typst-runtime-theme-{}", std::process::id()));
        fs::create_dir_all(root.join("base/components"))?;
        fs::create_dir_all(root.join("overrides"))?;
        fs::write(
            root.join("base/page.hbs"),
            "{{> components/label}} {{> panel}} {{title}}",
        )?;
        fs::write(root.join("base/components/label.hbs"), "Nested")?;
        fs::write(root.join("base/panel.hbs"), "Base")?;
        fs::write(root.join("overrides/panel.hbs"), "Version 1")?;
        fs::write(root.join("a.css"), "/* first */")?;
        fs::write(root.join("b.css"), "/* second */")?;
        fs::write(root.join("image.bin"), [0, 255, 128])?;
        fs::write(
            root.join("theme.toml"),
            r#"
template-dirs = ["base", "overrides"]
stylesheets = ["_theme/custom/b.css", "_theme/custom/a sheet.css"]
[assets]
"_theme/custom/a sheet.css" = "a.css"
"_theme/custom/b.css" = "b.css"
"_theme/custom/image.bin" = "image.bin"
"#,
        )?;
        let options = ThemeOptions {
            name: "custom".into(),
            directory: Some(root.clone()),
            ..Default::default()
        };
        let first = Theme::load(&options)?;
        assert_eq!(
            first
                .templates
                .render("page", &serde_json::json!({"title": "Title"}))?,
            "Nested Version 1 Title"
        );
        assert_eq!(
            first
                .assets
                .iter()
                .find(|asset| asset.path.ends_with("image.bin"))
                .unwrap()
                .bytes,
            [0, 255, 128]
        );
        let view = first.view("../../", &first.revision);
        assert!(view.stylesheets[0].starts_with("../../_theme/custom/b.css?v="));
        assert!(view.stylesheets[1].starts_with("../../_theme/custom/a%20sheet.css?v="));
        fs::write(root.join("overrides/panel.hbs"), "Version 2")?;
        fs::write(root.join("a.css"), "/* changed */")?;
        let second = Theme::load(&options)?;
        assert_eq!(
            second
                .templates
                .render("page", &serde_json::json!({"title": "Title"}))?,
            "Nested Version 2 Title"
        );
        assert_ne!(first.revision, second.revision);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn rejects_invalid_packages_and_unsafe_outputs_without_fallback() -> Result<()> {
        let root = std::env::temp_dir().join(format!("typst-invalid-theme-{}", std::process::id()));
        fs::create_dir_all(root.join("templates"))?;
        let options = ThemeOptions {
            directory: Some(root.clone()),
            ..Default::default()
        };
        let error = Theme::load(&options)
            .err()
            .expect("missing manifest must fail");
        assert!(error.to_string().contains("theme.toml"));
        fs::write(root.join("theme.toml"), "template-dirs = ['templates']")?;
        assert!(
            Theme::load(&options)
                .err()
                .unwrap()
                .to_string()
                .contains("page.hbs")
        );
        fs::write(root.join("templates/page.hbs"), "{{title}}")?;
        fs::write(
            root.join("theme.toml"),
            "template-dirs = ['templates']\nstylesheets = ['_theme/missing.css']",
        )?;
        assert!(
            Theme::load(&options)
                .err()
                .unwrap()
                .to_string()
                .contains("not declared")
        );
        fs::write(root.join("file.bin"), [0])?;
        for output in [
            "../escape.bin",
            "/absolute.bin",
            "_theme/../escape.bin",
            "index.html",
        ] {
            fs::write(
                root.join("theme.toml"),
                format!("template-dirs = ['templates']\n[assets]\n'{output}' = 'file.bin'"),
            )?;
            assert!(
                Theme::load(&options)
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("relative path")
            );
        }
        assert!(
            Theme::load(&ThemeOptions {
                name: "../escape".into(),
                ..Default::default()
            })
            .is_err()
        );
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
