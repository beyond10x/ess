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

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
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

/// Runs `go test -v` in `directory`, returning whether it passed and what it printed.
fn go_test(go: &Path, directory: &Path, broken: Option<&str>) -> (bool, String) {
    let mut command = Command::new(go);
    command.args(["test", "-v", "./..."]).current_dir(directory);
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
            "billing.invoice.Invoice/invariant/after/billing.invoice.CancelInvoice/cancelled",
            "billing.invoice.Invoice/invariant/after/billing.invoice.CreateInvoice/accepted",
            "billing.invoice.Invoice/invariant/after/billing.invoice.IssueInvoice/issued",
            "billing.invoice.Invoice/invariant/after/billing.invoice.PayInvoice/settled",
            "billing.invoice.Money/invariant/at/billing.invoice.InvoiceById/total",
            "billing.invoice.Money/invariant/at/billing.invoice.OutstandingInvoices/total",
        ],
        "the six scenarios that read a total are the six that must catch a negative one, and a \
         suite that failed more would not be telling anybody which check found it:\n{printed}"
    );
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
