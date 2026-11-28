//! Whether a row arranged beside a creator leaves the creating branch to answer an input is
//! decided once per synthesis, not once per arrangement of that creator.
//!
//! `Join` reads three rows through its input, the map and both stops, refuses a stop that lies on
//! another map, and refuses a path already joined with a row-set refusal. Every run that needs a
//! path, the row-set refusals of `Terminus` and `Audit` among them, tries to create one through
//! `Join`'s accepting branch with values it chose, and that branch is arranged around the map with
//! both stops beside it. Each arrangement searched its rows and inputs and checked every stop
//! beside for every input it tried on every row it reached. The choices differed only in the
//! identities of the rows `Join` reads, which the arrangement names itself, so the same
//! arrangement was built again for each of them. On a downstream specification of that shape the
//! checks held most of a synthesis that took five times as long as the release before.
//!
//! The synthesis of this model keeps its suite and is held to a bound on the stops beside it
//! checks, a work bound rather than a clock.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{beside_rows_checked, synthesize};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

/// The stops beside the synthesis may check: 120 when each answer is kept for the synthesis and an
/// arrangement is built once for choices differing only in the rows it names, 26,264 when every
/// arrangement checks every stop for every input anew.
const BOUND: u64 = 500;

/// The suite, the same whether the checks are kept or not.
const SCENARIOS: &[&str] = &[
    "demo.trail.Audit/outcome/clear",
    "demo.trail.Audit/outcome/no-such-map",
    "demo.trail.Close/outcome/closed",
    "demo.trail.Join/outcome/map-closed",
    "demo.trail.Join/outcome/no-such-from",
    "demo.trail.Join/outcome/no-such-map",
    "demo.trail.Join/outcome/no-such-to",
    "demo.trail.Join/outcome/same-stop",
    "demo.trail.Map/state/Closed/refuses/demo.trail.Close",
    "demo.trail.Map/transition/close/by/demo.trail.Close/closed",
    "demo.trail.Open/outcome/opened",
    "demo.trail.Place/outcome/map-closed",
    "demo.trail.Place/outcome/no-such-map",
    "demo.trail.Place/outcome/placed",
    "demo.trail.Terminus/outcome/no-such-map",
];

const MODEL: &str = r"format: ess/37
system: demo
version: v1
domain: demo.trail
summary: Stops on a map, joined by paths; a stop is marked a terminus only where a path reaches it.
types:
  - {name: demo.trail.MapId, kind: newtype, of: Uuid}
  - {name: demo.trail.StopId, kind: newtype, of: Uuid}
  - {name: demo.trail.PathId, kind: newtype, of: Uuid}
  - {name: demo.trail.StopKind, kind: enum, variants: [Halt, Junction, Depot]}
  - {name: demo.trail.Mode, kind: enum, variants: [Walk, Ride]}
entities:
  - name: demo.trail.Map
    identity: {name: map_id, type: demo.trail.MapId}
    fields:
      - {name: title, type: String}
    relations:
      - {name: stops, kind: owns, target: demo.trail.Stop, cardinality: many, via: map_id}
      - {name: paths, kind: owns, target: demo.trail.Path, cardinality: many, via: map_id}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
  - name: demo.trail.Stop
    identity: {name: stop_id, type: demo.trail.StopId}
    fields:
      - {name: map_id, type: demo.trail.MapId}
      - {name: kind, type: demo.trail.StopKind}
      - {name: terminus, type: Boolean}
    lifecycle: {initial: Placed, states: [Placed], terminal: [Placed]}
  - name: demo.trail.Path
    identity: {name: path_id, type: demo.trail.PathId}
    fields:
      - {name: map_id, type: demo.trail.MapId}
      - {name: from_id, type: demo.trail.StopId}
      - {name: to_id, type: demo.trail.StopId}
      - {name: mode, type: demo.trail.Mode}
      - {name: to_kind, type: demo.trail.StopKind}
    relations:
      - {name: from, kind: references, target: demo.trail.Stop, cardinality: one, via: from_id}
      - {name: to, kind: references, target: demo.trail.Stop, cardinality: one, via: to_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: demo.trail.NoSuchMap, summary: No map carries this identity., fields: []}
  - {name: demo.trail.NoSuchStop, summary: No stop carries this identity., fields: []}
  - {name: demo.trail.ClosedMap, summary: The map is closed., fields: []}
  - {name: demo.trail.ForeignStop, summary: The stop lies on another map., fields: []}
  - {name: demo.trail.SameStop, summary: A path joins two stops., fields: []}
  - {name: demo.trail.NotADepot, summary: A ride ends at a depot., fields: []}
  - {name: demo.trail.PathExists, summary: The stops are joined already., fields: []}
  - {name: demo.trail.Unreached, summary: No path reaches the stop., fields: []}
  - {name: demo.trail.RideLeft, summary: A ride is still joined on the map., fields: []}
events:
  - name: demo.trail.MapOpened
    fields: [{name: map_id, type: demo.trail.MapId}]
  - name: demo.trail.MapClosed
    fields: [{name: map_id, type: demo.trail.MapId}]
  - name: demo.trail.StopPlaced
    fields: [{name: stop_id, type: demo.trail.StopId}]
  - name: demo.trail.StopMarked
    fields: [{name: stop_id, type: demo.trail.StopId}]
  - name: demo.trail.PathJoined
    fields: [{name: path_id, type: demo.trail.PathId}]
actors:
  - name: demo.trail.Planner
    may: [demo.trail.Open, demo.trail.Close, demo.trail.Place, demo.trail.Join, demo.trail.Terminus, demo.trail.Audit]
commands:
  - name: demo.trail.Open
    input:
      - {name: title, type: String}
    outcomes:
      - name: opened
        creates: demo.trail.Map
        instance: map_id
        sets: {title: input.title}
        emits: [demo.trail.MapOpened]
        payload: {demo.trail.MapOpened: {map_id: {generated: true}}}
  - name: demo.trail.Close
    input:
      - {name: map_id, type: demo.trail.MapId}
    outcomes:
      - name: closed
        moves: demo.trail.Map.close
        instance: map_id
        emits: [demo.trail.MapClosed]
        payload: {demo.trail.MapClosed: {map_id: input.map_id}}
  - name: demo.trail.Place
    input:
      - {name: stop_id, type: demo.trail.StopId}
      - {name: map_id, type: demo.trail.MapId}
      - {name: kind, type: demo.trail.StopKind}
    outcomes:
      - name: no-such-map
        when_related: {via: input.map_id, exists: false}
        error: demo.trail.NoSuchMap
      - name: map-closed
        when_related: {via: input.map_id, predicate: state == Closed}
        error: demo.trail.ClosedMap
      - name: placed
        creates: demo.trail.Stop
        instance: stop_id
        sets: {map_id: input.map_id, kind: input.kind, terminus: false}
        emits: [demo.trail.StopPlaced]
        payload: {demo.trail.StopPlaced: {stop_id: input.stop_id}}
  - name: demo.trail.Join
    input:
      - {name: map_id, type: demo.trail.MapId}
      - {name: from_id, type: demo.trail.StopId}
      - {name: to_id, type: demo.trail.StopId}
      - {name: mode, type: demo.trail.Mode}
    outcomes:
      - name: same-stop
        when: from_id == to_id
        error: demo.trail.SameStop
      - name: no-such-map
        when_related: {via: input.map_id, exists: false}
        error: demo.trail.NoSuchMap
      - name: no-such-from
        when_related: {via: input.from_id, exists: false}
        error: demo.trail.NoSuchStop
      - name: no-such-to
        when_related: {via: input.to_id, exists: false}
        error: demo.trail.NoSuchStop
      - name: map-closed
        when_related: {via: input.map_id, predicate: state == Closed}
        error: demo.trail.ClosedMap
      - name: foreign-from
        when_related: {via: input.from_id, predicate: map_id != input.map_id}
        error: demo.trail.ForeignStop
      - name: foreign-to
        when_related: {via: input.to_id, predicate: map_id != input.map_id}
        error: demo.trail.ForeignStop
      - name: ride-to-depot
        when_related:
          via: input.to_id
          predicate:
            all:
              - input.mode == Ride
              - kind != Depot
        error: demo.trail.NotADepot
      - name: joined-already
        when_related:
          entity: demo.trail.Path
          where:
            all:
              - from_id == input.from_id
              - to_id == input.to_id
              - mode == input.mode
          exists: true
        error: demo.trail.PathExists
      - name: joined
        creates: demo.trail.Path
        instance: path_id
        sets:
          map_id: input.map_id
          from_id: input.from_id
          to_id: input.to_id
          mode: input.mode
          to_kind: {related: {via: input.to_id, field: kind}}
        emits: [demo.trail.PathJoined]
        payload: {demo.trail.PathJoined: {path_id: {generated: true}}}
  - name: demo.trail.Terminus
    input:
      - {name: map_id, type: demo.trail.MapId}
      - {name: stop_id, type: demo.trail.StopId}
    outcomes:
      - name: no-such-map
        when_related: {via: input.map_id, exists: false}
        error: demo.trail.NoSuchMap
      - name: no-such-stop
        when_related: {via: input.stop_id, exists: false}
        error: demo.trail.NoSuchStop
      - name: foreign-stop
        when_related: {via: input.stop_id, predicate: map_id != input.map_id}
        error: demo.trail.ForeignStop
      - name: unreached
        when_related:
          entity: demo.trail.Path
          where:
            all:
              - to_id == input.stop_id
              - mode == Walk
          exists: false
        error: demo.trail.Unreached
      - name: marked
        updates: demo.trail.Stop
        instance: stop_id
        sets: {terminus: true}
        emits: [demo.trail.StopMarked]
        payload: {demo.trail.StopMarked: {stop_id: input.stop_id}}
  - name: demo.trail.Audit
    input:
      - {name: map_id, type: demo.trail.MapId}
    outcomes:
      - name: no-such-map
        when_related: {via: input.map_id, exists: false}
        error: demo.trail.NoSuchMap
      - name: ride-left
        when_related:
          entity: demo.trail.Path
          where:
            all:
              - map_id == input.map_id
              - mode == Ride
              - to_kind == Depot
          exists: true
        error: demo.trail.RideLeft
      - {name: clear, accepts: nothing}
views:
  - name: demo.trail.Paths
    source: demo.trail.Path
    consistency: read_your_writes
    fields:
      - {name: path_id, type: demo.trail.PathId}
      - {name: from_id, type: demo.trail.StopId}
      - {name: to_id, type: demo.trail.StopId}
      - {name: mode, type: demo.trail.Mode}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("trail.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[test]
fn the_rows_beside_a_creator_are_checked_within_a_bound() {
    let ir = ir(MODEL);
    let (synthesis, checked) = beside_rows_checked(|| synthesize(&ir));
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        ids, SCENARIOS,
        "the suite moved\nrefusals: {:#?}",
        synthesis.refusals
    );
    assert!(
        checked <= BOUND,
        "the rows beside a creator were checked {checked} times, more than {BOUND}"
    );
}
