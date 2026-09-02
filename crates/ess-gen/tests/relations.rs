//! What a declared relation looks like once it has been projected.
//!
//! `docs/design/ess-entity-relations-design-v0.1.md` §4 decides that one extension key,
//! `x-ess-relation`, carries the whole relation on the property that carries the field — so a
//! reader of `account_id` is told what that column means rather than left to infer it from a type
//! name. These tests hold both contract projections to that, over the billing example, and then
//! hold the committed bytes to what the projection writes: an adopter reads `generated/`, not the
//! generator, and a projection that changed what it publishes without moving those files has
//! published two different contracts for one model.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::{run, Artifact, Generator};
use ess_gen::openapi::OpenApi;
use ess_gen::schema::JsonSchema;
use serde_json::{json, Value};

/// The repository root, from this crate's manifest.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root exists")
}

/// The billing example, compiled from the files it lives in.
fn billing() -> EssIr {
    let base = root().join("examples/billing");
    let mut found = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();

    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside the example")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{label} is readable: {error}"));
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the billing specification validates:\n{errors}"));
    compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("the billing specification resolves:\n{diagnostics}"))
}

/// Every artifact one projection writes for an IR, keyed by path.
fn artifacts(generator: &dyn Generator, ir: &EssIr) -> BTreeMap<String, Artifact> {
    run(generator, ir).expect("no two artifacts claim one path")
}

/// One artifact's contents.
fn contents(artifacts: &BTreeMap<String, Artifact>, path: &str) -> String {
    artifacts
        .get(path)
        .unwrap_or_else(|| panic!("{path} is published; the tree holds {:?}", artifacts.keys()))
        .contents
        .clone()
}

/// The relation the billing example declares, as every projection publishes it.
fn declared() -> Value {
    json!({
        "name": "invoices",
        "kind": "owns",
        "source": "billing.invoice.Account",
        "target": "billing.invoice.Invoice",
        "cardinality": "many",
        "via": "account_id",
    })
}

#[test]
fn the_entity_document_states_the_relation_on_the_property_that_carries_it() {
    let ir = billing();
    let published = artifacts(&JsonSchema, &ir);
    let document: Value = serde_json::from_str(&contents(
        &published,
        "schema/entities/billing.invoice.Invoice.schema.json",
    ))
    .expect("the entity document is JSON");

    assert_eq!(
        document["x-ess-kind"], "entity",
        "the document says what it describes"
    );
    assert_eq!(
        document["properties"]["account_id"]["x-ess-relation"],
        declared(),
        "the carrying property states the whole relation, so a reader of it needs nothing else"
    );
    assert!(
        document["properties"]["account_id"]["x-ess-relation"]["$ref"].is_null(),
        "a self-contained document holds no schema for another entity, and a pointer at one would \
         resolve to nothing"
    );

    // The other end declares the relation and does not carry it: `via` is a field on the target.
    let owner: Value = serde_json::from_str(&contents(
        &published,
        "schema/entities/billing.invoice.Account.schema.json",
    ))
    .expect("the owner's document is JSON");
    assert!(
        owner["properties"]
            .as_object()
            .expect("properties")
            .values()
            .all(|property| property["x-ess-relation"].is_null()),
        "an `owns` relation is carried by the target's field, not by the owner's"
    );
}

#[test]
fn the_openapi_document_states_the_relation_and_links_the_targets_schema() {
    let ir = billing();
    let published = artifacts(&OpenApi, &ir);
    let document: Value =
        serde_yaml::from_str(&contents(&published, "openapi/invoice-service.yaml"))
            .expect("the document is YAML");
    let model = &document["x-ess-entities"];

    let mut expected = declared();
    expected["$ref"] = json!("#/x-ess-entities/billing.invoice.Account");
    assert_eq!(
        model["billing.invoice.Invoice"]["properties"]["account_id"]["x-ess-relation"], expected,
        "the same key as the schema projection, plus the link a document holding the schema of what \
         the property identifies can make"
    );
    assert!(
        !model["billing.invoice.Account"].is_null(),
        "the link points at a schema this document publishes"
    );

    // The contract half is untouched: `components.schemas` is what `ess import openapi` reads back,
    // and an entity's shape reaches a `Map` and a tagged union, which that subset does not carry.
    assert!(
        document["components"]["schemas"]["billing.invoice.Invoice"].is_null(),
        "an entity is published beside the contract, not inside it"
    );
}

/// Compares one committed artifact with what the projection writes now.
fn assert_committed(generator: &dyn Generator, ir: &EssIr, artifact: &str, committed: &str) {
    let written = contents(&artifacts(generator, ir), artifact);
    let path = root().join(committed);
    let recorded = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} is committed ({error})", path.display()));
    assert_eq!(
        written,
        recorded,
        "`{committed}` is not what `{}` writes for the billing example. The committed tree is what \
         an adopter reads; regenerate it in the commit that changed the projection",
        generator.name()
    );
}

#[test]
fn the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes() {
    let ir = billing();
    for entity in ["billing.invoice.Account", "billing.invoice.Invoice"] {
        assert_committed(
            &JsonSchema,
            &ir,
            &format!("schema/entities/{entity}.schema.json"),
            &format!("generated/schema/entities/{entity}.schema.json"),
        );
    }
}

#[test]
fn the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes() {
    assert_committed(
        &OpenApi,
        &billing(),
        "openapi/invoice-service.yaml",
        "generated/openapi/invoice-service.yaml",
    );
}
