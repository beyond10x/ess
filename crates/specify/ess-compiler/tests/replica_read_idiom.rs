//! One write path, a read replica in each environment: the idiom the model already has for it, the
//! three refusals that keep the idiom the only spelling, and the guide section that teaches it.
//!
//! The model keeps placement out on purpose. One component accepts the writes; its views are
//! `eventual`; a `role` setting and a shared store carry the deployment fact; each environment's
//! binding supplies the values. Nothing here is new surface: these cases hold that it still works.

use ess_compiler::resolve::{compile, diagnose};
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use std::path::PathBuf;

/// The domain the guide's model is placed into.
const DOMAIN: &str = "\
format: ess/22
system: directory
version: v1
domain: directory.accounts
entities:
  - name: directory.accounts.Account
    identity: {name: account_id, type: Uuid}
    fields: [{name: email, type: String}]
    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}
";

/// The model the guide section shows, byte for byte.
const IDIOM: &str = "\
types:
  - {name: directory.accounts.Role, kind: enum, variants: [Primary, Replica]}
views:
  - name: directory.accounts.Accounts
    source: directory.accounts.Account
    consistency: eventual                                # a replica may trail the write
    fields: [{name: account_id, type: Uuid}, {name: email, type: String}]
components:
  - component: directory                                 # one component, every environment
    owns: {domains: [directory.accounts]}
    settings:
      - {name: role, type: directory.accounts.Role, required: true}
      - {name: write-endpoint, type: Optional<String>}   # absent on the primary
topology:
  workloads:
    directory: {stateless: true, replicas: {min: 2}, requires: [{postgres: directory-store}]}
";

/// Compiles one document, or returns every refusal rendered with its code, message and hint, as
/// `ess specify validate` reports it: a rule `ess-domain` enforces reaches a reader through
/// [`diagnose`], which gives it the same `ESS-<layer>-<n>` code.
fn compiled(text: &str) -> Result<(), String> {
    let parsed = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    let mut sources = SourceMap::new();
    sources.insert("directory.yaml".to_owned(), text.to_owned());
    let diagnostics = match Specification::assemble([(Source::new("directory.yaml"), parsed)]) {
        Ok(specification) => match compile(&specification, &sources) {
            Ok(_) => return Ok(()),
            Err(diagnostics) => diagnostics,
        },
        Err(errors) => diagnose(&errors, &sources),
    };
    Err(diagnostics
        .as_slice()
        .iter()
        .map(|diagnostic| {
            format!(
                "{} {} {}",
                diagnostic.code,
                diagnostic.message,
                diagnostic.hint.as_deref().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

fn model(idiom: &str) -> String {
    format!("{DOMAIN}{idiom}")
}

#[test]
fn replica_read_idiom_validates() {
    compiled(&model(IDIOM)).expect("the replica-read idiom validates with no diagnostic");
}

#[test]
fn stateful_replicas_without_a_shared_store_are_refused() {
    let stateful = IDIOM.replace(
        "{stateless: true, replicas: {min: 2}, requires: [{postgres: directory-store}]}",
        "{stateless: false, replicas: {min: 2}}",
    );
    assert_ne!(stateful, IDIOM);
    let refusal = compiled(&model(&stateful)).expect_err("two stateful instances are refused");
    assert!(
        refusal.contains("ESS-TOPOLOGY-004"),
        "stateful replicas with no shared store are ESS-TOPOLOGY-004: {refusal}"
    );
    assert!(
        refusal.contains("nothing says how they share it"),
        "the refusal says the state is not shared: {refusal}"
    );
    assert!(
        refusal.contains("a store they share is a `requires:` entry"),
        "the refusal carries the shared-store hint: {refusal}"
    );
}

#[test]
fn topology_environment_placement_key_is_refused() {
    let placed = IDIOM.replace(
        "directory: {stateless: true,",
        "directory: {environments: [central, edge], stateless: true,",
    );
    assert_ne!(placed, IDIOM);
    let refusal = compiled(&model(&placed)).expect_err("a placement key is refused");
    assert!(
        refusal.contains("unknown field `environments`")
            && refusal.contains("`replicas`, `stateless`, `requires`"),
        "`environments:` is an unknown field naming the three a workload has: {refusal}"
    );
}

#[test]
fn replica_only_component_is_refused() {
    let replica = IDIOM.replace(
        "topology:",
        "  - component: directory-replica                         # owns, accepts and publishes nothing\n\
         topology:",
    );
    assert_ne!(replica, IDIOM);
    let refusal = compiled(&model(&replica)).expect_err("an empty component is refused");
    assert!(
        refusal.contains("ESS-COMPONENT-007"),
        "a component owning, accepting and publishing nothing is ESS-COMPONENT-007: {refusal}"
    );
}

/// The `## ` section of `page` titled `heading`, up to the next `## ` heading outside a fence.
fn section<'a>(page: &'a str, heading: &str) -> Option<&'a str> {
    let start = page.find(&format!("\n{heading}\n"))? + 1;
    let mut fenced = false;
    let mut offset = start;
    for line in page[start..].split_inclusive('\n') {
        if line.starts_with("```") {
            fenced = !fenced;
        }
        if offset > start && !fenced && line.starts_with("## ") {
            return Some(&page[start..offset]);
        }
        offset += line.len();
    }
    Some(&page[start..])
}

#[test]
fn replica_read_guide_section_states_the_idiom() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("website/docs").is_dir())
        .expect("the workspace holds website/docs")
        .join("website/docs/guides/specify/values-and-views.md");
    let page = std::fs::read_to_string(&path).expect("values-and-views.md");
    let headings: Vec<&str> = page.lines().filter(|l| l.starts_with("## ")).collect();
    let consistency = headings
        .iter()
        .position(|h| *h == "## A view declares its consistency")
        .expect("the page keeps `## A view declares its consistency`");
    assert_eq!(
        headings.get(consistency + 1).copied(),
        Some("## A view served from a replica"),
        "`## A view served from a replica` directly follows `## A view declares its consistency`"
    );
    let section = section(&page, "## A view served from a replica").expect("the section");
    for phrase in [
        "one component",
        "`consistency: eventual`",
        "`role`",
        "`requires:`",
        "`ess-environment/1`",
        "write-primary",
        "out of scope",
    ] {
        assert!(
            section.contains(phrase),
            "`## A view served from a replica` is missing {phrase}"
        );
    }
    let fence = section
        .split("```yaml\n")
        .nth(1)
        .and_then(|rest| rest.split("```").next())
        .expect("the section shows its model in a ```yaml fence");
    assert_eq!(
        fence, IDIOM,
        "the section's model is the one replica_read_idiom_validates compiles"
    );
}
