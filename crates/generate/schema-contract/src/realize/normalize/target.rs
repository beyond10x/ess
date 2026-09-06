//! Executable-target packaging from the sealed recipe, separate from structural type reports.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::Serialize;

use super::{finding, Plan, Refused, Root};
use crate::bundle::source_digest;
use crate::realize::TargetConfiguration;

/// Complete executable-target files; no successful partial adapter is returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Realization {
    /// UTF-8 files relative to the selected output directory, including the target report.
    pub files: BTreeMap<String, String>,
    /// Target identity, checked inputs and deterministic artifact digests.
    pub report: Report,
}

/// Provenance for an executable normalization target, not a types-only coverage claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    format: &'static str,
    generator_version: &'static str,
    configuration: TargetConfiguration,
    recipe_digest: String,
    roots: Vec<SchemaIdentity>,
    files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct SchemaIdentity {
    pub(super) identity: Root,
    pub(super) schema_path: String,
    schema_digest: String,
}

fn rust_package(package: &str) -> Result<(), Refused> {
    if !crate::realize::rust::package_name(package) || package == "jsonschema" {
        return Err(Refused(vec![finding(
            "/",
            "rust_package",
            "use a non-reserved lowercase Cargo package name, distinct from runtime dependencies",
        )]));
    }
    Ok(())
}

pub(super) fn rust(plan: &Plan, package: &str) -> Result<Realization, Refused> {
    rust_package(package)?;
    let mut files = BTreeMap::from([
        ("source.recipe.json".to_owned(), plan.to_json()),
        (
            "src/lib.rs".to_owned(),
            include_str!("rust_runtime.rs.txt").to_owned(),
        ),
        (
            "src/recipe.rs".to_owned(),
            include_str!("recipe.rs").to_owned(),
        ),
        ("src/eval.rs".to_owned(), include_str!("eval.rs").to_owned()),
        (
            "src/numeric.rs".to_owned(),
            include_str!("numeric.rs").to_owned(),
        ),
        (
            "src/input.rs".to_owned(),
            include_str!("input.rs").to_owned(),
        ),
        (
            "src/execute.rs".to_owned(),
            include_str!("execute.rs").to_owned(),
        ),
        (
            "src/diagnostic.rs".to_owned(),
            include_str!("../diagnostic.rs").to_owned(),
        ),
    ]);
    files.insert("Cargo.toml".to_owned(), format!(
        "[package]\nname = {package:?}\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[dependencies]\nserde = {{ version = \"=1.0.229\", features = [\"derive\"] }}\nserde_json = {{ version = \"=1.0.151\", features = [\"raw_value\"] }}\njsonschema = {{ version = \"=0.52.1\", default-features = false }}\n\n[workspace]\n"
    ));
    let (identities, source_files) = sources(plan)?;
    files.extend(source_files);
    let mut embedded = "//! Generated root bindings; source identities are data, never Rust identifiers.\n\npub(super) const ROOTS: &[(&str, &str)] = &[\n".to_owned();
    for root in &identities {
        writeln!(
            embedded,
            "    ({:?}, include_str!({:?})),",
            serde_json::to_string(&root.identity).expect("typed root serializes"),
            format!("../{}", root.schema_path)
        )
        .expect("String write");
    }
    embedded.push_str("];\n");
    files.insert("src/schemas.rs".to_owned(), embedded);
    Ok(finish(
        plan,
        TargetConfiguration::Rust {
            package: package.to_owned(),
        },
        files,
        identities,
    ))
}

type SourceFiles = (Vec<SchemaIdentity>, BTreeMap<String, String>);

pub(super) fn sources(plan: &Plan) -> Result<SourceFiles, Refused> {
    let roots = plan
        .recipe
        .branches
        .values()
        .flatten()
        .flat_map(|stage| [&stage.input, &stage.output])
        .collect::<BTreeSet<_>>();
    let mut identities = Vec::new();
    let mut files = BTreeMap::new();
    for root in roots {
        let root_digest = source_digest(root.name());
        let (key, schema, source_path, source) = match root {
            Root::Bundle {
                bundle_digest,
                root,
            } => {
                let bundle = &plan.bundles[bundle_digest];
                let id = format!("urn:ess:normalization:{bundle_digest}:{root_digest}");
                let schema = bundle.schema(root, &id).map_err(|error| {
                    Refused(vec![finding("/roots", "target_schema", &error.to_string())])
                })?;
                let source = bundle.to_json().map_err(|error| {
                    Refused(vec![finding("/roots", "target_bundle", &error.to_string())])
                })?;
                (
                    bundle_digest.clone(),
                    schema,
                    format!("sources/{bundle_digest}.bundle.json"),
                    source,
                )
            }
            Root::Model { model, .. } => {
                let key = format!("model-{}", model.digest());
                (
                    key.clone(),
                    super::source::model_schema(plan, root),
                    format!("sources/model-{}.schema.json", model.projection_digest),
                    plan.models[model].to_json(),
                )
            }
        };
        let schema_path = format!("schemas/{key}-{root_digest}.schema.json");
        let schema = format!(
            "{}\n",
            serde_json::to_string_pretty(&schema).expect("schema serializes")
        );
        identities.push(SchemaIdentity {
            identity: root.clone(),
            schema_path: schema_path.clone(),
            schema_digest: source_digest(&schema),
        });
        files.insert(schema_path, schema);
        files.insert(source_path, source);
    }
    Ok((identities, files))
}

pub(super) fn finish(
    plan: &Plan,
    configuration: TargetConfiguration,
    mut files: BTreeMap<String, String>,
    identities: Vec<SchemaIdentity>,
) -> Realization {
    let report = Report {
        format: if plan.recipe.format == super::FORMAT_V3 {
            "ess-normalization-target/2"
        } else {
            "ess-normalization-target/1"
        },
        generator_version: env!("CARGO_PKG_VERSION"),
        configuration,
        recipe_digest: source_digest(&plan.to_json()),
        roots: identities,
        files: files
            .iter()
            .map(|(name, value)| (name.clone(), source_digest(value)))
            .collect(),
    };
    files.insert(
        "normalization-report.json".to_owned(),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&report).expect("typed report serializes")
        ),
    );
    Realization { files, report }
}
