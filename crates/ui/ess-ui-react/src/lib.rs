//! Generates a plain React + TypeScript application from an `ess-ui/1` document: `react` and
//! `react-dom` are its only runtime dependencies, its routing is generated into it, and one
//! esbuild step builds it.
//!
//! [`render`] turns a loaded [`Document`] into the project's files in memory; [`generate`] writes
//! them. The project has a generated History-API router that renders the page a path matches
//! under its shell, one component per page, section and overlay, a small component set for the
//! composite kinds and primitives (plain CSS, no UI library), state placed where each state's
//! `store` says, reads answered from the document's fixtures behind a replaceable data adapter,
//! and live channels that play their fixture scripts until pointed at a server. Every rendered
//! node carries `data-ui-path="<canonical node path>"`.
//!
//! Output is deterministic: the same document and fixtures give the same bytes. A construct the
//! document does not use contributes no code — a runtime module is emitted only when a generated
//! file imports it.
//!
//! This crate has no command line of its own; [`run`] is what `ess generate ui --target react`
//! wraps, and [`ReactArgs`] its arguments.

mod assets;
mod emit;
mod fixtures;
mod jsx;
mod ts;
mod types;

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use ess_ui::Document;

pub use emit::route_pattern;

/// Every file of a generated project, by path relative to the output directory.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneratedFiles {
    /// Relative path (`/`-separated) to file contents.
    pub files: BTreeMap<String, String>,
}

/// Why a project was not generated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateError(String);

impl GenerateError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for GenerateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for GenerateError {}

/// Renders the project in memory. Fixture paths resolve against `source_dir`, the directory of
/// the document.
pub fn render(document: &Document, source_dir: &Path) -> Result<GeneratedFiles, GenerateError> {
    let mut files = BTreeMap::new();
    let title = document
        .title
        .clone()
        .unwrap_or_else(|| document.app.clone());
    files.extend(assets::project(&document.app, &title));
    files.insert(
        "src/fixtures.ts".to_owned(),
        fixtures::emit(document, source_dir)?,
    );
    let mut gen = emit::Gen::new(document);
    files.insert("src/routes.ts".to_owned(), gen.routes());
    for (name, shell) in &document.shells {
        let file = format!("src/shells/{}Shell.tsx", ts::pascal(name));
        files.insert(file, gen.shell(name, shell));
    }
    for (name, page) in &document.pages {
        let file = format!("src/pages/{}.tsx", ts::pascal(name));
        if files.contains_key(&file) {
            return Err(GenerateError::new(format!(
                "pages/{name}: its component file {file} collides with another page's"
            )));
        }
        files.insert(file, gen.page(name, page));
    }
    files.insert("src/App.tsx".to_owned(), gen.app());
    if !document.channels.is_empty() {
        gen.used.insert("runtime/live".to_owned());
    }
    files.insert("src/model.ts".to_owned(), gen.model());
    if let Some(refusal) = gen.errors.first() {
        return Err(refusal.clone());
    }
    files.extend(assets::runtime(&gen.used));
    if files.contains_key("src/runtime/live.ts") {
        files.insert("src/channels.ts".to_owned(), gen.channels());
    }
    Ok(GeneratedFiles { files })
}

/// Renders the project and writes it under `out`, creating directories as needed.
pub fn generate(
    document: &Document,
    source_dir: &Path,
    out: &Path,
) -> Result<GeneratedFiles, GenerateError> {
    let files = render(document, source_dir)?;
    for (path, text) in &files.files {
        let target = out.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                GenerateError::new(format!("cannot create {}: {error}", parent.display()))
            })?;
        }
        std::fs::write(&target, text).map_err(|error| {
            GenerateError::new(format!("cannot write {}: {error}", target.display()))
        })?;
    }
    Ok(files)
}

/// Loads a document file and generates its project under `out`.
pub fn generate_path(document: &Path, out: &Path) -> Result<GeneratedFiles, GenerateError> {
    let loaded =
        ess_ui::load_path(document).map_err(|error| GenerateError::new(error.to_string()))?;
    let dir = document.parent().map(Path::to_path_buf).unwrap_or_default();
    generate(&loaded, &dir, out)
}

/// `ess generate ui --target react` arguments.
#[derive(Debug, Clone, clap::Args)]
pub struct ReactArgs {
    /// The `ess-ui/1` document.
    #[arg(long)]
    pub path: PathBuf,
    /// Directory the project is written to.
    #[arg(long)]
    pub out: PathBuf,
}

/// The entry point an `ess generate ui --target react` command wraps; returns a one-line summary.
pub fn run(args: &ReactArgs) -> Result<String, GenerateError> {
    let files = generate_path(&args.path, &args.out)?;
    Ok(format!(
        "{} files written to {}",
        files.files.len(),
        args.out.display()
    ))
}
