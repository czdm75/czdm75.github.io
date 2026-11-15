use anyhow::{Context, Result, bail};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use typst::foundations::{Repr, Str};
use typst::syntax::{FileId, RootedPath, VirtualPath, VirtualRoot};

pub struct Entry {
    pub id: FileId,
    pub generated: Option<String>,
}

pub fn prepare(root: &Path, input: &Path) -> Result<Entry> {
    let input = root.join(input).canonicalize().context("input path")?;
    let relative = input
        .strip_prefix(root)
        .context("input must be inside project root")?;
    if input.is_file() {
        return Ok(Entry {
            id: file_id(relative)?,
            generated: None,
        });
    }
    let files = discover(&input, &input)?;
    if files.is_empty() {
        bail!("no Typst articles found in {}", input.display());
    }
    // The entry exists only in memory. No generated source is written into the repository.
    Ok(Entry {
        id: file_id(Path::new(".ssg-entry.typ"))?,
        generated: Some(generate(root, files)?),
    })
}

fn generate(root: &Path, files: BTreeMap<PathBuf, PathBuf>) -> Result<String> {
    let articles = files
        .into_iter()
        .map(|(route, file)| -> Result<String> {
            let source = format!("/{}", utf8(file.strip_prefix(root)?)?);
            Ok(format!(
                "  (source: {}, output: {}),\n",
                literal(&source),
                literal(utf8(&route)?)
            ))
        })
        .collect::<Result<String>>()?;
    Ok(format!(
        "{}\n#let articles = (\n{articles});\n{}",
        include_str!("render.typ"),
        include_str!("bundle.typ")
    ))
}

fn discover(root: &Path, dir: &Path) -> Result<BTreeMap<PathBuf, PathBuf>> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        let discovered = if kind.is_dir() {
            discover(root, &path)?
        } else if kind.is_file()
            && matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("typ" | "typst")
            )
        {
            let route = path
                .strip_prefix(root)?
                .with_extension("")
                .join("index.html");
            BTreeMap::from([(route, path)])
        } else {
            continue;
        };
        for (route, source) in discovered {
            if files.insert(route.clone(), source).is_some() {
                bail!("duplicate article route: {}", route.display());
            }
        }
    }
    Ok(files)
}
fn utf8(path: &Path) -> Result<&str> {
    path.to_str().context("non-UTF-8 source path")
}
fn file_id(path: &Path) -> Result<FileId> {
    Ok(RootedPath::new(VirtualRoot::Project, VirtualPath::new(utf8(path)?)?).intern())
}
fn literal(text: &str) -> String {
    Str::from(text).repr().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_sorted_articles_and_rejects_route_collisions() -> Result<()> {
        let root = std::env::temp_dir().join(format!("ssg-discovery-{}", std::process::id()));
        fs::create_dir_all(root.join("content/nested"))?;
        fs::write(root.join("content/z.typst"), "= Z")?;
        fs::write(root.join("content/nested/a.typ"), "= A")?;
        fs::write(root.join("content/ignored.txt"), "ignored")?;
        let root = root.canonicalize()?;
        let entry = prepare(&root, Path::new("content"))?;
        let source = entry.generated.unwrap();
        assert!(source.find("nested/a.typ").unwrap() < source.find("z.typst").unwrap());
        assert!(source.contains("nested/a/index.html"));
        assert!(!source.contains("ignored.txt"));
        fs::write(root.join("content/z.typ"), "= Duplicate")?;
        let error = prepare(&root, Path::new("content")).err().unwrap();
        assert!(error.to_string().contains("duplicate article route"));
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
