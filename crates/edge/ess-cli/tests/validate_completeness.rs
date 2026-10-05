//! `ess specify validate --format json` says how complete a specification is (beyond10x/ess#434).
//!
//! The answer is what `ess verify conform synthesize` already prints as text — `refused:`,
//! `outside:` and `note:` lines — carried in one additive `completeness` object, so a merge can be
//! gated on it without scraping text. It is advisory: the exit status does not change, and the
//! object is left out when nothing is owed.

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

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `ess specify validate --path <path> --format json`, with any further arguments, parsed.
fn validated(path: &Path, extra: &[&str]) -> (Output, Value) {
    let path = path.to_str().unwrap();
    let mut args = vec!["specify", "validate", "--path", path, "--format", "json"];
    args.extend_from_slice(extra);
    let output = ess(&args);
    let report = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {output:?}"));
    (output, report)
}

/// `ess verify conform synthesize --path <path>` as text, with any further arguments.
fn synthesized(path: &Path, extra: &[&str]) -> String {
    let path = path.to_str().unwrap();
    let mut args = vec!["verify", "conform", "synthesize", "--path", path];
    args.extend_from_slice(extra);
    let output = ess(&args);
    assert!(output.status.success(), "{output:?}");
    text(&output.stdout)
}

/// Every line `synthesize` prints with this prefix, without it.
fn lines_after<'a>(printed: &'a str, prefix: &str) -> Vec<&'a str> {
    printed
        .lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .collect()
}

/// The number before `label` in the summary line `synthesize` prints last.
fn summary_count(printed: &str, label: &str) -> usize {
    let summary = printed.lines().last().unwrap();
    let before = summary
        .split(", ")
        .find_map(|part| part.strip_suffix(label))
        .unwrap_or_else(|| panic!("no `{label}` in {summary}"));
    before.trim().parse().unwrap()
}

/// A fresh directory under this test target's scratch space.
fn scratch(name: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "validate-completeness/{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(root.join("domains")).unwrap();
    root
}

const SYSTEM: &str = "format: ess/23
system: two
version: v1
summary: An item store and a ping, served by two components.
domains: [two.items, two.ping]
";

const PING: &str = "domain: two.ping
summary: Pings.
events:
  - name: two.ping.Pinged
    fields:
      - {name: note, type: String}
commands:
  - name: two.ping.Ping
    input:
      - {name: note, type: String}
    outcomes:
      - name: pinged
        emits: [two.ping.Pinged]
        payload:
          two.ping.Pinged: {note: input.note}
";

const COMPONENTS: &str = "components:
  - component: item-service
    summary: Keeps items.
    owns: {domains: [two.items]}
    accepts: {commands: [two.items.AddItem]}
    publishes: {events: [two.items.ItemAdded]}
  - component: ping-service
    summary: Answers pings.
    owns: {domains: [two.ping]}
    accepts: {commands: [two.ping.Ping]}
    publishes: {events: [two.ping.Pinged]}
";

/// The items domain. `invariants` is written under the entity; `view_quantity` decides whether the
/// view publishes the field an invariant would read.
fn items(invariants: &str, view_quantity: bool) -> String {
    format!(
        "domain: two.items
summary: Items.
types:
  - {{name: two.items.ItemId, kind: newtype, of: Uuid}}
entities:
  - name: two.items.Item
    identity: {{name: item_id, type: two.items.ItemId}}
    fields:
      - {{name: quantity, type: Integer}}
    lifecycle: {{initial: Listed, states: [Listed], terminal: [Listed]}}
{invariants}events:
  - name: two.items.ItemAdded
    fields:
      - {{name: item_id, type: two.items.ItemId}}
commands:
  - name: two.items.AddItem
    input:
      - {{name: quantity, type: Integer}}
    outcomes:
      - name: added
        creates: two.items.Item
        instance: item_id
        emits: [two.items.ItemAdded]
        payload:
          two.items.ItemAdded: {{item_id: {{generated: true}}}}
        sets:
          quantity: input.quantity
views:
  - name: two.items.Items
    source: two.items.Item
    consistency: read_your_writes
    fields:
      - {{name: item_id, type: two.items.ItemId}}
{}",
        if view_quantity {
            "      - {name: quantity, type: Integer}\n"
        } else {
            ""
        }
    )
}

/// Two domains served by two components. Whole-system synthesis owes nothing; scoped to
/// `item-service` it holds one scenario outside.
fn two(name: &str, items_text: &str) -> PathBuf {
    let root = scratch(name);
    fs::write(root.join("system.yaml"), SYSTEM).unwrap();
    fs::write(root.join("components.yaml"), COMPONENTS).unwrap();
    fs::write(root.join("domains/ping.yaml"), PING).unwrap();
    fs::write(root.join("domains/items.yaml"), items_text).unwrap();
    root
}

/// The same system, where one invariant reads a field no view publishes: synthesis refuses its one
/// scenario, `ESS-SYNTH-011`.
fn one_refusal(name: &str) -> PathBuf {
    two(name, &items("    invariants: [quantity >= 0]\n", false))
}

fn nothing_owed(name: &str) -> PathBuf {
    two(name, &items("", true))
}

/// The keys `validate --format json` has always written for a valid specification with no listed
/// scenarios.
const LEGACY_KEYS: [&str; 9] = [
    "valid",
    "system",
    "version",
    "files_read",
    "domains",
    "commands",
    "events",
    "components",
    "unresolved_references",
];

/// The report's keys, in name order.
fn keys(report: &Value) -> Vec<&str> {
    let mut keys: Vec<&str> = report
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys
}

#[test]
fn validate_json_reports_unscenarioed_constructs() {
    let root = one_refusal("unscenarioed");
    let printed = synthesized(&root, &[]);
    let refused = lines_after(&printed, "refused: ");
    assert_eq!(refused.len(), 1, "{printed}");

    let (output, report) = validated(&root, &[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(report["valid"], true, "{report}");
    let unscenarioed = report["completeness"]["unscenarioed"]
        .as_array()
        .unwrap_or_else(|| panic!("{report}"));
    assert_eq!(unscenarioed.len(), 1, "{report}");
    let entry = &unscenarioed[0];
    let code = entry["code"].as_str().unwrap();
    assert_eq!(code, "ESS-SYNTH-011", "{report}");
    assert!(
        refused[0].starts_with(&format!("refusal[{code}]: ")),
        "the code `conform synthesize` prints: {}",
        refused[0]
    );
    let subject = entry["subject"].as_str().unwrap();
    let scenario = entry["scenario"].as_str().unwrap();
    assert_eq!(subject, "entity two.items.Item", "{report}");
    assert_eq!(
        scenario, "two.items.Item/invariant/after/two.items.AddItem/added",
        "{report}"
    );
    assert!(
        refused[0].contains(&format!("{subject} has no scenario `{scenario}`")),
        "the subject and scenario `conform synthesize` prints: {}",
        refused[0]
    );
    assert_eq!(report["completeness"]["counts"]["unscenarioed"], 1);
}

#[test]
fn validate_json_reports_unanswered_notes() {
    let billing = repo().join("examples/billing");
    let printed = synthesized(&billing, &[]);
    let notes = lines_after(&printed, "note: ");
    assert_eq!(notes.len(), 1, "{printed}");

    let (output, report) = validated(&billing, &[]);
    assert!(output.status.success(), "{output:?}");
    let unanswered: Vec<&str> = report["completeness"]["unanswered"]
        .as_array()
        .unwrap_or_else(|| panic!("{report}"))
        .iter()
        .map(|note| note.as_str().unwrap())
        .collect();
    assert_eq!(unanswered, notes, "{report}");
    assert_eq!(report["completeness"]["counts"]["unanswered"], 1);
}

#[test]
fn validate_json_reports_outside_scenarios() {
    let root = nothing_owed("outside");
    let printed = synthesized(&root, &["--component", "item-service"]);
    let outside = lines_after(&printed, "outside: ");
    assert_eq!(outside.len(), 1, "{printed}");

    let (output, report) = validated(&root, &["--component", "item-service"]);
    assert!(output.status.success(), "{output:?}");
    let listed = report["completeness"]["outside"]
        .as_array()
        .unwrap_or_else(|| panic!("{report}"));
    assert_eq!(listed.len(), 1, "{report}");
    assert_eq!(listed[0]["scenario"], "two.ping.Ping/outcome/pinged");
    let needs: Vec<&str> = listed[0]["needs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|need| need.as_str().unwrap())
        .collect();
    assert_eq!(needs, ["command two.ping.Ping", "event two.ping.Pinged"]);
    // As `conform synthesize --component` prints it.
    let rendered: Vec<String> = needs.iter().map(|need| format!("`{need}`")).collect();
    assert_eq!(
        outside[0],
        format!(
            "`{}` needs {}",
            listed[0]["scenario"].as_str().unwrap(),
            rendered.join(", ")
        )
    );
    assert_eq!(report["completeness"]["counts"]["outside"], 1, "{report}");

    // Without a component nothing is held outside, and here nothing else is owed either.
    let (_, whole) = validated(&root, &[]);
    assert!(whole.get("completeness").is_none(), "{whole}");
}

#[test]
fn completeness_absent_when_nothing_is_owed() {
    let root = nothing_owed("absent");
    let printed = synthesized(&root, &[]);
    assert!(
        lines_after(&printed, "refused: ").is_empty()
            && lines_after(&printed, "outside: ").is_empty()
            && lines_after(&printed, "note: ").is_empty(),
        "{printed}"
    );
    let (output, report) = validated(&root, &[]);
    assert!(output.status.success(), "{output:?}");
    let mut legacy = LEGACY_KEYS;
    legacy.sort_unstable();
    assert_eq!(keys(&report), legacy, "today's JSON, unchanged: {report}");
    assert_eq!(
        text(&ess(&["specify", "validate", "--path", root.to_str().unwrap()]).stdout),
        "two v1 — 4 file(s), valid\n",
        "text mode is unchanged"
    );

    // The billing example prints one actor note, and carries it.
    let (_, billing) = validated(&repo().join("examples/billing"), &[]);
    let completeness = &billing["completeness"];
    assert_eq!(completeness["counts"]["unanswered"], 1, "{billing}");
    assert_eq!(completeness["counts"]["unscenarioed"], 0, "{billing}");
    assert_eq!(completeness["counts"]["outside"], 0, "{billing}");
    assert_eq!(
        completeness["unscenarioed"].as_array().map(Vec::len),
        Some(0),
        "{billing}"
    );
}

#[test]
fn completeness_counts_match_synthesis() {
    let cases: Vec<(PathBuf, Vec<&str>)> = vec![
        (repo().join("examples/gatepass"), vec![]),
        (repo().join("examples/oracle-fixture"), vec![]),
        (repo().join("examples/billing"), vec![]),
        (
            repo().join("examples/billing"),
            vec!["--component", "invoice-service"],
        ),
        (one_refusal("counts"), vec![]),
    ];
    for (path, extra) in cases {
        let printed = synthesized(&path, &extra);
        let (output, report) = validated(&path, &extra);
        assert!(output.status.success(), "{output:?}");
        let counts = &report["completeness"]["counts"];
        let refusals = summary_count(&printed, "refusal(s)");
        assert_eq!(
            counts["unscenarioed"].as_u64(),
            Some(refusals as u64),
            "{}: {printed}\n{report}",
            path.display()
        );
        assert_eq!(
            counts["unscenarioed"].as_u64(),
            Some(lines_after(&printed, "refused: ").len() as u64)
        );
        assert_eq!(
            counts["outside"].as_u64(),
            Some(lines_after(&printed, "outside: ").len() as u64),
            "{}: {report}",
            path.display()
        );
        assert_eq!(
            counts["unanswered"].as_u64(),
            Some(lines_after(&printed, "note: ").len() as u64),
            "{}: {report}",
            path.display()
        );
        for list in ["unscenarioed", "outside", "unanswered"] {
            assert_eq!(
                report["completeness"][list]
                    .as_array()
                    .map(|entries| entries.len() as u64),
                counts[list].as_u64(),
                "{list}: {report}"
            );
        }
    }
}

#[test]
fn completeness_does_not_change_exit_status() {
    // Valid, with a refusal owed: still exit 0.
    let refused = one_refusal("exit-valid");
    let (output, report) = validated(&refused, &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(report["completeness"]["counts"]["unscenarioed"], 1);
    let plain = ess(&["specify", "validate", "--path", refused.to_str().unwrap()]);
    assert_eq!(plain.status.code(), Some(0), "{plain:?}");

    // Invalid: still exit 1, and nothing about completeness, because nothing resolved.
    let invalid = two(
        "exit-invalid",
        &items("", true).replace("emits: [two.items.ItemAdded]", "emits: [two.items.Gone]"),
    );
    let (output, report) = validated(&invalid, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report["compiled"], false, "{report}");
    assert!(report.get("completeness").is_none(), "{report}");
}

#[test]
fn an_unknown_component_is_refused_as_synthesize_refuses_it() {
    let root = nothing_owed("unknown-component");
    let output = ess(&[
        "specify",
        "validate",
        "--path",
        root.to_str().unwrap(),
        "--component",
        "nowhere",
    ]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let synthesize = ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        root.to_str().unwrap(),
        "--component",
        "nowhere",
    ]);
    assert_eq!(synthesize.status.code(), Some(1), "{synthesize:?}");
    assert_eq!(text(&output.stderr), text(&synthesize.stderr));
}

#[test]
fn a_model_conformance_does_not_admit_is_named_not_counted() {
    // `conform synthesize` refuses a model with a finite `Binary64` outright; validate names why
    // instead of reporting nothing owed.
    let root = scratch("unadmitted");
    fs::write(
        root.join("system.yaml"),
        "format: ess/2\nsystem: sample\nversion: v1\ndomains: [sample.data]\ndomain: sample.data\n\
         types:\n  - {name: sample.data.Ratio, kind: newtype, of: Binary64}\n",
    )
    .unwrap();
    let synthesize = ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        root.to_str().unwrap(),
    ]);
    assert!(!synthesize.status.success(), "{synthesize:?}");

    let (output, report) = validated(&root, &[]);
    assert!(output.status.success(), "{output:?}");
    let unsynthesizable = report["completeness"]["unsynthesizable"]
        .as_array()
        .unwrap_or_else(|| panic!("{report}"));
    assert_eq!(unsynthesizable.len(), 1, "{report}");
    assert_eq!(unsynthesizable[0]["reason"], "UnsupportedPrimitive");
    assert!(
        unsynthesizable[0]["path"]
            .as_str()
            .unwrap()
            .contains("sample.data.Ratio"),
        "{report}"
    );
    assert_eq!(report["completeness"]["counts"]["unsynthesizable"], 1);
}

#[test]
fn validate_completeness_guide_section() {
    let page_path = repo().join("website/docs/guides/specify/layout-and-validation.md");
    let page = fs::read_to_string(&page_path).unwrap();
    let refusals = page
        .find("\n## Validate early, read the refusals\n")
        .expect("the page has `## Validate early, read the refusals`");
    let start = page
        .find("\n## How complete is it\n")
        .expect("the page has the heading `## How complete is it`");
    assert!(
        start > refusals,
        "`## How complete is it` comes after `## Validate early, read the refusals`"
    );
    let rest = &page[start + 1..];
    let section = rest[1..].find("\n## ").map_or(rest, |end| &rest[..=end]);
    for name in [
        "`completeness`",
        "`unscenarioed`",
        "`outside`",
        "`unanswered`",
        "`plan.json`",
        "`UNMAPPED:`",
    ] {
        assert!(
            section.contains(name),
            "`## How complete is it` names {name}"
        );
    }
}
