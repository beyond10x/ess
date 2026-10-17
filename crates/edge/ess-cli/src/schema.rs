//! CLI shell for adopter-owned JSON Schema contracts.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use schema_contract::validate::{self, JsonDocument};
use serde_json::{json, Value};

use crate::Format;

/// Operations over adopter-owned JSON Schema contracts.
#[derive(Debug, Subcommand)]
pub(crate) enum SchemaCommand {
    /// Extract a qualified structural component closure without inventing a service.
    ImportBundle(crate::schema_bundle::ImportArgs),
    /// Retain a JSON Schema document root and its local definitions without inventing a service.
    ImportDocument(crate::schema_bundle::DocumentArgs),
    /// Revalidate a qualified import and project one selected root as standalone JSON Schema.
    ProjectBundle(crate::schema_bundle::ProjectArgs),
    /// Validate unmodified JSON or YAML instances against one selected component root.
    ValidateBundle(crate::schema_bundle::ValidateArgs),
    /// Realize selected structural roots with source provenance and target accounting.
    TypesBundle(crate::schema_bundle::TypesArgs),
    /// Check every source-pinned normalization branch and emit the canonical recipe.
    NormalizeCheck(crate::normalize::CheckArgs),
    /// Execute one explicit normalization branch with checked input and output boundaries.
    NormalizeRun(crate::normalize::RunArgs),
    /// Emit a source-pinned normalization library, or check its generated file bytes.
    NormalizeGenerate(crate::normalize::GenerateArgs),
    /// Validate JSON or YAML instances against schemas selected by their `schema` property.
    Validate(ValidateArgs),
    /// Project a schema's structural types into a deterministic TypeScript module.
    Typescript(TypeScriptArgs),
}

/// Inputs for offline schema-contract validation.
#[derive(Debug, Args)]
pub(crate) struct ValidateArgs {
    /// Instance files or directories to validate. Directories are searched recursively. A file
    /// ending `.yaml` or `.yml` is read as the JSON value its YAML denotes, one ending `.json` as JSON.
    #[arg(required = true)]
    paths: Vec<PathBuf>,
    /// Directory containing the authoritative `*.schema.json` registry.
    #[arg(long, value_name = "DIR", required = true)]
    schemas: PathBuf,
    /// How to render the validation report.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

/// Inputs for deterministic TypeScript projection.
#[derive(Debug, Args)]
pub(crate) struct TypeScriptArgs {
    /// The schema's exact `$id`, not its filename.
    schema_id: String,
    /// The exported root type name.
    #[arg(long)]
    root: String,
    /// Directory containing the authoritative `*.schema.json` registry.
    #[arg(long, value_name = "DIR", required = true)]
    schemas: PathBuf,
    /// Write the generated module here. Omit to print it.
    #[arg(long)]
    out: Option<PathBuf>,
    /// Refuse when `--out` differs from the generated module, without writing it.
    #[arg(long, requires = "out")]
    check: bool,
}

pub(crate) fn run(command: SchemaCommand) -> Result<ExitCode> {
    match command {
        SchemaCommand::ImportBundle(args) => crate::schema_bundle::import(&args),
        SchemaCommand::ImportDocument(args) => crate::schema_bundle::import_document(&args),
        SchemaCommand::ProjectBundle(args) => crate::schema_bundle::project(&args),
        SchemaCommand::ValidateBundle(args) => crate::schema_bundle::validate(&args),
        SchemaCommand::TypesBundle(args) => crate::schema_bundle::types(&args),
        SchemaCommand::NormalizeCheck(args) => crate::normalize::check(&args),
        SchemaCommand::NormalizeRun(args) => crate::normalize::run(&args),
        SchemaCommand::NormalizeGenerate(args) => crate::normalize::generate(&args),
        SchemaCommand::Validate(args) => validate_instances(&args),
        SchemaCommand::Typescript(args) => typescript(&args),
    }
}

fn validate_instances(args: &ValidateArgs) -> Result<ExitCode> {
    let schema_paths = json_files(std::slice::from_ref(&args.schemas), true)?;
    if schema_paths.is_empty() {
        bail!("the schema registry contains no `*.schema.json` files");
    }
    let instance_paths = json_files(&args.paths, false)?;
    if instance_paths.is_empty() {
        bail!("the supplied paths contain no JSON or YAML instance files");
    }

    let schemas = read_json(&schema_paths)?;
    let instances = instance_paths
        .iter()
        .map(|path| read_instance(path))
        .collect::<Result<Vec<_>>>()?;
    let schema_documents = documents(&schema_paths, &schemas);
    let instance_documents = documents(&instance_paths, &instances);
    let report = validate::validate(&schema_documents, &instance_documents);

    match args.format {
        Format::Text => {
            for issue in &report.issues {
                println!("{issue}");
            }
            if report.is_valid() {
                println!(
                    "{} schema(s), {} instance(s): valid",
                    report.schema_count,
                    report.valid.len()
                );
            } else {
                println!(
                    "{} schema(s), {} valid instance(s), {} issue(s): invalid",
                    report.schema_count,
                    report.valid.len(),
                    report.issues.len()
                );
            }
        }
        Format::Json | Format::Yaml => {
            let value = json!({
                "schema_count": report.schema_count,
                "valid": report.valid.iter().map(|valid| json!({
                    "instance": valid.instance,
                    "schema_id": valid.schema_id,
                    "schema": valid.schema,
                })).collect::<Vec<_>>(),
                "issues": report.issues.iter().map(|issue| json!({
                    "code": issue.code.as_str(),
                    "document": issue.document,
                    "instance_path": issue.instance_path,
                    "message": issue.message,
                })).collect::<Vec<_>>(),
            });
            crate::render(&value, args.format)?;
        }
    }

    Ok(if report.is_valid() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn typescript(args: &TypeScriptArgs) -> Result<ExitCode> {
    let paths = json_files(std::slice::from_ref(&args.schemas), true)?;
    let values = read_json(&paths)?;
    let matching = paths
        .iter()
        .zip(&values)
        .filter(|(_, schema)| schema.get("$id").and_then(Value::as_str) == Some(&args.schema_id))
        .collect::<Vec<_>>();
    let [(source, schema)] = matching.as_slice() else {
        bail!(
            "expected exactly one schema with `$id: {}`, found {}",
            args.schema_id,
            matching.len()
        );
    };
    let generated = schema_contract::typescript::project(schema, &args.root)
        .with_context(|| format!("projecting {}", source.display()))?;

    let Some(output) = &args.out else {
        print!("{generated}");
        return Ok(ExitCode::SUCCESS);
    };
    if args.check {
        let _guard = crate::output_ownership::check(
            output
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new(".")),
        )?;
        crate::preflight_named_output(output)?;
        let existing = fs::read_to_string(output)
            .with_context(|| format!("reading generated projection {}", output.display()))?;
        if existing == generated {
            println!("{}: current", output.display());
            return Ok(ExitCode::SUCCESS);
        }
        println!("{}: stale", output.display());
        return Ok(ExitCode::from(1));
    }

    write_projection(output, &generated)?;
    println!("wrote {} from {}", output.display(), source.display());
    Ok(ExitCode::SUCCESS)
}

fn json_files(inputs: &[PathBuf], schemas: bool) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for input in inputs {
        collect_json(input, schemas, &mut paths)?;
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn collect_json(path: &Path, schemas: bool, output: &mut Vec<PathBuf>) -> Result<()> {
    if path.is_dir() {
        let mut entries = fs::read_dir(path)
            .with_context(|| format!("reading directory {}", path.display()))?
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            collect_json(&entry.path(), schemas, output)?;
        }
        return Ok(());
    }
    if !path.is_file() {
        bail!("{} is not a file or directory", path.display());
    }
    let is_json = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"));
    // Instances only: a registry holds the `*.schema.json` documents ESS projects.
    let is_yaml = !schemas && is_yaml(path);
    let is_schema = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem.to_ascii_lowercase().ends_with(".schema"));
    let selected = (is_json || is_yaml) && (schemas == is_schema);
    if selected {
        output.push(path.to_path_buf());
    }
    Ok(())
}

fn read_json(paths: &[PathBuf]) -> Result<Vec<Value>> {
    paths
        .iter()
        .map(|path| {
            let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
            // Not `serde_json::from_slice`: under `arbitrary_precision`, which `entity-core`
            // unifies into a build, that keeps a number's spelling and admits `1e400`.
            ess_primitives::json::from_slice(&bytes)
                .with_context(|| format!("parsing {}", path.display()))
        })
        .collect()
}

fn is_yaml(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("yaml") || extension.eq_ignore_ascii_case("yml")
        })
}

/// One instance file as the JSON value it denotes (beyond10x/ess#591).
///
/// A `.yaml` or `.yml` file is read through `ess-yaml`, the reader ESS sources are read with, as
/// exactly one document. A YAML document that has no JSON value is refused rather than coerced:
/// a tag, a key that is not a string, a number that is not finite, and a key written twice, each
/// named by its pointer. A number is held as ESS holds a JSON number ([`read_json`]), so `1.50`
/// in YAML and in JSON are one value. Every other file is JSON.
///
/// # Errors
///
/// When the file cannot be read, does not parse, or holds a YAML value outside JSON.
pub(crate) fn read_instance(path: &Path) -> Result<Value> {
    if !is_yaml(path) {
        return read_json(std::slice::from_ref(&path.to_path_buf()))
            .map(|mut values| values.remove(0));
    }
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let document: ess_yaml::Value =
        ess_yaml::from_str(&text).with_context(|| format!("parsing {} as YAML", path.display()))?;
    yaml_json(&document, "").map_err(|message| anyhow::anyhow!("{}: {message}", path.display()))
}

fn yaml_json(value: &ess_yaml::Value, pointer: &str) -> std::result::Result<Value, String> {
    Ok(match value {
        ess_yaml::Value::Null => Value::Null,
        ess_yaml::Value::Bool(value) => Value::Bool(*value),
        ess_yaml::Value::String(text) => Value::String(text.clone()),
        ess_yaml::Value::Number(number) => {
            if number.as_f64().is_some_and(|value| !value.is_finite()) {
                return Err(format!(
                    "{}: a number that is not finite has no JSON value",
                    at(pointer)
                ));
            }
            Value::Number(
                ess_primitives::json::number(&number.to_string())
                    .map_err(|error| format!("{}: {error}", at(pointer)))?,
            )
        }
        ess_yaml::Value::Sequence(items) => Value::Array(
            items
                .iter()
                .enumerate()
                .map(|(index, item)| yaml_json(item, &format!("{pointer}/{index}")))
                .collect::<std::result::Result<_, _>>()?,
        ),
        ess_yaml::Value::Mapping(entries) => {
            let mut object = serde_json::Map::new();
            for (key, item) in entries {
                let Some(key) = key.as_str() else {
                    return Err(format!(
                        "{}: a key that is not a string has no JSON value",
                        at(pointer)
                    ));
                };
                let escaped = key.replace('~', "~0").replace('/', "~1");
                let item = yaml_json(item, &format!("{pointer}/{escaped}"))?;
                if object.insert(key.to_owned(), item).is_some() {
                    return Err(format!("{}: the key `{key}` is written twice", at(pointer)));
                }
            }
            Value::Object(object)
        }
        ess_yaml::Value::Tagged(tagged) => {
            return Err(format!(
                "{}: the tag `{}` has no JSON value",
                at(pointer),
                tagged.tag
            ));
        }
    })
}

fn at(pointer: &str) -> &str {
    if pointer.is_empty() {
        "/"
    } else {
        pointer
    }
}

fn documents<'a>(paths: &'a [PathBuf], values: &'a [Value]) -> Vec<JsonDocument<'a>> {
    paths
        .iter()
        .zip(values)
        .map(|(path, value)| JsonDocument::new(path.to_str().unwrap_or("<non-UTF-8 path>"), value))
        .collect()
}

fn write_projection(path: &Path, contents: &str) -> Result<()> {
    crate::output_ownership::named(path, "typescript-file", contents)
}

#[cfg(test)]
mod tests {
    use super::read_json;

    /// An instance's numbers are read as the build without `serde_json`'s `arbitrary_precision`
    /// reads them, so a document validates the same way whichever `serde_json` ESS was built with.
    #[test]
    fn an_instance_reads_its_numbers_the_same_in_every_serde_json_build() {
        let directory = tempfile::tempdir().expect("a scratch directory");
        let path = directory.path().join("instance.json");
        std::fs::write(
            &path,
            r#"{"a":1.50,"b":1E2,"c":-0,"d":18446744073709551616}"#,
        )
        .expect("the instance writes");
        let values = read_json(std::slice::from_ref(&path)).expect("the instance reads");
        assert_eq!(
            values[0].to_string(),
            r#"{"a":1.5,"b":100.0,"c":-0.0,"d":1.8446744073709552e+19}"#
        );
        std::fs::write(&path, "[1e400]").expect("the instance writes");
        assert!(read_json(&[path]).is_err(), "a number past binary64");
    }
}
