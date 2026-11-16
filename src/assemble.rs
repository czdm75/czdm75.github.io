use crate::inspect::Page;
use anyhow::{Context, Result, ensure};
use handlebars::Handlebars;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

const CSS: &str = include_str!("../theme/default/main.css");
const JS: &str = include_str!("../theme/default/main.js");

fn templates() -> Result<Handlebars<'static>> {
    let mut registry = Handlebars::new();
    registry.set_strict_mode(true);
    for (name, source) in [
        ("page", include_str!("../theme/default/templates/page.hbs")),
        (
            "article",
            include_str!("../theme/default/templates/article.hbs"),
        ),
        (
            "index",
            include_str!("../theme/default/templates/index.hbs"),
        ),
        ("tree", include_str!("../theme/default/templates/tree.hbs")),
        ("toc", include_str!("../theme/default/templates/toc.hbs")),
        (
            "index-tree",
            include_str!("../theme/default/templates/index-tree.hbs"),
        ),
    ] {
        registry
            .register_template_string(name, source)
            .with_context(|| format!("register theme template {name}"))?;
    }
    Ok(registry)
}

#[derive(Serialize)]
struct PageView<'a> {
    title: &'a str,
    lang: &'a str,
    prefix: &'a str,
    revision: &'a str,
    // Only compiler-produced HTML bypasses Handlebars escaping.
    head_html: &'a str,
    has_title: bool,
    body_html: String,
    is_index: bool,
    show_title: bool,
    article_count: usize,
    topic_count: usize,
    tree: TreeView,
    toc: Vec<TocItem>,
}

pub fn assemble(
    mut files: BTreeMap<String, Vec<u8>>,
    pages: &[Page],
) -> Result<BTreeMap<String, Vec<u8>>> {
    let registry = templates()?;
    let tree = article_tree(pages);
    let revision = format!(
        "{:x}",
        CSS.bytes()
            .chain(JS.bytes())
            .fold(0xcbf29ce484222325u64, |hash, byte| (hash ^ u64::from(byte))
                .wrapping_mul(0x100000001b3))
    );
    for page in pages {
        let html = std::str::from_utf8(files.get(&page.path).context("missing page HTML")?)?;
        let prefix = "../".repeat(page.path.matches('/').count());
        let view = article(html, page, &tree, &prefix, &revision)?;
        let rendered = registry
            .render("page", &view)
            .with_context(|| format!("render {}", page.path))?;
        files.insert(page.path.clone(), rendered.into_bytes());
    }
    for (path, bytes) in [
        ("_theme/default/main.css", CSS.as_bytes()),
        ("_theme/default/main.js", JS.as_bytes()),
    ] {
        ensure!(
            !files.contains_key(path),
            "theme output conflicts with bundle file: {path}"
        );
        files.insert(path.into(), bytes.to_vec());
    }
    if !files.contains_key("index.html") {
        let view = PageView {
            title: "文章与笔记",
            lang: "zh",
            prefix: "",
            revision: &revision,
            head_html: "",
            has_title: false,
            body_html: String::new(),
            is_index: true,
            show_title: false,
            article_count: pages.len(),
            topic_count: tree
                .entries
                .iter()
                .filter(|entry| matches!(entry, TreeEntry::Folder(..)))
                .count(),
            tree: tree.view("", "", ""),
            toc: vec![],
        };
        files.insert(
            "index.html".into(),
            registry
                .render("page", &view)
                .context("render site index")?
                .into_bytes(),
        );
    }
    Ok(files)
}

fn article<'a>(
    html: &'a str,
    page: &'a Page,
    tree: &ArticleTree<'_>,
    prefix: &'a str,
    revision: &'a str,
) -> Result<PageView<'a>> {
    let head_start = html.find("<head>").context("missing HTML head")? + 6;
    let head_end = html.find("</head>").context("missing HTML head end")?;
    let body_tag = html.find("<body").context("missing HTML body")?;
    let body_start = html[body_tag..].find('>').context("missing body start")? + body_tag + 1;
    let body_end = html.rfind("</body>").context("missing HTML body end")?;
    let lang = html
        .find("<html")
        .and_then(|i| html[i..].split('>').next())
        .and_then(|t| attribute(t, "lang"))
        .unwrap_or("zh");
    let (body_html, targets) = heading_targets(&html[body_start..body_end], page)?;
    let mut cursor = 0;
    let toc = toc_items(page, &targets, &mut cursor, 0);
    Ok(PageView {
        title: page.title.as_deref().unwrap_or(&page.path),
        lang,
        prefix,
        revision,
        head_html: &html[head_start..head_end],
        has_title: tags(&html[head_start..head_end])
            .iter()
            .any(|(_, _, name)| name == "title"),
        body_html,
        is_index: false,
        show_title: !page.toc.first().is_some_and(|h| h.level == 1),
        article_count: 0,
        topic_count: 0,
        tree: tree.view("", prefix, &page.path),
        toc,
    })
}

#[derive(Serialize)]
struct TocItem {
    text: String,
    url: Option<String>,
    children: Vec<TocItem>,
}
// Nest by semantic level without inventing nodes for skipped levels.
fn toc_items(
    page: &Page,
    targets: &[Option<String>],
    cursor: &mut usize,
    parent_level: usize,
) -> Vec<TocItem> {
    let mut result = Vec::new();
    while *cursor < page.toc.len() && page.toc[*cursor].level > parent_level {
        let i = *cursor;
        let heading = &page.toc[i];
        *cursor += 1;
        result.push(TocItem {
            text: heading.text.clone(),
            url: targets[i].as_ref().map(|id| format!("#{}", escape_url(id))),
            children: toc_items(page, targets, cursor, heading.level),
        });
    }
    result
}

#[derive(Default)]
struct ArticleTree<'a> {
    collapsed: bool,
    entries: Vec<TreeEntry<'a>>,
}
enum TreeEntry<'a> {
    Folder(&'a str, &'a str, ArticleTree<'a>),
    Page(&'a Page),
}
fn article_tree(pages: &[Page]) -> ArticleTree<'_> {
    let mut root = ArticleTree::default();
    let mut pages: Vec<_> = pages.iter().collect();
    pages.sort_by_key(|page| page.navigation.as_ref().map_or(usize::MAX, |nav| nav.order));
    for page in &pages {
        let parts: Vec<_> = page.path.split('/').collect();
        let groups: Vec<_> = if let Some(nav) = &page.navigation {
            nav.groups
                .iter()
                .map(|group| (group.key.as_str(), group.title.as_str(), group.collapsed))
                .collect()
        } else {
            parts[..parts.len().saturating_sub(2)]
                .iter()
                .map(|part| (*part, *part, false))
                .collect()
        };
        let mut node = &mut root;
        for (key, name, collapsed) in groups {
            let index = node
                .entries
                .iter()
                .position(
                    |entry| matches!(entry, TreeEntry::Folder(existing, _, _) if *existing == key),
                )
                .unwrap_or_else(|| {
                    node.entries.push(TreeEntry::Folder(
                        key,
                        name,
                        ArticleTree {
                            collapsed,
                            ..ArticleTree::default()
                        },
                    ));
                    node.entries.len() - 1
                });
            let TreeEntry::Folder(_, _, child) = &mut node.entries[index] else {
                unreachable!()
            };
            node = child;
        }
        node.entries.push(TreeEntry::Page(page));
    }
    // Retain the existing folders-first order when no summary is supplied.
    if pages.iter().all(|page| page.navigation.is_none()) {
        root.sort_default();
    }
    root
}
#[derive(Serialize)]
struct TreeView {
    is_folder: bool,
    collapsed: bool,
    name: String,
    path: String,
    entries: Vec<TreeItem>,
}
#[derive(Serialize)]
#[serde(untagged)]
enum TreeItem {
    Folder(TreeView),
    Page(ArticleLink),
}
#[derive(Serialize)]
struct ArticleLink {
    is_folder: bool,
    title: String,
    url: String,
    path: String,
    current: bool,
}
impl ArticleTree<'_> {
    fn sort_default(&mut self) {
        self.entries.sort_by(|a, b| match (a, b) {
            (TreeEntry::Folder(a, _, _), TreeEntry::Folder(b, _, _)) => a.cmp(b),
            (TreeEntry::Folder(..), TreeEntry::Page(..)) => std::cmp::Ordering::Less,
            (TreeEntry::Page(..), TreeEntry::Folder(..)) => std::cmp::Ordering::Greater,
            (TreeEntry::Page(a), TreeEntry::Page(b)) => a.path.cmp(&b.path),
        });
        for entry in &mut self.entries {
            if let TreeEntry::Folder(_, _, child) = entry {
                child.sort_default();
            }
        }
    }
    fn view(&self, path: &str, prefix: &str, current: &str) -> TreeView {
        TreeView {
            is_folder: true,
            collapsed: self.collapsed,
            name: path.rsplit(" / ").next().unwrap_or("").into(),
            path: path.into(),
            entries: self
                .entries
                .iter()
                .map(|entry| match entry {
                    TreeEntry::Folder(_, name, child) => {
                        let child_path = if path.is_empty() {
                            name.to_string()
                        } else {
                            format!("{path} / {name}")
                        };
                        let mut view = child.view(&child_path, prefix, current);
                        view.name = (*name).into();
                        TreeItem::Folder(view)
                    }
                    TreeEntry::Page(page) => TreeItem::Page(ArticleLink {
                        is_folder: false,
                        title: page
                            .navigation
                            .as_ref()
                            .and_then(|nav| nav.title.as_deref())
                            .or(page.title.as_deref())
                            .unwrap_or_else(|| {
                                page.path
                                    .trim_end_matches("/index.html")
                                    .rsplit('/')
                                    .next()
                                    .unwrap_or(&page.path)
                            })
                            .into(),
                        url: format!("{prefix}{}", escape_url(&page.path)),
                        path: page.path.trim_end_matches("/index.html").into(),
                        current: page.path == current,
                    }),
                })
                .collect(),
        }
    }
}

// Tokenize start tags in Typst's exported HTML, respecting quoted attributes and
// raw text elements. Never search heading text: repeated titles remain distinct.
fn tags(html: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(offset) = html[pos..].find('<') {
        let start = pos + offset;
        if html[start..].starts_with("<!--") {
            pos = html[start..]
                .find("-->")
                .map_or(html.len(), |i| start + i + 3);
            continue;
        }
        let mut quote = None;
        let mut end = html.len();
        for (i, c) in html[start + 1..].char_indices() {
            if Some(c) == quote {
                quote = None;
            } else if quote.is_none() {
                if c == '\'' || c == '"' {
                    quote = Some(c);
                } else if c == '>' {
                    end = start + i + 2;
                    break;
                }
            }
        }
        let name = html[start + 1..end]
            .split(|c: char| c.is_whitespace() || c == '>')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        out.push((start, end, name.clone()));
        pos = end;
        if matches!(name.as_str(), "script" | "style" | "textarea" | "title") {
            pos = html[pos..]
                .find(&format!("</{name}"))
                .map_or(html.len(), |i| pos + i);
        }
    }
    out
}
fn attribute<'a>(tag: &'a str, key: &str) -> Option<&'a str> {
    let mut rest = tag.trim_start_matches('<');
    rest = &rest[rest.find(char::is_whitespace)?..];
    loop {
        rest = rest.trim_start();
        if rest.is_empty() || rest.starts_with('>') {
            return None;
        }
        let n = rest
            .find(|c: char| c.is_whitespace() || c == '=' || c == '>')
            .unwrap_or(rest.len());
        let name = &rest[..n];
        rest = rest[n..].trim_start();
        if !rest.starts_with('=') {
            continue;
        }
        rest = rest[1..].trim_start();
        let (value, tail) = if rest.starts_with(['"', '\'']) {
            let q = rest.as_bytes()[0] as char;
            let end = rest[1..].find(q)? + 1;
            (&rest[1..end], &rest[end + 1..])
        } else {
            let end = rest
                .find(|c: char| c.is_whitespace() || c == '>')
                .unwrap_or(rest.len());
            (&rest[..end], &rest[end..])
        };
        if name == key {
            return Some(value);
        }
        rest = tail;
    }
}
fn heading_targets(body: &str, page: &Page) -> Result<(String, Vec<Option<String>>)> {
    let tags = tags(body);
    let mut used: BTreeSet<String> = tags
        .iter()
        .filter_map(|(s, e, _)| attribute(&body[*s..*e], "id").map(str::to_owned))
        .collect();
    let headings: Vec<_> = tags
        .iter()
        .filter(|(_, _, n)| matches!(n.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6"))
        .collect();
    let mut targets: Vec<_> = page.toc.iter().map(|h| h.anchor.clone()).collect();
    let mut result = body.to_owned();
    // Custom show rules may remove or replace headings. In that case only use
    // compiler-provided anchors that actually exist; don't guess correspondence.
    if headings.len() != targets.len() {
        for target in &mut targets {
            if !target.as_ref().is_some_and(|id| used.contains(id)) {
                *target = None;
            }
        }
        return Ok((result, targets));
    }
    let mut insertions = Vec::new();
    for (i, (s, e, _)) in headings.iter().enumerate() {
        if let Some(id) = attribute(&body[*s..*e], "id") {
            targets[i] = Some(id.into());
        } else if !targets[i].as_ref().is_some_and(|id| used.contains(id)) {
            let mut id = format!("ssg-heading-{}", i + 1);
            while used.contains(&id) {
                id.push('-');
            }
            used.insert(id.clone());
            insertions.push((*e - 1, format!(" id=\"{id}\"")));
            targets[i] = Some(id);
        }
    }
    for (pos, text) in insertions.into_iter().rev() {
        result.insert_str(pos, &text);
    }
    Ok((result, targets))
}
fn escape_url(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~/".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::{Heading, PageKind};
    fn page() -> Page {
        Page {
            path: "nested/post/index.html".into(),
            kind: PageKind::Page,
            title: Some("A < B".into()),
            navigation: None,
            metadata: vec![],
            code: vec![],
            toc: vec![1, 2, 1]
                .into_iter()
                .map(|level| Heading {
                    level,
                    text: "Repeated".into(),
                    anchor: None,
                })
                .collect(),
        }
    }
    #[test]
    fn assembly_preserves_styles_resources_and_unique_targets() -> Result<()> {
        let html = "<!DOCTYPE html><html lang=\"en\"><head><title>A &lt; B</title><style>.typst{color:red}</style></head><body><h2 id=\"existing\">Repeated</h2><h3>Repeated</h3><div id=\"ssg-heading-3\"></div><h2>Repeated</h2><pre>&lt;h2&gt;code&lt;/h2&gt;</pre><img src=\"image.svg\"></body></html>";
        let page = page();
        let files = assemble(
            BTreeMap::from([
                (page.path.clone(), html.as_bytes().to_vec()),
                ("nested/post/image.svg".into(), b"image".to_vec()),
            ]),
            &[page],
        )?;
        let output = std::str::from_utf8(&files["nested/post/index.html"])?;
        for value in [
            ".typst{color:red}",
            "../../_theme/default/main.css",
            "#existing",
            "#ssg-heading-2",
            "#ssg-heading-3-",
            "A &lt; B",
            "image.svg",
        ] {
            assert!(output.contains(value), "{value}");
        }
        assert_eq!(output.matches("<title>").count(), 1);
        assert!(files.contains_key("index.html"));
        assert_eq!(files["nested/post/image.svg"], b"image");
        Ok(())
    }
    #[test]
    fn tree_is_shared_and_marks_current_article() -> Result<()> {
        let page = page();
        let tree = article_tree(std::slice::from_ref(&page));
        let view = tree.view("", "../../", &page.path);
        let templates = templates()?;
        let navigation = templates.render("tree", &view)?;
        assert!(navigation.contains("<summary>nested</summary>"));
        assert!(navigation.contains("../../nested/post/index.html"));
        assert!(navigation.contains("aria-current=\"page\""));
        assert!(!navigation.contains("<summary>post</summary>"));
        assert!(templates.render("index-tree", &view)?.contains("A &lt; B"));
        Ok(())
    }
    #[test]
    fn templates_escape_labels_and_reject_missing_fields() -> Result<()> {
        let registry = templates()?;
        let tree = TreeView {
            name: String::new(),
            path: String::new(),
            is_folder: true,
            collapsed: false,
            entries: vec![TreeItem::Page(ArticleLink {
                is_folder: false,
                title: "<script>bad</script> & \"quoted\"".into(),
                url: "post%20name/index.html".into(),
                path: "post name".into(),
                current: false,
            })],
        };
        let rendered = registry.render("tree", &tree)?;
        assert!(!rendered.contains("<script>"));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(rendered.contains("post%20name/index.html"));
        assert!(registry.render("page", &tree).is_err());
        Ok(())
    }
    #[test]
    fn toc_nests_skipped_levels_and_keeps_unlinked_items() {
        let mut page = page();
        page.toc[1].level = 4;
        let targets = vec![Some("first".into()), None, Some("last".into())];
        let items = toc_items(&page, &targets, &mut 0, 0);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].children.len(), 1);
        assert!(items[0].children[0].url.is_none());
        assert_eq!(items[1].url.as_deref(), Some("#last"));
    }
    #[test]
    fn custom_heading_output_does_not_get_guessed_targets() -> Result<()> {
        let (body, targets) = heading_targets(
            "<script>const x='<h2>fake</h2>';</script><h2>one</h2>",
            &page(),
        )?;
        assert!(!body.contains("ssg-heading"));
        assert!(targets.iter().all(Option::is_none));
        Ok(())
    }
}
