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
    std::fs::write(
        &embedded,
        suite.to_canonical_json().expect("admitted suite"),
    )
    .expect("the suite writes");

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
    std::fs::write(
        &embedded,
        suite.to_canonical_json().expect("admitted suite"),
    )
    .expect("the suite writes");
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
    std::fs::write(
        &embedded,
        suite.to_canonical_json().expect("admitted suite"),
    )
    .expect("the suite writes");
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

#[test]
fn count_report_skip_only_is_inconclusive_without_actual_failures() {
    let go = go().expect("count-stage verification requires an actual Go toolchain");
    let directory = module("count-skip-only");
    let fixture = std::fs::read_to_string(directory.join("target.go")).unwrap();
    let needle = "func (";
    assert!(fixture.contains(needle));
    std::fs::write(directory.join("target_test.go"), r#"package billing
import ("testing"; "essbilling/essconform")
type unavailable struct { essconform.Target }
func (unavailable) Identity() (essconform.Identity, error) { return essconform.Identity{Name:"skip-control", Version:"1"}, nil }
func (unavailable) BeginScenario(essconform.ScenarioContext) error { return essconform.ErrUnsupported }
func TestConformance(t *testing.T) { essconform.Run(t, func() essconform.Target { return unavailable{} }) }
"#).unwrap();
    let output = Command::new(go)
        .args(["test", "-count=1", "-v", "./..."])
        .current_dir(&directory)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", report_path(&directory))
        .output()
        .unwrap();
    let raw = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{raw}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(scenarios(&raw, "SKIP").len(), 30, "{raw}");
    let text = std::fs::read_to_string(report_path(&directory)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["format"], "ess-conformance-report/2", "{text}");
    assert_eq!(value["counts"]["failed"], 0);
    assert_eq!(value["counts"]["skipped"], 30);
    assert_eq!(value["execution_status"], "inconclusive");
    assert_eq!(value["conformance_status"], "inconclusive");
}

fn predicate_path_and_operator_cases() -> Vec<(serde_json::Value, bool)> {
    use serde_json::json;

    let mut cases = Vec::new();
    for (path, accepted) in [
        ("ready", true),
        ("ready.0_done-now", true),
        ("A_-.0_-", true),
        ("", false),
        ("ready..done", false),
        (".ready", false),
        ("ready.", false),
        ("0ready", false),
        ("réady", false),
    ] {
        for predicate in [
            json!({path: {"eq": true}}),
            json!(path),
            json!(format!("{path} == true")),
            json!(format!("defined ( {path} )")),
            json!(format!("exists({path})")),
            json!(format!("missing({path})")),
        ] {
            cases.push((predicate, accepted));
        }
    }
    for operator in [
        "eq",
        "equals",
        "==",
        "ne",
        "not_equals",
        "!=",
        "lt",
        "<",
        "le",
        "lte",
        "<=",
        "gt",
        ">",
        "ge",
        "gte",
        ">=",
    ] {
        cases.push((json!({"ready": {operator: "other..literal"}}), true));
        cases.push((json!({"ready": {operator: {}}}), false));
    }
    for operator in ["any_of", "in", "one_of", "none_of", "not_in"] {
        for operand in [
            json!(null),
            json!([]),
            json!("a.b"),
            json!([true, 1.5, "a.b"]),
        ] {
            cases.push((json!({"ready": {operator: operand}}), true));
        }
        for operand in [json!([null]), json!([[]]), json!([{}]), json!({})] {
            cases.push((json!({"ready": {operator: operand}}), false));
        }
    }
    for operator in ["exists", "defined"] {
        for operand in [json!(true), json!(false)] {
            cases.push((json!({"ready": {operator: operand}}), true));
        }
        for operand in [json!(null), json!("true"), json!(1), json!([])] {
            cases.push((json!({"ready": {operator: operand}}), false));
        }
    }
    for operand in [json!(null), json!({"future": [null, {}]}), json!([])] {
        cases.push((json!({"ready": {"truthy": operand}}), true));
    }
    cases
}

#[test]
fn count_go_predicate_admission_matches_rust_leaf_grammar() {
    use serde_json::json;

    let directory = count_module("predicate-leaves");
    let mut cases = predicate_path_and_operator_cases();
    for predicate in [
        json!(true),
        json!(false),
        json!("true"),
        json!("false"),
        json!("always"),
        json!("never"),
        json!({}),
        json!([]),
        json!({"ready": {}}),
        json!({"ready": [true, 1, "word"]}),
        json!({"ready": ""}),
        json!({"ready": 1.5}),
        json!({"ready": {"eq": true, "ne": false}}),
        json!("ready == other..literal"),
        json!("ready == 'a.b'"),
        json!("ready == other.value"),
        json!("ready < quoted == text"),
        json!("not defined (ready.0)"),
    ] {
        cases.push((predicate, true));
    }
    for predicate in [
        json!(null),
        json!(1),
        json!("ready == "),
        json!("ready ==\t"),
        json!("'ready' == true"),
        json!({"ready": null}),
        json!({"ready": [null]}),
        json!({"ready": {"eq": []}}),
        json!({"ready": {"eq": null}}),
        json!({"ready": {"eq": true, "future": true}}),
        json!({"not": null}),
    ] {
        cases.push((predicate, false));
    }
    for group in ["all", "and", "all_of", "any", "or", "none", "none_of_these"] {
        cases.push((json!({group: null}), true));
        cases.push((json!({group: [{"ready": {"eq": true}}]}), true));
        cases.push((json!({group: ["ready..done"]}), false));
    }
    let template: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(directory.join("essconform/suite.json")).unwrap(),
    )
    .unwrap();
    let vectors: Vec<_> = cases.into_iter().enumerate().map(|(index, (predicate, accepted))| {
        let mut document = template.clone();
        document["scenarios"]["example.domain/authored/control"]["steps"] = json!([
            {"step":"expect_view", "view":"example.domain.Rows",
                "expectation":{"expect":"satisfies", "predicate":predicate}}
        ]);
        let original = document.to_string();
        assert_eq!(ess_conformance::AdmittedSuite::from_json(&original).is_ok(), accepted,
            "Rust grammar control {index}: {predicate}");
        json!({"name":format!("{index}: {predicate}"), "original":original, "accepted":accepted})
    }).collect();
    std::fs::write(
        directory.join("essconform/predicate_vectors.json"),
        serde_json::to_string_pretty(&vectors).unwrap(),
    )
    .unwrap();
    std::fs::write(directory.join("essconform/predicate_admission_test.go"), r#"package essconform
import (
    _ "embed"
    "encoding/json"
    "testing"
)
//go:embed predicate_vectors.json
var predicateVectors []byte
func TestPredicateAdmission(t *testing.T) {
    var cases []struct { Name string; Original string; Accepted bool }
    if err := json.Unmarshal(predicateVectors, &cases); err != nil { t.Fatal(err) }
    for _, c := range cases {
        _, err := admitSuite(c.Original)
        if (err == nil) != c.Accepted { t.Errorf("%s: admitted=%v, expected=%v, error=%v", c.Name, err == nil, c.Accepted, err) }
    }
    t.Logf("checked %d independently expected Rust/Go predicate originals", len(cases))
}
"#).unwrap();
    let output = invoke_count(
        &directory,
        "predicate-leaf-grammar",
        &[],
        "^TestPredicateAdmission$",
    );
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn count_module(label: &str) -> PathBuf {
    let directory = scratch(label);
    let suite = serde_json::json!({
        "provenance": {"suite_version":"ess-conformance/4", "system":"example", "specification_version":"v1", "spec_digest":"a".repeat(64), "contract_digest":"a".repeat(64)},
        "scenarios": {"example.domain/authored/control": {"purpose":"A controlled terminal command", "steps":[{"step":"execute_command","command":"example.domain.Execute"}],"source":[]}}
    });
    let suite = ConformanceSuite::from_json(&suite.to_string()).unwrap();
    for file in ess_conformance::go::emit(&suite).expect("admitted suite") {
        let path = directory.join(file.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, file.contents).unwrap();
    }
    std::fs::write(directory.join("go.mod"), "module countfixture\n\ngo 1.24\n").unwrap();
    std::fs::write(directory.join("essconform/count_test.go"), r#"package essconform
import ("errors"; "os"; "runtime"; "strconv"; "testing")
type countTarget struct { Target; mode string }
func (c countTarget) Identity() (Identity,error) { return Identity{Name:"count-fixture",Version:"1"},nil }
func (c countTarget) BeginScenario(ScenarioContext) error { if c.mode=="begin-skip" { return ErrUnsupported }; if c.mode=="begin-error" {return errors.New("begin control")};return nil }
func (c countTarget) EndScenario(ScenarioContext) error { if c.mode=="teardown" {return ErrUnsupported};return nil }
func (c countTarget) ExecuteCommand(CommandRequest) (CommandResult,error) {
    switch c.mode {case "skip","teardown":return CommandResult{},ErrUnsupported;case "failure":return CommandResult{},errors.New("ordinary control");case "goexit":runtime.Goexit();case "panic":panic("abnormal control")}
    return CommandResult{},nil
}
func TestCount(t *testing.T) {
    if clock:=os.Getenv("COUNT_CLOCK");clock!="" { now,err:=strconv.ParseInt(clock,10,64);if err!=nil{t.Fatal(err)};countReportNow=func()int64{return now} }
    Run(t,func()Target { if marker:=os.Getenv("COUNT_MARKER");marker!=""{if err:=os.WriteFile(marker,[]byte("constructed\n"),0600);err!=nil{panic(err)}};return countTarget{mode:os.Getenv("COUNT_MODE")} })
}
func TestCountUnsigned(t *testing.T) {
    for _,raw:=range []string{"0","9007199254740993","9223372036854775807","9223372036854775808","18446744073709551615"} {
        value,err:=strictJSON(raw);if err!=nil{t.Fatal(err)};n,err:=unsigned(value);if err!=nil{t.Fatal(err)};if strconv.FormatUint(n,10)!=raw{t.Fatal("integer rounded")}
    }
    for _,raw:=range []string{"-1","-0","1.0","1e0","0.5","\"1\"","18446744073709551616"} { value,err:=strictJSON(raw);if err==nil{_,err=unsigned(value)};if err==nil{t.Fatalf("admitted %s",raw)} }
    raw,err:=countCanonical(map[string]any{"z":uint64(18446744073709551615),"a":"e\u0301<>&/\u2028\n\u0001"});if err!=nil{t.Fatal(err)}
    if string(raw)!="{\n  \"a\": \"e\u0301<>&/\u2028\\n\\u0001\",\n  \"z\": 18446744073709551615\n}\n"{t.Fatalf("canonical bytes: %s",raw)}
}
"#).unwrap();
    directory
}

fn invoke_count(
    directory: &Path,
    label: &str,
    env: &[(&str, &str)],
    filter: &str,
) -> std::process::Output {
    let mut command = Command::new(go().expect("count verification requires Go"));
    command
        .args(["test", "-count=1", "-v", "./...", "-run", filter])
        .current_dir(directory)
        .env_remove("ESS_REPORT_FORMAT")
        .env_remove("ESS_CONFORMANCE_STRICT")
        .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
        .env_remove("ESS_REPORT_OUT")
        .env_remove("COUNT_MODE")
        .env_remove("COUNT_CLOCK")
        .env_remove("COUNT_MARKER");
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output().unwrap();
    let log = format!(
        "cwd: {}\ncommand: {command:?}\nexit: {:?}\n{}{}",
        directory.display(),
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let evidence = root().join("target/review-boundaries-8/go-matrix");
    std::fs::create_dir_all(&evidence).unwrap();
    std::fs::write(evidence.join(format!("{label}.log")), log).unwrap();
    output
}

#[test]
fn count_go_actual_producers_keep_skip_error_and_teardown_categories() {
    let directory = count_module("count-matrix");
    let output = invoke_count(&directory, "unsigned", &[], "^TestCountUnsigned$");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for (mode, success, passed, failed, skipped, status) in [
        ("passed", true, 1, 0, 0, "passed"),
        ("skip", true, 0, 0, 1, "inconclusive"),
        ("begin-skip", true, 0, 0, 1, "inconclusive"),
        ("failure", false, 0, 1, 0, "failed"),
        ("begin-error", false, 0, 1, 0, "failed"),
        ("teardown", false, 0, 1, 0, "failed"),
    ] {
        let destination = directory.join(format!("{mode}.json"));
        let output = invoke_count(
            &directory,
            mode,
            &[
                ("COUNT_MODE", mode),
                ("ESS_REPORT_FORMAT", "2"),
                ("COUNT_CLOCK", "1788680000000"),
                ("ESS_REPORT_OUT", destination.to_str().unwrap()),
            ],
            "^TestCount$",
        );
        assert_eq!(
            output.status.success(),
            success,
            "{mode}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let raw = std::fs::read_to_string(&destination).unwrap();
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(
            value["counts"],
            serde_json::json!({"total":1,"passed":passed,"failed":failed,"error":0,"unsupported":0,"skipped":skipped})
        );
        assert_eq!(value["execution_status"], status);
        assert_eq!(
            value["conformance_status"],
            if status == "failed" {
                "failed"
            } else {
                "inconclusive"
            }
        );
        let original = std::fs::read_to_string(directory.join("essconform/suite.json")).unwrap();
        let admitted = ess_conformance::AdmittedSuite::from_json(&original).unwrap();
        let report = ess_conformance::CountReport::from_json(&raw, &admitted).unwrap();
        assert_eq!(
            report.to_canonical_json().unwrap(),
            raw,
            "cross-language canonical bytes"
        );
        let export = root()
            .join("target/review-boundaries-8/producer-pairs/go")
            .join(mode);
        std::fs::create_dir_all(&export).unwrap();
        std::fs::write(export.join("suite.json"), original).unwrap();
        std::fs::write(export.join("report.json"), raw).unwrap();
        for relative in [
            "go.mod",
            "essconform/runtime.go",
            "essconform/predicate.go",
            "essconform/suite.go",
            "essconform/suite.json",
            "essconform/count_test.go",
        ] {
            let to = export.join("generated").join(relative);
            std::fs::create_dir_all(to.parent().unwrap()).unwrap();
            std::fs::copy(directory.join(relative), to).unwrap();
        }
        let expected = serde_json::json!({"producer":"actual generated Go Run + writeCountReport", "command":format!("COUNT_MODE={mode} COUNT_CLOCK=1788680000000 ESS_REPORT_FORMAT=2 ESS_REPORT_OUT=report.json go test -count=1 -v ./... -run '^TestCount$'"),"runtime":String::from_utf8(Command::new("go").arg("version").output().unwrap().stdout).unwrap(),"report":"report.json","suite":"suite.json","generated_module":"generated","expected_model_digest":"a".repeat(64),"expected_selected_ids":["example.domain/authored/control"],"expected":{"total":1,"passed":passed,"failed":failed,"error":0,"unsupported":0,"skipped":skipped,"execution_status":status,"conformance_status":if status=="failed"{"failed"}else{"inconclusive"}},"clock":1_788_680_000_000_u64});
        std::fs::write(
            export.join("fixture.json"),
            serde_json::to_string_pretty(&expected).unwrap() + "\n",
        )
        .unwrap();
    }
    for mode in ["passed", "skip"] {
        let output = invoke_count(
            &directory,
            &format!("strict-{mode}"),
            &[
                ("COUNT_MODE", mode),
                ("ESS_REPORT_FORMAT", "2"),
                ("ESS_CONFORMANCE_STRICT", "1"),
            ],
            "^TestCount$",
        );
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("strict conformance"));
    }
}

#[test]
fn count_go_refusals_precede_targets_and_incomplete_runs_never_publish() {
    let directory = count_module("count-refusals");
    let marker = directory.join("constructed");
    let destination = directory.join("report.json");
    for (index, settings) in [
        vec![("ESS_REPORT_FORMAT", "3")],
        vec![("ESS_REPORT_FORMAT", "")],
        vec![("ESS_CONFORMANCE_STRICT", "")],
        vec![("ESS_CONFORMANCE_ALLOW_INCOMPLETE", "")],
        vec![("ESS_CONFORMANCE_STRICT", "1")],
        vec![
            ("ESS_REPORT_FORMAT", "2"),
            ("ESS_CONFORMANCE_STRICT", "1"),
            ("ESS_CONFORMANCE_ALLOW_INCOMPLETE", "1"),
        ],
        vec![("ESS_CONFORMANCE_STRICT", "0")],
        vec![("ESS_CONFORMANCE_ALLOW_INCOMPLETE", "true")],
    ]
    .into_iter()
    .enumerate()
    {
        let mut settings = settings;
        settings.push(("COUNT_MARKER", marker.to_str().unwrap()));
        settings.push(("ESS_REPORT_OUT", destination.to_str().unwrap()));
        let output = invoke_count(
            &directory,
            &format!("configuration-{index}"),
            &settings,
            "^TestCount$",
        );
        assert!(!output.status.success());
        assert!(!marker.exists());
        assert!(!destination.exists());
    }
    for (label, mode, clock, filter) in [
        ("filtered", "passed", "0", "^TestCount$/omitted"),
        ("negative-clock", "passed", "-1", "^TestCount$"),
        ("abnormal-goexit", "goexit", "0", "^TestCount$"),
        ("abnormal-panic", "panic", "0", "^TestCount$"),
    ] {
        for with_destination in [false, true] {
            let mut settings = vec![
                ("ESS_REPORT_FORMAT", "2"),
                ("COUNT_MODE", mode),
                ("COUNT_CLOCK", clock),
            ];
            if with_destination {
                settings.push(("ESS_REPORT_OUT", destination.to_str().unwrap()));
            }
            let output = invoke_count(
                &directory,
                &format!("{label}-{with_destination}"),
                &settings,
                filter,
            );
            assert!(!output.status.success(), "{label}");
            assert!(!destination.exists());
        }
    }
    let output = invoke_count(
        &directory,
        "report-io-failure",
        &[
            ("ESS_REPORT_FORMAT", "2"),
            ("ESS_REPORT_OUT", directory.to_str().unwrap()),
        ],
        "^TestCount$",
    );
    assert!(!output.status.success());
    assert_count_suite_refusals(&directory, &marker, &destination);
}

fn assert_count_suite_refusals(directory: &Path, marker: &Path, destination: &Path) {
    let original = std::fs::read_to_string(directory.join("essconform/suite.json")).unwrap();
    let mut cases = Vec::new();
    for version in ["ess-conformance/5", "ess-conformance/99"] {
        cases.push(original.replace("ess-conformance/4", version));
    }
    cases.push(original.replacen("\"steps\": [", "\"unknown\": 1, \"steps\": [", 1));
    cases.push(original.replacen(
        "\"command\": \"example.domain.Execute\"",
        "\"command\": \"example.domain.Execute\", \"comm\\u0061nd\": \"example.domain.Execute\"",
        1,
    ));
    cases.push(original.replace("A controlled terminal command", "\\ud800"));
    let mut old: serde_json::Value = serde_json::from_str(&original).unwrap();
    old["provenance"]["suite_version"] = serde_json::json!("ess-conformance/2");
    old["scenarios"]["example.domain/authored/control"]["steps"] =
        serde_json::json!([{"step":"mark_instant","instant":"start"}]);
    cases.push(old.to_string());
    for (index, bad) in cases.iter().enumerate() {
        std::fs::write(directory.join("essconform/suite.json"), bad).unwrap();
        let output = invoke_count(
            directory,
            &format!("suite-refusal-{index}"),
            &[
                ("ESS_REPORT_FORMAT", "2"),
                ("COUNT_MARKER", marker.to_str().unwrap()),
                ("ESS_REPORT_OUT", destination.to_str().unwrap()),
            ],
            "^TestCount$",
        );
        assert!(!output.status.success());
        assert!(!marker.exists());
        assert!(!destination.exists());
    }
}

#[test]
fn count_retained_runtime_preserves_legacy_behavior_and_does_not_gain_version_checks() {
    let directory = count_module("count-retained-runtime");
    let legacy = include_str!("fixtures/go-count-legacy/runtime.go");
    std::fs::write(directory.join("essconform/runtime.go"), legacy).unwrap();
    let mut fixture = std::fs::read_to_string(directory.join("essconform/count_test.go")).unwrap();
    fixture = fixture.replace("; \"strconv\"", "");
    fixture = fixture
        .lines()
        .filter(|line| !line.contains("if clock:="))
        .collect::<Vec<_>>()
        .join("\n");
    fixture.truncate(fixture.find("func TestCountUnsigned").unwrap());
    std::fs::write(directory.join("essconform/count_test.go"), fixture).unwrap();
    let destination = directory.join("legacy.json");
    let epoch_millis = || {
        u64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        )
        .unwrap()
    };
    let observed_before = epoch_millis();
    let output = invoke_count(
        &directory,
        "retained-runtime-v1",
        &[
            ("COUNT_MODE", "skip"),
            ("ESS_REPORT_FORMAT", "2"),
            ("ESS_REPORT_OUT", destination.to_str().unwrap()),
        ],
        "^TestCount$",
    );
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let observed_after = epoch_millis();
    let bytes = std::fs::read_to_string(&destination).unwrap();
    let report = StandaloneConformanceReport::from_json(&bytes).unwrap();
    assert_eq!(report.format, "ess-conformance-report/1");
    assert_eq!(report.scenarios_total, 1);
    assert_eq!(report.scenarios_failed, 1);
    assert_eq!(report.status, VerificationStatus::Inconclusive);
    assert_eq!(report.to_canonical_json(), bytes);
    let export = root().join("target/review-boundaries-8/producer-pairs/go/legacy");
    std::fs::create_dir_all(&export).unwrap();
    std::fs::write(export.join("report.json"), bytes).unwrap();
    let manifest = serde_json::json!({"producer":"actual retained generated Go Run/writeReport at bd6d82f0d551fcf1cc2ec2eab65aab2fe7539947","command":"COUNT_MODE=skip ESS_REPORT_FORMAT=2 ESS_REPORT_OUT=report.json go test -count=1 -v ./... -run '^TestCount$'","runtime":String::from_utf8(Command::new("go").arg("version").output().unwrap().stdout).unwrap(),"report":"report.json","suite":"suite.json","expected_model_digest":"a".repeat(64),"expected_selected_ids":["example.domain/authored/control"],"expected":{"format":"ess-conformance-report/1","scenarios_total":1,"scenarios_failed":1,"status":"inconclusive","failed_scenarios":["skipped example.domain/authored/control"]},"clock":null,"clock_source":"unmodified legacy time.Now().UnixMilli()","expected_clock_range":[observed_before,observed_after]});
    std::fs::write(
        export.join("fixture.json"),
        serde_json::to_string_pretty(&manifest).unwrap() + "\n",
    )
    .unwrap();
    std::fs::copy(
        directory.join("essconform/suite.json"),
        export.join("suite.json"),
    )
    .unwrap();
    let suite = std::fs::read_to_string(directory.join("essconform/suite.json"))
        .unwrap()
        .replace("ess-conformance/4", "ess-conformance/5");
    std::fs::write(directory.join("essconform/suite.json"), suite).unwrap();
    let output = invoke_count(
        &directory,
        "retained-runtime-future-suite",
        &[
            ("COUNT_MODE", "skip"),
            ("ESS_REPORT_OUT", destination.to_str().unwrap()),
        ],
        "^TestCount$",
    );
    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(destination).unwrap()).unwrap();
    assert_eq!(report["suite_version"], "ess-conformance/5");
    assert_eq!(report["scenarios_failed"], 1);
    // This is an observed old-runtime defect, never a claim that the upgraded runtime admits /5.
}

#[test]
fn count_go_empty_selection_is_inconclusive_and_clock_conversion_is_checked() {
    let directory = count_module("count-empty-and-clock");
    let path = directory.join("essconform/suite.json");
    let mut suite: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    suite["scenarios"] = serde_json::json!({});
    std::fs::write(&path, serde_json::to_string_pretty(&suite).unwrap() + "\n").unwrap();
    let destination = directory.join("report.json");
    for instant in ["0", "9223372036854775807"] {
        let output = invoke_count(
            &directory,
            &format!("empty-clock-{instant}"),
            &[
                ("ESS_REPORT_FORMAT", "2"),
                ("COUNT_CLOCK", instant),
                ("ESS_REPORT_OUT", destination.to_str().unwrap()),
            ],
            "^TestCount$",
        );
        assert!(output.status.success());
        let original = std::fs::read_to_string(&path).unwrap();
        let admitted = ess_conformance::AdmittedSuite::from_json(&original).unwrap();
        let raw = std::fs::read_to_string(&destination).unwrap();
        let report = ess_conformance::CountReport::from_json(&raw, &admitted).unwrap();
        assert_eq!(report.counts().total, 0);
        assert_eq!(
            report.execution_status(),
            ess_conformance::CountStatus::Passed
        );
        assert_eq!(
            report.conformance_status(),
            ess_conformance::CountStatus::Inconclusive
        );
        assert_eq!(report.completed_at(), instant.parse::<u64>().unwrap());
    }
    let output = invoke_count(
        &directory,
        "empty-strict",
        &[("ESS_REPORT_FORMAT", "2"), ("ESS_CONFORMANCE_STRICT", "1")],
        "^TestCount$",
    );
    assert!(!output.status.success());
}
