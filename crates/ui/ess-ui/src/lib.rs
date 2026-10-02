//! `ess-ui/1`: a renderer-neutral UI document, loaded into typed Rust.
//!
//! The format is defined as data in `schemas/ui/ess-ui.schema.yaml`, embedded here as [`SCHEMA`].
//! Loading a document runs four steps, each refusing with the [`NodePath`] of the node at fault:
//!
//! 1. every shorthand the schema's `shorthands.index` lists is expanded, from that index's own
//!    `expands_to` templates where the expansion is a template;
//! 2. every page is merged over its page kind (built-in kinds come from the schema), and every
//!    `same_as` overlay over the overlay it names;
//! 3. every widget instance receives its widget's body with the arguments substituted, and then
//!    every `tone_by.tones` is resolved to the `tone_maps` entry it names;
//! 4. the result is read into [`Document`], whose structs refuse any key they do not declare.
//!
//! [`Document::nodes`] then yields every node with its canonical path.
//!
//! This crate has no command line. [`check`] is the entry point a command wraps.
//!
//! [`binding`] is the contract between a document and the HTTP surface ESS synthesizes: the route
//! table a renderer reads, and the one classification of a command's answer.

pub mod binding;
mod expand;
mod locate;
mod model;
mod path;
mod schema;
mod style;

use std::fmt;
use std::path::Path;

pub use model::*;
pub use path::{Located, NodePath, NodeRef, SEGMENT_PATTERN};
pub use schema::{Positions, Shape};

/// Every key at which the schema expects a `Node`, `Action`, `Field` or `Reads`: the positions
/// at which expansion applies their shorthands and derived names.
pub fn positions() -> &'static Positions {
    schema::Schema::embedded().positions()
}

/// The format marker every document carries.
pub const FORMAT: &str = "ess-ui/1";

/// The schema, `schemas/ui/ess-ui.schema.yaml`, as the bytes this build was compiled with.
pub const SCHEMA: &str = include_str!("../../../../schemas/ui/ess-ui.schema.yaml");

/// A document refused, naming the node at fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    path: NodePath,
    message: String,
}

impl LoadError {
    pub(crate) fn new(path: NodePath, message: impl Into<String>) -> Self {
        Self {
            path,
            message: message.into(),
        }
    }

    /// The node the refusal is about.
    pub fn path(&self) -> &NodePath {
        &self.path
    }

    #[must_use]
    pub(crate) fn with_hint(mut self, hint: &str) -> Self {
        self.message.push_str("; ");
        self.message.push_str(hint);
        self
    }

    /// What is wrong with it.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for LoadError {}

/// Loads a document from YAML text.
pub fn load_str(text: &str) -> Result<Document, LoadError> {
    let raw: serde_yaml::Value = serde_yaml::from_str(text)
        .map_err(|error| LoadError::new(NodePath::root(), error.to_string()))?;
    let schema = schema::Schema::embedded();
    let expanded = expand::expand(raw, &schema)?;
    let document: Document = match serde_yaml::from_value(expanded.clone()) {
        Ok(document) => document,
        Err(error) => {
            return Err(locate::locate(&expanded)
                .unwrap_or_else(|| LoadError::new(NodePath::root(), error.to_string())));
        }
    };
    path::check(&document)?;
    Ok(document)
}

/// Loads a document from a file.
pub fn load_path(file: &Path) -> Result<Document, LoadError> {
    let text = std::fs::read_to_string(file).map_err(|error| {
        LoadError::new(
            NodePath::root(),
            format!("cannot read {}: {error}", file.display()),
        )
    })?;
    load_str(&text)
}

/// What a successful [`check`] found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckSummary {
    /// Pages in the document.
    pub pages: usize,
    /// Nodes with a canonical path.
    pub nodes: usize,
}

impl fmt::Display for CheckSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{FORMAT}: {} pages, {} addressed nodes",
            self.pages, self.nodes
        )
    }
}

/// Loads and checks a document file; the entry point an `ess ui check <file>` command wraps.
pub fn check(file: &Path) -> Result<CheckSummary, LoadError> {
    let document = load_path(file)?;
    Ok(CheckSummary {
        pages: document.pages.len(),
        nodes: document.nodes().len(),
    })
}
