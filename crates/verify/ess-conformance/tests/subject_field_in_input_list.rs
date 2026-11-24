//! A `when_subject` guard tests whether a stored field is an element of a `List` input, and its
//! negation, with the existing quantifier `exists: {in: input.<list>, as: x, that: <field> == x}`
//! (story:guard-tests-list-membership).
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{synthesize::synthesize, ConformanceScenario, ScenarioStep, ScenarioValue};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const MEMBER: &str = "{exists: {in: input.frontier_actions, as: candidate, that: {candidate: {eq: {fact: action}}}}}";

/// The walk domain, its `off-frontier` refusal guarded by `guard` and its list input typed `list`.
fn walk(guard: &str, list: &str) -> String {
    format!(
        r"format: ess/30
system: demo
version: v1
domain: demo.walk
summary: A walker steps only to an action the frontier lists.
types:
  - {{name: demo.walk.WalkerId, kind: newtype, of: Uuid}}
entities:
  - name: demo.walk.Walker
    identity: {{name: id, type: demo.walk.WalkerId}}
    fields:
      - {{name: action, type: String}}
    lifecycle:
      initial: Walking
      states: [Walking, Done]
      terminal: [Done]
      transitions:
        - {{name: finish, from: [Walking], to: Done}}
actors:
  - name: demo.walk.Operator
    may: [demo.walk.Start, demo.walk.Step]
errors:
  - {{name: demo.walk.UnknownWalker, summary: No such walker., fields: []}}
  - {{name: demo.walk.NotWalking, summary: The walker is done., fields: []}}
  - {{name: demo.walk.OffFrontier, summary: The frontier does not list the action., fields: []}}
events:
  - name: demo.walk.Started
    fields: [{{name: id, type: demo.walk.WalkerId}}]
  - name: demo.walk.Stepped
    fields: [{{name: id, type: demo.walk.WalkerId}}]
commands:
  - name: demo.walk.Start
    input:
      - {{name: action, type: String}}
    outcomes:
      - name: started
        creates: demo.walk.Walker
        instance: id
        sets: {{action: input.action}}
        emits: [demo.walk.Started]
        payload: {{demo.walk.Started: {{id: {{generated: true}}}}}}
  - name: demo.walk.Step
    input:
      - {{name: id, type: demo.walk.WalkerId}}
      - {{name: frontier_actions, type: {list}}}
    outcomes:
      - name: unknown-walker
        unknown_instance: true
        error: demo.walk.UnknownWalker
      - name: not-walking
        wrong_state: true
        error: demo.walk.NotWalking
      - name: off-frontier
        when_subject: {{predicate: {guard}}}
        error: demo.walk.OffFrontier
      - name: stepped
        moves: demo.walk.Walker.finish
        instance: id
        emits: [demo.walk.Stepped]
        payload: {{demo.walk.Stepped: {{id: input.id}}}}
views:
  - name: demo.walk.Walkers
    source: demo.walk.Walker
    consistency: read_your_writes
    fields:
      - {{name: id, type: demo.walk.WalkerId}}
      - {{name: action, type: String}}
      - {{name: state, type: demo.walk.Walker.State}}
"
    )
}

fn not_member() -> String {
    format!("{{not: {MEMBER}}}")
}

fn assembled(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("walk.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn ir(text: &str) -> EssIr {
    let spec = assembled(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

fn requires(scenario: &ConformanceScenario, outcome: &str) -> bool {
    scenario.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectOutcome { outcome: expected }
            if expected.to_string() == outcome)
    })
}

fn inputs(scenario: &ConformanceScenario, command: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .collect()
}

/// For each scenario requiring `outcome`: the stored action and the list the `Step` sent.
fn witnessed(text: &str, outcome: &str) -> Vec<(Node, Vec<Node>)> {
    let suite = synthesize(&ir(text)).suite;
    suite
        .scenarios
        .values()
        .filter(|scenario| requires(scenario, outcome))
        .filter_map(|scenario| {
            let start = inputs(scenario, "demo.walk.Start");
            let step = inputs(scenario, "demo.walk.Step");
            let action = start.last()?.get("action")?.as_literal()?.clone();
            let Some(Node::Seq(list)) = step.last()?.get("frontier_actions")?.as_literal().cloned()
            else {
                return None;
            };
            Some((action, list))
        })
        .collect()
}

#[test]
fn membership_and_its_negation_validate() {
    for guard in [MEMBER.to_owned(), not_member()] {
        let text = walk(&guard, "List<String>");
        assert!(
            assembled(&text).is_ok(),
            "{guard}: {:?}",
            assembled(&text).err()
        );
        let _ = ir(&text);
    }
}

#[test]
fn an_element_type_other_than_the_field_type_is_refused() {
    let text = walk(&not_member(), "List<Integer>");
    let error = assembled(&text).map_or_else(|error| error, |_| panic!("must be refused"));
    assert!(error.contains("[type_mismatch]"), "{error}");
}

/// Whether one of `found` sends a list holding the stored action behind another element, so a
/// target reading only the first element answers it wrongly.
fn holds_behind_another(found: &[(Node, Vec<Node>)]) -> bool {
    found
        .iter()
        .any(|(action, list)| list.contains(action) && list.first() != Some(action))
}

/// Whether one of `found` sends a non-empty list without the stored action, so a target that
/// ignores the list, or tests only that it is empty, answers it wrongly.
fn lacks_non_empty(found: &[(Node, Vec<Node>)]) -> bool {
    found
        .iter()
        .any(|(action, list)| !list.is_empty() && !list.contains(action))
}

#[test]
fn each_side_of_the_negated_guard_is_witnessed_with_a_non_empty_list() {
    let text = walk(&not_member(), "List<String>");
    let refused = witnessed(&text, "demo.walk.Step/off-frontier");
    assert!(lacks_non_empty(&refused), "off-frontier: {refused:#?}");
    let taken = witnessed(&text, "demo.walk.Step/stepped");
    assert!(holds_behind_another(&taken), "stepped: {taken:#?}");
}

#[test]
fn each_side_of_the_guard_is_witnessed_with_a_non_empty_list() {
    let text = walk(MEMBER, "List<String>");
    let refused = witnessed(&text, "demo.walk.Step/off-frontier");
    assert!(holds_behind_another(&refused), "off-frontier: {refused:#?}");
    let taken = witnessed(&text, "demo.walk.Step/stepped");
    assert!(lacks_non_empty(&taken), "stepped: {taken:#?}");
}

/// The compact spellings an adopter tries are refused naming the quantifier, and the form named,
/// applied as written, validates.
#[test]
fn the_compact_membership_spelling_names_a_form_that_validates() {
    for (written, hint) in [
        (
            "\"action in input.frontier_actions\"",
            "exists: {in: input.frontier_actions, as: x, that: action == x}",
        ),
        (
            "\"action not in input.frontier_actions\"",
            "not: {exists: {in: input.frontier_actions, as: x, that: action == x}}",
        ),
    ] {
        let error = assembled(&walk(written, "List<String>"))
            .map_or_else(|error| error, |_| panic!("{written} must be refused"));
        assert!(
            error.contains("a predicate is either a comparison"),
            "{written}: {error}"
        );
        assert!(error.contains(hint), "{written} hints `{hint}`: {error}");
        let applied = walk(&format!("{{{hint}}}"), "List<String>");
        if let Err(error) = assembled(&applied) {
            panic!("{written}: the hinted `{hint}` is refused:\n{error}");
        }
        let _ = ir(&applied);
    }
}

/// A profile whose stored `parties` the creating `DeclareProfile` checks against the roles of its
/// `semantics` input, and whose `ReviseProfile` refuses when a stored party is a role of none of
/// its own `semantics` (beyond10x/ess#734).
const PROFILE: &str = "format: ess/35
system: example
version: v1
domain: example.profile
summary: A profile's parties must each be a role of a declared semantic.
types:
  - {name: example.profile.ProfileId, kind: newtype, of: Uuid}
  - name: example.profile.Semantic
    kind: struct
    fields:
      - {name: name, type: String}
      - {name: roles, type: List<String>}
entities:
  - name: example.profile.Profile
    identity: {name: profile_id, type: example.profile.ProfileId}
    fields:
      - {name: parties, type: List<String>}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
actors:
  - name: example.profile.Operator
    may: [example.profile.DeclareProfile, example.profile.ReviseProfile]
errors:
  - {name: example.profile.PartyNotARole, summary: A party is no role., fields: []}
  - {name: example.profile.UnknownProfile, summary: No such profile., fields: []}
events:
  - name: example.profile.Declared
    fields: [{name: profile_id, type: example.profile.ProfileId}]
  - name: example.profile.Revised
    fields: [{name: profile_id, type: example.profile.ProfileId}]
commands:
  - name: example.profile.DeclareProfile
    input:
      - {name: profile_id, type: example.profile.ProfileId}
      - {name: parties, type: List<String>}
      - {name: semantics, type: List<example.profile.Semantic>}
    outcomes:
      - name: party-not-a-role
        when: {not: {forall: {in: parties, as: p, that: {exists: {in: semantics, as: s, that: {exists: {in: s.roles, as: r, that: r == p}}}}}}}
        error: example.profile.PartyNotARole
      - name: declared
        creates: example.profile.Profile
        instance: profile_id
        sets: {parties: input.parties}
        emits: [example.profile.Declared]
        payload: {example.profile.Declared: {profile_id: input.profile_id}}
  - name: example.profile.ReviseProfile
    input:
      - {name: profile_id, type: example.profile.ProfileId}
      - {name: semantics, type: List<example.profile.Semantic>}
    outcomes:
      - {name: unknown, unknown_instance: true, error: example.profile.UnknownProfile}
      - name: party-not-a-role
        when_subject:
          predicate: {not: {forall: {in: parties, as: p, that: {exists: {in: input.semantics, as: s, that: {exists: {in: s.roles, as: r, that: r == p}}}}}}}
        error: example.profile.PartyNotARole
      - name: revised
        updates: example.profile.Profile
        instance: profile_id
        emits: [example.profile.Revised]
        payload: {example.profile.Revised: {profile_id: input.profile_id}}
views:
  - name: example.profile.Profiles
    source: example.profile.Profile
    consistency: read_your_writes
    fields:
      - {name: profile_id, type: example.profile.ProfileId}
      - {name: parties, type: List<String>}
";

/// [`PROFILE`] with a flat `roles: List<String>` input in place of `semantics`.
fn flat_profile() -> String {
    PROFILE
        .replace(
            "      - {name: semantics, type: List<example.profile.Semantic>}",
            "      - {name: roles, type: List<String>}",
        )
        .replace(
            "{in: semantics, as: s, that: {exists: {in: s.roles, as: r, that: r == p}}}",
            "{in: roles, as: r, that: r == p}",
        )
        .replace(
            "{in: input.semantics, as: s, that: {exists: {in: s.roles, as: r, that: r == p}}}",
            "{in: input.roles, as: r, that: r == p}",
        )
}

/// Every string a node holds, at any depth.
fn strings(node: &Node, out: &mut Vec<String>) {
    match node {
        Node::Text(text) => out.push(text.clone()),
        Node::Seq(items) => items.iter().for_each(|item| strings(item, out)),
        Node::Map(entries) => entries.values().for_each(|value| strings(value, out)),
        _ => {}
    }
}

/// The roles one command input sends: every string under `semantics` or `roles`.
fn roles(input: &BTreeMap<String, ScenarioValue>) -> Vec<String> {
    let mut out = Vec::new();
    for name in ["semantics", "roles"] {
        if let Some(node) = input.get(name).and_then(ScenarioValue::as_literal) {
            strings(node, &mut out);
        }
    }
    out
}

/// The refusal of `ReviseProfile` gets a scenario that declares a profile with a non-empty
/// `parties`, each a role the declaration sends, and revises it with roles that miss a party; and
/// the interpreted target passes every scenario of the suite.
fn assert_arranged_non_empty(text: &str) {
    let model = ir(text);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    let scenario = suite
        .scenarios
        .values()
        .find(|scenario| requires(scenario, "example.profile.ReviseProfile/party-not-a-role"))
        .expect("the refusal has a scenario");
    let declared = inputs(scenario, "example.profile.DeclareProfile");
    let declared = declared.last().expect("a declaration");
    let Some(Node::Seq(parties)) = declared.get("parties").and_then(ScenarioValue::as_literal)
    else {
        panic!("parties: {declared:?}");
    };
    let mut held = Vec::new();
    for party in parties {
        strings(party, &mut held);
    }
    assert!(!held.is_empty(), "a non-empty parties: {declared:?}");
    let declared_roles = roles(declared);
    assert!(
        held.iter().all(|party| declared_roles.contains(party)),
        "every party a declared role: {declared:?}"
    );
    let revised = inputs(scenario, "example.profile.ReviseProfile");
    let revised = roles(revised.last().expect("a revision"));
    assert!(
        held.iter().any(|party| !revised.contains(party)),
        "the revision misses a party: {held:?} against {revised:?}"
    );
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
    let report = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model),
        )
        .into_report();
    for run in &report.scenarios {
        assert_eq!(
            run.status,
            ess_conformance::report::Status::Passed,
            "{}: {:?}",
            run.scenario,
            run.checks
        );
    }
}

#[test]
fn a_stored_list_tied_to_a_creation_input_is_arranged_non_empty() {
    assert_arranged_non_empty(PROFILE);
}

#[test]
fn a_stored_list_tied_to_a_flat_creation_input_is_arranged_non_empty() {
    assert_arranged_non_empty(&flat_profile());
}

#[test]
fn a_creation_guard_no_list_can_pass_still_refuses() {
    let text = PROFILE.replace(
        "when: {not: {forall: {in: parties, as: p, that: {exists: {in: semantics, as: s, that: {exists: {in: s.roles, as: r, that: r == p}}}}}}}",
        "when: parties.count >= 1",
    );
    let refusals = synthesize(&ir(&text)).refusals;
    assert_eq!(refusals.len(), 1, "{refusals:#?}");
    let text = refusals[0].to_string();
    for line in [
        "refusal[ESS-SYNTH-003]: outcome example.profile.ReviseProfile/party-not-a-role has no \
         scenario `example.profile.ReviseProfile/outcome/party-not-a-role`",
        "no candidate of the 1 tried satisfies ``example.profile.Profile` stored parties \
         selecting this branch, over the rows 1 bounded arrangements left`",
    ] {
        assert!(text.contains(line), "{text}");
    }
}

#[test]
fn the_interpreted_target_passes_every_scenario() {
    for guard in [MEMBER.to_owned(), not_member()] {
        let model = ir(&walk(&guard, "List<String>"));
        let suite = synthesize(&model).suite;
        let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
        let report = ess_conformance::Runner::for_suite(&suite)
            .run_admitted(
                &admitted,
                &ess_conformance::interpret::Interpreted::for_model(model),
            )
            .into_report();
        assert_ne!(report.scenarios.len(), 0, "{guard}: no scenario ran");
        for run in &report.scenarios {
            assert_eq!(
                run.status,
                ess_conformance::report::Status::Passed,
                "{guard}: {}: {:?}",
                run.scenario,
                run.checks
            );
        }
    }
}

/// An order admitted when every capability its stored `requires` lists is granted by the input
/// list `granted` (beyond10x/ess#726).
const ORDER: &str = "format: ess/34
system: example
version: v1
domain: example.order
summary: An order is admitted when every capability it requires is granted.
entities:
  - name: example.order.Order
    identity: {name: order_id, type: Uuid}
    fields:
      - {name: requires, type: List<String>}
    lifecycle:
      initial: Pending
      states: [Pending, Admitted]
      terminal: [Admitted]
      transitions:
        - {name: admit, from: [Pending], to: Admitted}
actors:
  - name: example.order.Operator
    may: [example.order.Place, example.order.Admit]
errors:
  - {name: example.order.OrderStateConflict, summary: The order is not pending., fields: []}
  - {name: example.order.DecisionInvalid, summary: A requirement is not granted., fields: []}
events:
  - name: example.order.Placed
    fields: [{name: order_id, type: Uuid}]
  - name: example.order.Admitted
    fields: [{name: order_id, type: Uuid}]
commands:
  - name: example.order.Place
    input:
      - {name: requires, type: List<String>}
    outcomes:
      - name: placed
        creates: example.order.Order
        instance: order_id
        sets: {requires: input.requires}
        emits: [example.order.Placed]
        payload: {example.order.Placed: {order_id: {generated: true}}}
  - name: example.order.Admit
    input:
      - {name: order_id, type: Uuid}
      - {name: granted, type: List<String>}
    outcomes:
      - name: admitted
        when_subject:
          predicate:
            forall:
              in: requires
              as: c
              that: {exists: {in: input.granted, as: g, that: g == c}}
        moves: example.order.Order.admit
        instance: order_id
        emits: [example.order.Admitted]
        payload: {example.order.Admitted: {order_id: input.order_id}}
      - {name: wrong-state, wrong_state: true, error: example.order.OrderStateConflict}
      - {name: not-covered, error: example.order.DecisionInvalid}
views:
  - name: example.order.Orders
    source: example.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: Uuid}
      - {name: requires, type: List<String>}
      - {name: state, type: example.order.Order.State}
";

/// [`ORDER`] with a second move `refuse` from `Pending`, bindings checked for duplicates, a
/// `missing` list refused when it holds anything, and a guard reading both lists both ways.
fn bound_order() -> String {
    ORDER
        .replace(
            "entities:\n",
            "types:\n  - name: example.order.Binding\n    kind: struct\n    fields:\n      \
             - {name: capability, type: String}\nentities:\n",
        )
        .replace(
            "      states: [Pending, Admitted]\n      terminal: [Admitted]\n",
            "      states: [Pending, Admitted, Refused]\n      terminal: [Admitted, Refused]\n",
        )
        .replace(
            "        - {name: admit, from: [Pending], to: Admitted}\n",
            "        - {name: admit, from: [Pending], to: Admitted}\n        \
             - {name: refuse, from: [Pending], to: Refused}\n",
        )
        .replace(
            "    may: [example.order.Place, example.order.Admit]",
            "    may: [example.order.Place, example.order.Admit, example.order.Refuse]",
        )
        .replace(
            "  - name: example.order.Admitted\n    fields: [{name: order_id, type: Uuid}]\n",
            "  - name: example.order.Admitted\n    fields: [{name: order_id, type: Uuid}]\n  \
             - name: example.order.Refused\n    fields: [{name: order_id, type: Uuid}]\n",
        )
        .replace(
            "  - name: example.order.Admit\n",
            "  - name: example.order.Refuse
    input:
      - {name: order_id, type: Uuid}
    outcomes:
      - name: refused
        moves: example.order.Order.refuse
        instance: order_id
        emits: [example.order.Refused]
        payload: {example.order.Refused: {order_id: input.order_id}}
      - {name: wrong-state, wrong_state: true, error: example.order.OrderStateConflict}
  - name: example.order.Admit\n",
        )
        .replace(
            "      - {name: granted, type: List<String>}\n",
            "      - {name: bindings, type: List<example.order.Binding>}\n      \
             - {name: missing, type: List<String>}\n",
        )
        .replace(
            "      - name: admitted
        when_subject:
          predicate:
            forall:
              in: requires
              as: c
              that: {exists: {in: input.granted, as: g, that: g == c}}\n",
            "      - name: duplicate-binding
        when: {not: {distinct: {in: bindings, as: b, by: b.capability}}}
        error: example.order.DecisionInvalid
      - name: missing-capability
        when: missing.count >= 1
        error: example.order.DecisionInvalid
      - name: admitted
        when_subject:
          predicate:
            all:
              - input.missing.count == 0
              - {forall: {in: requires, as: c, that: {exists: {in: input.bindings, as: b, that: b.capability == c}}}}
              - {forall: {in: input.bindings, as: b, that: {exists: {in: requires, as: c, that: b.capability == c}}}}\n",
        )
}

/// The suite of `text`, every scenario of it passing on the interpreted target.
fn passing_suite(text: &str) -> ess_conformance::Synthesis {
    let model = ir(text);
    let synthesis = synthesize(&model);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = ess_conformance::Runner::for_suite(&synthesis.suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model),
        )
        .into_report();
    for run in &report.scenarios {
        assert_eq!(
            run.status,
            ess_conformance::report::Status::Passed,
            "{}: {:?}",
            run.scenario,
            run.checks
        );
    }
    synthesis
}

/// The scenario `id` of `synthesis`, which refuses nothing under that id.
fn written<'s>(synthesis: &'s ess_conformance::Synthesis, id: &str) -> &'s ConformanceScenario {
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains(id))
        .collect();
    assert!(refused.is_empty(), "{refused:#?}");
    synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

#[test]
fn a_wrong_state_row_is_sent_an_input_list_holding_every_stored_element() {
    let synthesis = passing_suite(ORDER);
    let scenario = written(
        &synthesis,
        "example.order.Order/state/Admitted/refuses/example.order.Admit",
    );
    assert!(requires(scenario, "example.order.Admit/wrong-state"));
    let placed = inputs(scenario, "example.order.Place");
    let Some(Node::Seq(stored)) = placed
        .first()
        .and_then(|input| input.get("requires"))
        .and_then(ScenarioValue::as_literal)
        .cloned()
    else {
        panic!("the order is placed with its requires: {placed:?}");
    };
    let admits = inputs(scenario, "example.order.Admit");
    let Some(Node::Seq(granted)) = admits
        .last()
        .and_then(|input| input.get("granted"))
        .and_then(ScenarioValue::as_literal)
        .cloned()
    else {
        panic!("the wrong-state send carries granted: {admits:?}");
    };
    assert!(
        stored.iter().all(|element| granted.contains(element)),
        "{stored:?} within {granted:?}"
    );
}

#[test]
fn each_wrong_state_keeps_its_refusal_beside_list_guards_both_ways() {
    let synthesis = passing_suite(&bound_order());
    for state in ["Admitted", "Refused"] {
        let scenario = written(
            &synthesis,
            &format!("example.order.Order/state/{state}/refuses/example.order.Admit"),
        );
        assert!(
            requires(scenario, "example.order.Admit/wrong-state"),
            "{state}"
        );
    }
}
