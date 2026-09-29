//! Adversary, pass 1, for synthesis of a branch guarded by a row of another entity
//! (`when_related:`, ess/18, beyond10x/ess#211).
//!
//! The issue's "Expected": "The conformance witness arranges the other record present or absent
//! (with the field set) and asserts each refusal." Each case asks for that on a shape the fixture
//! does not cover.
use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::synthesize::Synthesis;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const SIGN_IN: &str = include_str!("fixtures/related-guard-sign-in.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("adversary.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn refusals_about(result: &Synthesis, command: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains(command))
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn has_scenario(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .iter()
        .any(|(key, _)| key.to_string() == id)
}

/// The fixture with the configuration carrying a plan the tenant is configured with, and the
/// predicate branch reading that stored field alone: "the configuration is on the basic plan".
/// The model is admitted; both sides of the predicate are reached only if the arranged row can be
/// on either plan.
fn planned() -> String {
    let text = replaced(
        SIGN_IN,
        "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n  - name: demo.signin.Plan\n    kind: enum\n    variants: [Basic, Premium]\n",
    );
    let text = replaced(
        &text,
        "      - {name: redirect_client, type: demo.signin.ClientId}\n    lifecycle",
        "      - {name: redirect_client, type: demo.signin.ClientId}\n      - {name: plan, type: demo.signin.Plan}\n    lifecycle",
    );
    let text = replaced(
        &text,
        "      - {name: redirect_client, type: demo.signin.ClientId}\n    outcomes:",
        "      - {name: redirect_client, type: demo.signin.ClientId}\n      - {name: plan, type: demo.signin.Plan}\n    outcomes:",
    );
    let text = replaced(
        &text,
        "          redirect_client: input.redirect_client\n",
        "          redirect_client: input.redirect_client\n          plan: input.plan\n",
    );
    replaced(
        &text,
        "        when_related: {via: input.tenant, predicate: redirect_client != input.client}\n",
        "        when_related: {via: input.tenant, predicate: plan == Basic}\n",
    )
}

#[test]
fn adversary_pass1_a_predicate_over_a_related_enum_field_is_witnessed_on_both_sides() {
    let result = ess_conformance::synthesize::synthesize(&ir(&planned()));
    let refusals = refusals_about(&result, "demo.signin.InitiateSignIn");
    let refused = has_scenario(
        &result,
        "demo.signin.InitiateSignIn/outcome/no-redirect-entry",
    );
    let initiated = has_scenario(&result, "demo.signin.InitiateSignIn/outcome/initiated");
    assert!(
        refusals.is_empty() && refused && initiated,
        "`plan == Basic` over the related row must be witnessed true (no-redirect-entry: {refused}) \
         and false (initiated: {initiated}); refusals: {refusals:#?}"
    );
}

/// A folder is created inside a parent folder that must exist; a root folder has no parent. The
/// related entity is the created entity itself, one hop through the input — the ordinary tree
/// shape. Synthesis must terminate, and arrange the parent through `CreateRoot`.
const FOLDERS: &str = "format: ess/18
system: demo
version: v1
domain: demo.folders
types:
  - {name: demo.folders.FolderId, kind: newtype, of: Uuid}
  - {name: demo.folders.Label, kind: newtype, of: String}
entities:
  - name: demo.folders.Folder
    identity: {name: folder_id, type: demo.folders.FolderId}
    fields:
      - {name: label, type: demo.folders.Label}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
errors:
  - {name: demo.folders.NoParent, summary: The parent folder does not exist., fields: []}
events:
  - name: demo.folders.FolderCreated
    fields:
      - {name: folder_id, type: demo.folders.FolderId}
actors:
  - name: demo.folders.Owner
    may: [demo.folders.CreateFolder, demo.folders.CreateRoot]
commands:
  - name: demo.folders.CreateFolder
    input:
      - {name: parent, type: demo.folders.FolderId}
      - {name: label, type: demo.folders.Label}
    outcomes:
      - name: no-parent
        when_related: {via: input.parent, exists: false}
        error: demo.folders.NoParent
      - name: created
        creates: demo.folders.Folder
        instance: folder_id
        emits: [demo.folders.FolderCreated]
        payload:
          demo.folders.FolderCreated: {folder_id: {generated: true}}
        sets:
          label: input.label
  - name: demo.folders.CreateRoot
    input:
      - {name: label, type: demo.folders.Label}
    outcomes:
      - name: created
        creates: demo.folders.Folder
        instance: folder_id
        emits: [demo.folders.FolderCreated]
        payload:
          demo.folders.FolderCreated: {folder_id: {generated: true}}
        sets:
          label: input.label
views:
  - name: demo.folders.Folders
    source: demo.folders.Folder
    consistency: read_your_writes
    fields:
      - {name: folder_id, type: demo.folders.FolderId}
      - {name: label, type: demo.folders.Label}
";

/// [`FOLDERS`] with `CreateRoot` declared before `CreateFolder`: the creator a row is first
/// arranged through (#198, declaration order) is the unguarded one.
#[test]
fn adversary_pass1_a_folder_inside_an_existing_folder_is_witnessed_with_the_root_creator_first() {
    let (head, commands) = FOLDERS.split_once("commands:\n").unwrap();
    let (folder, rest) = commands
        .split_once("  - name: demo.folders.CreateRoot\n")
        .unwrap();
    let (root, views) = rest.split_once("views:\n").unwrap();
    let text = format!(
        "{head}commands:\n  - name: demo.folders.CreateRoot\n{root}{folder}views:\n{views}"
    );
    let result = ess_conformance::synthesize::synthesize(&ir(&text));
    let refusals = refusals_about(&result, "demo.folders.CreateFolder");
    assert!(
        refusals.is_empty()
            && has_scenario(&result, "demo.folders.CreateFolder/outcome/no-parent")
            && has_scenario(&result, "demo.folders.CreateFolder/outcome/created"),
        "both branches of CreateFolder are witnessed; refusals: {refusals:#?}"
    );
}

#[test]
fn adversary_pass1_a_folder_inside_an_existing_folder_is_witnessed() {
    let result = ess_conformance::synthesize::synthesize(&ir(FOLDERS));
    let refusals = refusals_about(&result, "demo.folders.CreateFolder");
    assert!(
        refusals.is_empty()
            && has_scenario(&result, "demo.folders.CreateFolder/outcome/no-parent")
            && has_scenario(&result, "demo.folders.CreateFolder/outcome/created"),
        "both branches of CreateFolder are witnessed; refusals: {refusals:#?}"
    );
}
