//! `a1_timestamp_sibling_instant_order` in the suite runtimes (`docs/design/expression-family-source22.md`,
//! final review decision 2): an ess/22 invariant ordering two `Timestamp` siblings is carried as
//! `{compare: {…, as: timestamp}}` in suite `/40`, and the Rust, Go and TypeScript runners compare
//! the instants, never the spellings.
//!
//! Every target here publishes the same instants under spellings whose byte order disagrees with
//! their instant order, so a runner comparing spellings gives the opposite verdict: the healthy
//! target publishes `valid_until` at `-02:00`, which spells it lower than a `Z` `valid_from` at the
//! same or a later instant; the broken one publishes it an hour before `valid_from` at `+02:00`,
//! which spells it higher.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

fn model(format: u32) -> String {
    format!(
        r"format: ess/{format}
system: lease
version: v1
domain: lease.window
entities:
  - name: lease.window.Lease
    identity: {{name: lease_id, type: Uuid}}
    fields:
      - {{name: valid_from, type: Timestamp}}
      - {{name: valid_until, type: Timestamp}}
    invariants:
      - valid_until >= valid_from
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
events:
  - name: lease.window.Opened
    fields:
      - {{name: lease_id, type: Uuid}}
commands:
  - name: lease.window.Open
    input:
      - {{name: valid_from, type: Timestamp}}
      - {{name: valid_until, type: Timestamp}}
    outcomes:
      - name: opened
        creates: lease.window.Lease
        instance: lease_id
        emits: [lease.window.Opened]
        payload:
          lease.window.Opened: {{lease_id: {{generated: true}}}}
        sets: {{valid_from: input.valid_from, valid_until: input.valid_until}}
views:
  - name: lease.window.Leases
    source: lease.window.Lease
    consistency: read_your_writes
    fields:
      - {{name: lease_id, type: Uuid}}
      - {{name: valid_from, type: Timestamp}}
      - {{name: valid_until, type: Timestamp}}
"
    )
}

fn ir(format: u32) -> EssIr {
    let raw = RawSpecFile::parse(&model(format)).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("lease.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(22));
    assert_eq!(synthesis.refusals.len(), 0, "{:#?}", synthesis.refusals);
    synthesis.suite
}

const INVARIANT: &str = "lease.window.Lease/invariant/after/lease.window.Open/opened";

// ---- instants, spelled at an offset ----------------------------------------------------------

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let shifted = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * shifted + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// `text`, a `Z` RFC 3339 instant as synthesis writes one, as epoch seconds and its fraction.
fn epoch(text: &str) -> (i64, String) {
    let number = |range: std::ops::Range<usize>| text[range].parse::<i64>().expect("digits");
    let seconds = days_from_civil(number(0..4), number(5..7), number(8..10)) * 86_400
        + number(11..13) * 3600
        + number(14..16) * 60
        + number(17..19);
    let fraction = text[19..].trim_end_matches(['Z', 'z']).to_owned();
    (seconds, fraction)
}

/// The instant `seconds` (with `fraction`) spelled at `offset` minutes east of UTC.
fn spelled(seconds: i64, fraction: &str, offset: i64) -> String {
    let local = seconds + offset * 60;
    let (year, month, day) = civil_from_days(local.div_euclid(86_400));
    let of_day = local.rem_euclid(86_400);
    let sign = if offset < 0 { '-' } else { '+' };
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}{fraction}{sign}{:02}:{:02}",
        of_day / 3600,
        of_day / 60 % 60,
        of_day % 60,
        offset.abs() / 60,
        offset.abs() % 60
    )
}

#[test]
fn the_respelling_names_the_same_instant() {
    use ess_primitives::time::Rfc3339Instant;
    for text in [
        "2020-01-01T00:30:00Z",
        "2020-03-01T01:00:00.25Z",
        "1999-12-31T23:59:59Z",
    ] {
        let (seconds, fraction) = epoch(text);
        for offset in [-120, 120, -330, 840] {
            assert_eq!(
                Rfc3339Instant::parse_rfc3339(&spelled(seconds, &fraction, offset)),
                Rfc3339Instant::parse_rfc3339(text),
                "{text} at {offset}"
            );
        }
    }
}

// ---- the targets -------------------------------------------------------------------------------

/// The interpreter, its view rows re-spelled: healthy keeps every instant, broken moves
/// `valid_until` to an hour before `valid_from`.
struct Leases {
    inner: Interpreted,
    broken: bool,
}

impl ConformanceTarget for Leases {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        for row in &mut result.rows {
            let (Some(Node::Text(from)), Some(Node::Text(until))) = (
                row.get("valid_from").cloned(),
                row.get("valid_until").cloned(),
            ) else {
                continue;
            };
            let respelled = if self.broken {
                let (seconds, fraction) = epoch(&from);
                spelled(seconds - 3600, &fraction, 120)
            } else {
                let (seconds, fraction) = epoch(&until);
                spelled(seconds, &fraction, -120)
            };
            row.insert("valid_until".to_owned(), Node::Text(respelled));
        }
        Ok(result)
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(r)
    }
    fn observe_events(
        &self,
        r: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(r)
    }
}

fn leases(broken: bool) -> Leases {
    Leases {
        inner: Interpreted::for_model(ir(22)),
        broken,
    }
}

fn rust_verdicts<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

// ---- the lanes ---------------------------------------------------------------------------------

#[test]
fn the_sibling_comparison_is_tagged_and_selects_suite40() {
    let suite = suite();
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/40"
    );
    let json = suite.to_canonical_json().expect("admitted");
    assert!(json.contains(r#""as": "timestamp""#), "{json}");
}

#[test]
fn a1_timestamp_sibling_instant_order_in_the_interpreter_and_the_rust_runner() {
    let suite = suite();
    let native = rust_verdicts(&suite, &Interpreted::for_model(ir(22)));
    assert!(
        native.values().all(|status| *status == Status::Passed),
        "{native:#?}"
    );
    let healthy = rust_verdicts(&suite, &leases(false));
    assert_eq!(
        healthy.get(INVARIANT),
        Some(&Status::Passed),
        "{healthy:#?}"
    );
    let broken = rust_verdicts(&suite, &leases(true));
    assert_eq!(broken.get(INVARIANT), Some(&Status::Failed), "{broken:#?}");
}

#[test]
fn a1_timestamp_sibling_instant_order_in_the_go_runner() {
    let suite = suite();
    let healthy = support_go::assert_parity("instant-healthy", &suite, leases(false));
    assert_eq!(
        healthy.get(INVARIANT).map(String::as_str),
        Some("passed"),
        "{healthy:#?}"
    );
    let broken = support_go::assert_parity("instant-broken", &suite, leases(true));
    assert_eq!(
        broken.get(INVARIANT).map(String::as_str),
        Some("failed"),
        "{broken:#?}"
    );
}

/// The lease in JavaScript, re-spelling `valid_until` as [`Leases`] does.
const JS_TARGET: &str = r"const mode = process.env.ESS_TARGET_MODE ?? 'healthy';
const pad = (n, w = 2) => String(n).padStart(w, '0');
function spelled(millis, fraction, offsetMinutes) {
  const local = new Date(millis + offsetMinutes * 60000);
  const sign = offsetMinutes < 0 ? '-' : '+';
  const magnitude = Math.abs(offsetMinutes);
  return `${pad(local.getUTCFullYear(), 4)}-${pad(local.getUTCMonth() + 1)}-${pad(local.getUTCDate())}T` +
    `${pad(local.getUTCHours())}:${pad(local.getUTCMinutes())}:${pad(local.getUTCSeconds())}${fraction}` +
    `${sign}${pad(Math.floor(magnitude / 60))}:${pad(magnitude % 60)}`;
}
function parts(text) {
  const fraction = text.slice(19).replace(/[Zz]$/, '');
  return [Date.parse(text.slice(0, 19) + 'Z'), fraction];
}
class Leases {
  rows = [];
  seq = 0;
  identity() { return { name: 'leases', version: '1' }; }
  beginScenario() { this.rows = []; }
  endScenario() {}
  executeCommand({ command, input }) {
    if (command !== 'lease.window.Open') throw new Error(`unexpected ${command}`);
    this.seq += 1;
    const id = `00000000-0000-4000-8000-${String(this.seq).padStart(12, '0')}`;
    this.rows.push({ lease_id: id, valid_from: input.valid_from, valid_until: input.valid_until });
    return {
      outcome: 'opened',
      consistency: `seq:${this.seq}`,
      directEvents: [{ event: 'lease.window.Opened', payload: { lease_id: id } }],
    };
  }
  queryView() {
    return {
      rows: this.rows.map((row) => {
        if (mode === 'broken') {
          const [from, fraction] = parts(row.valid_from);
          return { ...row, valid_until: spelled(from - 3600000, fraction, 120) };
        }
        const [until, fraction] = parts(row.valid_until);
        return { ...row, valid_until: spelled(until, fraction, -120) };
      }),
    };
  }
  observeEvents() { throw new Error('unused'); }
  configureExternalOutcome() { throw new Error('nothing is external'); }
  redeliverEvent() { throw new Error('no bindings'); }
}
export function makeTarget() { return new Leases(); }
";

fn typescript_verdicts(mode: &str) -> Option<BTreeMap<String, String>> {
    use std::process::Command;
    for name in ["tsc", "node"] {
        if !Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            println!("skipped: no `{name}` on PATH");
            return None;
        }
    }
    let admitted = AdmittedSuite::from_suite(&suite()).expect("admits");
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("expression-timestamp")
        .join(format!("{mode}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for artifact in ess_conformance::ts::emit(admitted.suite()).expect("the package emits") {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
        std::fs::write(path, artifact.contents).expect("writes");
    }
    let dir = root.join(ess_conformance::ts::PACKAGE);
    for (name, contents) in [
        ("target.mjs", JS_TARGET),
        (
            "driver.mjs",
            include_str!("fixtures/typescript-parity-driver.mjs"),
        ),
        (
            "runtime-test.tsconfig.json",
            r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
        ),
    ] {
        std::fs::write(dir.join(name), contents).expect("writes");
    }
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&dir)
        .output()
        .expect("tsc runs");
    assert!(
        compiled.status.success(),
        "{}{}",
        String::from_utf8_lossy(&compiled.stdout),
        String::from_utf8_lossy(&compiled.stderr)
    );
    let report = dir.join("report.json");
    let output = Command::new("node")
        .args(["--test", "driver.mjs"])
        .env("ESS_TARGET_MODE", mode)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .current_dir(&dir)
        .output()
        .expect("node runs");
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let text = std::fs::read_to_string(&report)
        .unwrap_or_else(|_| panic!("the run wrote no report:\n{printed}"));
    let document: serde_json::Value = serde_json::from_str(&text).expect("report/2 is JSON");
    let mut verdicts = BTreeMap::new();
    for (status, ids) in document["outcomes"].as_object().expect("outcomes") {
        for id in ids.as_array().expect("a list") {
            verdicts.insert(id.as_str().expect("an id").to_owned(), status.clone());
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    Some(verdicts)
}

#[test]
fn a1_timestamp_sibling_instant_order_in_the_typescript_runner() {
    let Some(healthy) = typescript_verdicts("healthy") else {
        return;
    };
    assert_eq!(
        healthy.get(INVARIANT).map(String::as_str),
        Some("passed"),
        "{healthy:#?}"
    );
    let broken = typescript_verdicts("broken").expect("tools were found once");
    assert_eq!(
        broken.get(INVARIANT).map(String::as_str),
        Some("failed"),
        "{broken:#?}"
    );
}
