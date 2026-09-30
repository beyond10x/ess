//! The `ess-ui/1` reference, generated from `schemas/ui/ess-ui.schema.yaml` as data.
//!
//! [`render_html`] writes one self-contained HTML page — chapters by the schema's `groups`, per
//! construct its summary, doc, a properties table with readable, cross-linked types, the short
//! forms it accepts, a coloured example and the checks that apply to it, and a quick reference at
//! the end. [`render_markdown`] writes the same content for the documentation site.
//!
//! A schema the reference could not document completely is refused: a construct without a
//! summary, doc or example, a type naming a construct that does not exist, a `{ref: kind}` naming
//! an undeclared kind, or two headings sharing an anchor.
//!
//! This crate has no binary. [`run`] is what `ess ui docs` wraps, with [`DocsArgs`] as its
//! arguments.

mod doc;
mod html;
mod markdown;
mod model;
mod yaml;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

/// Why a schema could not be rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsError {
    problems: Vec<String>,
}

impl DocsError {
    fn one(problem: String) -> Self {
        Self {
            problems: vec![problem],
        }
    }

    /// Every problem found, one line each.
    pub fn problems(&self) -> &[String] {
        &self.problems
    }
}

impl fmt::Display for DocsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.problems.join("\n"))
    }
}

impl std::error::Error for DocsError {}

/// The output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// One self-contained HTML page.
    Html,
    /// Markdown for the documentation site.
    Md,
}

/// `ess ui docs`: render the `ess-ui/1` reference.
#[derive(Debug, Clone, clap::Args)]
pub struct DocsArgs {
    /// The file to write (with `--check`, the file to compare).
    #[arg(long)]
    pub out: PathBuf,
    /// The format; inferred from the extension of `--out` (`.md` is Markdown) when absent.
    #[arg(long, value_enum)]
    pub format: Option<Format>,
    /// The schema to render; the schema this build embeds when absent.
    #[arg(long)]
    pub schema: Option<PathBuf>,
    /// Write nothing; fail when `--out` differs from a fresh render.
    #[arg(long)]
    pub check: bool,
}

/// Renders the schema at `schema` as one self-contained HTML page.
pub fn render_html(schema: &Path) -> Result<String, DocsError> {
    render_html_str(&read(schema)?)
}

/// Renders the schema at `schema` as Markdown for the documentation site.
pub fn render_markdown(schema: &Path) -> Result<String, DocsError> {
    render_markdown_str(&read(schema)?)
}

/// Renders schema text as one self-contained HTML page.
pub fn render_html_str(schema: &str) -> Result<String, DocsError> {
    render(schema, Format::Html)
}

/// Renders schema text as Markdown for the documentation site.
pub fn render_markdown_str(schema: &str) -> Result<String, DocsError> {
    render(schema, Format::Md)
}

/// Renders the reference as `args` asks, returning one line saying what was done.
pub fn run(args: &DocsArgs) -> Result<String, DocsError> {
    let format = args.format.unwrap_or_else(|| {
        if args.out.extension().is_some_and(|ext| ext == "md") {
            Format::Md
        } else {
            Format::Html
        }
    });
    let text = match &args.schema {
        Some(path) => read(path)?,
        None => ess_ui::SCHEMA.to_owned(),
    };
    let rendered = render(&text, format)?;
    let out = args.out.display();
    if args.check {
        let current = fs::read_to_string(&args.out).unwrap_or_default();
        if current == rendered {
            Ok(format!("{out} is current"))
        } else {
            Err(DocsError::one(format!(
                "{out} is not a fresh render of the schema; run `ess ui docs --out {out}` without `--check`"
            )))
        }
    } else {
        fs::write(&args.out, &rendered)
            .map_err(|error| DocsError::one(format!("write {out}: {error}")))?;
        Ok(format!("wrote {out} ({} bytes)", rendered.len()))
    }
}

fn read(schema: &Path) -> Result<String, DocsError> {
    fs::read_to_string(schema)
        .map_err(|error| DocsError::one(format!("read {}: {error}", schema.display())))
}

fn render(text: &str, format: Format) -> Result<String, DocsError> {
    let root: Value = serde_yaml::from_str(text)
        .map_err(|error| DocsError::one(format!("the schema is not YAML: {error}")))?;
    let schema = model::Schema::read(&root).map_err(|problems| DocsError { problems })?;
    let blocks = doc::build(&schema).map_err(|problems| DocsError { problems })?;
    Ok(match format {
        Format::Html => html::render(&blocks, &format!("{} reference", schema.format)),
        Format::Md => markdown::render(&blocks, schema.format),
    })
}
