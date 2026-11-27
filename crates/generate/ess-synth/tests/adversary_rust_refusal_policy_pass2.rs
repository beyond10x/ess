//! Adversary pass 2 on the Rust dispatch of a refusal-selected failure policy (ess/22,
//! beyond10x/ess#269) and of a plain bounded retry (ess/16, beyond10x/ess#165), after correction 1.
//!
//! The cases run the harness `task refusal-policy-harness` builds (`ESS_REFUSAL_POLICY_HARNESS`),
//! in the line protocol `refusal_policy_runtime.rs` documents; an order whose id starts with
//! `nobuild` makes the harness's escalation builder fail (adversary pass 1). A pump that returns
//! `Err` prints the policy `unmet`. Nothing here builds with cargo.
//!
//! Both cases attack the same seam: an attempt made from the held-back list (the second attempt
//! of an occurrence or later). The pump discards a held attempt's `Err` (`let _ =`), on the
//! promise that "a held-back attempt that fails again stays held". Under a per-refusal dispatch
//! `attempt_…` no longer holds a failure whose port was invoked, so a failure the design page and
//! the CHANGELOG say is reported out of `pump` is then neither held nor reported.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const HARNESS: &str = "ESS_REFUSAL_POLICY_HARNESS";
const RECORD: &str = "demo.ledger.Record";

fn harness() -> PathBuf {
    let Some(path) = std::env::var_os(HARNESS).map(PathBuf::from) else {
        panic!("{HARNESS} is not set: run `task refusal-policy-harness`");
    };
    let metadata = std::fs::metadata(&path).unwrap_or_else(|error| {
        panic!(
            "{HARNESS}={} names no harness ({error}): run `task refusal-policy-harness`",
            path.display()
        )
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            permissions.set_mode(permissions.mode() | 0o755);
            std::fs::set_permissions(&path, permissions).expect("the harness is made executable");
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    path
}

fn run(script: &[&str]) -> Vec<String> {
    let mut child = Command::new(harness())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the harness starts");
    {
        let mut stdin = child.stdin.take().expect("piped");
        for line in script {
            writeln!(stdin, "{line}").expect("the harness reads its script");
        }
    }
    let output = child.wait_with_output().expect("the harness ends");
    assert!(
        output.status.success(),
        "the harness fails: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("the harness prints UTF-8")
        .lines()
        .map(str::to_owned)
        .collect()
}

fn attempt(n: u32, order: &str, answer: &str, policy: &str) -> String {
    format!("attempt {RECORD} {order} {n} {answer} {policy} -")
}

/// Design page, § "Escalation is reached only after a recorded attempt": "A builder failure
/// remains a reported obligation/error, does not emit a partial event and does not restart the
/// original command's retry loop"; CHANGELOG (unreleased): "a failed escalation builder is
/// reported". The interpreter reports it on whichever attempt it happens
/// (`ess-conformance/src/interpret/bindings.rs`, `selected`).
///
/// `busy` selects the bounded retry, so the occurrence is held; its second attempt is answered
/// `at-limit`, which selects the fallback escalation, and the builder fails (`nobuild…`). That
/// failure must be reported by the pump that made the attempt (`unmet`), nothing published, and
/// the command not invoked a third time. Today the held attempt's `Err` is discarded by `pump`
/// and the attempt is not held either: the occurrence and its obligation vanish (`drop`).
#[test]
fn an_escalation_builder_failure_on_a_retried_attempt_is_reported() {
    let lines = run(&["place nobuild2 busy at-limit recorded"]);
    let expected = vec![
        attempt(1, "nobuild2", "busy", "retry"),
        attempt(2, "nobuild2", "at-limit", "unmet"),
        "end nobuild2".to_owned(),
    ];
    assert_eq!(
        lines, expected,
        "a failed escalation builder on a held-back attempt was neither held nor reported"
    );
}

/// CHANGELOG (unreleased): "An unmet obligation of the command port selects no policy and
/// propagates out of `pump`, and that invocation counts against `attempts`"; design page,
/// § Generated Rust dispatch, the same. Correction 1 made the port's invocation count, so the
/// third invocation (`unmet`) reaches `attempts: 3`. The retry did not select it — no policy did —
/// so the work is not lost "as under drop" in silence: the pump that made that attempt reports the
/// unmet obligation (`unmet`), and the scripted `recorded` is never asked for. Today the held
/// attempt at the bound is neither held nor reported (`drop`).
#[test]
fn a_port_unmet_obligation_at_the_bound_is_reported() {
    let lines = run(&["place u9 busy busy unmet recorded"]);
    let expected = vec![
        attempt(1, "u9", "busy", "retry"),
        attempt(2, "u9", "busy", "retry"),
        attempt(3, "u9", "unmet", "unmet"),
        "end u9".to_owned(),
    ];
    assert_eq!(
        lines, expected,
        "an unmet obligation of the port at the bound was neither held nor reported"
    );
}

const MODEL: &str = include_str!("fixtures/refusal-policy-harness/model.yaml.in");

fn ir(text: &str) -> ess_compiler::ir::EssIr {
    let raw = ess_domain::spec::RawSpecFile::parse(text).unwrap();
    let spec = ess_domain::spec::Specification::assemble([(
        ess_domain::system::Source::new("model.yaml"),
        raw,
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    ess_compiler::resolve::compile(&spec, &ess_compiler::source::SourceMap::new()).unwrap()
}

/// Correction 1, the coordinator's decision: "an invocation of the port counts against
/// `attempts:` whatever it answers"; the design page's Generated Rust bullet: "That invocation
/// still counts against `attempts`: an `at_least_once` binding holds the occurrence for the next
/// pump while it is below the bound."
///
/// A bounded retry whose every refusal is `final:` is legal (`final` must lie inside what the
/// retry selects, nothing more). `counts_attempts` then answers false, because no refusal is
/// answered with `Retry(Some(_))`, so `deliver_archive_order` takes no count at all, and
/// `port_unmet_hold` holds an unmet port answer with no bound (`made` absent, `bound` None): the
/// port is invoked on every pump without end, where `attempts: 2` says two.
#[test]
fn a_bounded_retry_whose_refusals_are_all_final_still_bounds_the_port_unmet_obligation() {
    let model = MODEL.replace(
        "retry: {attempts: 2, final: [demo.ledger.Unknown]}",
        "retry: {attempts: 2, final: [demo.ledger.Unavailable, demo.ledger.Unknown]}",
    );
    assert_ne!(model, MODEL, "the fixture's plain bounded retry moved");
    let ir = ir(&model);
    let synthesis = ess_synth::synthesize_for(&ir, ess_synth::Target::Rust)
        .unwrap_or_else(|failure| panic!("the Rust target refuses the model: {failure:?}"));
    let (path, artifact) = synthesis
        .artifacts
        .iter()
        .find(|(path, _)| path.ends_with("demo-system/src/lib.rs"))
        .expect("the system crate's lib.rs");
    let text = &artifact.contents;
    let start = text
        .find("fn deliver_archive_order(")
        .unwrap_or_else(|| panic!("{path} has no `deliver_archive_order`"));
    let rest = &text[start..];
    let end = rest[1..].find("\n    fn ").map_or(rest.len(), |at| at + 1);
    let function = &rest[..end];
    assert!(
        function.contains("made + 1 < 2"),
        "{path}: `deliver_archive_order` holds an unmet obligation of the port without counting \
         it against `attempts: 2`:\n{function}"
    );
}
