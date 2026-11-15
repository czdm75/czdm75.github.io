use crate::options::Options;
use anyhow::Result;
use std::path::Path;
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Features, Library, LibraryExt, World};
use typst_kit::{
    datetime::Time,
    downloader::SystemDownloader,
    files::{FileStore, FsRoot, SystemFiles},
    fonts::FontStore,
    packages::SystemPackages,
};
pub struct SiteWorld {
    library: LazyHash<Library>,
    files: FileStore<SystemFiles>,
    time: Time,
    fonts: FontStore,
    main: FileId,
    generated: Option<Source>,
}
impl World for SiteWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }
    fn main(&self) -> FileId {
        self.main
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if let Some(source) = &self.generated {
            if id == self.main {
                return Ok(source.clone());
            }
        }
        self.files.source(id)
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if let Some(source) = &self.generated {
            if id == self.main {
                return Ok(Bytes::from_string(source.text().to_owned()));
            }
        }
        self.files.file(id)
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }
    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.time.today(offset)
    }
}

impl SiteWorld {
    pub fn new(
        root: &Path,
        main: FileId,
        generated: Option<String>,
        options: &Options,
    ) -> Result<Self> {
        let mut fonts = FontStore::new();
        fonts.extend(typst_kit::fonts::embedded());
        fonts.extend(typst_kit::fonts::system());
        let inputs = options.inputs(root)?;
        Ok(Self {
            library: Library::builder()
                .with_features(Features::all())
                .with_inputs(inputs)
                .build()
                .into(),
            files: FileStore::new(SystemFiles::new(
                FsRoot::new(root.to_path_buf()),
                SystemPackages::new(SystemDownloader::new(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION")
                ))),
            )),
            time: Time::system(),
            fonts,
            main,
            generated: generated.map(|text| Source::new(main, text)),
        })
    }
}
