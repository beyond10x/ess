//! Adversary pass 1 on the Rust dispatch of a refusal-selected failure policy (ess/22,
//! beyond10x/ess#269) and of a plain bounded retry (ess/16, beyond10x/ess#165).
//!
//! The runtime cases run the harness `task refusal-policy-harness` builds
//! (`ESS_REFUSAL_POLICY_HARNESS`), in the line protocol `refusal_policy_runtime.rs` documents; an
//! order whose id starts with `nobuild` makes the harness's escalation builder fail. The expected
//! lines are the contract of `docs/design/conditional-binding-failure-policies.md`, § "Attempts
//! and actual refusal authority" and § "Escalation is reached only after a recorded attempt".
//! The static case reads the generated system crate; nothing here builds with cargo.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target};

const HARNESS: &str = "ESS_REFUSAL_POLICY_HARNESS";
const RECORD: &str = "demo.ledger.Record";
const RECORDED: &str = "demo.ledger.Recorded";
const MODEL: &str = include_str!("fixtures/refusal-policy-harness/model.yaml.in");

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

fn attempt(n: u32, order: &str, answer: &str, policy: &str, published: &[&str]) -> String {
    let published = if published.is_empty() {
        "-".to_owned()
    } else {
        published.join(",")
    };
    format!("attempt {RECORD} {order} {n} {answer} {policy} {published}")
}

/// Design page: "Escalation is reached only after a recorded attempt … A builder failure remains a
/// reported obligation/error, does not emit a partial event and does not restart the original
/// command's retry loop", and under § Required policy controls "an escalation-builder failure
/// publishes nothing and does not reenter retry". `at-limit` selects the fallback escalation; the
/// builder fails for `nobuild…`. The pump reports the failure (`unmet`) and nothing is published;
/// the command must not be invoked a second time for that occurrence.
#[test]
fn an_escalation_builder_failure_does_not_invoke_the_command_again() {
    let lines = run(&["place nobuild1 at-limit recorded"]);
    let expected = vec![
        attempt(1, "nobuild1", "at-limit", "unmet", &[]),
        "end nobuild1".to_owned(),
    ];
    assert_eq!(
        lines, expected,
        "a failed escalation builder re-entered the command's delivery"
    );
}

/// Design page, § Attempts: "A logical attempt begins at the dispatcher-to-command-port boundary
/// with a fully constructed, admitted input. Count and record it once before calling the port", and
/// "A bounded retry compares its total invocation count for this occurrence with attempts". The
/// second invocation reached the port with a complete input and the port answered with an unmet
/// obligation; it is the second of three attempts, so the third (`busy`) reaches the bound and the
/// occurrence is lost, and the scripted `recorded` is never asked for.
#[test]
fn a_port_invocation_answered_unmet_counts_against_the_bound() {
    let lines = run(&["place u1 busy unmet busy recorded"]);
    let expected = vec![
        attempt(1, "u1", "busy", "retry", &[]),
        attempt(2, "u1", "unmet", "retry", &[]),
        attempt(3, "u1", "busy", "drop", &[]),
        "end u1".to_owned(),
    ];
    assert_eq!(
        lines, expected,
        "an invocation of the port was not counted against `attempts: 3` ({RECORDED} was \
         reached after a fourth invocation)"
    );
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// The counted dispatch adds a parameter `made: u32` to `deliver_<binding>` and
/// `attempt_<binding>`, and the generated transformation is a free function named after the
/// binding, called as `<binding>(event)` inside `deliver_<binding>`. A binding named `made` is
/// legal (`event` and `input` are guarded by `rust::feasibility`); its call then names the `u32`
/// parameter, which does not compile (E0618). Either synthesis refuses the name, or the
/// transformation call is not shadowed.
#[test]
fn a_binding_named_made_is_not_shadowed_by_the_attempt_count() {
    let model = MODEL.replace("notify-ledger", "made");
    let ir = ir(&model);
    let Ok(synthesis) = synthesize_for(&ir, Target::Rust) else {
        return;
    };
    let (path, artifact) = synthesis
        .artifacts
        .iter()
        .find(|(path, _)| path.ends_with("demo-system/src/lib.rs"))
        .expect("the system crate's lib.rs");
    let text = &artifact.contents;
    let start = text
        .find("fn deliver_made(")
        .unwrap_or_else(|| panic!("{path} has no `deliver_made`"));
    let rest = &text[start..];
    let end = rest[1..].find("\n    fn ").map_or(rest.len(), |at| at + 1);
    let function = &rest[..end];
    let signature = function.lines().next().unwrap_or_default();
    assert!(
        !(signature.contains("made: u32") && function.contains(" made(event)")),
        "{path}: `deliver_made` takes `made: u32` and calls the transformation `made(event)`, \
         which names the parameter:\n{function}"
    );
}
