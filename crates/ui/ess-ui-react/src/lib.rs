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
//! [`render_bound`] binds the same project to the HTTP surface an ESS model serves, from the
//! route table `ess_ui_check::binding` computes ([`ess_ui::binding::Binding`]); without a binding
//! it gives exactly what [`render`] gives.
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

use ess_ui::binding::Binding;
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
    render_bound(document, source_dir, None)
}

/// Renders the project in memory, bound to the served surface when `binding` is given: the
/// project then carries `src/binding.ts`, its `httpAdapter` reads and commands the binding's
/// paths and is switched on in `main.tsx`, a command's answer is classified as
/// `ess_ui::binding::classify` classifies it, and a refusal is shown on the form, confirm or
/// action that sent it. A section with `live:` polls its read instead (no served surface
/// streams), or is refused where its `degrades: {no_live: refuse}` says so. Without a binding
/// the project is exactly what [`render`] gives.
pub fn render_bound(
    document: &Document,
    source_dir: &Path,
    binding: Option<&Binding>,
) -> Result<GeneratedFiles, GenerateError> {
    let mut files = BTreeMap::new();
    let title = document
        .title
        .clone()
        .unwrap_or_else(|| document.app.clone());
    files.extend(assets::project(&document.app, &title, binding));
    files.insert(
        "src/fixtures.ts".to_owned(),
        fixtures::emit(document, source_dir)?,
    );
    if let Some(binding) = binding {
        files.insert("src/binding.ts".to_owned(), assets::binding(binding));
    }
    let mut gen = emit::Gen::new(document, binding.is_some());
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
    // A bound project runs no channel: the served surface streams nothing to play them against.
    if !document.channels.is_empty() && binding.is_none() {
        gen.used.insert("runtime/live".to_owned());
    }
    files.insert("src/model.ts".to_owned(), gen.model());
    if let Some(refusal) = gen.errors.first() {
        return Err(refusal.clone());
    }
    files.extend(assets::runtime(&gen.used, binding.is_some()));
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
    generate_bound(document, source_dir, out, None)
}

/// [`render_bound`], written under `out` as [`generate`] writes.
pub fn generate_bound(
    document: &Document,
    source_dir: &Path,
    out: &Path,
    binding: Option<&Binding>,
) -> Result<GeneratedFiles, GenerateError> {
    let files = render_bound(document, source_dir, binding)?;
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
    /// The ESS specification the document's `model:` names (a directory, its `ess-inputs.yaml`,
    /// or one file): the app is bound to the HTTP surface the specification's `reached_by:
    /// network` components serve, instead of answering from fixtures.
    #[arg(long)]
    pub model: Option<PathBuf>,
}

/// The entry point an `ess generate ui --target react` command wraps; returns a one-line summary.
///
/// `binding` is the route table `--model` resolves to (`ess_ui_check::binding`), which the caller
/// computes: this crate reads a binding and never compiles a model. `--model` without one is
/// refused.
pub fn run(args: &ReactArgs, binding: Option<&Binding>) -> Result<String, GenerateError> {
    if let (Some(model), None) = (&args.model, binding) {
        return Err(GenerateError::new(format!(
            "--model {}: no binding was computed for it",
            model.display()
        )));
    }
    let loaded =
        ess_ui::load_path(&args.path).map_err(|error| GenerateError::new(error.to_string()))?;
    let dir = args
        .path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let files = generate_bound(&loaded, &dir, &args.out, binding)?;
    Ok(format!(
        "{} files written to {}",
        files.files.len(),
        args.out.display()
    ))
}
