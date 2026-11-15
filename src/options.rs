use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use typst::foundations::{Bytes, Dict, Value};

#[derive(Debug, Clone, Copy, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum MathFormat {
    Mathml,
    Svg,
}

#[derive(Debug, Clone, Copy, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum CodeRenderer {
    Typst,
    HighlightJs,
}

#[derive(Debug, Parser)]
#[command(version, about = "Compile a Typst bundle and inspect its pages")]
pub struct Options {
    /// Content directory or an explicit Typst bundle entry point.
    #[arg(long, default_value = "content")]
    pub input: PathBuf,
    /// Directory for exported files.
    #[arg(long, default_value = "public")]
    pub output: PathBuf,
    /// Format for equations in the HTML output.
    #[arg(long, value_enum, default_value = "mathml")]
    pub math: MathFormat,
    /// Typst colors, or uncolored HTML for future highlight.js processing.
    #[arg(long, value_enum, default_value = "typst")]
    pub code_renderer: CodeRenderer,
    /// Typst .tmTheme file; for highlight-js, reserved for a later rendering stage.
    #[arg(long)]
    pub code_theme: Option<PathBuf>,
    /// Write ssg-report.json for inspecting the compiled pages.
    #[arg(long)]
    pub report: bool,
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
    fn accepts_named_options_and_rejects_legacy_arguments() {
        assert!(
            Options::try_parse_from([
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
        assert!(Options::try_parse_from(["typst-ssg"]).is_ok());
        for args in [
            vec!["typst-ssg", "content", "public"],
            vec!["typst-ssg", "--code-renderer", "compiled"],
            vec!["typst-ssg", "--code-renderer", "client"],
        ] {
            assert!(Options::try_parse_from(args).is_err());
        }
    }
}
