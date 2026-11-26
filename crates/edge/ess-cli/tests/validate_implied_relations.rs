//! `ess specify validate` warns where a specification implies a relation it does not declare
//! (beyond10x/ess#437).
//!
//! Two exact rules, both advisory: a stored entity field typed as the named identity of exactly one
//! entity that no `references` or `owns` carries (`ESS-ENTITY-019`), and a `when_related:` guard
//! whose row the identity type alone settled (`ESS-COMMAND-019`). A warning leaves the exit status
//! and the compiled model as they were.

use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

/// The fit-review model: `Agent.pool: PoolId`, with no relation.
fn fixture() -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/implied-relation.yaml"),
    )
    .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `model`, written to a fresh file under this test target's scratch space.
fn written(name: &str, model: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "validate-implied-relations/{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    let path = root.join("system.yaml");
    fs::write(&path, model).unwrap();
    path
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}

fn validate_text(path: &Path) -> Output {
    ess(&["specify", "validate", "--path", path.to_str().unwrap()])
}

fn validate_json(path: &Path) -> (Output, Value) {
    let output = ess(&[
        "specify",
        "validate",
        "--path",
        path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    let report = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {output:?}"));
    (output, report)
}

/// The codes of every warning the JSON report carries, in order.
fn warning_codes(report: &Value) -> Vec<String> {
    report
        .get("warnings")
        .and_then(Value::as_array)
        .map(|warnings| {
            warnings
                .iter()
                .map(|warning| warning["code"].as_str().unwrap().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// Inserts `relation` as the agent's `relations:` list, just before its lifecycle.
fn agent_relations(model: &str, relations: &str) -> String {
    let anchor = "      - {name: pool, type: probe.staff.PoolId}\n    lifecycle:";
    assert!(
        model.contains(anchor),
        "the fixture still has the agent's field"
    );
    model.replace(
        anchor,
        &format!(
            "      - {{name: pool, type: probe.staff.PoolId}}\n    relations:\n{relations}    lifecycle:"
        ),
    )
}

const DECLARED: &str =
    "      - {name: pool, kind: references, target: probe.staff.Pool, cardinality: one, via: pool}\n";

/// `AddAgent` guarded on the pool its input names: refused when no pool carries the identity, or
/// when the pool is closed.
fn guarded(model: &str) -> String {
    let anchor = "    outcomes:\n      - name: added\n        creates: probe.staff.Agent\n";
    assert!(model.contains(anchor), "the fixture still creates an agent");
    model
        .replace(
            anchor,
            "    outcomes:\n      - name: no-pool\n        when_related: {via: input.pool, exists: false}\n        error: probe.staff.NoPool\n      - name: closed\n        when_related: {via: input.pool, predicate: open == false}\n        error: probe.staff.PoolClosed\n      - name: added\n        creates: probe.staff.Agent\n",
        )
        .replace(
            "\nviews:\n",
            "\nerrors:\n  - {name: probe.staff.NoPool, summary: No pool carries the identity., fields: []}\n  - {name: probe.staff.PoolClosed, summary: The pool is closed., fields: []}\nviews:\n",
        )
}

#[test]
fn stored_named_identity_without_relation_warns() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/implied-relation.yaml");
    let output = validate_text(&path);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(text(&output.stdout), "probe v1 — 1 file(s), valid\n");
    let stderr = text(&output.stderr);
    assert_eq!(
        stderr.matches("warning[").count(),
        1,
        "exactly one warning: {stderr}"
    );
    assert_eq!(
        stderr.matches("warning[ESS-ENTITY-019]").count(),
        1,
        "{stderr}"
    );
    for named in [
        "probe.staff.Agent",
        "`pool`",
        "probe.staff.Pool",
        "references",
    ] {
        assert!(stderr.contains(named), "names {named}: {stderr}");
    }
    // Located in the file, at the agent's declaration where the field's own line is not unique.
    assert!(stderr.contains("--> implied-relation.yaml:"), "{stderr}");
}

#[test]
fn declared_relation_silences_the_warning() {
    let path = written("declared", &agent_relations(&fixture(), DECLARED));
    let output = validate_text(&path);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        text(&output.stderr),
        "",
        "no warning once the relation is declared"
    );
    let (_, report) = validate_json(&path);
    assert!(report.get("warnings").is_none(), "{report}");
}

#[test]
fn owns_on_target_silences_the_warning() {
    let anchor = "      - {name: open, type: Boolean}\n    lifecycle:";
    let model = fixture();
    assert!(model.contains(anchor));
    let model = model.replace(
        anchor,
        "      - {name: open, type: Boolean}\n    relations:\n      - {name: agents, kind: owns, target: probe.staff.Agent, cardinality: many, via: pool}\n    lifecycle:",
    );
    let path = written("owns", &model);
    let output = validate_text(&path);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        text(&output.stderr),
        "",
        "the owner's relation carries the field"
    );
}

#[test]
fn optional_and_list_identity_fields_warn() {
    // The `pool` field is declared, so only the two new fields can warn.
    let model = agent_relations(&fixture(), DECLARED).replace(
        "      - {name: pool, type: probe.staff.PoolId}\n    relations:",
        "      - {name: pool, type: probe.staff.PoolId}\n      - {name: backup, type: 'Optional<probe.staff.PoolId>'}\n      - {name: history, type: 'List<probe.staff.PoolId>'}\n    relations:",
    );
    let path = written("optional-list", &model);
    let (output, report) = validate_json(&path);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        warning_codes(&report),
        ["ESS-ENTITY-019", "ESS-ENTITY-019"],
        "{report}"
    );
    let paths: Vec<&str> = report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|warning| warning["span"]["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        [
            "entity probe.staff.Agent.fields.backup",
            "entity probe.staff.Agent.fields.history"
        ]
    );
    let hints: Vec<&str> = report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|warning| warning["hint"].as_str().unwrap())
        .collect();
    assert!(hints[0].contains("cardinality: one"), "{}", hints[0]);
    assert!(hints[1].contains("cardinality: many"), "{}", hints[1]);
}

#[test]
fn bare_primitive_identity_is_not_linted() {
    let model = fixture()
        .replace(
            "    identity: {name: pool_id, type: probe.staff.PoolId}",
            "    identity: {name: pool_id, type: Uuid}",
        )
        .replace(
            "    identity: {name: agent_id, type: probe.staff.AgentId}",
            "    identity: {name: agent_id, type: Uuid}",
        )
        .replace("type: probe.staff.PoolId}", "type: Uuid}")
        .replace("type: probe.staff.AgentId}", "type: Uuid}");
    assert!(!model.contains("type: probe.staff.PoolId"), "{model}");
    let path = written("bare-primitive", &model);
    let output = validate_text(&path);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        text(&output.stderr),
        "",
        "`Uuid` identifies both entities, so no field typed `Uuid` names one"
    );

    // A guard whose row a bare `Uuid` settles is not linted either: only the pool is identified by
    // `Uuid` here, so the row resolves, and the rule still reads named identity types only.
    let guarded_model = guarded(&fixture())
        .replace(
            "    identity: {name: pool_id, type: probe.staff.PoolId}",
            "    identity: {name: pool_id, type: Uuid}",
        )
        .replace("type: probe.staff.PoolId}", "type: Uuid}");
    assert!(
        !guarded_model.contains("type: probe.staff.PoolId}"),
        "{guarded_model}"
    );
    let path = written("bare-primitive-guard", &guarded_model);
    let output = validate_text(&path);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        text(&output.stderr),
        "",
        "a `when_related:` row settled by a bare primitive is not linted"
    );
}

#[test]
fn related_guard_by_fallback_warns() {
    let path = written("guard", &guarded(&fixture()));
    let (output, report) = validate_json(&path);
    assert!(output.status.success(), "{output:?}");
    let codes = warning_codes(&report);
    assert!(
        codes.contains(&"ESS-COMMAND-019".to_owned()),
        "the guard warns: {report}"
    );
    let guard = report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|warning| warning["code"] == "ESS-COMMAND-019")
        .unwrap();
    assert_eq!(
        guard["span"]["path"], "command.probe.staff.AddAgent.outcomes.no-pool.when_related",
        "{guard}"
    );
    assert!(
        guard["message"].as_str().unwrap().contains("input.pool"),
        "{guard}"
    );
    assert!(
        guard["hint"].as_str().unwrap().contains("references"),
        "{guard}"
    );

    // Declared: the relation settles the row, and nothing warns.
    let declared = written(
        "guard-declared",
        &agent_relations(&guarded(&fixture()), DECLARED),
    );
    let (output, report) = validate_json(&declared);
    assert!(output.status.success(), "{output:?}");
    assert!(report.get("warnings").is_none(), "{report}");
    assert_eq!(text(&validate_text(&declared).stderr), "");
}

#[test]
fn warnings_do_not_change_exit_or_ir() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/implied-relation.yaml");
    let (output, report) = validate_json(&path);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(report["valid"], true, "{report}");
    let warnings = report["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 1, "{report}");
    let warning = &warnings[0];
    assert_eq!(warning["code"], "ESS-ENTITY-019");
    assert_eq!(warning["severity"], "warning");
    assert_eq!(
        warning["span"]["path"],
        "entity probe.staff.Agent.fields.pool"
    );
    assert!(warning["span"]["source"].is_string(), "{warning}");
    assert!(
        warning["hint"].as_str().unwrap().contains("references"),
        "{warning}"
    );

    // The model compiles to the IR the compiler makes of it, with no trace of the warning.
    let compiled = ess(&[
        "specify",
        "compile",
        "--path",
        path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(compiled.status.success(), "{compiled:?}");
    assert_eq!(text(&compiled.stderr), "");
    let source = fixture();
    let raw = ess_domain::spec::RawSpecFile::parse(&source).unwrap();
    let specification = ess_domain::spec::Specification::assemble(vec![(
        ess_domain::system::Source::new("system.yaml"),
        raw,
    )])
    .unwrap();
    let ir =
        ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap();
    assert_eq!(text(&compiled.stdout), ir.to_canonical_json());

    // And the warning is not a refusal anywhere else: synthesis runs as it did.
    let synthesized = ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        path.to_str().unwrap(),
    ]);
    assert!(synthesized.status.success(), "{synthesized:?}");
    assert!(
        !text(&synthesized.stderr).contains("warning"),
        "{synthesized:?}"
    );
}

#[test]
fn implied_relation_docs_state_the_rule() {
    let reference =
        fs::read_to_string(repo().join("website/docs/reference/diagnostics.md")).unwrap();
    let classes = reference
        .split("\n### Classes\n")
        .nth(1)
        .and_then(|rest| rest.split("\n### ").next())
        .expect("the reference has `### Classes`");
    let row = classes
        .lines()
        .find(|line| line.starts_with("| `019` | `IMPLIED_RELATION` |"))
        .expect("`### Classes` has the row `019` `IMPLIED_RELATION`");
    for phrase in ["warning", "`references`"] {
        assert!(row.contains(phrase), "the `019` row says {phrase}: {row}");
    }
    let names = reference
        .split("\n### Validation names\n")
        .nth(1)
        .expect("the reference has `### Validation names`");
    assert!(
        names.contains("| `implied_relation` | `019` `IMPLIED_RELATION` |"),
        "the validation-name table maps `implied_relation` to `019`"
    );

    let guide =
        fs::read_to_string(repo().join("website/docs/guides/specify/values-and-views.md")).unwrap();
    let resolution = guide
        .find("Where several\nentities share that identity type, the relation on the subject field says which one")
        .expect("the relation-resolution text");
    let start = guide
        .find("\n### A relation the model only implies\n")
        .expect("the guide has `### A relation the model only implies`");
    assert!(
        start > resolution,
        "the section comes after the relation-resolution text"
    );
    let rest = &guide[start + 1..];
    let section = rest[1..].find("\n#").map_or(rest, |end| &rest[..=end]);
    for phrase in [
        "`ESS-ENTITY-019`",
        "warning",
        "exit status",
        "declare a `references` relation",
        "bare primitive",
        "row set",
    ] {
        assert!(
            section.contains(phrase),
            "`### A relation the model only implies` says {phrase}"
        );
    }
}

/// Adversary, W3-3 pass 1: an `ess/23` rename (beyond10x/ess#429) of the entity a field names.
/// `Grant.secret` stores a secret's identity with no relation. Both repairs a hint could name — a
/// `references` on the field or an `owns` carried by it — make the secret a relation-carried
/// entity, whose re-key validate refuses. A warning whose only repair is a refusal cannot be
/// silenced, so none is given (coordinator decision, correction round 1: the case first asserted
/// the hinted relation validates, which #429's rule forbids).
#[test]
fn adversary_a_field_naming_a_renamed_entity_does_not_warn() {
    let (output, report) = validate_json(&written("rename", &renamed_with_grant()));
    assert!(output.status.success(), "{output:?}");
    assert!(
        !warning_codes(&report)
            .iter()
            .any(|code| code == "ESS-ENTITY-019"),
        "`demo.vault.Secret` is re-keyed, so no relation to it can be declared: {report}"
    );
}

/// The `ess/23` vault model with a `Grant` that stores a secret's identity and declares nothing.
fn renamed_with_grant() -> String {
    let rename = fs::read_to_string(
        repo().join("crates/verify/ess-conformance/tests/fixtures/identity-changing-updates.yaml"),
    )
    .unwrap();
    let model = rename
        .replace(
            "  - {name: demo.vault.SecretName, kind: newtype, of: String}\n",
            "  - {name: demo.vault.SecretName, kind: newtype, of: String}\n  - {name: demo.vault.GrantId, kind: newtype, of: Uuid}\n",
        )
        .replace(
            "\nerrors:\n",
            "\n  - name: demo.vault.Grant\n    identity: {name: grant_id, type: demo.vault.GrantId}\n    fields:\n      - {name: secret, type: demo.vault.SecretName}\n    lifecycle: {initial: Held, states: [Held], terminal: [Held]}\nerrors:\n",
        );
    assert!(model.contains("  - name: demo.vault.Grant\n"), "{model}");
    model
}

/// The same class for a guard: a `when_related:` row of a re-keyed entity, settled by its type,
/// gets no `ESS-COMMAND-019` either, because the relation that would settle it is refused.
#[test]
fn a_guard_reading_a_renamed_entity_does_not_warn() {
    let model = renamed_with_grant()
        .replace(
            "  - {name: demo.vault.Keeper, may: [demo.vault.StoreSecret, demo.vault.RenameSecret]}\n",
            "  - {name: demo.vault.Keeper, may: [demo.vault.StoreSecret, demo.vault.RenameSecret, demo.vault.GrantSecret]}\n",
        )
        .replace(
            "\nviews:\n",
            "\n  - name: demo.vault.GrantSecret\n    input:\n      - {name: secret, type: demo.vault.SecretName}\n    outcomes:\n      - name: no-secret\n        when_related: {via: input.secret, exists: false}\n        error: demo.vault.NoSuchSecret\n      - name: granted\n        creates: demo.vault.Grant\n        instance: grant_id\n        sets: {secret: input.secret}\n        emits: [demo.vault.SecretGranted]\n        payload:\n          demo.vault.SecretGranted: {grant_id: {generated: true}}\nviews:\n",
        )
        .replace(
            "\nactors:\n",
            "\n  - name: demo.vault.SecretGranted\n    fields:\n      - {name: grant_id, type: demo.vault.GrantId}\nactors:\n",
        );
    assert!(
        model.contains("when_related: {via: input.secret"),
        "{model}"
    );
    let (output, report) = validate_json(&written("rename-guard", &model));
    assert!(output.status.success(), "{output:?}");
    assert!(
        !warning_codes(&report)
            .iter()
            .any(|code| code.ends_with("-019")),
        "`demo.vault.Secret` is re-keyed, so no relation to it can be declared: {report}"
    );
}

/// Adversary, W3-3 pass 1: an `owns` is carried only by a field typed exactly the owner's
/// identity, so for an `Optional<…>` or `List<…>` field that alternative would be refused. The
/// hint names only `references` there (coordinator decision, correction round 1: the case first
/// asserted the offered `owns` validates).
#[test]
fn adversary_the_hint_for_an_optional_or_list_field_names_only_references() {
    for wrapped in [
        "'Optional<probe.staff.PoolId>'",
        "'List<probe.staff.PoolId>'",
    ] {
        // The field, the input `AddAgent` copies into it and the view that projects it, alike.
        let model = fixture().replace(
            "{name: pool, type: probe.staff.PoolId}",
            &format!("{{name: pool, type: {wrapped}}}"),
        );
        assert!(model.contains(wrapped), "{model}");
        let (output, report) = validate_json(&written("wrapped-hint", &model));
        assert!(output.status.success(), "{output:?}");
        assert_eq!(warning_codes(&report), ["ESS-ENTITY-019"], "{report}");
        let hint = report["warnings"][0]["hint"].as_str().unwrap();
        assert!(hint.contains("`references`"), "{hint}");
        assert!(
            !hint.contains("`owns`"),
            "an `owns` is refused for a {wrapped} field, so the hint must not offer it: {hint}"
        );
    }

    // A field typed exactly the identity may still be carried by an `owns`, and the hint says so.
    let (_, report) = validate_json(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/implied-relation.yaml"),
    );
    let hint = report["warnings"][0]["hint"].as_str().unwrap();
    assert!(hint.contains("`owns`"), "{hint}");
}

/// `CloseShelf` addresses the shelf through `input.shelf_id` (its moving branch's `instance:`),
/// and its `when_related:` guards read the row that same input names: the command's own subject.
fn reads_own_subject(format: &str) -> String {
    format!(
        "format: {format}
system: probe
version: v1
domain: probe.shop
types:
  - {{name: probe.shop.ShelfId, kind: newtype, of: String}}
entities:
  - name: probe.shop.Shelf
    identity: {{name: shelf_id, type: probe.shop.ShelfId}}
    fields:
      - {{name: capacity, type: Integer}}
      - {{name: count, type: Integer}}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {{name: close, from: [Open], to: Closed}}
errors:
  - {{name: probe.shop.NoShelf, fields: [{{name: shelf_id, type: probe.shop.ShelfId}}]}}
  - {{name: probe.shop.OverCapacity}}
  - {{name: probe.shop.ShelfClosed}}
commands:
  - name: probe.shop.CloseShelf
    input:
      - {{name: shelf_id, type: probe.shop.ShelfId}}
      - {{name: count, type: Integer}}
    outcomes:
      - name: no-shelf
        when_related: {{via: input.shelf_id, exists: false}}
        error: probe.shop.NoShelf
        payload: {{probe.shop.NoShelf: {{shelf_id: input.shelf_id}}}}
      - name: over-capacity
        when_related: {{via: input.shelf_id, predicate: capacity < input.count}}
        error: probe.shop.OverCapacity
      - name: closed
        moves: probe.shop.Shelf.close
        instance: shelf_id
        sets: {{count: input.count}}
        emits: [probe.shop.ShelfDone]
        payload: {{probe.shop.ShelfDone: {{shelf_id: input.shelf_id}}}}
      - name: wrong-state
        wrong_state: true
        error: probe.shop.ShelfClosed
  - name: probe.shop.OpenShelf
    input:
      - {{name: shelf_id, type: probe.shop.ShelfId}}
      - {{name: capacity, type: Integer}}
    outcomes:
      - name: opened
        creates: probe.shop.Shelf
        instance: shelf_id
        sets: {{capacity: input.capacity, count: 0}}
        emits: [probe.shop.ShelfOpened]
        payload: {{probe.shop.ShelfOpened: {{shelf_id: input.shelf_id}}}}
events:
  - {{name: probe.shop.ShelfDone, fields: [{{name: shelf_id, type: probe.shop.ShelfId}}]}}
  - {{name: probe.shop.ShelfOpened, fields: [{{name: shelf_id, type: probe.shop.ShelfId}}]}}
views:
  - name: probe.shop.Shelves
    source: probe.shop.Shelf
    consistency: read_your_writes
    fields:
      - {{name: shelf_id, type: probe.shop.ShelfId}}
      - {{name: capacity, type: Integer}}
      - {{name: count, type: Integer}}
      - {{name: state, type: probe.shop.Shelf.State}}
"
    )
}

/// From `ess/38` a `when_related:` read of the row the command addresses itself is refused
/// (`ESS-COMMAND-004`), its hint naming `when_subject:` and `unknown_instance:`; at `ess/37` the
/// same text is a warning and the specification validates.
#[test]
fn a_related_read_of_the_commands_own_subject_is_refused_from_ess_38_and_warns_below() {
    let path = written("own-subject-38", &reads_own_subject("ess/38"));
    let output = validate_text(&path);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(stderr.contains("error[ESS-COMMAND-004]"), "{stderr}");
    for named in [
        "probe.shop.CloseShelf",
        "input.shelf_id",
        "when_subject: {predicate:",
        "unknown_instance: true",
    ] {
        assert!(stderr.contains(named), "names {named}: {stderr}");
    }

    let path = written("own-subject-37", &reads_own_subject("ess/37"));
    let (output, report) = validate_json(&path);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let codes = warning_codes(&report);
    assert!(
        codes.iter().any(|code| code == "ESS-COMMAND-004"),
        "{report}"
    );
    let warning = report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|warning| warning["code"] == "ESS-COMMAND-004")
        .unwrap();
    let hint = warning["hint"].as_str().unwrap();
    assert!(
        hint.contains("when_subject: {predicate:") && hint.contains("unknown_instance: true"),
        "{hint}"
    );
}

/// [`reads_own_subject`], where `CloseShelf` also refuses while an item of the shelf at or past
/// the count is kept: a guard on a row set, beside which a `when_subject` guard is refused
/// (`ESS-COMMAND-009`).
fn beside_row_set(model: &str) -> String {
    let replaced = [
        (
            "  - {name: probe.shop.ShelfId, kind: newtype, of: String}\n",
            "  - {name: probe.shop.ShelfId, kind: newtype, of: String}\n  - {name: probe.shop.ItemId, kind: newtype, of: String}\n",
        ),
        (
            "errors:\n",
            "  - name: probe.shop.Item\n    identity: {name: item_id, type: probe.shop.ItemId}\n    fields:\n      - {name: shelf_id, type: probe.shop.ShelfId}\n      - {name: seq, type: Integer}\n    relations:\n      - {name: shelf, kind: references, target: probe.shop.Shelf, cardinality: one, via: shelf_id}\n    lifecycle: {initial: Kept, states: [Kept], terminal: [Kept]}\nerrors:\n  - {name: probe.shop.ItemsLeft}\n",
        ),
        (
            "      - name: closed\n",
            "      - name: items-left\n        when_related:\n          entity: probe.shop.Item\n          where: {all: [shelf_id == input.shelf_id, seq >= input.count]}\n          exists: true\n        error: probe.shop.ItemsLeft\n      - name: closed\n",
        ),
    ];
    let mut model = model.to_owned();
    for (anchor, with) in replaced {
        assert!(model.contains(anchor), "the model still has {anchor}");
        model = model.replacen(anchor, with, 1);
    }
    model
}

/// The form the row-set hint names: `unknown_instance: true` for the missing shelf, and the guard
/// over the shelf's capacity as a row set selecting the shelf by its key.
fn row_set_form(model: &str) -> String {
    let replaced = [
        (
            "        when_related: {via: input.shelf_id, exists: false}\n",
            "        unknown_instance: true\n",
        ),
        (
            "        when_related: {via: input.shelf_id, predicate: capacity < input.count}\n",
            "        when_related:\n          entity: probe.shop.Shelf\n          where: {all: [shelf_id == input.shelf_id, capacity < input.count]}\n          exists: true\n",
        ),
    ];
    let mut model = model.to_owned();
    for (anchor, with) in replaced {
        assert!(model.contains(anchor), "the model still has {anchor}");
        model = model.replacen(anchor, with, 1);
    }
    model
}

/// Beside a row-set guard the refusal's hint does not send the author into `ESS-COMMAND-009`: it
/// names the row-set form, and that form validates at `ess/38` with no `ESS-COMMAND-004`.
#[test]
fn beside_a_row_set_the_own_subject_hint_names_the_row_set_form_and_that_form_validates() {
    let model = beside_row_set(&reads_own_subject("ess/38"));
    let output = validate_text(&written("own-subject-row-set", &model));
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(stderr.contains("error[ESS-COMMAND-004]"), "{stderr}");
    assert!(!stderr.contains("ESS-COMMAND-009"), "{stderr}");
    for named in [
        "as a row set selecting it by its key",
        "entity: probe.shop.Shelf",
        "shelf_id == input.shelf_id",
        "unknown_instance: true",
    ] {
        assert!(stderr.contains(named), "names {named}: {stderr}");
    }
    assert!(!stderr.contains("when_subject: {predicate:"), "{stderr}");

    let (output, report) =
        validate_json(&written("own-subject-row-set-form", &row_set_form(&model)));
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(
        !warning_codes(&report)
            .iter()
            .any(|code| code == "ESS-COMMAND-004"),
        "{report}"
    );
}
