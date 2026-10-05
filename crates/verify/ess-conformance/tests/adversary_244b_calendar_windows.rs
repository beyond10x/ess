//! Adversary cases, beyond10x/ess#244 part b, pass 1: calendar-window guards
//! (`docs/design/calendar-window-guards.md`) against the synthesized suite, the mutation audit and
//! authored scenarios.
//!
//! Each case states the design sentence it holds the implementation to.

mod support_go;
mod support_occurrence_clock;

use std::collections::BTreeMap;

use ess_conformance::authored::{compile as compile_authored, Source as AuthoredSource};
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, MutantClass, Verdict};
use ess_conformance::synthesize::synthesize;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_primitives::node::Node;
use support_occurrence_clock::{begin, model, outcome, request, text};

const RELEASES: &str = include_str!("fixtures/calendar-windows.yaml");

// ---- a target that reads an instant's written clock as UTC ---------------------------------------

/// A target wrong in one way: every RFC 3339 instant it is sent has its own offset dropped and is
/// read as UTC (`2020-01-06T02:30:00-05:00` is taken as `02:30Z`). That is "comparing text instead of
/// instants": the written clock, not the instant. On an instant already spelled `Z` the rewrite is
/// the identity.
struct WrittenClockAsUtc<T>(T);

/// `text` with a trailing `±HH:MM` replaced by `Z`, where it is an RFC 3339 date-time.
fn drop_offset(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    if bytes.len() < 25 || bytes.get(10) != Some(&b'T') {
        return None;
    }
    let tail = &text[text.len() - 6..];
    let tail_bytes = tail.as_bytes();
    if (tail_bytes[0] == b'+' || tail_bytes[0] == b'-') && tail_bytes[3] == b':' {
        Some(format!("{}Z", &text[..text.len() - 6]))
    } else {
        None
    }
}

fn rewrite(node: &Node) -> Node {
    match node {
        Node::Text(text) => drop_offset(text).map_or_else(|| node.clone(), Node::Text),
        Node::Map(fields) => Node::Map(
            fields
                .iter()
                .map(|(key, value)| (key.clone(), rewrite(value)))
                .collect(),
        ),
        Node::Seq(items) => Node::Seq(items.iter().map(rewrite).collect()),
        other => other.clone(),
    }
}

impl<T: ConformanceTarget> ConformanceTarget for WrittenClockAsUtc<T> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        request.input = request
            .input
            .iter()
            .map(|(key, value)| (key.clone(), rewrite(value)))
            .collect::<BTreeMap<_, _>>();
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
}

/// Design, "The instant is compared, never its spelling", and Synthesis: the boundary instants are
/// "written in UTC with `Z`". A target that reads the written clock as UTC decides otherwise than
/// the specification for any instant written at another offset (asserted first, so the fault is
/// shown real), and the synthesized suite must fail it. It does not: every instant synthesis sends
/// is spelled `Z`, on which that target is exact.
#[test]
fn adv244b_a_target_reading_the_written_clock_fails_a_synthesized_scenario() {
    // The fault is real: 02:30 at -05:00 is Monday 08:30 at +01:00, inside `Schedule`'s window.
    let healthy = Interpreted::for_model(model(RELEASES));
    begin(&healthy);
    let taken = healthy
        .execute_command(request(
            "demo.releases.Schedule",
            &[("starts_at", text("2020-01-06T02:30:00-05:00"))],
        ))
        .expect("decided");
    assert_eq!(outcome(&taken), "scheduled");
    let faulty = WrittenClockAsUtc(Interpreted::for_model(model(RELEASES)));
    begin(&faulty);
    let taken = faulty
        .execute_command(request(
            "demo.releases.Schedule",
            &[("starts_at", text("2020-01-06T02:30:00-05:00"))],
        ))
        .expect("decided");
    assert_eq!(
        outcome(&taken),
        "outside-hours",
        "the fault decides otherwise"
    );

    let suite = synthesize(&model(RELEASES)).suite;
    let run = support_go::rust_outcomes(&suite, &faulty);
    assert!(run.len() >= 8, "{run:#?}");
    let failed = support_go::not_passed(&run);
    assert!(
        !failed.is_empty(),
        "a target reading every instant's written clock as UTC passes all {} synthesized \
         scenarios: no window witness is spelled at any offset but `Z`: {run:#?}",
        run.len()
    );
}

// ---- a window over a stored instant -------------------------------------------------------------

/// `website/docs/reference/predicates.md`, "A calendar window": "A generated suite witnesses a
/// window over an input, or over a stored instant arranged through the input that writes it, a
/// second either side of every `from` and `to` on the week of Monday 2020-01-06". Each boundary
/// fault written into the fixture's stored-instant window (`Promote`) alone must then fail a
/// synthesized scenario.
#[test]
fn adv244b_each_boundary_fault_in_a_stored_instant_window_fails_a_scenario() {
    let promote = "        when_subject:\n          predicate:\n            window: {at: ready_at, days: [mon, tue, wed, thu], from: \"08:00\", to: \"16:00\", offset: \"+01:00\"}";
    assert_eq!(RELEASES.matches(promote).count(), 1);
    let suite = synthesize(&model(RELEASES)).suite;
    let mut missed = Vec::new();
    for (fault, written) in [
        (
            "to inclusive",
            promote.replace(r#"to: "16:00""#, r#"to: "16:01""#),
        ),
        (
            "from exclusive",
            promote.replace(r#"from: "08:00""#, r#"from: "08:01""#),
        ),
        ("offset ignored", promote.replace(r#""+01:00""#, "Z")),
        ("Friday listed", promote.replace("thu]", "thu, fri]")),
        ("Monday dropped", promote.replace("[mon, tue", "[tue")),
    ] {
        let wrong = RELEASES.replacen(promote, &written, 1);
        let run = support_go::rust_outcomes(&suite, &Interpreted::for_model(model(&wrong)));
        if support_go::not_passed(&run).is_empty() {
            missed.push(fault);
        }
    }
    assert_eq!(
        missed,
        Vec::<&str>::new(),
        "faults in the stored-instant window no scenario fails"
    );
}

// ---- two intervals, written as the design says: `any:` of two windows --------------------------

/// Design, "Not in this design": "more than one interval per window (write `any:` of two
/// windows)". The two-interval spelling the design prescribes.
const DESK: &str = r#"format: ess/22
system: demo
version: v1
domain: demo.desk
summary: A desk takes bookings Monday 08:00-12:00 and 13:00-17:00 at +01:00.
errors:
  - {name: demo.desk.Closed, summary: The desk is closed., fields: []}
events:
  - name: demo.desk.Booked
    fields:
      - {name: starts_at, type: Timestamp}
commands:
  - name: demo.desk.Book
    input:
      - {name: starts_at, type: Timestamp}
    outcomes:
      - name: booked
        when:
          any:
            - window: {at: starts_at, days: [mon], from: "08:00", to: "12:00", offset: "+01:00"}
            - window: {at: starts_at, days: [mon], from: "13:00", to: "17:00", offset: "+01:00"}
        emits: [demo.desk.Booked]
        payload:
          demo.desk.Booked: {starts_at: input.starts_at}
      - name: closed
        error: demo.desk.Closed
"#;

/// The faults the design's "Faulty targets" table names, written into the first or the second
/// interval of [`DESK`].
fn desk_faults() -> Vec<(&'static str, String)> {
    let first =
        r#"window: {at: starts_at, days: [mon], from: "08:00", to: "12:00", offset: "+01:00"}"#;
    let second =
        r#"window: {at: starts_at, days: [mon], from: "13:00", to: "17:00", offset: "+01:00"}"#;
    assert!(DESK.contains(first) && DESK.contains(second));
    vec![
        (
            "first interval, `to` inclusive",
            DESK.replacen(first, &first.replace(r#"to: "12:00""#, r#"to: "12:01""#), 1),
        ),
        (
            "second interval, `to` inclusive",
            DESK.replacen(
                second,
                &second.replace(r#"to: "17:00""#, r#"to: "17:01""#),
                1,
            ),
        ),
        (
            "second interval, `from` exclusive",
            DESK.replacen(
                second,
                &second.replace(r#"from: "13:00""#, r#"from: "13:01""#),
                1,
            ),
        ),
    ]
}

/// Design, Faulty targets: "`to` inclusive ... must fail a synthesized scenario under the Rust
/// runner". Held for the two-interval guard the design tells an author to write.
#[test]
fn adv244b_two_intervals_as_any_of_two_windows_fail_each_boundary_fault() {
    let suite = synthesize(&model(DESK)).suite;
    let healthy = support_go::rust_outcomes(&suite, &Interpreted::for_model(model(DESK)));
    assert_eq!(
        support_go::not_passed(&healthy),
        Vec::<&str>::new(),
        "{healthy:#?}"
    );
    let mut missed = Vec::new();
    for (fault, written) in desk_faults() {
        let run = support_go::rust_outcomes(&suite, &Interpreted::for_model(model(&written)));
        if support_go::not_passed(&run).is_empty() {
            missed.push(fault);
        }
    }
    assert_eq!(
        missed,
        Vec::<&str>::new(),
        "faults no synthesized scenario fails (of {} scenarios)",
        healthy.len()
    );
}

/// Design, Lanes, mutation: "both mutants of a window over an input killed by the synthesized
/// suite". Held for each window of the two-interval guard.
#[test]
fn adv244b_two_intervals_as_any_of_two_windows_kill_every_window_mutant() {
    let verdicts = window_mutant_verdicts(DESK, "desk.yaml");
    assert!(verdicts.len() >= 4, "{verdicts:#?}");
    let surviving: Vec<_> = verdicts
        .iter()
        .filter(|(_, _, verdict)| *verdict != Verdict::Killed)
        .collect();
    assert_eq!(surviving.len(), 0, "{surviving:#?}");
}

/// Every guard-boundary mutant whose change names a window: its id, its change and its verdict.
fn window_mutant_verdicts(text: &str, file: &str) -> Vec<(String, String, Verdict)> {
    let mut texts = ess_compiler::source::SourceMap::new();
    texts.insert(file.to_owned(), text.to_owned());
    let raw = ess_domain::spec::RawSpecFile::parse(text).expect("parses");
    let documents = vec![(ess_domain::system::Source::new(file), raw)];
    let original = mutate::compile(documents.clone(), &texts).expect("compiles");
    mutate::mutants(&documents, &[MutantClass::GuardBoundary])
        .iter()
        .filter(|mutant| mutant.change.contains("window("))
        .map(|mutant| {
            let entry = mutate::evaluate(&documents, &texts, mutant, || {
                Interpreted::for_model(original.clone())
            })
            .unwrap_or_else(|refusal| panic!("{}: {refusal:?}", mutant.id));
            (mutant.id.clone(), mutant.change.clone(), entry.verdict)
        })
        .collect()
}

// ---- a window over `input.<field>` in a stored row's predicate ----------------------------------

/// Design, "Where it is admitted": "`at` reads what the site reads: ... a stored field or
/// `input.<path>` in a stored row's predicate". Here the instant is the command's own input, read
/// from a `when_subject:` predicate.
const APPROVALS: &str = r#"format: ess/22
system: demo
version: v1
domain: demo.approvals
summary: A release is approved only for a slot Monday to Thursday 08:00-16:00 at +01:00.
types:
  - {name: demo.approvals.ReleaseId, kind: newtype, of: Uuid}
entities:
  - name: demo.approvals.Release
    identity: {name: release_id, type: demo.approvals.ReleaseId}
    fields:
      - {name: ready_at, type: Timestamp}
    lifecycle:
      initial: Ready
      states: [Ready, Approved]
      terminal: [Approved]
      transitions:
        - {name: approve, from: [Ready], to: Approved}
errors:
  - {name: demo.approvals.OutsideHours, summary: Outside hours., fields: []}
  - {name: demo.approvals.NotApprovable, summary: Only a ready release is approved., fields: []}
  - {name: demo.approvals.UnknownRelease, summary: No release carries the identity., fields: []}
events:
  - name: demo.approvals.Prepared
    fields:
      - {name: release_id, type: demo.approvals.ReleaseId}
  - name: demo.approvals.Approved
    fields:
      - {name: release_id, type: demo.approvals.ReleaseId}
commands:
  - name: demo.approvals.Prepare
    input:
      - {name: ready_at, type: Timestamp}
    outcomes:
      - name: prepared
        creates: demo.approvals.Release
        instance: release_id
        sets: {ready_at: input.ready_at}
        emits: [demo.approvals.Prepared]
        payload:
          demo.approvals.Prepared: {release_id: {generated: true}}
  - name: demo.approvals.Approve
    input:
      - {name: release_id, type: demo.approvals.ReleaseId}
      - {name: slot, type: Timestamp}
    outcomes:
      - name: approved
        when_subject:
          predicate:
            window: {at: input.slot, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}
        moves: demo.approvals.Release.approve
        instance: release_id
        emits: [demo.approvals.Approved]
        payload:
          demo.approvals.Approved: {release_id: input.release_id}
      - name: outside-hours
        error: demo.approvals.OutsideHours
      - name: not-approvable
        wrong_state: true
        error: demo.approvals.NotApprovable
      - name: unknown-release
        unknown_instance: true
        error: demo.approvals.UnknownRelease
views:
  - name: demo.approvals.Releases
    source: demo.approvals.Release
    consistency: read_your_writes
    fields:
      - {name: release_id, type: demo.approvals.ReleaseId}
      - {name: ready_at, type: Timestamp}
      - {name: state, type: demo.approvals.Release.State}
"#;

/// Design, Synthesis: "A window over an input fact is witnessed both ways at each side of each
/// boundary"; Faulty targets: `to` inclusive and the offset ignored each "must fail a synthesized
/// scenario". Held for a window over the command's input read from its `when_subject:` predicate.
#[test]
fn adv244b_a_window_over_input_in_a_stored_row_predicate_fails_each_fault() {
    let window = r#"window: {at: input.slot, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}"#;
    assert!(APPROVALS.contains(window));
    let synthesis = synthesize(&model(APPROVALS));
    let healthy =
        support_go::rust_outcomes(&synthesis.suite, &Interpreted::for_model(model(APPROVALS)));
    assert_eq!(
        support_go::not_passed(&healthy),
        Vec::<&str>::new(),
        "{healthy:#?}"
    );
    for decided in [
        "demo.approvals.Approve/outcome/approved",
        "demo.approvals.Approve/outcome/outside-hours",
    ] {
        assert!(
            healthy.contains_key(decided),
            "{decided} is synthesized: {healthy:#?}\n{:#?}",
            synthesis.refusals
        );
    }
    let mut missed = Vec::new();
    for (fault, written) in [
        (
            "to inclusive",
            window.replace(r#"to: "16:00""#, r#"to: "16:01""#),
        ),
        (
            "from exclusive",
            window.replace(r#"from: "08:00""#, r#"from: "08:01""#),
        ),
        ("offset ignored", window.replace(r#""+01:00""#, "Z")),
    ] {
        let wrong = APPROVALS.replacen(window, &written, 1);
        let run =
            support_go::rust_outcomes(&synthesis.suite, &Interpreted::for_model(model(&wrong)));
        if support_go::not_passed(&run).is_empty() {
            missed.push(fault);
        }
    }
    assert_eq!(
        missed,
        Vec::<&str>::new(),
        "faults no synthesized scenario fails (of {} scenarios)",
        healthy.len()
    );
}

/// Control for the case above: the same model with the window replaced by an ordering over another
/// input field read the same way, `input.seats >= 3`, synthesizes its `approved` branch. Green: the
/// `when_subject:` reading of `input.<field>` is witnessed; the window over it is what is not.
#[test]
fn adv244b_control_an_ordering_over_input_in_a_stored_row_predicate_is_witnessed() {
    let window = r#"window: {at: input.slot, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}"#;
    let ordered = APPROVALS.replacen(window, "input.seats >= 3", 1).replacen(
        "      - {name: slot, type: Timestamp}\n",
        "      - {name: slot, type: Timestamp}\n      - {name: seats, type: Integer}\n",
        1,
    );
    assert_ne!(ordered, APPROVALS);
    let synthesis = synthesize(&model(&ordered));
    let healthy =
        support_go::rust_outcomes(&synthesis.suite, &Interpreted::for_model(model(&ordered)));
    assert!(
        healthy.contains_key("demo.approvals.Approve/outcome/approved"),
        "{healthy:#?}\n{:#?}",
        synthesis.refusals
    );
    assert_eq!(
        healthy
            .get("demo.approvals.Approve/outcome/approved")
            .map(String::as_str),
        Some("passed")
    );
}

// ---- a window whose `from` cannot move a minute later -------------------------------------------

/// A night shift opening at 23:59, and a one-minute window: `window_from_later` answers `None` for
/// both (`from + 1` is 24:00, or equals `to`), yet `boundary_sites` still lists a `guard-boundary`
/// mutant for the leaf, and `swap_strictness` then leaves the window as it was.
const NIGHT: &str = r#"format: ess/22
system: demo
version: v1
domain: demo.night
summary: A night desk opens Monday 23:59 to Tuesday 06:00 at Z, and a one-minute check at 12:00.
errors:
  - {name: demo.night.Closed, summary: The desk is closed., fields: []}
events:
  - name: demo.night.Booked
    fields:
      - {name: starts_at, type: Timestamp}
  - name: demo.night.Checked
    fields:
      - {name: checked_at, type: Timestamp}
commands:
  - name: demo.night.Book
    input:
      - {name: starts_at, type: Timestamp}
    outcomes:
      - name: booked
        when:
          window: {at: starts_at, days: [mon], from: "23:59", to: "06:00", offset: Z}
        emits: [demo.night.Booked]
        payload:
          demo.night.Booked: {starts_at: input.starts_at}
      - name: closed
        error: demo.night.Closed
  - name: demo.night.Check
    input:
      - {name: checked_at, type: Timestamp}
    outcomes:
      - name: checked
        when:
          window: {at: checked_at, days: [mon], from: "12:00", to: "12:01", offset: Z}
        emits: [demo.night.Checked]
        payload:
          demo.night.Checked: {checked_at: input.checked_at}
      - name: missed
        error: demo.night.Closed
"#;

/// Design, Lanes, mutation: "`guard-boundary` moves a window's `from` a minute later". A
/// guard-boundary mutant that changes nothing is reported `survived` against a suite that is not
/// at fault: the audit's survivor count then names a gap no suite can close.
#[test]
fn adv244b_no_window_guard_boundary_mutant_is_the_unchanged_window() {
    let verdicts = window_mutant_verdicts(NIGHT, "night.yaml");
    assert!(
        !verdicts.is_empty(),
        "the two windows have guard-boundary sites"
    );
    let unchanged: Vec<_> = verdicts
        .iter()
        .filter(|(_, change, _)| {
            // "`<before>` becomes `<after>`"
            let parts: Vec<&str> = change.split('`').collect();
            parts.len() >= 4 && parts[1] == parts[3]
        })
        .collect();
    assert_eq!(unchanged.len(), 0, "identity mutants: {unchanged:#?}");
    let survived: Vec<_> = verdicts
        .iter()
        .filter(|(_, _, verdict)| *verdict == Verdict::Survived)
        .collect();
    assert_eq!(survived.len(), 0, "survivors: {survived:#?}");
}

// ---- a window in an authored scenario's suite predicate -----------------------------------------

/// Design, "Where it is admitted": anywhere but a command guard "it is refused as `type_mismatch`
/// ... That keeps windows out of every suite predicate (`satisfies`, observed selections), so no
/// suite format changes." An authored row's `satisfies:` is a suite predicate.
#[test]
fn adv244b_an_authored_satisfies_window_is_refused() {
    let ir = model(RELEASES);
    let mut admitted = Vec::new();
    for at in ["ready_at", "now"] {
        let text = format!(
            "type: ess-scenario/1\ndomain: demo.releases\nscenario: ready-in-hours\nsummary: A \
             prepared release is ready inside promotion hours.\ntimeline:\n  - at: \
             2026-01-05T09:00:00Z\n    command: demo.releases.Prepare\n    input:\n      ready_at: \
             2020-01-06T09:00:00Z\n    outcome: prepared\nassert:\n  - view: \
             demo.releases.Releases\n    satisfies: {{window: {{at: {at}, days: [mon], from: \
             \"08:00\", to: \"16:00\", offset: \"+01:00\"}}}}\n"
        );
        let authored = compile_authored(&ir, &[AuthoredSource::new("ready.yaml", text)]);
        if !authored.scenarios.is_empty() || authored.refusals.is_empty() {
            let mut suite = synthesize(&ir).suite;
            suite.scenarios.extend(authored.scenarios.clone());
            let format = ess_conformance::expression_format::used_by(&suite);
            suite.select_fresh_format_for(&ir);
            let run = support_go::rust_outcomes(&suite, &Interpreted::for_model(model(RELEASES)));
            let verdicts: Vec<String> = run
                .iter()
                .filter(|(id, _)| authored.scenarios.keys().any(|key| key.to_string() == **id))
                .map(|(id, verdict)| format!("{id} {verdict}"))
                .collect();
            admitted.push(format!(
                "at: {at}: {} scenario(s), {} refusal(s), /40 vocabulary seen: {format}, suite \
                 {}, healthy run: {verdicts:?}",
                authored.scenarios.len(),
                authored.refusals.len(),
                suite.provenance.suite_version
            ));
        }
    }
    assert_eq!(
        admitted,
        Vec::<String>::new(),
        "a window entered an authored suite predicate"
    );
}
