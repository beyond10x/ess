//! Explicit schema-only import, qualified reload and root-selected instance validation.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Args, ValueEnum};
use schema_contract::bundle::{self, Bundle};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Dialect {
    /// Explicitly interpret structural schemas as JSON Schema 2020-12, not as an `OpenAPI` service.
    #[value(name = "draft-2020-12")]
    Draft202012,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    /// Original JSON document containing components/schemas. Its exact bytes are retained.
    #[arg(long)]
    path: PathBuf,
    /// Select a named component root. Repeat to select multiple roots and their closure.
    #[arg(long, required = true)]
    component: Vec<String>,
    /// Required structural interpretation; never inferred from the envelope's version claim.
    #[arg(long, value_enum)]
    dialect: Dialect,
    /// Write the qualified import envelope here; omit to print it.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct DocumentArgs {
    /// Original JSON Schema 2020-12 document, including its root and local $defs.
    #[arg(long)]
    path: PathBuf,
    /// Explicit identity for the document root, distinct from every existing definition.
    #[arg(long)]
    root: String,
    /// Additional $defs roots to make independently selectable.
    #[arg(long)]
    definition: Vec<String>,
    /// Required interpretation; an incompatible declared dialect is refused.
    #[arg(long, value_enum)]
    dialect: Dialect,
    /// Write the replay-checked document bundle here; omit to print it.
    #[arg(long)]
    out: Option<PathBuf>,
}

pub fn import_document(args: &DocumentArgs) -> Result<ExitCode> {
    let source = fs::read_to_string(&args.path)
        .with_context(|| format!("reading {}", args.path.display()))?;
    let dialect = match args.dialect {
        Dialect::Draft202012 => bundle::Dialect::Draft202012,
    };
    let imported = bundle::import_document(
        &source,
        &args.root,
        &args.definition.iter().cloned().collect(),
        dialect,
    )?;
    emit(&args.path, args.out.as_deref(), &imported.to_json()?)?;
    Ok(ExitCode::SUCCESS)
}

#[derive(Debug, Args)]
pub struct ProjectArgs {
    /// Persisted component (/1) or document-root (/2) bundle to revalidate.
    #[arg(long)]
    bundle: PathBuf,
    /// One root explicitly selected by that import.
    #[arg(long)]
    root: String,
    /// Absolute identity for the standalone projected schema, without a fragment.
    #[arg(long)]
    schema_id: String,
    /// Write the qualified standalone schema here; omit to print it.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ValidateArgs {
    /// Persisted component (/1) or document-root (/2) bundle to revalidate.
    #[arg(long)]
    bundle: PathBuf,
    /// One root explicitly selected by that import.
    #[arg(long)]
    root: String,
    /// JSON instance files. No schema selector field is added to their data.
    #[arg(required = true)]
    instances: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TypesTarget {
    /// Structural TypeScript declarations; not a decoder or runtime validator.
    Typescript,
    /// Standalone Cargo data library with explicit field presence and wire serialization.
    Rust,
    /// Standalone Go module with explicit field presence and exact JSON numbers.
    Go,
}

#[derive(Debug, Args)]
pub struct TypesArgs {
    /// Persisted component (/1) or document-root (/2) bundle, revalidated before type planning.
    #[arg(long)]
    bundle: PathBuf,
    /// Root explicitly selected by the import. Repeat to select a shared closure.
    #[arg(long, required = true)]
    root: Vec<String>,
    #[command(flatten)]
    options: TypeOptions,
    /// Directory for declarations, target accounting and the retained source bundle.
    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug, Args)]
pub struct TypeOptions {
    /// Data library target. Unsupported language targets are not silently substituted.
    #[arg(long, value_enum)]
    target: TypesTarget,
    /// Native package identity, required for Rust and Go.
    #[arg(long)]
    package: Option<String>,
    /// Go module identity, required only for Go.
    #[arg(long)]
    module: Option<String>,
}

pub fn types(args: &TypesArgs) -> Result<ExitCode> {
    let bundle = read(&args.bundle)?;
    let plan =
        schema_contract::realize::Plan::from_bundle(&bundle, &args.root.iter().cloned().collect())?;
    let mut files = type_files(&plan, &args.options)?;
    files.insert("source.bundle.json".to_owned(), bundle.to_json()?);
    let destination = crate::resolve_output_directory(&args.out)?;
    let input = fs::canonicalize(&args.bundle)?;
    if files
        .iter()
        .any(|(name, _)| destination.join(name) == input)
    {
        bail!("type realization output must not replace its source input");
    }
    crate::write_owned_files(
        &destination,
        "types-bundle",
        files
            .iter()
            .map(|(path, contents)| (path.as_str(), contents.as_str())),
    )?;
    println!(
        "{} component type(s), written to {}; runtime obligations in types-report.json",
        plan.declarations().len(),
        args.out.display()
    );
    Ok(ExitCode::SUCCESS)
}

pub fn type_files(
    plan: &schema_contract::realize::Plan,
    args: &TypeOptions,
) -> Result<std::collections::BTreeMap<String, String>> {
    let (declaration_path, realization) = match args.target {
        TypesTarget::Typescript => {
            if args.package.is_some() || args.module.is_some() {
                bail!("--package is only supported for a native library target");
            }
            ("types.ts", plan.typescript())
        }
        TypesTarget::Rust => {
            if args.module.is_some() {
                bail!("--module is only supported for the Go target");
            }
            let package = args
                .package
                .as_deref()
                .context("the Rust target requires --package")?;
            ("types.rs", plan.rust(package)?)
        }
        TypesTarget::Go => {
            let package = args
                .package
                .as_deref()
                .context("the Go target requires --package")?;
            let module = args
                .module
                .as_deref()
                .context("the Go target requires --module")?;
            ("types.go", plan.go(package, module)?)
        }
    };
    let report = format!("{}\n", serde_json::to_string_pretty(&realization.report)?);
    let mut files = realization.supporting;
    files.insert(declaration_path.to_owned(), realization.declarations);
    files.insert("types-report.json".to_owned(), report);
    Ok(files)
}

pub fn import(args: &ImportArgs) -> Result<ExitCode> {
    let source = fs::read_to_string(&args.path)
        .with_context(|| format!("reading {}", args.path.display()))?;
    let dialect = match args.dialect {
        Dialect::Draft202012 => bundle::Dialect::Draft202012,
    };
    let imported = bundle::import(&source, &args.component.iter().cloned().collect(), dialect)?;
    emit(&args.path, args.out.as_deref(), &imported.to_json()?)?;
    Ok(ExitCode::SUCCESS)
}

pub fn project(args: &ProjectArgs) -> Result<ExitCode> {
    let imported = read(&args.bundle)?;
    let schema = imported.schema(&args.root, &args.schema_id)?;
    let output = format!("{}\n", serde_json::to_string_pretty(&schema)?);
    emit(&args.bundle, args.out.as_deref(), &output)?;
    Ok(ExitCode::SUCCESS)
}

pub fn validate(args: &ValidateArgs) -> Result<ExitCode> {
    let imported = read(&args.bundle)?;
    let mut valid = 0;
    for path in &args.instances {
        let instance = serde_json::from_slice(&fs::read(path)?)
            .with_context(|| format!("parsing {}", path.display()))?;
        let errors = imported.validate(&args.root, &instance)?;
        if errors.is_empty() {
            valid += 1;
        }
        for error in errors {
            println!("{}{}: {}", path.display(), error.pointer, error.message);
        }
    }
    println!(
        "{} instance(s), {valid} valid, root {}",
        args.instances.len(),
        args.root
    );
    Ok(if valid == args.instances.len() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn read(path: &Path) -> Result<Bundle> {
    Bundle::read(&fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?)
        .with_context(|| format!("checking qualified import {}", path.display()))
}

fn emit(source: &Path, output: Option<&Path>, contents: &str) -> Result<()> {
    if let Some(output) = output {
        let destination = crate::preflight_named_output(output)?;
        if fs::canonicalize(source)? == destination {
            bail!("projection output must not replace its source input");
        }
        crate::write_preflighted_files([(destination, contents)])?;
        println!("wrote {}", output.display());
    } else {
        print!("{contents}");
    }
    Ok(())
}
