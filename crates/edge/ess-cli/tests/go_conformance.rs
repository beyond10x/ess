//! `ess conform synthesize --target go`, run end to end against a Go implementation.
//!
//! What this exists to rule out is the failure the whole conformance milestone is about, one step
//! further back. A suite that is regenerated on every model change and that nothing can execute is
//! not a weak suite, it is no suite: `ess conform run` reaches only the Rust targets in this
//! workspace, and every adopter's implementation is somewhere else.
//!
//! So the emitted package is held to a real implementation, twice: once correct, where all 30
//! scenarios must pass, and once with a single deliberate defect, where the scenarios responsible
//! for that defect must fail and **no others**. A suite that failed everything would prove nothing
//! about which check caught what.
//!
//! One of the thirty is not synthesized. `examples/billing/scenarios/` carries an authored scenario,
//! and `ess conform synthesize` compiles it into the same suite — so what this file also shows is
//! that a scenario a person wrote runs on the emitted runner **unchanged**: no step it uses is new,
//! no method of `Target` moved, and the fixture beside it was not touched for it. It earns its place
//! twice over in the matrix below, because it is the only check that catches both of the two defects
//! a page can have while every value in it is right.
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

use ess_conformance::scenario::{CommandRef, EventRef, InstantName, OutcomeRef};
use ess_conformance::{
    ConformanceScenario, ConformanceSuite, Elapsed, InstanceName, Position, ScenarioId,
    ScenarioStep, ScenarioValue, StandaloneConformanceReport, ViewExpectation,
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
        // The authored half, named rather than discovered: a specification directory holds `ess/1`
        // documents and nothing else, so the scenarios a person wrote about this model sit beside
        // it instead of inside it.
        .arg("--scenarios")
        .arg(root().join("examples/billing-scenarios"))
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
        30,
        "every scenario must run, and a suite that skipped them all would also pass:\n{printed}"
    );

    // The report says the same thing the log does, in the shape a workflow system reads. The
    // digest is the field a passing run is worth anything for, so it is checked against the suite
    // rather than against a constant.
    let written = report(&directory);
    assert_eq!(written.status, VerificationStatus::Passed);
    assert_eq!(written.scenarios_total, 30);
    assert_eq!(written.scenarios_failed, 0);
    assert!(written.failed_scenarios.is_empty());
    assert_eq!(written.spec_digest.as_str(), suite_digest(&directory));
    assert_eq!(written.specification, "billing/v3");
    assert_eq!(written.suite_version, "ess-conformance/4");
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
    assert_eq!(written.scenarios_total, 30);
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
            "billing.invoice/authored/outstanding-invoices-rank-latest-first",
        ],
        "exactly the scenarios that assert `OutstandingInvoices`'s declared order, and no others: \
         a reversed page is the right multiset, so every other check in the suite still holds and \
         a suite that failed more would not be saying which check found it. The last of them is \
         the authored one, and it is the row that shows what authoring buys: synthesis can say the \
         rows are in order and will not say which row is first, so the generated checks catch a \
         reversal only through the pair they arranged, and a person's claim about the first row \
         catches it directly:\n{printed}"
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
            "billing.invoice/authored/outstanding-invoices-rank-latest-first",
        ],
        "exactly the scenarios that arranged more than one row in `OutstandingInvoices`, the \
         authored one included: it asserts a first row and a last one, and a page of one row has \
         no last:\n{printed}"
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

/// A window over the invoice this scenario created, and the twenty seconds it must stay quiet for.
///
/// Built here rather than authored in `examples/billing-scenarios/`, for the same reason the
/// positional case above is: the committed suite is wave 4's artifact and a scenario added to it
/// would change what an unrelated test is pinning. What this needs to show is narrower — that the
/// four steps the emitted runner gained really run, against a target with a clock — and a scenario
/// injected into the emitted document shows exactly that.
fn held_window() -> Vec<ScenarioStep> {
    let created: EventRef = "billing.invoice.InvoiceCreated".parse().expect("an event");
    let bridged: InstantName = "created".parse().expect("an instant name");
    let mut steps = create_and_issue("held", 5.0);
    steps.push(ScenarioStep::MarkInstant {
        instant: bridged.clone(),
    });
    steps.push(ScenarioStep::ExpectNotBefore {
        instant: bridged.clone(),
        elapsed: Elapsed::seconds(20),
    });
    // `InvoiceCreated` was published before the mark, so it is outside the window. A target that
    // answered "have you ever published this" rather than "did you publish it in here" fails a
    // claim that is true, which is the one way a windowed negative can be wrong without anybody
    // noticing.
    steps.push(ScenarioStep::ExpectQuiet {
        event: created,
        instant: bridged.clone(),
        elapsed: Elapsed::seconds(20),
    });
    steps.push(ScenarioStep::ExpectWithin {
        instant: bridged,
        elapsed: Elapsed::seconds(60),
    });
    steps
}

/// The emitted suite with one elapsed-time scenario added, written back where the package embeds it.
fn with_a_window(directory: &Path) {
    let embedded = directory.join("essconform/suite.json");
    let mut suite = ConformanceSuite::from_json(
        &std::fs::read_to_string(&embedded).expect("the emitted suite is readable"),
    )
    .expect("the emitted suite parses");
    suite
        .insert(
            hand_written("held-for-twenty-seconds"),
            ConformanceScenario::new(
                "twenty seconds really pass, and nothing is published while they do"
                    .parse()
                    .expect("a purpose"),
                held_window(),
                [],
            ),
        )
        .expect("the id is free");
    std::fs::write(&embedded, suite.to_canonical_json()).expect("the suite writes");
}

#[test]
fn the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves() {
    // The acceptance for the Go half of elapsed time, and both directions of it. Green shows the
    // four steps are executed rather than skipped — a step the emitted runner did not implement
    // would be *skipped*, which is the shape of green this whole milestone exists to rule out. Red
    // shows the claim bites: `never-holds` is a system that fires every timer the instant it is
    // armed, and it is wrong in no other way, so nothing else in the suite can see it.
    let Some(go) = go() else {
        eprintln!("no Go toolchain on this machine; the Go emitter is unchecked here");
        return;
    };
    let held = "billing.invoice.IssueInvoice/outcome/held-for-twenty-seconds";

    let directory = module("window");
    with_a_window(&directory);
    let (passed, printed) = go_test(&go, &directory, None);
    assert!(
        passed,
        "a target with a clock did not pass the window:\n{printed}"
    );
    assert!(
        scenarios(&printed, "PASS").contains(&held.to_owned()),
        "the window scenario has to PASS, not SKIP: a skipped one is what an emitted runner that \
         does not implement the steps would produce, and it would look like nothing was wrong:\n\
         {printed}"
    );
    let _ = std::fs::remove_dir_all(&directory);

    let directory = module("nohold");
    with_a_window(&directory);
    let (passed, printed) = go_test(&go, &directory, Some("never-holds"));
    assert!(
        !passed,
        "a system whose timers all fire immediately passed a suite that claims one waits:\n{printed}"
    );
    assert_eq!(
        scenarios(&printed, "FAIL"),
        vec![held.to_owned()],
        "exactly the scenario that claims a length of time, and no others: the target is right \
         about every value, every branch and every view, and wrong only about how long it \
         took:\n{printed}"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

/// Three issued invoices, and the claim that a reader of the ranked listing stops it after two.
///
/// Built here rather than authored in `examples/billing-scenarios/`, for the reason the window above
/// gives: the committed suite is wave 4's artifact and a scenario added to it would change what an
/// unrelated test is pinning. Three rather than two, because the claim is only worth making where
/// there is a third row a producer could have gone on to build.
fn stopped_scan() -> Vec<ScenarioStep> {
    let mut steps = create_and_issue("first", 1.0);
    steps.extend(create_and_issue("second", 2.0));
    steps.extend(create_and_issue("third", 3.0));
    steps.push(ScenarioStep::ExpectHalt {
        view: "billing.invoice.OutstandingInvoices"
            .parse()
            .expect("a view"),
        params: BTreeMap::new(),
        after: 2,
    });
    steps
}

/// The emitted suite with one early-stop scenario added, written back where the package embeds it.
fn with_a_stop(directory: &Path) {
    let embedded = directory.join("essconform/suite.json");
    let mut suite = ConformanceSuite::from_json(
        &std::fs::read_to_string(&embedded).expect("the emitted suite is readable"),
    )
    .expect("the emitted suite parses");
    suite
        .insert(
            hand_written("stopped-after-two-rows"),
            ConformanceScenario::new(
                "a reader of the ranked listing takes two rows and the producer stops too"
                    .parse()
                    .expect("a purpose"),
                stopped_scan(),
                [],
            ),
        )
        .expect("the id is free");
    std::fs::write(&embedded, suite.to_canonical_json()).expect("the suite writes");
}

#[test]
fn the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing() {
    // The acceptance for the Go half of the early stop, and both directions of it. Green shows the
    // step is executed rather than skipped — a step the emitted runner did not implement would be
    // *skipped*, which is the shape of green this whole feature exists to rule out. Red shows the
    // claim bites: `never-stops` builds every row before the reader sees the first one, and it is
    // wrong in no other way. Its rows are the right rows, in the right order, in the right number,
    // so nothing else in the suite can see it — which is the entire argument for the step existing.
    let Some(go) = go() else {
        eprintln!("no Go toolchain on this machine; the Go emitter is unchecked here");
        return;
    };
    let stopped = "billing.invoice.IssueInvoice/outcome/stopped-after-two-rows";

    let directory = module("scan");
    with_a_stop(&directory);
    let (passed, printed) = go_test(&go, &directory, None);
    assert!(
        passed,
        "a target that reads a listing a row at a time did not pass the early stop:\n{printed}"
    );
    assert!(
        scenarios(&printed, "PASS").contains(&stopped.to_owned()),
        "the early-stop scenario has to PASS, not SKIP: a skipped one is what an emitted runner \
         that does not implement the step would produce, and it would look like nothing was \
         wrong:\n{printed}"
    );
    let _ = std::fs::remove_dir_all(&directory);

    let directory = module("nostop");
    with_a_stop(&directory);
    let (passed, printed) = go_test(&go, &directory, Some("never-stops"));
    assert!(
        !passed,
        "a listing built in full before anybody read a row passed a suite that claims a reader \
         stopped it:\n{printed}"
    );
    assert_eq!(
        scenarios(&printed, "FAIL"),
        vec![stopped.to_owned()],
        "exactly the scenario that claims a reader stopped a scan, and no others: the target is \
         right about every value, every branch, every order and every count, and wrong only about \
         how much it built to answer:\n{printed}"
    );
    let _ = std::fs::remove_dir_all(&directory);
}
