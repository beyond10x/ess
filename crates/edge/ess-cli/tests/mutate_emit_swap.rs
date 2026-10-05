//! `ess verify conform mutate --class emit-swap`: single-event substitution at the CLI
//! (beyond10x/ess#295, `docs/design/mutation-scope-and-known-failures.md`).
//!
//! On billing, the two moving outcomes whose events carry only `invoice_id` swap those events and
//! are killed; the other three single-event sites have no alternative and are listed as
//! unavailable, which keeps the audit at exit 3. Every report here is written by a real runner:
//! the built-in target, the native runner (`verify conform run`) over each emitted suite, and the
//! generated Go runner against `fixtures/go-billing`. The unavailable sites and the exit status are
//! the same through every route. A Go runner patched to discard its event expectations lets both
//! swaps survive, so the kill is the event observation's.
//!
//! The Go cases are skipped, and said out loud, where the machine has no `go`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn mutate(args: &[&str]) -> Output {
    let mut all = vec!["verify", "conform", "mutate"];
    all.extend_from_slice(args);
    ess(&all)
}

fn text(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8")
}

fn s(path: &Path) -> &str {
    path.to_str().expect("a UTF-8 path")
}

fn scratch(label: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("mutate-emit-swap")
        .join(format!("{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn json(path: &Path) -> Value {
    serde_json::from_str(&read(path)).expect("JSON")
}

fn tool(name: &str) -> bool {
    let found = Command::new(name)
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success());
    if !found {
        println!("skipped: no `{name}` on PATH, so this case was not run");
    }
    found
}

const ISSUE: &str = "emit-swap/billing.invoice.IssueInvoice/issued/billing.invoice.InvoiceIssued/\
                     billing.invoice.InvoiceCancelled";
const CANCEL: &str = "emit-swap/billing.invoice.CancelInvoice/cancelled/\
                      billing.invoice.InvoiceCancelled/billing.invoice.InvoiceIssued";
const UNAVAILABLE: [&str; 3] = [
    "emit-swap/billing.email.SendEmail/sent/billing.email.EmailSent",
    "emit-swap/billing.invoice.CreateInvoice/accepted/billing.invoice.InvoiceCreated",
    "emit-swap/billing.invoice.PayInvoice/settled/billing.invoice.InvoicePaid",
];

/// The built-in audit of billing's `emit-swap` sites, as JSON written to `out`.
fn direct(out: &Path) -> Output {
    mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--class",
        "emit-swap",
        "--format",
        "json",
        "--report-out",
        s(out),
    ])
}

/// Every suite directory of the emission at `emitted`, the baseline first.
fn suite_dirs(emitted: &Path) -> Vec<PathBuf> {
    let manifest = json(&emitted.join("manifest.json"));
    let mut dirs = vec![emitted.join(manifest["baseline"]["dir"].as_str().unwrap())];
    dirs.extend(
        manifest["mutants"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|it| it["dir"].as_str().map(|dir| emitted.join(dir))),
    );
    dirs
}

fn emit(emitted: &Path) {
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--class",
        "emit-swap",
        "--emit",
        s(emitted),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output));
    assert!(
        stdout(&output).contains(
            "3 selected site(s) have no mutant and are listed as unavailable_sites: single-event \
             substitution is not audited there"
        ),
        "{}",
        text(&output)
    );
    let manifest = json(&emitted.join("manifest.json"));
    assert_eq!(manifest["format"], "ess-mutation-manifest/4");
    let ids: Vec<&str> = manifest["unavailable_sites"]
        .as_array()
        .unwrap()
        .iter()
        .map(|site| site["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, UNAVAILABLE);
}

fn collect(emitted: &Path, out: &Path) -> Output {
    mutate(&[
        "--collect",
        s(emitted),
        "--format",
        "json",
        "--report-out",
        s(out),
    ])
}

fn verdict(report: &Value, id: &str) -> String {
    report["mutants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == id)
        .unwrap_or_else(|| panic!("`{id}` in {report:#}"))["verdict"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// What two routes must agree on: the mutants' verdicts and killers, the counts, and the
/// unavailable sites, byte for byte.
fn agreed(report: &Value) -> Value {
    let mut kept = report.clone();
    let object = kept.as_object_mut().unwrap();
    object.remove("implementation");
    kept
}

#[test]
fn billings_swaps_are_killed_and_its_sites_without_an_alternative_exit_three() {
    let directory = scratch("direct");
    let out = directory.join("report.json");
    let output = direct(&out);
    assert_eq!(output.status.code(), Some(3), "{}", text(&output));
    assert_eq!(
        output.stdout,
        std::fs::read(&out).unwrap(),
        "JSON is the report bytes"
    );
    let report = json(&out);
    assert_eq!(report["format"], "ess-mutation-report/4");
    assert_eq!(report["counts"]["killed"], 2);
    assert_eq!(report["counts"]["mutants"], 2);
    assert_eq!(report["counts"]["stillborn"], 0);
    for id in [ISSUE, CANCEL] {
        assert_eq!(verdict(&report, id), "killed");
    }
    let sites = report["unavailable_sites"].as_array().unwrap();
    assert_eq!(sites.len(), 3);
    for site in sites {
        assert_eq!(site["reason"], "no_compatible_event_alternative");
    }

    let output = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--class",
        "emit-swap",
    ]);
    assert_eq!(output.status.code(), Some(3), "{}", text(&output));
    let printed = stdout(&output);
    assert!(
        printed.lines().next().unwrap().ends_with("; 3 unavailable"),
        "{printed}"
    );
    for id in UNAVAILABLE {
        assert!(
            printed.lines().any(|line| line.starts_with(&format!(
                "unavailable {id}: no_compatible_event_alternative — single-event substitution \
                 was not audited here"
            ))),
            "{id}: {printed}"
        );
    }
    assert!(
        printed.contains(&format!(
            "killed {ISSUE}: `emits: [billing.invoice.InvoiceIssued]` becomes `emits: \
             [billing.invoice.InvoiceCancelled]` — by "
        )),
        "{printed}"
    );
}

/// The swap and drop of one class each run together: the unavailable sites still exit 3 although
/// every mutant that ran was killed.
#[test]
fn an_unavailable_site_keeps_an_otherwise_killed_audit_at_exit_three() {
    let error_swap = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--class",
        "error-swap",
    ]);
    assert_eq!(error_swap.status.code(), Some(0), "{}", text(&error_swap));
    let both = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--class",
        "error-swap",
        "--class",
        "emit-swap",
    ]);
    assert_eq!(both.status.code(), Some(3), "{}", text(&both));
    assert!(
        stdout(&both).starts_with(
            "mutation audit of billing v3 against billing-reference: 7 mutant(s), 7 killed, 0 \
             survived"
        ),
        "{}",
        text(&both)
    );
}

#[test]
fn the_native_runner_through_emit_and_collect_scores_what_the_built_in_audit_scores() {
    let directory = scratch("native");
    let direct_out = directory.join("direct.json");
    let direct_run = direct(&direct_out);
    let emitted = directory.join("emitted");
    emit(&emitted);
    for dir in suite_dirs(&emitted) {
        let output = ess(&[
            "verify",
            "conform",
            "run",
            "--target",
            "billing",
            "--report-format",
            "2",
            "--suite",
            s(&dir.join("suite.json")),
            "--report-out",
            s(&dir.join("report.json")),
        ]);
        assert!(dir.join("report.json").is_file(), "{}", text(&output));
    }
    let collected_out = directory.join("collected.json");
    let collected = collect(&emitted, &collected_out);
    assert_eq!(
        collected.status.code(),
        direct_run.status.code(),
        "{}",
        text(&collected)
    );
    assert_eq!(collected.status.code(), Some(3));
    assert_eq!(agreed(&json(&collected_out)), agreed(&json(&direct_out)));
    // And the text route agrees on every unavailable line.
    let lines = |output: &Output| -> Vec<String> {
        stdout(output)
            .lines()
            .filter(|line| line.starts_with("unavailable "))
            .map(str::to_owned)
            .collect()
    };
    let direct_text = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--class",
        "emit-swap",
    ]);
    let collected_text = mutate(&["--collect", s(&emitted)]);
    assert_eq!(lines(&collected_text), lines(&direct_text));
    assert_eq!(lines(&direct_text).len(), 3);
}

// ---- the generated Go runner ---------------------------------------------------------------------

/// The billing Go package `synthesize --target go` emits, with the hand-written billing target.
fn go_billing(directory: &Path) -> PathBuf {
    let module = directory.join("go-billing");
    let output = ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        "examples/billing",
        "--target",
        "go",
        "--out",
        s(&module),
    ]);
    assert!(output.status.success(), "{}", text(&output));
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/go-billing");
    for file in ["target.go", "target_test.go"] {
        std::fs::copy(fixture.join(file), module.join(file)).unwrap();
    }
    std::fs::write(module.join("go.mod"), "module essbilling\n\ngo 1.24\n").unwrap();
    module
}

/// Runs the Go package over every suite of the emission, writing each report/2 beside its suite.
fn go_over_emission(module: &Path, emitted: &Path) {
    for dir in suite_dirs(emitted) {
        std::fs::copy(dir.join("suite.json"), module.join("essconform/suite.json")).unwrap();
        let output = Command::new("go")
            .args(["test", "-count=1", "-run", "^TestConformance$", "./..."])
            .current_dir(module)
            .env("GOWORK", "off")
            .env("GOFLAGS", "-mod=mod")
            .env("GOPROXY", "off")
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", dir.join("report.json"))
            .env_remove("ESS_BREAK")
            .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
            .env_remove("ESS_CONFORMANCE_STRICT")
            .output()
            .expect("go test runs");
        assert!(
            dir.join("report.json").is_file(),
            "{}: {}",
            dir.display(),
            text(&output)
        );
        let counts = &json(&dir.join("report.json"))["counts"];
        println!(
            "go runner {}: `{}`; report/2 counts {counts}",
            dir.strip_prefix(emitted).unwrap().display(),
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .rfind(|line| line.starts_with("ok ") || line.starts_with("FAIL"))
                .unwrap_or_default()
        );
    }
}

/// The Go runtime's event expectations, each answered as met without looking: a runner that
/// discards them.
const DISCARDED: [(&str, &str); 4] = [
    (
        "\tcase \"expect_event\":\n\t\treturn r.expectEvent(index, step)\n",
        "\tcase \"expect_event\":\n\t\treturn true\n",
    ),
    (
        "\tcase \"expect_event_values\":\n\t\treturn r.expectEventValues(index, step)\n",
        "\tcase \"expect_event_values\":\n\t\treturn true\n",
    ),
    (
        "\tcase \"expect_no_event\":\n\t\treturn r.expectNoEvent(index, step)\n",
        "\tcase \"expect_no_event\":\n\t\treturn true\n",
    ),
    (
        "\tcase \"eventually_event\":\n\t\treturn r.eventuallyEvent(index, step)\n",
        "\tcase \"eventually_event\":\n\t\treturn true\n",
    ),
];

#[test]
fn the_generated_go_runner_kills_the_swaps_through_their_events_with_the_same_unavailable_sites() {
    if !tool("go") {
        return;
    }
    let directory = scratch("go");
    let direct_out = directory.join("direct.json");
    let direct_run = direct(&direct_out);
    let module = go_billing(&directory);
    let emitted = directory.join("emitted");
    emit(&emitted);
    go_over_emission(&module, &emitted);
    let collected_out = directory.join("collected.json");
    let collected = collect(&emitted, &collected_out);
    assert_eq!(collected.status.code(), Some(3), "{}", text(&collected));
    assert_eq!(collected.status.code(), direct_run.status.code());
    let report = json(&collected_out);
    let baseline = json(&emitted.join("baseline/report.json"));
    assert_eq!(baseline["producer_profile"], "go-scenario-status/2");
    for id in [ISSUE, CANCEL] {
        assert_eq!(verdict(&report, id), "killed", "{report:#}");
    }
    assert_eq!(
        report["unavailable_sites"],
        json(&direct_out)["unavailable_sites"],
        "the same unavailable sites through both routes"
    );

    // The same runner with its event expectations discarded: both swaps survive and the audit
    // fails with exit 1.
    let runtime = module.join("essconform/runtime.go");
    let mut source = read(&runtime);
    for (kept, discarded) in DISCARDED {
        assert_eq!(
            source.matches(kept).count(),
            1,
            "one `{kept}` in the runtime"
        );
        source = source.replace(kept, discarded);
    }
    std::fs::write(&runtime, source).unwrap();
    let blind = directory.join("blind");
    emit(&blind);
    go_over_emission(&module, &blind);
    let blind_out = directory.join("blind.json");
    let collected = collect(&blind, &blind_out);
    assert_eq!(collected.status.code(), Some(1), "{}", text(&collected));
    let report = json(&blind_out);
    for id in [ISSUE, CANCEL] {
        assert_eq!(verdict(&report, id), "survived", "{report:#}");
    }
}
