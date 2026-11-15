use serde::Serialize;
use typst::foundations::Value;
use typst::foundations::{NativeElement, Selector};
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
    pub metadata: Vec<Metadata>,
    pub toc: Vec<Heading>,
    pub code: Vec<Code>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PageKind {
    Page,
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
            Some(Page {
                path: path.get_without_slash().to_string(),
                kind: PageKind::Page,
                title: doc.info().title.as_ref().map(ToString::to_string),
                metadata: metadata(introspector),
                toc: toc(introspector, bundle),
                code: code(introspector),
            })
        })
        .collect()
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
