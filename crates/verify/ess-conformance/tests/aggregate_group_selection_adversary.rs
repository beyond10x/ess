//! Adversarial cases for aggregate group selection and exact unscoped views (beyond10x/ess#361,
//! beyond10x/ess#362, `docs/design/aggregate-group-selection.md`).
//!
//! Each case synthesizes a fresh suite (which carries `scenario_initial_state: empty`) and runs its
//! aggregate scenarios against the interpreted target — the specification itself selected as the
//! implementation. A healthy target owns state and query evaluation, so an aggregate scenario that
//! fails or errors against it asserts something the specification does not say.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::{
    scenario::{ScenarioId, ScenarioInitialState, ViewExpectation},
    synthesize::{synthesize, Synthesis},
    AdmittedSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("adv.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The aggregate refusals of a synthesis, by scenario, rendered.
fn aggregate_refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| {
            matches!(
                refusal.code().to_string().as_str(),
                "ESS-SYNTH-016" | "ESS-SYNTH-017"
            )
        })
        .map(ToString::to_string)
        .collect()
}

/// Every aggregate scenario's status against the interpreted target, with the diagnostics of each
/// check that did not pass.
fn interpreted(text: &str) -> (Synthesis, BTreeMap<String, (Status, Vec<String>)>) {
    let model = ir(text);
    let result = synthesize(&model);
    assert_eq!(
        result.suite.provenance.scenario_initial_state,
        Some(ScenarioInitialState::Empty)
    );
    let mut suite = result.suite.clone();
    suite
        .scenarios
        .retain(|id, _| matches!(id, ScenarioId::Aggregate { .. }));
    eprintln!(
        "synthesized: {:?}\nrefused: {:#?}",
        suite
            .scenarios
            .keys()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        result
            .refusals
            .iter()
            .filter(|refusal| refusal
                .scenario
                .as_ref()
                .is_some_and(|id| matches!(id, ScenarioId::Aggregate { .. })))
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let target = ess_conformance::interpret::Interpreted::for_model(model);
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    let statuses = report
        .scenarios
        .into_iter()
        .map(|run| {
            let failing = if run.status == Status::Passed {
                Vec::new()
            } else {
                vec![run.to_string()]
            };
            (run.scenario.to_string(), (run.status, failing))
        })
        .collect();
    (result, statuses)
}

/// The scenarios that did not pass, with why.
fn not_passed(statuses: &BTreeMap<String, (Status, Vec<String>)>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, (status, _))| *status != Status::Passed)
        .map(|(id, (status, why))| format!("{id}: {status:?}\n    {}", why.join("\n    ")))
        .collect()
}

/// The parameters of every read of a scenario, with what each read expects.
fn reads(
    result: &Synthesis,
    id: &str,
) -> Vec<(BTreeMap<String, ScenarioValue>, Vec<ViewExpectation>)> {
    let scenario = result
        .suite
        .scenarios
        .iter()
        .find(|(held, _)| held.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .1;
    let mut out: Vec<(BTreeMap<String, ScenarioValue>, Vec<ViewExpectation>)> = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::QueryView { params, .. } => out.push((params.clone(), Vec::new())),
            ScenarioStep::ExpectView { expectation, .. } => out
                .last_mut()
                .expect("an expectation follows a query")
                .1
                .push(expectation.clone()),
            _ => {}
        }
    }
    out
}

const HEAD: &str = "format: ess/20
system: demo
version: v1
domain: demo.adv
";

/// Selectors whose literal spelling and JSON form can differ, or whose domain is constrained:
/// `Decimal`, a nominal `String`, a walked `Integer`, an `Optional` parameter, the
/// lifecycle state, an enum, and an `Integer` newtype bounded to 5..7.
const TYPED: &str = "types:
  - {name: demo.adv.Team, kind: newtype, of: String}
  - {name: demo.adv.Level, kind: newtype, of: Integer, invariants: ['value >= 5', 'value <= 7']}
  - {name: demo.adv.Kind, kind: enum, variants: [Rush, Plain, Hold]}
events:
  - name: demo.adv.Opened
    fields: [{name: id, type: Uuid}]
  - name: demo.adv.Finished
    fields: [{name: id, type: Uuid}]
entities:
  - name: demo.adv.Item
    identity: {name: id, type: Uuid}
    fields:
      - {name: team, type: demo.adv.Team}
      - {name: price, type: Decimal}
      - {name: at, type: Timestamp}
      - {name: cents, type: Integer}
      - {name: channel, type: Optional<String>}
      - {name: kind, type: demo.adv.Kind}
      - {name: level, type: demo.adv.Level}
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions: [{name: finish, from: [Open], to: Done}]
commands:
  - name: demo.adv.Open
    input:
      - {name: team, type: demo.adv.Team}
      - {name: price, type: Decimal}
      - {name: at, type: Timestamp}
      - {name: cents, type: Integer}
      - {name: channel, type: Optional<String>}
      - {name: kind, type: demo.adv.Kind}
      - {name: level, type: demo.adv.Level}
    outcomes:
      - name: opened
        creates: demo.adv.Item
        instance: id
        sets:
          team: input.team
          price: input.price
          at: input.at
          cents: input.cents
          channel: input.channel
          kind: input.kind
          level: input.level
        emits: [demo.adv.Opened]
        payload: {demo.adv.Opened: {id: {generated: true}}}
  - name: demo.adv.Finish
    input: [{name: id, type: Uuid}]
    outcomes:
      - name: finished
        moves: demo.adv.Item.finish
        instance: id
        emits: [demo.adv.Finished]
        payload: {demo.adv.Finished: {id: input.id}}
      - {name: unavailable, wrong_state: true, refuses: false}
views:
  - name: demo.adv.ByPrice
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: price, type: Decimal}]
    filter: price == param.price
    group_by: [price]
    fields:
      - {name: price, type: Decimal}
      - {name: count, type: Integer, aggregate: {count: {}}}
      - {name: total, type: Integer, aggregate: {sum: cents}}
  - name: demo.adv.ByTeam
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: team, type: demo.adv.Team}]
    filter: team == param.team
    group_by: [team]
    fields:
      - {name: team, type: demo.adv.Team}
      - {name: count, type: Integer, aggregate: {count: {}}}
      - {name: total, type: Integer, aggregate: {sum: cents}}
  - name: demo.adv.ByCents
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: cents, type: Integer}]
    filter: cents == param.cents
    group_by: [cents]
    fields:
      - {name: cents, type: Integer}
      - {name: count, type: Integer, aggregate: {count: {}}}
  - name: demo.adv.ByChannel
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: channel, type: Optional<String>}]
    filter: channel == param.channel
    group_by: [channel]
    fields:
      - {name: channel, type: Optional<String>}
      - {name: count, type: Integer, aggregate: {count: {}}}
  - name: demo.adv.InState
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: state, type: demo.adv.Item.State}]
    filter: state == param.state
    group_by: [state]
    fields:
      - {name: state, type: demo.adv.Item.State}
      - {name: count, type: Integer, aggregate: {count: {}}}
      - {name: total, type: Integer, aggregate: {sum: cents}}
  - name: demo.adv.OfKind
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: kind, type: demo.adv.Kind}]
    filter: kind == param.kind
    group_by: [kind]
    fields:
      - {name: kind, type: demo.adv.Kind}
      - {name: count, type: Integer, aggregate: {count: {}}}
  - name: demo.adv.ByLevel
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: level, type: demo.adv.Level}]
    filter: level == param.level
    group_by: [level]
    fields:
      - {name: level, type: demo.adv.Level}
      - {name: count, type: Integer, aggregate: {count: {}}}
";

/// Every typed selector is either refused by name or runs green against the interpreted target.
#[test]
fn adversary_typed_selectors_pass_against_the_interpreted_target() {
    let (result, statuses) = interpreted(&format!("{HEAD}{TYPED}"));
    let failed = not_passed(&statuses);
    assert_eq!(
        failed.len(),
        0,
        "aggregate scenarios a healthy target does not pass:\n{}\nrefused: {:#?}",
        failed.join("\n"),
        aggregate_refusals(&result)
    );
}

/// A nonmatching read sends a value the parameter's own type admits: the bounded `Level` (5..7)
/// is never asked for a level outside its invariants.
#[test]
fn adversary_a_nonmatching_read_sends_a_valid_parameter() {
    let model = ir(&format!("{HEAD}{TYPED}"));
    let result = synthesize(&model);
    let id = "demo.adv.ByLevel/aggregate";
    if !result
        .suite
        .scenarios
        .keys()
        .any(|held| held.to_string() == id)
    {
        panic!(
            "ByLevel was not synthesized; refusals {:#?}",
            aggregate_refusals(&result)
        );
    }
    let mut invalid = Vec::new();
    for (params, _) in reads(&result, id) {
        for (name, value) in &params {
            let ScenarioValue::Literal { value } = value else {
                continue;
            };
            // `demo.adv.Level`: `value >= 5`, `value <= 7`.
            let valid = matches!(value, ess_primitives::node::Node::Number(number)
                if number.as_i64().is_some_and(|level| (5..=7).contains(&level)));
            if !valid {
                invalid.push(format!("{name} = {value:?}"));
            }
        }
    }
    assert_eq!(
        invalid.len(),
        0,
        "reads send parameters their type refuses: {invalid:?}"
    );
}

/// A command that changes the group key after creation (`Reassign` moves the row to `Done` and
/// rewrites its `team`), so grouping by the key the row was created with is stale.
const RETAGGED: &str = "events:
  - name: demo.adv.Opened
    fields: [{name: id, type: Uuid}]
  - name: demo.adv.Reassigned
    fields: [{name: id, type: Uuid}]
entities:
  - name: demo.adv.Item
    identity: {name: id, type: Uuid}
    fields: [{name: team, type: String}, {name: cents, type: Integer}]
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions: [{name: reassign, from: [Open], to: Done}]
commands:
  - name: demo.adv.Open
    input: [{name: team, type: String}, {name: cents, type: Integer}]
    outcomes:
      - name: opened
        creates: demo.adv.Item
        instance: id
        sets: {team: input.team, cents: input.cents}
        emits: [demo.adv.Opened]
        payload: {demo.adv.Opened: {id: {generated: true}}}
  - name: demo.adv.Reassign
    input: [{name: id, type: Uuid}, {name: team, type: String}]
    outcomes:
      - name: reassigned
        moves: demo.adv.Item.reassign
        instance: id
        sets: {team: input.team}
        emits: [demo.adv.Reassigned]
        payload: {demo.adv.Reassigned: {id: input.id}}
      - {name: unavailable, wrong_state: true, refuses: false}
views:
  - name: demo.adv.OpenByTeam
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: team, type: String}]
    filter: [state == Open, team == param.team]
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: count, type: Integer, aggregate: {count: {}}}
      - {name: total, type: Integer, aggregate: {sum: cents}}
  - name: demo.adv.ByTeamState
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: team, type: String}]
    filter: team == param.team
    group_by: [team, state]
    fields:
      - {name: team, type: String}
      - {name: state, type: demo.adv.Item.State}
      - {name: count, type: Integer, aggregate: {count: {}}}
  - name: demo.adv.ByStateOnly
    source: demo.adv.Item
    consistency: read_your_writes
    group_by: [state]
    fields:
      - {name: state, type: demo.adv.Item.State}
      - {name: count, type: Integer, aggregate: {count: {}}}
      - {name: total, type: Integer, aggregate: {sum: cents}}
";

/// Rows whose key a later command rewrites are grouped under the key they actually hold.
#[test]
fn adversary_a_key_a_later_command_rewrites_is_grouped_as_held() {
    let (result, statuses) = interpreted(&format!("{HEAD}{RETAGGED}"));
    let failed = not_passed(&statuses);
    assert_eq!(
        failed.len(),
        0,
        "aggregate scenarios a healthy target does not pass:\n{}\nrefused: {:#?}",
        failed.join("\n"),
        aggregate_refusals(&result)
    );
}

/// Finite and unscoped domains: a two-variant enum and a `Boolean` alone under a filter (three
/// tuples over two values), an `Optional` key alone, a single-state lifecycle's state, `avg` and
/// `count_distinct` over `Optional` values, and ungrouped views with no `count` or `sum`, empty
/// and not.
const FINITE: &str = "types:
  - {name: demo.adv.Lane, kind: enum, variants: [Left, Right]}
events:
  - name: demo.adv.Opened
    fields: [{name: id, type: Uuid}]
  - name: demo.adv.Finished
    fields: [{name: id, type: Uuid}]
  - name: demo.adv.Logged
    fields: [{name: id, type: Uuid}]
entities:
  - name: demo.adv.Item
    identity: {name: id, type: Uuid}
    fields:
      - {name: lane, type: demo.adv.Lane}
      - {name: urgent, type: Boolean}
      - {name: cents, type: Integer}
      - {name: tip, type: Optional<Integer>}
      - {name: note, type: Optional<String>}
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions: [{name: finish, from: [Open], to: Done}]
  - name: demo.adv.Entry
    identity: {name: id, type: Uuid}
    fields: [{name: cents, type: Integer}, {name: tag, type: Optional<String>}]
    lifecycle: {initial: Logged, states: [Logged], terminal: [Logged]}
commands:
  - name: demo.adv.Open
    input:
      - {name: lane, type: demo.adv.Lane}
      - {name: urgent, type: Boolean}
      - {name: cents, type: Integer}
      - {name: tip, type: Optional<Integer>}
      - {name: note, type: Optional<String>}
    outcomes:
      - name: opened
        creates: demo.adv.Item
        instance: id
        sets: {lane: input.lane, urgent: input.urgent, cents: input.cents, tip: input.tip, note: input.note}
        emits: [demo.adv.Opened]
        payload: {demo.adv.Opened: {id: {generated: true}}}
  - name: demo.adv.Finish
    input: [{name: id, type: Uuid}]
    outcomes:
      - name: finished
        moves: demo.adv.Item.finish
        instance: id
        emits: [demo.adv.Finished]
        payload: {demo.adv.Finished: {id: input.id}}
      - {name: unavailable, wrong_state: true, refuses: false}
  - name: demo.adv.Log
    input: [{name: cents, type: Integer}, {name: tag, type: Optional<String>}]
    outcomes:
      - name: logged
        creates: demo.adv.Entry
        instance: id
        sets: {cents: input.cents, tag: input.tag}
        emits: [demo.adv.Logged]
        payload: {demo.adv.Logged: {id: {generated: true}}}
views:
  - name: demo.adv.OpenByLane
    source: demo.adv.Item
    consistency: read_your_writes
    filter: state == Open
    group_by: [lane]
    fields:
      - {name: lane, type: demo.adv.Lane}
      - {name: count, type: Integer, aggregate: {count: {}}}
      - {name: total, type: Integer, aggregate: {sum: cents}}
  - name: demo.adv.OpenByUrgent
    source: demo.adv.Item
    consistency: read_your_writes
    filter: state == Open
    group_by: [urgent]
    fields:
      - {name: urgent, type: Boolean}
      - {name: count, type: Integer, aggregate: {count: {}}}
  - name: demo.adv.ByNote
    source: demo.adv.Item
    consistency: read_your_writes
    group_by: [note]
    fields:
      - {name: note, type: Optional<String>}
      - {name: count, type: Integer, aggregate: {count: {}}}
      - {name: total, type: Integer, aggregate: {sum: cents}}
  - name: demo.adv.LaneStats
    source: demo.adv.Item
    consistency: read_your_writes
    group_by: [lane]
    fields:
      - {name: lane, type: demo.adv.Lane}
      - {name: mean, type: Optional<Decimal>, aggregate: {avg: tip, skip_absent: true}}
      - {name: notes, type: Integer, aggregate: {count_distinct: note, skip_absent: true}}
  - name: demo.adv.EntriesByState
    source: demo.adv.Entry
    consistency: read_your_writes
    group_by: [state]
    fields:
      - {name: state, type: demo.adv.Entry.State}
      - {name: count, type: Integer, aggregate: {count: {}}}
      - {name: tags, type: Integer, aggregate: {count_distinct: tag, skip_absent: true}}
  - name: demo.adv.DoneExtremes
    source: demo.adv.Item
    consistency: read_your_writes
    filter: state == Done
    fields:
      - {name: top, type: Optional<Integer>, aggregate: {max: cents}}
      - {name: low, type: Optional<Integer>, aggregate: {min: cents}}
      - {name: mean, type: Optional<Decimal>, aggregate: {avg: cents}}
  - name: demo.adv.AllDistinct
    source: demo.adv.Item
    consistency: read_your_writes
    fields:
      - {name: notes, type: Integer, aggregate: {count_distinct: note, skip_absent: true}}
      - {name: mean, type: Optional<Decimal>, aggregate: {avg: tip, skip_absent: true}}
";

/// Every finite-domain and unscoped exact view runs green against the interpreted target.
#[test]
fn adversary_finite_and_unscoped_domains_pass_against_the_interpreted_target() {
    let (result, statuses) = interpreted(&format!("{HEAD}{FINITE}"));
    let failed = not_passed(&statuses);
    assert_eq!(
        failed.len(),
        0,
        "aggregate scenarios a healthy target does not pass:\n{}\nrefused: {:#?}",
        failed.join("\n"),
        aggregate_refusals(&result)
    );
}

/// An exact ungrouped read expects one row and a grouped empty read zero rows: every `Counts` an
/// exact read carries is `[n, n]`, and an ungrouped view's is `[1, 1]`.
#[test]
fn adversary_ungrouped_exact_reads_expect_exactly_one_row() {
    let result = synthesize(&ir(&format!("{HEAD}{FINITE}")));
    let mut wrong = Vec::new();
    for id in [
        "demo.adv.DoneExtremes/aggregate",
        "demo.adv.AllDistinct/aggregate",
    ] {
        if !result
            .suite
            .scenarios
            .keys()
            .any(|held| held.to_string() == id)
        {
            wrong.push(format!(
                "{id} not synthesized: {:?}",
                aggregate_refusals(&result)
            ));
            continue;
        }
        for (params, expectations) in reads(&result, id) {
            let counts: Vec<&ViewExpectation> = expectations
                .iter()
                .filter(|expectation| matches!(expectation, ViewExpectation::Counts { .. }))
                .collect();
            let one = ViewExpectation::Counts {
                at_least: Some(1),
                at_most: Some(1),
            };
            if counts != [&one] {
                wrong.push(format!("{id} {params:?}: {counts:?}"));
            }
        }
    }
    assert_eq!(wrong.len(), 0, "{wrong:#?}");
}

/// An `Integer` newtype bounded to `0..=bound`, a field of it the creating command sets, and the
/// views over it named in `views`.
fn bounded(bound: u32, views: &str) -> String {
    format!(
        "{HEAD}types:
  - {{name: demo.adv.Level, kind: newtype, of: Integer, invariants: ['value >= 0', 'value <= {bound}']}}
events:
  - name: demo.adv.Opened
    fields: [{{name: id, type: Uuid}}]
  - name: demo.adv.Finished
    fields: [{{name: id, type: Uuid}}]
entities:
  - name: demo.adv.Item
    identity: {{name: id, type: Uuid}}
    fields: [{{name: team, type: String}}, {{name: level, type: demo.adv.Level}}, {{name: cents, type: Integer}}]
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions: [{{name: finish, from: [Open], to: Done}}]
commands:
  - name: demo.adv.Open
    input: [{{name: team, type: String}}, {{name: level, type: demo.adv.Level}}, {{name: cents, type: Integer}}]
    outcomes:
      - name: opened
        creates: demo.adv.Item
        instance: id
        sets: {{team: input.team, level: input.level, cents: input.cents}}
        emits: [demo.adv.Opened]
        payload: {{demo.adv.Opened: {{id: {{generated: true}}}}}}
  - name: demo.adv.Finish
    input: [{{name: id, type: Uuid}}]
    outcomes:
      - name: finished
        moves: demo.adv.Item.finish
        instance: id
        emits: [demo.adv.Finished]
        payload: {{demo.adv.Finished: {{id: input.id}}}}
      - {{name: unavailable, wrong_state: true, refuses: false}}
views:
{views}"
    )
}

const OPEN_BY_LEVEL: &str = "  - name: demo.adv.OpenByLevel
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: level, type: demo.adv.Level}]
    filter: [state == Open, level == param.level]
    group_by: [level]
    fields:
      - {name: level, type: demo.adv.Level}
      - {name: count, type: Integer, aggregate: {count: {}}}
      - {name: total, type: Integer, aggregate: {sum: cents}}
";

const TEAM_LEVELS: &str = "  - name: demo.adv.TeamLevels
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: team, type: String}]
    filter: team == param.team
    group_by: [level]
    fields:
      - {name: level, type: demo.adv.Level}
      - {name: count, type: Integer, aggregate: {count: {}}}
";

/// The selection's own levels: every read's `level`, as an integer.
fn levels(result: &Synthesis, id: &str) -> Vec<i64> {
    reads(result, id)
        .into_iter()
        .filter_map(|(params, _)| match params.get("level") {
            Some(ScenarioValue::Literal {
                value: ess_primitives::node::Node::Number(number),
            }) => number.as_i64(),
            _ => None,
        })
        .collect()
}

/// `Level` admits `0..=3`, and the selection arranges rows at every one of those levels. The read
/// the design calls a "valid nonmatching selection" must then be absent — no value of the domain
/// is left — and never a level the parameter's type refuses.
#[test]
fn adversary_a_nonmatching_read_stays_inside_a_bounded_domain() {
    let text = bounded(3, OPEN_BY_LEVEL);
    let result = synthesize(&ir(&text));
    let id = "demo.adv.OpenByLevel/aggregate";
    let read = levels(&result, id);
    let outside: Vec<i64> = read
        .iter()
        .copied()
        .filter(|level| !(0..=3).contains(level))
        .collect();
    let (_, statuses) = interpreted(&text);
    assert!(
        outside.is_empty() && not_passed(&statuses).is_empty(),
        "reads {read:?}; outside 0..=3: {outside:?}\n{}",
        not_passed(&statuses).join("\n")
    );
}

/// The same bounded key under a scope parameter rather than a selector: a view that synthesized
/// before this change. It separates the ladder's disregard of the newtype's invariants (the
/// arranged rows) from the selection's.
#[test]
fn adversary_a_scoped_view_over_a_bounded_key_passes() {
    let text = bounded(7, TEAM_LEVELS).replace("'value >= 0'", "'value >= 5'");
    let (_, statuses) = interpreted(&text);
    let failed = not_passed(&statuses);
    assert_eq!(failed.len(), 0, "{}", failed.join("\n"));
}

/// Selectors over the identity, over a `Uuid` the creating command takes as input, and over an
/// `Optional` key that is also a `skip_absent` aggregate's input (so the pattern's absent row of A
/// holds no key at all).
const IDENTITIES: &str = "events:
  - name: demo.adv.Opened
    fields: [{name: id, type: Uuid}]
entities:
  - name: demo.adv.Item
    identity: {name: id, type: Uuid}
    fields:
      - {name: ref, type: Uuid}
      - {name: channel, type: Optional<String>}
      - {name: cents, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
commands:
  - name: demo.adv.Open
    input:
      - {name: ref, type: Uuid}
      - {name: channel, type: Optional<String>}
      - {name: cents, type: Integer}
    outcomes:
      - name: opened
        creates: demo.adv.Item
        instance: id
        sets: {ref: input.ref, channel: input.channel, cents: input.cents}
        emits: [demo.adv.Opened]
        payload: {demo.adv.Opened: {id: {generated: true}}}
views:
  - name: demo.adv.ById
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: id, type: Uuid}]
    filter: id == param.id
    group_by: [id]
    fields:
      - {name: id, type: Uuid}
      - {name: count, type: Integer, aggregate: {count: {}}}
      - {name: total, type: Integer, aggregate: {sum: cents}}
  - name: demo.adv.ByRef
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: ref, type: Uuid}]
    filter: ref == param.ref
    group_by: [ref]
    fields:
      - {name: ref, type: Uuid}
      - {name: count, type: Integer, aggregate: {count: {}}}
  - name: demo.adv.ChannelSpread
    source: demo.adv.Item
    consistency: read_your_writes
    params: [{name: channel, type: String}]
    filter: channel == param.channel
    group_by: [channel]
    fields:
      - {name: channel, type: Optional<String>}
      - {name: count, type: Integer, aggregate: {count: {}}}
      - {name: channels, type: Integer, aggregate: {count_distinct: channel, skip_absent: true}}
";

/// Identity, input-`Uuid` and absent-capable selectors are refused by name or pass.
#[test]
fn adversary_identity_and_absent_capable_selectors_pass_against_the_interpreted_target() {
    let (result, statuses) = interpreted(&format!("{HEAD}{IDENTITIES}"));
    let failed = not_passed(&statuses);
    assert_eq!(
        failed.len(),
        0,
        "aggregate scenarios a healthy target does not pass:\n{}\nrefused: {:#?}",
        failed.join("\n"),
        aggregate_refusals(&result)
    );
}
