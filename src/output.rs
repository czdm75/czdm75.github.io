use crate::inspect::Page;
use crate::options::{CodeRenderer, MathFormat, Options};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{fs, path::Path};
use typst_bundle::Bundle;

pub fn write(bundle: &Bundle, pages: &[Page], options: &Options) -> Result<()> {
    let exported = typst_bundle::export(bundle, &Default::default())
        .map_err(|errors| anyhow::anyhow!("{errors:#?}"))?;
    let metadata = pages
        .iter()
        .map(|page| {
            let path = Path::new(&page.path).with_extension("json");
            ensure!(
                !exported
                    .keys()
                    .any(|file| Path::new(file.get_without_slash()) == path),
                "metadata output conflicts with bundle file: {}",
                path.display()
            );
            let bytes = serde_json::to_vec_pretty(page)
                .with_context(|| format!("serialize metadata for {}", page.path))?;
            Ok((path, bytes))
        })
        .collect::<Result<Vec<_>>>()?;
    let output = &options.output;
    fs::create_dir_all(output)?;
    for (path, bytes) in exported {
        let dest = output.join(path.get_without_slash());
        fs::create_dir_all(dest.parent().unwrap())?;
        fs::write(dest, bytes.as_slice())?;
    }
    metadata
        .into_iter()
        .try_for_each(|(path, bytes)| fs::write(output.join(path), bytes))?;
    if options.report {
        let report = Report {
            math: options.math,
            code_renderer: options.code_renderer,
            code_theme: options.code_theme.as_deref(),
            pages,
        };
        fs::write(
            output.join("ssg-report.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
    }
    Ok(())
}

#[derive(Serialize)]
struct Report<'a> {
    math: MathFormat,
    code_renderer: CodeRenderer,
    code_theme: Option<&'a Path>,
    pages: &'a [Page],
}
