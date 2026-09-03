//! `ess conform synthesize --target go`, run end to end against a Go implementation.
//!
//! What this exists to rule out is the failure the whole conformance milestone is about, one step
//! further back. A suite that is regenerated on every model change and that nothing can execute is
//! not a weak suite, it is no suite: `ess conform run` reaches only the Rust targets in this
//! workspace, and every adopter's implementation is somewhere else.
//!
//! So the emitted package is held to a real implementation, twice: once correct, where all 29
//! scenarios must pass, and once with a single deliberate defect, where the scenarios responsible
//! for that defect must fail and **no others**. A suite that failed everything would prove nothing
//! about which check caught what.
//!
//! `fixtures/go-billing/target.go` is that implementation — hand-written, small, and not a
//! reference. `ESS_BREAK=negative-total` makes its views publish a negative total, which is exactly
//! what `billing.invoice.Money`'s `amount >= 0` and `Invoice`'s `total.amount >= 0` forbid.
//! `ESS_BREAK=reversed-order` returns the right rows of `billing.invoice.OutstandingInvoices` in
//! the wrong order, which is the defect the view's `order_by:` exists to forbid — and the one that
//! was uncatchable until synthesis arranged a second row for it to be compared against.
//! `ESS_BREAK=one-row` returns the first of those rows and drops the rest, which is right in every
//! value and wrong in its count.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_conformance::scenario::{CommandRef, EventRef, OutcomeRef};
use ess_conformance::{
    ConformanceScenario, ConformanceSuite, InstanceName, Position, ScenarioId, ScenarioStep,
    ScenarioValue, StandaloneConformanceReport, ViewExpectation,
};
use ess_domain::view::{Direction, Ranking};
use ess_primitives::facts::Number;
use ess_primitives::node::Node;
use ess_primitives::verification::VerificationStatus;

/// The repository root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

/// Where Go is, or `None` when this machine has none.
///
/// Skipped rather than failed, and said out loud: a machine without a Go toolchain cannot answer
/// this question, and a test that silently passed there would report the emitter as checked.
fn go() -> Option<PathBuf> {
    let output = Command::new("go").arg("version").output().ok()?;
    output.status.success().then(|| PathBuf::from("go"))
}

/// A directory of this test's own, under the cache rather than the source tree.
fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("ess-go-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

/// Emits the package, copies the fixture beside it, and returns the module directory.
fn module(name: &str) -> PathBuf {
    let directory = scratch(name);

    let emitted = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["conform", "synthesize", "--path"])
        .arg(root().join("examples/billing"))
        .args(["--target", "go", "--out"])
        .arg(&directory)
        .output()
        .expect("the ess binary runs");
    assert!(
        emitted.status.success(),
        "synthesis failed: {}",
        String::from_utf8_lossy(&emitted.stderr)
    );

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/go-billing");
    for file in ["target.go", "target_test.go"] {
        std::fs::copy(fixture.join(file), directory.join(file)).expect("the fixture copies");
    }
    // The module the fixture imports the emitted package from. Written here rather than committed,
    // because it names a directory that does not exist until the emitter has run.
    std::fs::write(directory.join("go.mod"), "module essbilling\n\ngo 1.24\n")
        .expect("the module file writes");
    directory
}

/// Where a run in `directory` writes its `ess-conformance-report/1`.
fn report_path(directory: &Path) -> PathBuf {
    directory.join("report.json")
}

/// The report a run in `directory` wrote, read back through the closed shape the Rust side
/// publishes — so a Go runner that drifted from it fails to parse here rather than being adapted
/// by a workflow system into a claim it never made.
fn report(directory: &Path) -> StandaloneConformanceReport {
    let text = std::fs::read_to_string(report_path(directory)).expect("the runner wrote a report");
    StandaloneConformanceReport::from_json(&text).unwrap_or_else(|error| {
        panic!("the report is not ess-conformance-report/1: {error}\n{text}")
    })
}

/// The digest the emitted `suite.json` carries, which is what the report has to repeat.
fn suite_digest(directory: &Path) -> String {
    let text = std::fs::read_to_string(directory.join("essconform/suite.json"))
        .expect("the emitted suite exists");
    let suite: serde_json::Value = serde_json::from_str(&text).expect("the suite is JSON");
    suite["provenance"]["spec_digest"]
        .as_str()
        .expect("the suite names its digest")
        .to_owned()
}

/// Runs `go test -v` in `directory`, returning whether it passed and what it printed.
///
/// Every run asks for a report, because the report is part of what the emitted runner is held to.
fn go_test(go: &Path, directory: &Path, broken: Option<&str>) -> (bool, String) {
    let mut command = Command::new(go);
    command
        .args(["test", "-v", "./..."])
        .current_dir(directory)
        .env("ESS_REPORT_OUT", report_path(directory));
    if let Some(defect) = broken {
        command.env("ESS_BREAK", defect);
    }
    let output = command.output().expect("go test runs");
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), printed)
}

/// The scenario ids `go test -v` reported at `verdict`.
fn scenarios(printed: &str, verdict: &str) -> Vec<String> {
    let marker = format!("--- {verdict}: TestConformance/");
    let mut found: Vec<String> = printed
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&marker).map(ToOwned::to_owned))
        .map(|line| {
            line.split_once(' ')
                .map_or(line.clone(), |(id, _)| id.to_owned())
        })
        .collect();
    found.sort();
    found
}

#[test]
fn the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite() {
    let Some(go) = go() else {
        eprintln!("no Go toolchain on this machine; the Go emitter is unchecked here");
        return;
    };
    let directory = module("green");
    let (passed, printed) = go_test(&go, &directory, None);

    assert!(passed, "a correct implementation did not pass:\n{printed}");
    assert_eq!(
        scenarios(&printed, "PASS").len(),
        29,
        "every scenario must run, and a suite that skipped them all would also pass:\n{printed}"
    );

    // The report says the same thing the log does, in the shape a workflow system reads. The
    // digest is the field a passing run is worth anything for, so it is checked against the suite
    // rather than against a constant.
    let written = report(&directory);
    assert_eq!(written.status, VerificationStatus::Passed);
    assert_eq!(written.scenarios_total, 29);
    assert_eq!(written.scenarios_failed, 0);
    assert!(written.failed_scenarios.is_empty());
    assert_eq!(written.spec_digest.as_str(), suite_digest(&directory));
    assert_eq!(written.specification, "billing/v3");
    assert_eq!(written.suite_version, "ess-conformance/2");
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others() {
    let Some(go) = go() else {
        eprintln!("no Go toolchain on this machine; the Go emitter is unchecked here");
        return;
    };
    let directory = module("red");
    let (passed, printed) = go_test(&go, &directory, Some("negative-total"));

    assert!(!passed, "a negative total passed the suite:\n{printed}");
    assert_eq!(
        scenarios(&printed, "FAIL"),
        vec![
            "billing.invoice.CancelInvoice/outcome/cancelled",
            "billing.invoice.CreateInvoice/outcome/accepted",
            "billing.invoice.Invoice/invariant/after/billing.invoice.CancelInvoice/cancelled",
            "billing.invoice.Invoice/invariant/after/billing.invoice.CreateInvoice/accepted",
            "billing.invoice.Invoice/invariant/after/billing.invoice.IssueInvoice/issued",
            "billing.invoice.Invoice/invariant/after/billing.invoice.PayInvoice/settled",
            "billing.invoice.Invoice/transition/cancel/by/billing.invoice.CancelInvoice/cancelled",
            "billing.invoice.Invoice/transition/issue/by/billing.invoice.IssueInvoice/issued",
            "billing.invoice.Invoice/transition/settle/by/billing.invoice.PayInvoice/settled",
            "billing.invoice.IssueInvoice/outcome/issued",
            "billing.invoice.Money/invariant/at/billing.invoice.InvoiceById/total",
            "billing.invoice.Money/invariant/at/billing.invoice.OutstandingInvoices/total",
            "billing.invoice.PayInvoice/outcome/settled",
        ],
        "the thirteen scenarios that read a total are the thirteen that must catch a negative one, \
         and a suite that failed more would not be telling anybody which check found it. Six of \
         them read it through an invariant and seven through the value `sets:` says the row holds \
         — a scenario that only found the invoice and never looked at it was in the second group \
         before that block existed:\n{printed}"
    );

    // The report names the same thirteen, as failures, and calls the run failed.
    let written = report(&directory);
    assert_eq!(written.status, VerificationStatus::Failed);
    assert_eq!(written.scenarios_total, 29);
    assert_eq!(written.scenarios_failed, 13);
    let named: Vec<String> = scenarios(&printed, "FAIL")
        .into_iter()
        .map(|id| format!("failed {id}"))
        .collect();
    assert_eq!(written.failed_scenarios, named);
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order() {
    // The check that makes `order_by:` worth declaring, and the one the suite could not make until
    // a scenario arranged two rows for it. The rows are the right rows and every value in them is
    // right; only the order is wrong, so nothing but the declared order can catch it.
    let Some(go) = go() else {
        eprintln!("no Go toolchain on this machine; the Go emitter is unchecked here");
        return;
    };
    let directory = module("reversed");
    let (passed, printed) = go_test(&go, &directory, Some("reversed-order"));

    assert!(
        !passed,
        "a view that answers backwards passed a suite that declares its order:\n{printed}"
    );
    assert_eq!(
        scenarios(&printed, "FAIL"),
        vec![
            "billing.invoice.CancelInvoice/outcome/cancelled",
            "billing.invoice.CreateInvoice/outcome/accepted",
            "billing.invoice.Invoice/transition/cancel/by/billing.invoice.CancelInvoice/cancelled",
            "billing.invoice.Invoice/transition/issue/by/billing.invoice.IssueInvoice/issued",
            "billing.invoice.Invoice/transition/settle/by/billing.invoice.PayInvoice/settled",
            "billing.invoice.IssueInvoice/outcome/issued",
            "billing.invoice.PayInvoice/outcome/settled",
        ],
        "exactly the scenarios that assert `OutstandingInvoices`'s declared order, and no others: \
         a reversed page is the right multiset, so every other check in the suite still holds and \
         a suite that failed more would not be saying which check found it:\n{printed}"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds() {
    // The other half of an ordering claim. `ranked` holds on fewer than two rows by design, so a
    // page that answered with one row would pass every ordering assertion in the suite; the floor
    // beside it is what says the rows were there to be compared. Every row this target does return
    // is right, and in the right order, so nothing else can see the defect.
    let Some(go) = go() else {
        eprintln!("no Go toolchain on this machine; the Go emitter is unchecked here");
        return;
    };
    let directory = module("short");
    let (passed, printed) = go_test(&go, &directory, Some("one-row"));

    assert!(
        !passed,
        "a page that drops rows passed a suite that says how many there are:\n{printed}"
    );
    assert_eq!(
        scenarios(&printed, "FAIL"),
        vec![
            "billing.invoice.CancelInvoice/outcome/cancelled",
            "billing.invoice.CreateInvoice/outcome/accepted",
            "billing.invoice.Invoice/transition/cancel/by/billing.invoice.CancelInvoice/cancelled",
            "billing.invoice.Invoice/transition/issue/by/billing.invoice.IssueInvoice/issued",
            "billing.invoice.Invoice/transition/settle/by/billing.invoice.PayInvoice/settled",
            "billing.invoice.IssueInvoice/outcome/issued",
            "billing.invoice.PayInvoice/outcome/settled",
        ],
        "exactly the scenarios that arranged more than one row in `OutstandingInvoices`:\n{printed}"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

/// A suite step's value: a literal number, wrapped as the model's own dynamic value.
fn number(value: f64) -> Node {
    Node::Number(Number::new(value).expect("a finite witness"))
}

/// `billing.invoice.Money`, as a command input carries one.
fn money(amount: f64) -> Node {
    Node::Map(BTreeMap::from([
        ("amount".to_owned(), number(amount)),
        ("currency".to_owned(), Node::Text("EUR".to_owned())),
    ]))
}

/// Creating one invoice and issuing it, bound under `instance`.
///
/// Written by hand because synthesis will not write one: nothing in the model relates a command's
/// input to the field a view ranks by, so no generator knows which of two invoices the
/// implementation will put first. An adapter that *does* know writes exactly this.
fn create_and_issue(instance: &str, amount: f64) -> Vec<ScenarioStep> {
    let create: CommandRef = "billing.invoice.CreateInvoice".parse().expect("a command");
    let issue: CommandRef = "billing.invoice.IssueInvoice".parse().expect("a command");
    let created: EventRef = "billing.invoice.InvoiceCreated".parse().expect("an event");
    let bound: InstanceName = instance.parse().expect("a lower-kebab instance name");
    vec![
        ScenarioStep::ExecuteCommand {
            command: create.clone(),
            actor: None,
            input: BTreeMap::from([
                (
                    "account_id".to_owned(),
                    ScenarioValue::literal(Node::Text(
                        "00000000-0000-4000-8000-000000000042".to_owned(),
                    )),
                ),
                (
                    "customer_email".to_owned(),
                    ScenarioValue::literal(Node::Text(format!("{instance}@example.com"))),
                ),
                ("amount".to_owned(), ScenarioValue::literal(money(amount))),
            ]),
        },
        ScenarioStep::ExpectOutcome {
            outcome: OutcomeRef::new(create, "accepted".parse().expect("an outcome name")),
        },
        ScenarioStep::CaptureInstance {
            instance: bound.clone(),
            entity: "billing.invoice.Invoice".parse().expect("an entity"),
            event: created,
            field: "invoice_id".to_owned(),
        },
        ScenarioStep::ExecuteCommand {
            command: issue.clone(),
            actor: None,
            input: BTreeMap::from([("invoice_id".to_owned(), ScenarioValue::instance(bound))]),
        },
        ScenarioStep::ExpectOutcome {
            outcome: OutcomeRef::new(issue, "issued".parse().expect("an outcome name")),
        },
    ]
}

/// The declared order of `billing.invoice.OutstandingInvoices`, as the view writes it.
fn issued_at_descending() -> Vec<Ranking> {
    vec![Ranking {
        field: "issued_at".to_owned(),
        direction: Direction::Descending,
    }]
}

/// One positional assertion about `billing.invoice.OutstandingInvoices`.
fn at(position: Position, instance: &str) -> ScenarioStep {
    ScenarioStep::ExpectView {
        view: "billing.invoice.OutstandingInvoices"
            .parse()
            .expect("a view"),
        expectation: ViewExpectation::At {
            order_by: issued_at_descending(),
            position,
            fields: BTreeMap::from([(
                "invoice_id".to_owned(),
                ScenarioValue::instance(instance.parse().expect("an instance name")),
            )]),
        },
    }
}

/// The scenario id of a hand-written check, which is a shape [`ScenarioId`] already has.
fn hand_written(branch: &str) -> ScenarioId {
    ScenarioId::parse(&format!("billing.invoice.IssueInvoice/outcome/{branch}"))
        .expect("a scenario id")
}

#[test]
fn the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view() {
    // What `ViewExpectation::At` is in the vocabulary for, and the one check that can catch the Go
    // runner and the Rust runner disagreeing about it — a variant Go did not implement would be
    // skipped, which is the shape of green this whole milestone exists to rule out.
    //
    // Synthesis writes no positional assertion, for the reason the variant's own documentation
    // gives, so this suite is the emitted one with two scenarios added: an adapter's, in the types
    // an adapter writes them in.
    let Some(go) = go() else {
        eprintln!("no Go toolchain on this machine; the Go emitter is unchecked here");
        return;
    };
    let directory = module("positional");
    let embedded = directory.join("essconform/suite.json");
    let mut suite = ConformanceSuite::from_json(
        &std::fs::read_to_string(&embedded).expect("the emitted suite is readable"),
    )
    .expect("the emitted suite parses");

    // `issued_at desc`, and the fixture issues in scenario order — so the invoice issued second is
    // the one the view puts first, and the one issued first is the one it puts last.
    let mut steps = create_and_issue("earlier", 1.0);
    steps.extend(create_and_issue("later", 2.0));
    steps.push(ScenarioStep::QueryView {
        view: "billing.invoice.OutstandingInvoices"
            .parse()
            .expect("a view"),
        params: BTreeMap::new(),
    });
    steps.push(at(Position::First, "later"));
    steps.push(at(Position::Last, "earlier"));
    steps.push(at(Position::Nth { index: 1 }, "earlier"));
    suite
        .insert(
            hand_written("position-of-the-newest"),
            ConformanceScenario::new(
                "the newest issued invoice is the first row of the ranked view"
                    .parse()
                    .expect("a purpose"),
                steps,
                [],
            ),
        )
        .expect("the id is free");

    // And the same claim about a view that declares no order is a suite defect, not a coin toss:
    // `InvoiceById` says the rows come back in whatever order the implementation has.
    suite
        .insert(
            hand_written("position-without-an-order"),
            ConformanceScenario::new(
                "a position in a view that declares no order names no particular row"
                    .parse()
                    .expect("a purpose"),
                [
                    ScenarioStep::QueryView {
                        view: "billing.invoice.InvoiceById".parse().expect("a view"),
                        params: BTreeMap::new(),
                    },
                    ScenarioStep::ExpectView {
                        view: "billing.invoice.InvoiceById".parse().expect("a view"),
                        expectation: ViewExpectation::At {
                            order_by: Vec::new(),
                            position: Position::First,
                            fields: BTreeMap::new(),
                        },
                    },
                ],
                [],
            ),
        )
        .expect("the id is free");
    std::fs::write(&embedded, suite.to_canonical_json()).expect("the suite writes");

    let (passed, printed) = go_test(&go, &directory, None);
    assert!(
        !passed,
        "the unordered position is a suite defect and must be reported as one:\n{printed}"
    );
    let failed = scenarios(&printed, "FAIL");
    assert_eq!(
        failed,
        vec!["billing.invoice.IssueInvoice/outcome/position-without-an-order"],
        "only the assertion that names no particular row:\n{printed}"
    );
    assert!(
        scenarios(&printed, "PASS")
            .contains(&"billing.invoice.IssueInvoice/outcome/position-of-the-newest".to_owned()),
        "the emitted runner reads `first`, `last` and `nth` the way the Rust runner does:\n{printed}"
    );
    let _ = std::fs::remove_dir_all(&directory);
}
