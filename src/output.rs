use crate::inspect::Page;
use crate::options::{CodeRenderer, MathFormat, Options};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{collections::BTreeMap, fs, path::Path};
use typst_bundle::Bundle;

pub fn export(bundle: &Bundle) -> Result<BTreeMap<String, Vec<u8>>> {
    let exported = typst_bundle::export(bundle, &Default::default())
        .map_err(|errors| anyhow::anyhow!("{errors:#?}"))?;
    Ok(exported
        .into_iter()
        .map(|(path, bytes)| {
            (
                path.get_without_slash().to_string(),
                bytes.as_slice().to_vec(),
            )
        })
        .collect())
}

pub fn with_metadata(
    mut exported: BTreeMap<String, Vec<u8>>,
    pages: &[Page],
) -> Result<BTreeMap<String, Vec<u8>>> {
    for page in pages {
        let path = Path::new(&page.path).with_extension("json");
        let key = path.to_str().context("non-UTF-8 metadata path")?;
        ensure!(
            !exported.contains_key(key),
            "metadata output conflicts with bundle file: {key}"
        );
        let bytes = serde_json::to_vec_pretty(page)
            .with_context(|| format!("serialize metadata for {}", page.path))?;
        exported.insert(key.to_owned(), bytes);
    }
    Ok(exported)
}

pub fn write(files: BTreeMap<String, Vec<u8>>, pages: &[Page], options: &Options) -> Result<()> {
    let output = &options.output;
    fs::create_dir_all(output)?;
    for (path, bytes) in files {
        let dest = output.join(path);
        fs::create_dir_all(dest.parent().unwrap())?;
        fs::write(dest, bytes)?;
    }
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
