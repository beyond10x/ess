//! Filesystem edge for native compact compilation and the standard Go emitter.
use anyhow::{Context, Result};
use ess_conformance::{authored::Source, compact, models::Models, recipes::Library};
use std::{fs, path::PathBuf, process::ExitCode};

#[derive(Debug, clap::Args)]
pub(super) struct Args {
    /// Original composition specification with exact component/model pins.
    #[arg(long)]
    composition: PathBuf,
    /// Exact service-key=specification-directory binding; repeat for each import.
    #[arg(long = "service", required = true)]
    services: Vec<super::ServiceInput>,
    /// Explicit fixture recipe file; repeat to supply a library.
    #[arg(long = "recipe")]
    recipes: Vec<PathBuf>,
    /// Compact scenario files or explicit directories; repeat for an exact selection.
    #[arg(long, required = true)]
    scenarios: Vec<PathBuf>,
    /// Owned output directory containing the native Go package and bound source manifest.
    #[arg(long)]
    out: PathBuf,
}

pub(super) fn run(args: &Args) -> Result<ExitCode> {
    let composition_text = fs::read_to_string(&args.composition)
        .with_context(|| format!("reading {}", args.composition.display()))?;
    let specification = if args
        .composition
        .extension()
        .is_some_and(|ext| ext == "json")
    {
        ess_composition::CompositionSpec::from_json(&composition_text)?
    } else {
        ess_composition::CompositionSpec::from_yaml(&composition_text)?
    };
    let mut originals = Vec::new();
    for service in &args.services {
        let Ok((ir, _)) = super::resolved(&service.path, super::Format::Text)? else {
            return Ok(ExitCode::from(1));
        };
        originals.push((service.key.clone(), ir));
    }
    let models = Models::compile(
        &specification,
        &originals
            .iter()
            .map(|(key, ir)| (key.clone(), ir.as_ref()))
            .collect::<Vec<_>>(),
    )
    .map_err(|diagnostics| anyhow::anyhow!("{diagnostics}"))?;
    let mut recipe_sources = Vec::new();
    for path in &args.recipes {
        // Direct-file acquisition supplies a checked, portable source identity. Equal
        // basenames are refused by the library; no absolute workstation path is published.
        for source in crate::input_discovery::acquire(path, crate::input_discovery::Kind::Coverage)?
        {
            recipe_sources.push(Source::new(source.identity, source.text));
        }
    }
    let library = Library::parse(&recipe_sources).map_err(anyhow::Error::msg)?;
    let mut sources = Vec::new();
    for path in &args.scenarios {
        sources.extend(
            crate::input_discovery::authored(Some(path), true)?
                .into_iter()
                .map(|source| Source::new(source.identity, source.text)),
        );
    }
    let compiled = compact::compile(&models, &library, &sources).map_err(anyhow::Error::msg)?;
    let mut files = ess_conformance::go::emit_input(&compiled.input)?;
    files.push(ess_conformance::go::GoArtifact {
        path: "essconform/live-inputs.json".into(),
        contents: compiled.manifest,
    });
    super::write_owned_files(
        &args.out,
        "conformance-go",
        files
            .iter()
            .map(|file| (file.path.as_str(), file.contents.as_str())),
    )?;
    println!("{}: {} compact scenario(s), {} fixture source(s); native Go and exact input manifest written to {}",
        models.composition().composition(), compiled.input.selected().suite().len(), recipe_sources.len(), args.out.display());
    Ok(ExitCode::SUCCESS)
}
