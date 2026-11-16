use serde::Serialize;
use typst::foundations::Value;
use typst::foundations::{Array, Label, NativeElement, Selector};
use typst::introspection::{Introspector, MetadataElem};
use typst::model::{Document, HeadingElem};
use typst::text::{RawContent, RawElem};
use typst_bundle::{Bundle, BundleDocument, BundleFile};

#[derive(Debug, Serialize)]
pub struct Page {
    pub path: String,
    #[serde(rename = "type")]
    pub kind: PageKind,
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navigation: Option<Navigation>,
    pub metadata: Vec<Metadata>,
    pub toc: Vec<Heading>,
    pub code: Vec<Code>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PageKind {
    Page,
}

#[derive(Debug, Serialize)]
pub struct Navigation {
    pub groups: Vec<NavigationGroup>,
    pub title: Option<String>,
    pub order: usize,
}

#[derive(Debug, Serialize)]
pub struct NavigationGroup {
    pub key: String,
    pub title: String,
    pub collapsed: bool,
}

fn navigation(introspector: &dyn Introspector) -> Option<Navigation> {
    let content = introspector.query(&Selector::Label(
        Label::construct("ssg-navigation".into()).expect("nonempty internal label"),
    ));
    let value = &content.first()?.to_packed::<MetadataElem>()?.value;
    let Value::Dict(dict) = value else {
        return None;
    };
    Some(Navigation {
        groups: dict
            .get("groups")
            .ok()?
            .clone()
            .cast::<Array>()
            .ok()?
            .iter()
            .map(|value| {
                let Value::Dict(group) = value else {
                    return None;
                };
                Some(NavigationGroup {
                    key: group.get("key").ok()?.clone().cast::<String>().ok()?,
                    title: group.get("title").ok()?.clone().cast::<String>().ok()?,
                    collapsed: group
                        .get("collapsed")
                        .ok()
                        .map(|value| value.clone().cast::<bool>().ok())
                        .unwrap_or(Some(false))?,
                })
            })
            .collect::<Option<_>>()?,
        title: dict
            .get("title")
            .ok()?
            .clone()
            .cast::<Option<String>>()
            .ok()?,
        order: dict.get("order").ok()?.clone().cast::<usize>().ok()?,
    })
}

// Keep arbitrary extension metadata in its native Typst representation.
#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct Metadata {
    pub value: Value,
}

#[derive(Debug, Serialize)]
pub struct Heading {
    pub level: usize,
    pub text: String,
    pub anchor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Code {
    pub language: Option<String>,
    pub block: bool,
    pub text: String,
}

pub fn pages(bundle: &Bundle) -> Vec<Page> {
    bundle
        .files
        .iter()
        .filter_map(|(path, file)| {
            let BundleFile::Document(BundleDocument::Html(doc)) = file else {
                return None;
            };
            let introspector = doc.introspector().as_ref();
            let toc = toc(introspector, bundle);
            let title = doc.info().title.as_ref().map(ToString::to_string);
            Some(Page {
                path: path.get_without_slash().to_string(),
                kind: PageKind::Page,
                title: page_title(title, &toc),
                navigation: navigation(introspector),
                metadata: metadata(introspector),
                toc,
                code: code(introspector),
            })
        })
        .collect()
}

// An opening H1 can name an otherwise untitled article, but multiple H1s
// represent peer sections rather than an unambiguous document title.
fn page_title(explicit: Option<String>, headings: &[Heading]) -> Option<String> {
    explicit.or_else(|| {
        let first = headings.first()?;
        if first.level != 1 || headings.iter().skip(1).any(|heading| heading.level == 1) {
            return None;
        }
        let text = first.text.trim();
        (!text.is_empty()).then(|| text.to_owned())
    })
}

fn code(introspector: &dyn Introspector) -> Vec<Code> {
    introspector
        .query(&Selector::Elem(RawElem::ELEM, None))
        .iter()
        .filter_map(|content| {
            let raw = content.to_packed::<RawElem>()?;
            Some(Code {
                language: raw
                    .lang
                    .get_ref(Default::default())
                    .as_ref()
                    .map(ToString::to_string),
                block: raw.block.get(Default::default()),
                text: raw_text(&raw.text),
            })
        })
        .collect()
}

fn raw_text(text: &RawContent) -> String {
    match text {
        RawContent::Text(text) => text.to_string(),
        RawContent::Lines(lines) => lines
            .iter()
            .map(|(line, _)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn metadata(introspector: &dyn Introspector) -> Vec<Metadata> {
    introspector
        .query(&Selector::Elem(MetadataElem::ELEM, None))
        .iter()
        .filter(|content| {
            content.label()
                != Some(Label::construct("ssg-navigation".into()).expect("nonempty internal label"))
        })
        .filter_map(|content| content.to_packed::<MetadataElem>())
        .map(|element| Metadata {
            value: element.value.clone(),
        })
        .collect()
}

fn toc(introspector: &dyn Introspector, bundle: &Bundle) -> Vec<Heading> {
    introspector
        .query(&Selector::Elem(HeadingElem::ELEM, None))
        .iter()
        .filter_map(|content| {
            let heading = content.to_packed::<HeadingElem>()?;
            let location = content.location()?;
            Some(Heading {
                level: heading.resolve_level(Default::default()).get(),
                text: heading.body.plain_text().to_string(),
                anchor: bundle
                    .introspector
                    .anchor(location)
                    .map(ToString::to_string),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headings(items: &[(usize, &str)]) -> Vec<Heading> {
        items
            .iter()
            .map(|(level, text)| Heading {
                level: *level,
                text: (*text).into(),
                anchor: None,
            })
            .collect()
    }

    #[test]
    fn title_prefers_explicit_then_a_unique_opening_h1() {
        let toc = headings(&[(1, "  Article title  "), (2, "Section"), (3, "Details")]);
        assert_eq!(page_title(None, &toc).as_deref(), Some("Article title"));
        assert_eq!(
            page_title(Some("Explicit".into()), &toc).as_deref(),
            Some("Explicit")
        );
        for items in [
            vec![],
            vec![(2, "Section")],
            vec![(2, "Section"), (1, "Later H1")],
            vec![(1, "First"), (2, "Child"), (1, "Second")],
            vec![(1, "  ")],
        ] {
            assert_eq!(page_title(None, &headings(&items)), None);
        }
    }
}
