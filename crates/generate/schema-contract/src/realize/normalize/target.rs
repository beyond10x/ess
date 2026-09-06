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
struct SchemaIdentity {
    identity: Root,
    schema_path: String,
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
    let roots = plan
        .recipe
        .branches
        .values()
        .flatten()
        .flat_map(|stage| [&stage.input, &stage.output])
        .collect::<BTreeSet<_>>();
    let mut identities = Vec::new();
    let mut embedded = "//! Generated root bindings; source identities are data, never Rust identifiers.\n\npub(super) const ROOTS: &[(&str, &str, &str)] = &[\n".to_owned();
    for root in roots {
        let bundle = &plan.bundles[&root.bundle_digest];
        let root_digest = source_digest(&root.root);
        let schema_path = format!("schemas/{}-{root_digest}.schema.json", root.bundle_digest);
        let id = format!("urn:ess:normalization:{}:{root_digest}", root.bundle_digest);
        let schema = bundle.schema(&root.root, &id).map_err(|error| {
            Refused(vec![finding("/roots", "target_schema", &error.to_string())])
        })?;
        let schema = format!(
            "{}\n",
            serde_json::to_string_pretty(&schema).expect("schema serializes")
        );
        writeln!(
            embedded,
            "    ({:?}, {:?}, include_str!({:?})),",
            root.bundle_digest,
            root.root,
            format!("../{schema_path}")
        )
        .expect("String write");
        identities.push(SchemaIdentity {
            identity: root.clone(),
            schema_path: schema_path.clone(),
            schema_digest: source_digest(&schema),
        });
        files.insert(schema_path, schema);
        files.insert(
            format!("sources/{}.bundle.json", root.bundle_digest),
            bundle.to_json().map_err(|error| {
                Refused(vec![finding("/roots", "target_bundle", &error.to_string())])
            })?,
        );
    }
    embedded.push_str("];\n");
    files.insert("src/schemas.rs".to_owned(), embedded);
    let report = Report {
        format: "ess-normalization-target/1",
        generator_version: env!("CARGO_PKG_VERSION"),
        configuration: TargetConfiguration::Rust {
            package: package.to_owned(),
        },
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
    Ok(Realization { files, report })
}
