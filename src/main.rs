mod inspect;
mod options;
mod output;
mod source;
mod world;

use anyhow::Result;
use clap::Parser;
use options::Options;
use typst_bundle::Bundle;
use world::SiteWorld;

fn compile(world: &SiteWorld) -> Result<Bundle> {
    let compiled = typst::compile::<Bundle>(world);
    for warning in compiled.warnings {
        eprintln!("warning: {}", warning.message);
    }
    compiled
        .output
        .map_err(|errors| anyhow::anyhow!("{errors:#?}"))
}

fn main() -> Result<()> {
    let options = Options::parse();
    let root = std::env::current_dir()?.canonicalize()?;
    let source = source::prepare(&root, &options.input)?;
    let world = SiteWorld::new(&root, source.id, source.generated, &options)?;
    let bundle = compile(&world)?;
    let pages = inspect::pages(&bundle);
    output::write(&bundle, &pages, &options)?;
    println!(
        "Exported {} pages to {}",
        pages.len(),
        options.output.display()
    );
    Ok(())
}
