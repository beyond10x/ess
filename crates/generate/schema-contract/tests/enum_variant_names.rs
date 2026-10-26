//! A model enum's variants keep their declared names in the types-only Rust, Go and TypeScript
//! outputs (beyond10x/ess#658): converted to the target's identifier case, each carrying its wire
//! spelling, and refused by name where a declared name has no identifier.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;

use schema_contract::realize::Plan;

const SOURCE: &str = r"format: ess/5
system: probe
version: v1
domains: [probe.flow]
domain: probe.flow
types:
  - name: probe.flow.Status
    kind: enum
    variants:
      - pending
      - in_review
      - {name: done, wire: DONE}
";

fn plan(source: &str) -> Plan {
    Plan::from_model(&model::selection(source, &["probe.flow.Status"])).expect("plan")
}

#[test]
fn variants_keep_declared_names() {
    let rust = plan(SOURCE).rust("probe-flow").expect("rust").declarations;
    for expected in [
        "#[serde(rename = \"pending\")]\n    Pending,",
        "#[serde(rename = \"in_review\")]\n    InReview,",
    ] {
        assert!(
            rust.contains(expected),
            "Rust is missing `{expected}`:\n{rust}"
        );
    }
    assert!(!rust.contains("    V0,"), "no positional variant:\n{rust}");
}

#[test]
fn a_wire_spelling_differs_from_the_name() {
    let rust = plan(SOURCE).rust("probe-flow").expect("rust").declarations;
    assert!(
        rust.contains("#[serde(rename = \"DONE\")]\n    Done,"),
        "`done` is `Done`, renamed to `DONE`:\n{rust}"
    );
}

/// The enum still generates, and every output names its variants by position, as it did before
/// declared names were read: no declared name reaches Rust, Go or TypeScript.
fn assert_positional(source: &str, case: &str) {
    let plan = Plan::from_model(&model::selection(source, &["probe.flow.Status"]))
        .unwrap_or_else(|refused| panic!("{case}: the enum still generates: {refused}"));
    let rust = plan.rust("probe-flow").expect("rust").declarations;
    for expected in ["    V0,", "    V1,", "    V2,"] {
        assert!(
            rust.contains(expected),
            "{case}: Rust keeps `{expected}`:\n{rust}"
        );
    }
    assert!(
        !rust.contains("    Done,") && !rust.contains("    InReview,"),
        "{case}: no declared name in Rust:\n{rust}"
    );
    let go = plan
        .go("flow", "example.invalid/flow")
        .expect("go")
        .declarations;
    assert!(
        go.contains("ProbeFlowStatusV0") && !go.contains("ProbeFlowStatusDone"),
        "{case}: Go keeps positional constants:\n{go}"
    );
    let typescript = plan.typescript().declarations;
    assert!(
        !typescript.contains("export const ProbeFlowStatus"),
        "{case}: TypeScript has no members:\n{typescript}"
    );
}

#[test]
fn a_name_with_no_identifier_keeps_positional_variants() {
    for (case, entry) in [
        ("a leading digit", "'1st'"),
        ("no letter or digit", "{name: '--', wire: dash}"),
        ("the keyword Self", "self"),
    ] {
        let source = SOURCE.replace("      - pending\n", &format!("      - {entry}\n"));
        assert_positional(&source, case);
    }
}

#[test]
fn two_names_with_one_identifier_keep_positional_variants() {
    let source = SOURCE.replace("      - pending\n", "      - InReview\n");
    assert_positional(&source, "InReview and in_review");
}

#[test]
fn go_and_typescript_agree() {
    let go = plan(SOURCE)
        .go("flow", "example.invalid/flow")
        .expect("go")
        .declarations;
    for expected in [
        "\tProbeFlowStatusPending ProbeFlowStatus = \"pending\"",
        "\tProbeFlowStatusInReview ProbeFlowStatus = \"in_review\"",
        "\tProbeFlowStatusDone ProbeFlowStatus = \"DONE\"",
    ] {
        assert!(go.contains(expected), "Go is missing `{expected}`:\n{go}");
    }
    let typescript = plan(SOURCE).typescript().declarations;
    for expected in [
        "export const ProbeFlowStatus = {",
        "  Pending: \"pending\",",
        "  InReview: \"in_review\",",
        "  Done: \"DONE\",",
        "} as const;",
    ] {
        assert!(
            typescript.contains(expected),
            "TypeScript is missing `{expected}`:\n{typescript}"
        );
    }
}

/// A schema bundle declares no variant names, so its enums keep their positional variants.
#[test]
fn a_bundle_enum_keeps_positional_variants() {
    use schema_contract::bundle::{import, Dialect};
    use std::collections::BTreeSet;
    let roots = BTreeSet::from(["Status".to_owned()]);
    let bundle = import(
        &serde_json::json!({"components": {"schemas": {
            "Status": {"type": "string", "enum": ["pending", "in_review"]}
        }}})
        .to_string(),
        &roots,
        Dialect::Draft202012,
    )
    .expect("bundle");
    let plan = Plan::from_bundle(&bundle, &roots).expect("plan");
    let rust = plan.rust("probe").expect("rust").declarations;
    assert!(
        rust.contains("    V0,") && rust.contains("    V1,"),
        "{rust}"
    );
    assert!(!plan
        .typescript()
        .declarations
        .contains("export const Status"));
}

/// A lifecycle state's wire name (`ess/32`, beyond10x/ess#639) is the wire value of the
/// generated state enum's variant, which keeps the declared state's name.
#[test]
fn generated_types_spell_a_state_by_its_wire_name() {
    let source = include_str!("../../ess-gen/tests/fixtures/state-wire.yaml");
    let plan =
        Plan::from_model(&model::selection(source, &["jobs.queue.Job.State"])).expect("plan");
    let rust = plan.rust("jobs-queue").expect("rust").declarations;
    assert!(
        rust.contains("#[serde(rename = \"indexing\")]\n    Indexing,"),
        "Rust decodes `indexing` as `Indexing`:\n{rust}"
    );
    assert!(
        !rust.contains("rename = \"Indexing\""),
        "Rust refuses `Indexing`:\n{rust}"
    );
    let go = plan
        .go("queue", "example.invalid/queue")
        .expect("go")
        .declarations;
    assert!(
        go.contains("\tJobsQueueJobStateIndexing JobsQueueJobState = \"indexing\""),
        "Go spells `Indexing` as `indexing`:\n{go}"
    );
    assert!(
        !go.contains("= \"Indexing\""),
        "Go refuses `Indexing`:\n{go}"
    );
    let typescript = plan.typescript().declarations;
    assert!(
        typescript.contains("  Indexing: \"indexing\","),
        "TypeScript spells `Indexing` as `indexing`:\n{typescript}"
    );
    assert!(
        !typescript.contains("\"Indexing\""),
        "TypeScript admits no `Indexing` value:\n{typescript}"
    );
}
