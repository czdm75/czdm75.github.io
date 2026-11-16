use crate::theme::ThemeOptions;
use anyhow::{Context, Result};
use clap::{Args, Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use typst::foundations::{Bytes, Dict, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum MathFormat {
    Mathml,
    Svg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum CodeRenderer {
    Typst,
    HighlightJs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    Site,
    Raw,
}

/// Fully resolved engine settings, with a separate theme-owned section.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Options {
    pub mode: Mode,
    pub input: PathBuf,
    pub output: PathBuf,
    pub math: MathFormat,
    pub code_renderer: CodeRenderer,
    pub code_theme: Option<PathBuf>,
    pub report: bool,
    pub theme: ThemeOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            mode: Mode::Site,
            input: "content".into(),
            output: "public".into(),
            math: MathFormat::Mathml,
            code_renderer: CodeRenderer::Typst,
            code_theme: None,
            report: false,
            theme: ThemeOptions::default(),
        }
    }
}

#[derive(Debug, Default, Args)]
struct ThemeOverrides {
    /// Override [theme].name.
    #[arg(long)]
    theme: Option<String>,
    /// Directory containing a runtime theme.toml manifest.
    #[arg(long)]
    theme_dir: Option<PathBuf>,
    /// Compatibility alias for theme.options.site-tree.
    #[arg(long, action = clap::ArgAction::Set)]
    site_tree: Option<bool>,
    /// Compatibility alias for theme.options.toc.enabled.
    #[arg(long, action = clap::ArgAction::Set)]
    toc: Option<bool>,
    /// Compatibility alias for theme.options.toc.mode.
    #[arg(long)]
    toc_mode: Option<String>,
    /// Set a theme option using a dotted KEY and a TOML VALUE, e.g. toc.enabled=false.
    #[arg(long = "theme-option", value_name = "KEY=VALUE")]
    options: Vec<String>,
}

#[derive(Debug, Parser)]
#[command(version, about = "Compile a Typst bundle and inspect its pages")]
pub struct Cli {
    /// TOML configuration; otherwise load typst-ssg.toml in the current directory if present.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Assemble site pages, or export raw HTML and metadata.
    #[arg(long, value_enum)]
    mode: Option<Mode>,
    #[command(flatten)]
    theme: ThemeOverrides,
    /// Content directory or an explicit Typst bundle entry point.
    #[arg(long)]
    input: Option<PathBuf>,
    /// Directory for exported files.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Format for equations in the HTML output.
    #[arg(long, value_enum)]
    math: Option<MathFormat>,
    /// Typst colors, or uncolored HTML for future highlight.js processing.
    #[arg(long, value_enum)]
    code_renderer: Option<CodeRenderer>,
    /// Typst .tmTheme file; reserved in highlight-js mode.
    #[arg(long)]
    code_theme: Option<PathBuf>,
    /// Write ssg-report.json; use --report=false to override a configured true.
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true", action = clap::ArgAction::Set)]
    report: Option<bool>,
}

impl Cli {
    pub fn resolve(self, root: &Path) -> Result<Options> {
        let path = root.join(
            self.config
                .as_deref()
                .unwrap_or(Path::new("typst-ssg.toml")),
        );
        let mut options = match fs::read_to_string(&path) {
            Ok(source) => {
                let mut config: Options = toml::from_str(&source)
                    .with_context(|| format!("parse config {}", path.display()))?;
                let base = path.parent().context("config has no parent directory")?;
                config.input = base.join(config.input);
                config.output = base.join(config.output);
                config.code_theme = config.code_theme.map(|value| base.join(value));
                config.theme.directory = config.theme.directory.map(|value| base.join(value));
                config
            }
            Err(error) if self.config.is_none() && error.kind() == std::io::ErrorKind::NotFound => {
                Options::default()
            }
            Err(error) => {
                return Err(error).with_context(|| format!("read config {}", path.display()));
            }
        };
        // CLI fields have no defaults: only explicit arguments override the file.
        macro_rules! override_field {
            ($($field:ident),* $(,)?) => { $(if let Some(value) = self.$field { options.$field = value; })* };
        }
        override_field!(mode, input, output, math, code_renderer, report);
        if let Some(value) = self.code_theme {
            options.code_theme = Some(value);
        }
        if let Some(value) = self.theme.theme {
            options.theme.name = value;
        }
        if let Some(value) = self.theme.theme_dir {
            options.theme.directory = Some(value);
        }
        if let Some(value) = self.theme.site_tree {
            options
                .theme
                .set_option("site-tree", toml::Value::Boolean(value))?;
        }
        if let Some(value) = self.theme.toc {
            options
                .theme
                .set_option("toc.enabled", toml::Value::Boolean(value))?;
        }
        if let Some(value) = self.theme.toc_mode {
            options
                .theme
                .set_option("toc.mode", toml::Value::String(value))?;
        }
        for entry in self.theme.options {
            let (key, value) = entry
                .split_once('=')
                .context("--theme-option requires KEY=VALUE")?;
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct ParsedValue {
                value: toml::Value,
            }
            let parsed: ParsedValue = toml::from_str(&format!("value = {value}"))
                .with_context(|| format!("parse --theme-option {key}"))?;
            options.theme.set_option(key, parsed.value)?;
        }
        Ok(options)
    }
}

impl Options {
    pub fn inputs(&self, root: &Path) -> Result<Dict> {
        let renderer = match self.code_renderer {
            CodeRenderer::Typst => "typst",
            CodeRenderer::HighlightJs => "highlight-js",
        };
        let math = match self.math {
            MathFormat::Mathml => "mathml",
            MathFormat::Svg => "svg",
        };
        let theme = match (&self.code_theme, self.code_renderer) {
            (Some(path), CodeRenderer::Typst) => Value::Bytes(Bytes::new(
                fs::read(root.join(path))
                    .with_context(|| format!("read code theme {}", path.display()))?,
            )),
            _ => Value::Auto,
        };
        Ok([
            ("code-renderer".into(), Value::Str(renderer.into())),
            ("math".into(), Value::Str(math.into())),
            ("code-theme".into(), theme),
        ]
        .into_iter()
        .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_cli_options_preserve_types_and_override_compatibility_aliases() -> Result<()> {
        let options = Cli::try_parse_from([
            "typst-ssg",
            "--site-tree=false",
            "--theme-option=site-tree=true",
            "--theme-option=toolbar.enabled=false",
            "--theme-option=toolbar.count=3",
            "--theme-option=toolbar.title=\"Custom\"",
            "--theme-option=tags=[\"a\",\"b\"]",
            "--theme-option=toc.mode=\"custom-mode\"",
        ])?
        .resolve(&std::env::temp_dir())?;
        let values = &options.theme.options;
        assert_eq!(values["site-tree"].as_bool(), Some(true));
        assert_eq!(values["toolbar"]["enabled"].as_bool(), Some(false));
        assert_eq!(values["toolbar"]["count"].as_integer(), Some(3));
        assert_eq!(values["toolbar"]["title"].as_str(), Some("Custom"));
        assert_eq!(values["tags"].as_array().unwrap().len(), 2);
        assert_eq!(values["toc"]["mode"].as_str(), Some("custom-mode"));
        for argument in [
            "--theme-option=missing-value",
            "--theme-option=a..b=true",
            "--theme-option=x=bare-string",
        ] {
            assert!(
                Cli::try_parse_from(["typst-ssg", argument])?
                    .resolve(&std::env::temp_dir())
                    .is_err()
            );
        }
        assert!(
            Cli::try_parse_from([
                "typst-ssg",
                "--theme-option=x=false",
                "--theme-option=x.child=true"
            ])?
            .resolve(&std::env::temp_dir())
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn loads_default_file_and_explicit_cli_overrides() -> Result<()> {
        let root = std::env::temp_dir().join(format!("typst-ssg-config-{}", std::process::id()));
        fs::create_dir_all(root.join("config"))?;
        let source = r#"
mode = "raw"
input = "articles"
output = "build"
math = "svg"
code-renderer = "highlight-js"
code-theme = "syntax.tmTheme"
report = true
[theme]
name = "mdbook"
directory = "skin"
[theme.options]
site-tree = false
[theme.options.toc]
enabled = false
mode = "embedded"
"#;
        fs::write(root.join("typst-ssg.toml"), source)?;
        fs::write(root.join("config/site.toml"), source)?;
        let options = Cli::try_parse_from(["typst-ssg"])?.resolve(&root)?;
        assert_eq!(options.mode, Mode::Raw);
        assert_eq!(options.input, root.join("articles"));
        assert_eq!(options.output, root.join("build"));
        assert_eq!(options.math, MathFormat::Svg);
        assert_eq!(options.code_renderer, CodeRenderer::HighlightJs);
        assert_eq!(options.code_theme, Some(root.join("syntax.tmTheme")));
        assert!(options.report);
        assert_eq!(options.theme.name, "mdbook");
        assert_eq!(options.theme.directory, Some(root.join("skin")));
        assert_eq!(options.theme.options["site-tree"].as_bool(), Some(false));
        assert_eq!(
            options.theme.options["toc"]["enabled"].as_bool(),
            Some(false)
        );
        assert_eq!(
            options.theme.options["toc"]["mode"].as_str(),
            Some("embedded")
        );

        let options = Cli::try_parse_from([
            "typst-ssg",
            "--config=config/site.toml",
            "--mode=site",
            "--output=cli-output",
            "--math=mathml",
            "--code-renderer=typst",
            "--theme=default",
            "--theme-dir=local-theme",
            "--site-tree=true",
            "--toc=true",
            "--toc-mode=split",
            "--report=false",
        ])?
        .resolve(&root)?;
        assert_eq!(options.mode, Mode::Site);
        assert_eq!(options.input, root.join("config/articles"));
        assert_eq!(options.output, PathBuf::from("cli-output"));
        assert_eq!(options.math, MathFormat::Mathml);
        assert_eq!(options.code_renderer, CodeRenderer::Typst);
        assert_eq!(options.code_theme, Some(root.join("config/syntax.tmTheme")));
        assert!(!options.report);
        assert_eq!(options.theme.name, "default");
        assert_eq!(options.theme.directory, Some(PathBuf::from("local-theme")));
        assert_eq!(options.theme.options["site-tree"].as_bool(), Some(true));
        assert_eq!(
            options.theme.options["toc"]["enabled"].as_bool(),
            Some(true)
        );
        assert_eq!(options.theme.options["toc"]["mode"].as_str(), Some("split"));
        // Bare --report keeps its existing meaning even with the new optional boolean.
        assert!(
            Cli::try_parse_from(["typst-ssg", "--report"])?
                .resolve(&root)?
                .report
        );
        let options =
            Cli::try_parse_from(["typst-ssg", "--input=cli-input", "--code-theme=cli.tmTheme"])?
                .resolve(&root)?;
        assert_eq!(options.input, PathBuf::from("cli-input"));
        assert_eq!(options.code_theme, Some(PathBuf::from("cli.tmTheme")));
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn rejects_invalid_config_and_requires_explicit_files() -> Result<()> {
        for source in [
            "toc = true",
            "[theme]\ntco = true",
            "[theme]\nname = 42",
            "[theme]\ntoc-mode = 'unknown'",
            "math = 'unknown'",
            "report = 'yes'",
            "[theme",
        ] {
            assert!(toml::from_str::<Options>(source).is_err(), "{source}");
        }
        let root =
            std::env::temp_dir().join(format!("typst-ssg-config-errors-{}", std::process::id()));
        fs::create_dir_all(&root)?;
        let defaults = Cli::try_parse_from(["typst-ssg"])?.resolve(&root)?;
        assert_eq!(defaults.input, PathBuf::from("content"));
        let error = Cli::try_parse_from(["typst-ssg", "--config=missing.toml"])?
            .resolve(&root)
            .unwrap_err();
        assert!(error.to_string().contains("missing.toml"));
        fs::write(root.join("typst-ssg.toml"), "invalid = true")?;
        let error = Cli::try_parse_from(["typst-ssg"])?
            .resolve(&root)
            .unwrap_err();
        assert!(error.to_string().contains("parse config"));
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn accepts_project_config_and_defaults_partial_sections() -> Result<()> {
        // The project file is user-editable; validate its schema rather than fixed values.
        let _: Options = toml::from_str(include_str!("../typst-ssg.toml"))?;
        let defaults: Options = toml::from_str("")?;
        assert_eq!(defaults.mode, Mode::Site);
        assert_eq!(defaults.theme.name, "default");
        assert!(defaults.theme.options.is_empty());
        let options: Options =
            toml::from_str("[theme]\nname = 'mdbook'\n[theme.options.toc]\nenabled = false")?;
        assert_eq!(options.input, PathBuf::from("content"));
        assert_eq!(options.theme.name, "mdbook");
        assert_eq!(
            options.theme.options["toc"]["enabled"].as_bool(),
            Some(false)
        );
        assert!(
            !options.theme.options["toc"]
                .as_table()
                .unwrap()
                .contains_key("mode")
        );
        Ok(())
    }

    #[test]
    fn selects_theme_and_independent_navigation_components() {
        let defaults = Cli::try_parse_from(["typst-ssg"])
            .unwrap()
            .resolve(&std::env::temp_dir())
            .unwrap();
        assert_eq!(defaults.theme.name, "default");
        assert!(defaults.theme.options.is_empty());
        let options = Cli::try_parse_from([
            "typst-ssg",
            "--theme",
            "mdbook",
            "--site-tree=false",
            "--toc=true",
            "--toc-mode=embedded",
        ])
        .unwrap()
        .resolve(&std::env::temp_dir())
        .unwrap();
        assert_eq!(options.theme.name, "mdbook");
        assert_eq!(options.theme.options["site-tree"].as_bool(), Some(false));
        assert_eq!(
            options.theme.options["toc"]["enabled"].as_bool(),
            Some(true)
        );
        assert_eq!(
            options.theme.options["toc"]["mode"].as_str(),
            Some("embedded")
        );
        assert!(Cli::try_parse_from(["typst-ssg", "--toc-mode=custom-mode"]).is_ok());
        assert!(Cli::try_parse_from(["typst-ssg", "--theme", "custom"]).is_ok());
        assert!(Cli::try_parse_from(["typst-ssg", "--toc=maybe"]).is_err());
    }

    #[test]
    fn accepts_named_options_and_rejects_legacy_arguments() {
        assert!(
            Cli::try_parse_from([
                "typst-ssg",
                "--input",
                "content",
                "--output",
                "public",
                "--math",
                "svg",
                "--code-renderer",
                "highlight-js",
                "--code-theme",
                "theme.css",
                "--report",
            ])
            .is_ok()
        );
        assert!(Cli::try_parse_from(["typst-ssg"]).is_ok());
        for args in [
            vec!["typst-ssg", "content", "public"],
            vec!["typst-ssg", "--code-renderer", "compiled"],
            vec!["typst-ssg", "--code-renderer", "client"],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
    }
}
