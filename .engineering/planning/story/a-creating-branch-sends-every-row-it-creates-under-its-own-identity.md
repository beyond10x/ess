---
format: aep.planning-md/3
id: story:a-creating-branch-sends-every-row-it-creates-under-its-own-identity
kind: story
status: active
title: A creating branch's scenario sends every row it creates under an identity of its own
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:a-creating-branchs-scenario-sends-every-row-it-creates-under-an-identity-of-its-own
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: cited
  path: crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/existing_instance_beside_related_guards.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/absent-reference-identity-view.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/create-if-absent.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T17:24:04Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-11-22T17:24:05Z", actor: "human:timo", revision: 11}
---
## Outcome

- A command whose creating branch is guarded on the absence of the row it creates (`when_related: {via: input.shelf_id, exists: false}` with `creates: demo.depot.Shelf`, `instance: shelf_id`) gets its scenarios: `demo.depot.OpenShelf/outcome/opened` and `/outcome/shelf-exists` are synthesized with 0 refusals and pass against the interpreter.
- The further run a creating branch makes with an absent Optional reference (`Witness::RelatedAbsent`) sends a `label_id` that no earlier row of its scenario holds, also beside a view filtered on that identity: `demo.depot.FileLabel/outcome/filed` is synthesized with 0 refusals.
- A guard on the identity that leaves one value (the `item_id.count > 10` forms of `arranged_identities_stay_distinct.rs`) is still refused with the same text.
- No pinned row of the three byte-pin tables moves; two rows are added, one per new fixture. No format version changes, no `ValidationCode` changes.

## What to change

Line numbers are at `16cde5949b` (release 0.97.4, the wave base). Record the base before you start: `BASE=$(git rev-parse HEAD)`. Two commits, A then B. Each commit carries its own fixture, its own test and its own table rows.

Where the 0.97.4 memo sits: `related_guard.rs` `chooses` (line 398) reads the branch a present related row and an input select from `row_picks` (line 432). The memo is keyed by command name, entity, the row's `state`, `settled` and `unwritten` (lines 445-448), and then by the whole input (`subject_fact.rs` `RowPicks::answer`, line 6129, compares the whole input). It is consulted only from `on_held` (line 3154, used at 3167) and `with_row_where` (line 5227, used at 5323), which read a present row. Both changes below are on paths that read no present row (the `exists: false` arm of `drive`, and the further run that leaves the reference out). They neither invalidate, key nor bypass the memo: a moved identity is a different input or a different `settled`, so it lands under a new key. Do not add any thread-local, flag or context read inside `selects` (line 2064) or `chooses`: the memo is correct only because `selects` reads nothing but the model, the command, the row and the input.

### Commit A: a creation guarded on its own row's absence

The defect: an arranging run of such a branch goes through `related_guard::drive` (line 2572). Its `exists: false` arm (lines 2634-2647) builds the input with `without_row` (line 4851), which takes the identity from `without_row_each` (line 5112) at line 5119: `fresh_identity(ir, command, field, None)` (`synthesize.rs` line 18577). That is always the one value at `Distinction::UNKNOWN`, whatever distinction the arrangement is at. So every row the branch creates for an arrangement is created under that one identity (`shelf_id-1048576`), and the branch's own scenario (`arranged_at`, line 4020, `exists: false` arm from line 4060: two decoy rows through `row_at` (line 3276), then its own send, also at the `UNKNOWN` value) creates it three times. `repeated_identity::withdraw_repeated_identities` (called at `synthesize.rs` line 3624) then withdraws the scenario, and every command whose arrangement needs two such rows loses its scenarios too.

1. `crates/verify/ess-conformance/src/synthesize/related_guard.rs`, directly after `fn is_absent` (lines 2543-2551, before the doc comment `/// One run of a branch of a command reading a related row as an arranging act` at line 2553), add:

   ```rust
   /// Whether `outcome` creates the row its own `exists: false` guard reads: `when_related: {via:
   /// input.f, exists: false}` on a `creates:` of the entity `f` names, whose new identity is `f`
   /// (story:a-creating-branchs-scenario-sends-every-row-it-creates-under-an-identity-of-its-own).
   ///
   /// The guard is then the creation's own existence test, as `existing_instance:` beside it is. A
   /// row such a branch creates for an arrangement is created under the arrangement's own identity
   /// ([`drive`]), never under the one identity no row carries, which every such row would share.
   pub(super) fn creates_if_absent(outcome: &ResolvedOutcome) -> bool {
       let ResolvedCondition::Related {
           via: via @ ResolvedRelatedVia::Input { .. },
           entity,
           test: ResolvedRelatedTest::Absent,
           also,
           ..
       } = &outcome.condition
       else {
           return false;
       };
       also.is_empty()
           && outcome.subject.as_ref().is_some_and(|subject| {
               subject.effect == ResolvedEffect::Creates && &subject.entity == entity
           })
           && super::existence::identity_input(outcome) == Some(via.field())
   }
   ```

   Every name used is already imported at the top of the file (lines 36-40: `ResolvedCondition`, `ResolvedEffect`, `ResolvedOutcome`, `ResolvedRelatedTest`, `ResolvedRelatedVia`). `ResolvedRelatedVia::field()` returns `&str`; `existence::identity_input` is `pub(super) fn identity_input(outcome: &ResolvedOutcome) -> Option<&str>` (`existence.rs` line 71).

2. Same file, `drive`, the `exists: false` arm. Today (lines 2634-2647):

   ```rust
       if is_absent(driver.outcome) {
           let input = match input {
               ...
               None => without_row(ir, driver.command, entity, field, distinction)?,
           };
           return Ok(super::invoke_with(
               ir, driver, instance, actors, bound, &input,
           ));
       }
   ```

   Insert between the `let input = match input { ... };` (ends line 2643) and `return Ok(super::invoke_with(` (line 2644):

   ```rust
           // A creation guarded on its own row's absence creates that row, so the row is created
           // under this arrangement's own identity, as any creating driver's row is: two rows of
           // one scenario never share the identity no row carries
           // (story:a-creating-branchs-scenario-sends-every-row-it-creates-under-an-identity-of-its-own).
           let input = if creates_if_absent(driver.outcome) {
               super::own_identity(ir, driver, distinction, input)
           } else {
               input
           };
   ```

   `own_identity` is the private `fn own_identity(ir: &EssIr, driver: &Driver<'_>, distinction: Distinction, input: BTreeMap<String, Node>) -> BTreeMap<String, Node>` at `synthesize.rs` line 9448. `related_guard` is a child module of `synthesize`, so `super::own_identity` is visible without changing its visibility. It moves the identity named by `existence::identity_input(driver.outcome)` to the first candidate's value at `distinction`, and only where `keeps_branch` (line 18435) holds, so a guard on the identity that the far value breaks keeps the old input.

3. Do not change `without_row_each`, `without_row`, `fresh_identity`, `missing_sends` or the scenario's own send in `arranged_at`. The branch's own send keeps the identity no row carries (the `UNKNOWN` value), which differs from every decoy once the decoys are created through step 2. The conformance runner empties the modelled namespace before each scenario, so no other scenario's rows are visible to it.

4. Add the fixture `crates/verify/ess-conformance/tests/fixtures/create-if-absent.yaml` with exactly the content under "Fixture A" in `## Tests first`, the tests of `## Tests first` part A, and the table rows (`## Commands` 6). Commit A, message file:

   ```text
   fix(conformance): a creation guarded on its own row's absence creates each row under its own identity

   An arranging run of a creating branch guarded by `when_related: {via: input.f, exists: false}`
   on the row it creates sent the one identity no row carries, whatever its distinction, so the
   branch's own scenario created that identity three times and was withdrawn as ESS-SYNTH-001,
   and every scenario needing two such rows went with it. Such a run now sends its
   distinction's own identity, as every other creating driver does.
   ```

### Commit B: the absent-reference run takes a distinction of its own

The defect: `outcome_scenario_in` (`synthesize.rs` line 4585) witnesses a branch an absent Optional reference selects once more, at lines 4809-4823, as `Witness::RelatedAbsent(nth)` with `nth = boundary_goals(..).len() + 1`. `exercise_as` (line 5081) arranges it at `Distinction::further(nth)` (lines 5732-5735) through `related_guard::prepare_absent_in` (line 3912) and `absent_input` (line 3842), which takes the identity from the candidate at that distinction and checks it against nothing. A view filtered on the created row's identity (`label_id == param.label_id`) makes the full run arrange a companion row (`arrange_matching`, numbered by `Beside::next`) at the same further distinction. Both send `label_id-1` and both capture `label-2`, so the scenario is withdrawn as creating one identity twice. Without that view the scenario stands (checked on the released binary).

5. `crates/verify/ess-conformance/src/synthesize.rs`. Directly above the doc comment of `fn outcome_scenario_in` (`/// One scenario per declared outcome (§10), or the refusal that says why there is none.`, line 4584), add:

   ```rust
   /// How many further distinctions past its own the absent-reference run of a branch an absent
   /// Optional reference selects is tried at, where the run at its own would create a row under an
   /// identity a row of the scenario still holds
   /// (story:a-creating-branchs-scenario-sends-every-row-it-creates-under-an-identity-of-its-own).
   const RELATED_ABSENT_TRIES: usize = 4;
   ```

6. Same file, replace lines 4809-4823 (the whole `if related_guard::absent_selects(models.arrangement, command, outcome) { ... }` block, keeping the comment at 4806-4808 above it) with:

   ```rust
       if related_guard::absent_selects(models.arrangement, command, outcome) {
           let first = related_guard::boundary_goals(models.arrangement, command, outcome).len() + 1;
           let mut found = exercise_as(
               models,
               command,
               outcome,
               actors,
               &id,
               refusals,
               Witness::RelatedAbsent(first),
           );
           // A row the scenario already holds may carry the identity this run creates at its own
           // distinction: the companion a view filtered on the identity arranges beside the full
           // run is created at the same further distinction. The next distinctions are tried, and
           // the first run is kept where each repeats, so a scenario that never repeated keeps
           // its bytes (story:a-creating-branchs-scenario-sends-every-row-it-creates-under-an-identity-of-its-own).
           let repeats = |more: &[ScenarioStep]| {
               repeated_identity::repeats(models.arrangement, &[steps.as_slice(), more].concat())
           };
           if found.as_ref().is_some_and(|(more, ..)| repeats(more)) {
               let apart = (1..=RELATED_ABSENT_TRIES).find_map(|step| {
                   let mut passed_over = Vec::new();
                   exercise_as(
                       models,
                       command,
                       outcome,
                       actors,
                       &id,
                       &mut passed_over,
                       Witness::RelatedAbsent(first + step),
                   )
                   .filter(|(more, ..)| !repeats(more))
               });
               if apart.is_some() {
                   found = apart;
               }
           }
           if let Some((more, depends, _)) = found {
               steps.extend(more);
               source.extend(depends);
           }
       }
   ```

   `repeated_identity::repeats(ir: &EssIr, steps: &[ScenarioStep]) -> bool` is `repeated_identity.rs` line 52, already used this way in `related_boundaries` (line 4961). The closure borrows `steps` immutably and is last used before `steps.extend`, so it compiles under the borrow checker as written. A tried run records its captured names into `earlier_runs` (line 5369) even when it is passed over. That only happens where the first run repeated, so no committed suite changes.

7. Add the fixture `crates/verify/ess-conformance/tests/fixtures/absent-reference-identity-view.yaml` with exactly the content under "Fixture B" in `## Tests first`, the test of part B, and its table rows. Commit B, message file:

   ```text
   fix(conformance): the absent-reference run of a creating branch takes an identity no row of its scenario holds

   The further run that leaves an Optional reference out was arranged at one fixed further
   distinction. A view filtered on the created row's identity arranges a companion at the same
   distinction, so both created the same identity and the scenario was withdrawn as ESS-SYNTH-001.
   Where the run would repeat an identity, the next four distinctions are tried and the first
   that repeats none is kept; every other suite keeps its bytes.
   ```

Commit each with `b10x-gates --repository beyond10x/ess bot -- commit -F <message file>` (`AGENTS.md` § Commits), no attribution lines.

## Tests first

All tests go into the existing target `crates/verify/ess-conformance/tests/existing_instance_beside_related_guards.rs` (no new test binary). It already has `ir`, `refusals`, `ids` and `run` (interpreter) helpers (lines 246-288). Each test synthesizes one small model (two to four commands) and runs its suite against the interpreter, with no generated code compiled. That is well under 60 s in a debug build; time each new test and name any over 5 s in your report.

The two models are files on purpose: every YAML file in the repository is a row of the three byte-pin tables, which is how they get pinned. Add one sentence to the module comment, after the paragraph `The fixtures are inline: …` (lines 19-20): `The models of story:a-creating-branchs-scenario-sends-every-row-it-creates-under-an-identity-of-its-own are files under tests/fixtures/, so the byte-pin tables pin them too.`

The names below (`demo.depot`, `Shelf`, `Label`, `Keeper`) are invented. Keep the fixtures free of any project, company or session name.

### Fixture A: `crates/verify/ess-conformance/tests/fixtures/create-if-absent.yaml`

```yaml
format: ess/37
system: demo
version: v1
domain: demo.depot
summary: A shelf is opened under an identity no shelf carries yet, and closed once.
types:
  - {name: demo.depot.ShelfId, kind: newtype, of: String}
entities:
  - name: demo.depot.Shelf
    identity: {name: shelf_id, type: demo.depot.ShelfId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
actors:
  - name: demo.depot.Keeper
    may: [demo.depot.OpenShelf, demo.depot.CloseShelf]
errors:
  - name: demo.depot.ShelfExists
    fields: [{name: shelf_id, type: demo.depot.ShelfId}]
  - name: demo.depot.NoSuchShelf
    fields: [{name: shelf_id, type: demo.depot.ShelfId}]
  - name: demo.depot.ShelfStateConflict
    fields: [{name: shelf_id, type: demo.depot.ShelfId}]
commands:
  - name: demo.depot.OpenShelf
    input:
      - {name: shelf_id, type: demo.depot.ShelfId}
      - {name: label, type: String}
    outcomes:
      - name: opened
        when_related: {via: input.shelf_id, exists: false}
        creates: demo.depot.Shelf
        instance: shelf_id
        sets:
          label: input.label
        emits: [demo.depot.ShelfOpened]
        payload:
          demo.depot.ShelfOpened:
            shelf_id: input.shelf_id
      - name: shelf-exists
        error: demo.depot.ShelfExists
  - name: demo.depot.CloseShelf
    input:
      - {name: shelf_id, type: demo.depot.ShelfId}
    outcomes:
      - name: no-shelf
        unknown_instance: true
        error: demo.depot.NoSuchShelf
      - name: closed
        moves: demo.depot.Shelf.close
        instance: shelf_id
        emits: [demo.depot.ShelfClosed]
        payload:
          demo.depot.ShelfClosed:
            shelf_id: input.shelf_id
      - name: wrong-state
        wrong_state: true
        error: demo.depot.ShelfStateConflict
events:
  - name: demo.depot.ShelfOpened
    fields: [{name: shelf_id, type: demo.depot.ShelfId}]
  - name: demo.depot.ShelfClosed
    fields: [{name: shelf_id, type: demo.depot.ShelfId}]
views:
  - name: demo.depot.Shelves
    source: demo.depot.Shelf
    consistency: read_your_writes
    fields:
      - {name: shelf_id, type: demo.depot.ShelfId}
      - {name: label, type: String}
      - {name: state, type: demo.depot.Shelf.State}
```

On the released 0.97.3 binary, `ess verify conform synthesize --path create-if-absent.yaml` prints `5 scenario(s) (0 authored), 1 refusal(s)`, the refusal being `refusal[ESS-SYNTH-001]: outcome demo.depot.OpenShelf/opened has no scenario `demo.depot.OpenShelf/outcome/opened`` with `no witness: `shelf_id` is `demo.depot.ShelfId`, which names one row per value, …`. Every scenario sends `OpenShelf` the same `shelf_id-1048576`.

### Fixture B: `crates/verify/ess-conformance/tests/fixtures/absent-reference-identity-view.yaml`

```yaml
format: ess/37
system: demo
version: v1
domain: demo.depot
summary: A label is filed under its own identity, on an open shelf or on none, and read by that identity.
types:
  - {name: demo.depot.ShelfId, kind: newtype, of: String}
  - {name: demo.depot.LabelId, kind: newtype, of: String}
entities:
  - name: demo.depot.Shelf
    identity: {name: shelf_id, type: demo.depot.ShelfId}
    fields:
      - {name: title, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
  - name: demo.depot.Label
    identity: {name: label_id, type: demo.depot.LabelId}
    fields:
      - {name: shelf_id, type: Optional<demo.depot.ShelfId>}
      - {name: text, type: String}
      - {name: filed_at, type: Timestamp}
    relations:
      - {name: shelf, kind: references, target: demo.depot.Shelf, cardinality: one, via: shelf_id}
    lifecycle:
      initial: Filed
      states: [Filed, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Filed], to: Done}
actors:
  - name: demo.depot.Keeper
    may: [demo.depot.OpenShelf, demo.depot.CloseShelf, demo.depot.FileLabel, demo.depot.FinishLabel]
errors:
  - name: demo.depot.ShelfExists
    fields: [{name: shelf_id, type: demo.depot.ShelfId}]
  - name: demo.depot.NoSuchShelf
    fields: [{name: shelf_id, type: demo.depot.ShelfId}]
  - name: demo.depot.ShelfStateConflict
    fields: [{name: shelf_id, type: demo.depot.ShelfId}]
  - name: demo.depot.ShelfClosedToLabels
    fields: [{name: label_id, type: demo.depot.LabelId}]
  - name: demo.depot.NoSuchLabel
    fields: [{name: label_id, type: demo.depot.LabelId}]
  - name: demo.depot.LabelStateConflict
    fields: [{name: label_id, type: demo.depot.LabelId}]
commands:
  - name: demo.depot.OpenShelf
    input:
      - {name: shelf_id, type: demo.depot.ShelfId}
      - {name: title, type: String}
    outcomes:
      - name: opened
        creates: demo.depot.Shelf
        instance: shelf_id
        sets:
          title: input.title
        emits: [demo.depot.ShelfOpened]
        payload:
          demo.depot.ShelfOpened:
            shelf_id: input.shelf_id
      - name: shelf-exists
        existing_instance: true
        error: demo.depot.ShelfExists
  - name: demo.depot.CloseShelf
    input:
      - {name: shelf_id, type: demo.depot.ShelfId}
    outcomes:
      - name: no-shelf
        unknown_instance: true
        error: demo.depot.NoSuchShelf
      - name: closed
        moves: demo.depot.Shelf.close
        instance: shelf_id
        emits: [demo.depot.ShelfClosed]
        payload:
          demo.depot.ShelfClosed:
            shelf_id: input.shelf_id
      - name: wrong-state
        wrong_state: true
        error: demo.depot.ShelfStateConflict
  - name: demo.depot.FileLabel
    input:
      - {name: label_id, type: demo.depot.LabelId}
      - {name: shelf_id, type: Optional<demo.depot.ShelfId>}
      - {name: text, type: String}
      - {name: at, type: Timestamp}
    outcomes:
      - name: no-shelf
        when_related: {via: input.shelf_id, exists: false}
        error: demo.depot.NoSuchShelf
      - name: shelf-closed
        when_related: {via: input.shelf_id, predicate: state == Closed}
        error: demo.depot.ShelfClosedToLabels
      - name: filed
        creates: demo.depot.Label
        instance: label_id
        sets:
          shelf_id: input.shelf_id
          text: input.text
          filed_at: input.at
        emits: [demo.depot.LabelFiled]
        payload:
          demo.depot.LabelFiled:
            label_id: input.label_id
  - name: demo.depot.FinishLabel
    input:
      - {name: label_id, type: demo.depot.LabelId}
    outcomes:
      - name: no-label
        unknown_instance: true
        error: demo.depot.NoSuchLabel
      - name: finished
        moves: demo.depot.Label.finish
        instance: label_id
        emits: [demo.depot.LabelFinished]
        payload:
          demo.depot.LabelFinished:
            label_id: input.label_id
      - name: wrong-state
        wrong_state: true
        error: demo.depot.LabelStateConflict
events:
  - name: demo.depot.ShelfOpened
    fields: [{name: shelf_id, type: demo.depot.ShelfId}]
  - name: demo.depot.ShelfClosed
    fields: [{name: shelf_id, type: demo.depot.ShelfId}]
  - name: demo.depot.LabelFiled
    fields: [{name: label_id, type: demo.depot.LabelId}]
  - name: demo.depot.LabelFinished
    fields: [{name: label_id, type: demo.depot.LabelId}]
views:
  - name: demo.depot.Shelves
    source: demo.depot.Shelf
    consistency: read_your_writes
    fields:
      - {name: shelf_id, type: demo.depot.ShelfId}
      - {name: title, type: String}
      - {name: state, type: demo.depot.Shelf.State}
  - name: demo.depot.Labels
    source: demo.depot.Label
    consistency: read_your_writes
    fields:
      - {name: label_id, type: demo.depot.LabelId}
      - {name: state, type: demo.depot.Label.State}
  - name: demo.depot.LabelDetail
    source: demo.depot.Label
    consistency: read_your_writes
    params:
      - {name: label_id, type: demo.depot.LabelId}
    filter: label_id == param.label_id
    fields:
      - {name: label_id, type: demo.depot.LabelId}
      - {name: shelf_id, type: Optional<demo.depot.ShelfId>}
      - {name: text, type: String}
      - {name: state, type: demo.depot.Label.State}
```

On 0.97.3: `12 scenario(s) (0 authored), 1 refusal(s)`, the refusal `outcome demo.depot.FileLabel/filed has no scenario `demo.depot.FileLabel/outcome/filed`` naming `label_id … names one row per value`. The same file without the view `demo.depot.LabelDetail` gives `13 scenario(s), 0 refusal(s)`, and its absent run sends `label_id-1` and captures `label-2`.

### The tests

Change the import at line 29 to `use ess_conformance::{AdmittedSuite, ConformanceScenario, Runner, ScenarioStep};` and add `use ess_primitives::node::Node;` (`ess-primitives` is a dependency of the crate, `Cargo.toml` line 12; `tests/arranged_identities_stay_distinct.rs` imports both the same way). The function `executed` (line 598) keeps its own inner `use ess_conformance::scenario::ScenarioStep;`. Then add:

```rust
/// A creation guarded on its own row's absence: `existing_instance:` written as a `when_related:`
/// guard (story:a-creating-branchs-scenario-sends-every-row-it-creates-under-an-identity-of-its-own).
const CREATE_IF_ABSENT: &str = include_str!("fixtures/create-if-absent.yaml");

/// A creating branch whose Optional reference is left out in a further run, beside a view read by
/// the created row's identity (same story).
const ABSENT_REFERENCE_IDENTITY_VIEW: &str =
    include_str!("fixtures/absent-reference-identity-view.yaml");

/// The scenario `id` of `synthesis`.
fn scenario<'s>(synthesis: &'s Synthesis, id: &str) -> &'s ConformanceScenario {
    synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
        .unwrap_or_else(|| panic!("no `{id}` in {:?}", ids(synthesis)))
}

/// Every send of `command` in `scenario` that the next step expects answered by `outcome`, as its
/// input, in order.
fn sends<'s>(
    scenario: &'s ConformanceScenario,
    command: &str,
    outcome: &str,
) -> Vec<&'s BTreeMap<String, ess_conformance::ScenarioValue>> {
    let mut found = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand {
            command: sent,
            input,
            ..
        } = step
        else {
            continue;
        };
        if sent.to_string() != command {
            continue;
        }
        if let Some(ScenarioStep::ExpectOutcome { outcome: answered }) = steps.peek() {
            if answered.outcome.to_string() == outcome {
                found.push(input);
            }
        }
    }
    found
}

/// The literal `field` of each of `sends`, a send naming a captured instance passed over.
fn literals(sends: &[&BTreeMap<String, ess_conformance::ScenarioValue>], field: &str) -> Vec<Node> {
    sends
        .iter()
        .filter_map(|input| input.get(field).and_then(|value| value.as_literal()).cloned())
        .collect()
}

/// The ids of the scenarios whose sends of `command` answered by `outcome` carry one literal
/// `field` twice.
fn repeating(synthesis: &Synthesis, command: &str, outcome: &str, field: &str) -> Vec<String> {
    synthesis
        .suite
        .scenarios
        .iter()
        .filter(|(_, scenario)| {
            let sent = literals(&sends(scenario, command, outcome), field);
            let distinct: std::collections::BTreeSet<&Node> = sent.iter().collect();
            distinct.len() != sent.len()
        })
        .map(|(id, _)| id.to_string())
        .collect()
}

/// Every scenario of `synthesis` passes against the interpreter.
fn all_pass(label: &str, model: EssIr, synthesis: &Synthesis) {
    let failing: Vec<_> = run(model, synthesis)
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "{label}: not passed against interpreted: {failing:?}");
}
```

If `ScenarioValue::as_literal` or the `ExpectOutcome` field names differ from what `tests/arranged_identities_stay_distinct.rs` lines 332-361 use, follow that file: it is the working model of these helpers.

Part A (commit A):

1. `a_creation_guarded_on_its_own_rows_absence_is_witnessed`:
   - `let model = ir(CREATE_IF_ABSENT); let synthesis = ess_conformance::synthesize::synthesize(&model);`
   - `assert_eq!(refusals(&synthesis), Vec::<String>::new());` Red before commit A: one entry beginning `demo.depot.OpenShelf/outcome/opened ESS-SYNTH-001:` and containing `names one row per value`.
   - `ids(&synthesis)` contains `demo.depot.OpenShelf/outcome/opened` and `demo.depot.OpenShelf/outcome/shelf-exists`.
   - `all_pass("create if absent", model, &synthesis)`.
2. `every_shelf_one_scenario_opens_has_an_identity_of_its_own`:
   - Same model. `assert_eq!(repeating(&synthesis, "demo.depot.OpenShelf", "opened", "shelf_id"), Vec::<String>::new());`
   - In `demo.depot.OpenShelf/outcome/opened`: `literals(&sends(..., "demo.depot.OpenShelf", "opened"), "shelf_id")` has length 3 (the two decoy rows of `arranged_at` and the branch's own send), all distinct. Red before commit A: the scenario is not in the suite and `scenario` panics. If the count comes out other than 3 with every other assertion green, print the scenario's steps in the failure message and report it; do not lower it below 2.

Part B (commit B):

3. `an_absent_reference_run_beside_an_identity_view_creates_a_label_of_its_own`:
   - `let model = ir(ABSENT_REFERENCE_IDENTITY_VIEW); let synthesis = ess_conformance::synthesize::synthesize(&model);`
   - `assert_eq!(refusals(&synthesis), Vec::<String>::new());` Red before commit B: one entry beginning `demo.depot.FileLabel/outcome/filed ESS-SYNTH-001:` containing `names one row per value`.
   - In `demo.depot.FileLabel/outcome/filed`, `let filed = sends(scenario(&synthesis, "demo.depot.FileLabel/outcome/filed"), "demo.depot.FileLabel", "filed");`: the literal `label_id`s of `filed` number at least 3 (the full run, the companion of `demo.depot.LabelDetail`, the absent run) and are all distinct; at least one send in `filed` has no `shelf_id` key (the absent run).
   - `assert_eq!(repeating(&synthesis, "demo.depot.FileLabel", "filed", "label_id"), Vec::<String>::new());`
   - `all_pass("absent reference beside an identity view", model, &synthesis)`.

The story's acceptance "a guard that leaves one value is still refused with the same text" is the existing `no_scenario_creates_one_identity_twice_under_a_bound_the_witness_breaks` in `tests/arranged_identities_stay_distinct.rs` (line 463). It must pass unchanged; do not edit that file.

## Commands

From the worktree root. Bounds: no single test over 60 s; a test step within 120 s; time each new test and name any over 5 s. A pool job ends after 10 minutes unless it names its time: the first build of a target below is named `SLOT_TIMEOUT=30m` (at most 30 minutes); never wrap a command in `timeout`. Run only the named `--test` targets, one per command, each with `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0`. `cargo` goes through the `slot` shim on its own. Do not set `RUSTC_WRAPPER`, `SCCACHE_DIR`, `SCCACHE_SERVER_PORT` or `SCCACHE_CACHE_SIZE`: the host's one compile cache applies (`~/beyond10x/AGENTS.md`, "One compile cache", replaces the "Local compile cache" paragraph of this repository's `AGENTS.md`). A script that waits or measures runs without `slot`. No Python anywhere.

0. `BASE=$(git rev-parse HEAD)`: the wave base (`16cde5949b`).

Commit A:

1. Write fixture A and tests 1 and 2, then `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test existing_instance_beside_related_guards`. Before steps 1-2: tests 1 and 2 fail as `## Tests first` says, every older test passes.
2. Apply `## What to change` steps 1-3, then the same command: all pass.
3. `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test arranged_identities_stay_distinct`: passes unchanged (the one-value guard is still refused with the same text).
4. One command each, all pass: `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test related_guard_absent_input`, `--test related_guard_optional`, `--test upsert_by_existence`, `--test interpreted_existence`.
5. `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table`: fails because the tree holds a model the table does not pin, and prints its re-pin command. Run that command as printed:
   `SYNTHESIS_SUITE_BYTES_TABLE_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table`
   then the plain command again: passes. This target holds two tables. The fixtures declare no component, so the component table (`synthesis-component-suites-table.tsv`, header `#	models=102	lines=124`) must not change. If the test prints the `SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE` command, stop and report instead of re-pinning it.
6. The same for the two other tables, each re-pinned with the command its own failure prints:
   - `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test selection_precedence_table`, then `SELECTION_PRECEDENCE_TABLE_WRITE="$PWD/crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test selection_precedence_table`, then the plain command: passes.
   - `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowered_definitions_table`, then `LOWERED_DEFINITIONS_TABLE_WRITE="$PWD/crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowered_definitions_table`, then the plain command: passes.
7. The proof that no pinned row moved, for each of the three tables `T` (`crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv`, `crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv`, `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv`):
   `comm -23 <(git show "$BASE:$T" | sort) <(sort "$T")` prints exactly one line, the old header (`#	models=483	lines=483`, `#	models=401	commands=824`, `#	models=483	lines=664` respectively), and `comm -13 <(git show "$BASE:$T" | sort) <(sort "$T")` prints the new header and only lines that begin with `crates/verify/ess-conformance/tests/fixtures/create-if-absent.yaml`. Expected after A: one new line in the suite table and in the lowered table, one `ir=` line and two command lines (`demo.depot.CloseShelf`, `demo.depot.OpenShelf`) in the selection table. The write mode sets the header counts. Any other line is a moved pin: stop and report it with its old and new text.
8. Byte pins of `AGENTS.md` this change could move, one command each, all pass with no re-pin: `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table_cli_read`, `--test adversary_related_values_all_bytes`, `--test external_beside_held_guard`.
9. `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`: no warnings. `cargo fmt -p ess-conformance`, then `slot task fmt-check`: clean. Never `cargo fmt --all`.
10. Commit A (message in step 4 of `## What to change`).

Commit B:

11. Write fixture B and test 3, then `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test existing_instance_beside_related_guards`: test 3 fails, tests 1 and 2 and the older ones pass.
12. Apply `## What to change` steps 5-6, then the same command: all pass.
13. Repeat commands 3 and 4: all pass.
14. Repeat 5, 6 and 7 for `crates/verify/ess-conformance/tests/fixtures/absent-reference-identity-view.yaml`, with `BASE` replaced by `HEAD` (commit A). Expected: one new line in the suite and lowered tables, one `ir=` line and four command lines (`demo.depot.CloseShelf`, `demo.depot.FileLabel`, `demo.depot.FinishLabel`, `demo.depot.OpenShelf`) in the selection table. Then once against `BASE` again: `comm -23` prints only the base header for each table.
15. Repeat command 8.
16. `ess-conformance` is a dependency of `ess-synth` and `ess-cli`, so run these byte pins of `AGENTS.md` too, one command each, all pass unchanged: `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test served_codec_names`, `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test go` (it needs `go` on `PATH`; if it is not there, say so in your report), `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test tutorial_page`. `ess-openapi` (`adversary_ess_issues_2_u6`) and `ess-compiler` (`string_alphabet_and_length`) do not depend on `ess-conformance` and cannot move: do not run them.
17. `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-conformance --all-targets --locked -- -D warnings` and `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-cli --all-targets --locked -- -D warnings`: no warnings. `slot task fmt-check`: clean.
18. Commit B (message in step 7 of `## What to change`).

Never the whole workspace, never `task check`, never `cargo test -p ess-conformance` without `--test`.

## Files this unit owns

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (commit A: `creates_if_absent`, the `exists: false` arm of `drive`)
- `crates/verify/ess-conformance/src/synthesize.rs` (commit B: `RELATED_ABSENT_TRIES`, the absent-reference block of `outcome_scenario_in`)
- `crates/verify/ess-conformance/tests/existing_instance_beside_related_guards.rs`
- `crates/verify/ess-conformance/tests/fixtures/create-if-absent.yaml` (new)
- `crates/verify/ess-conformance/tests/fixtures/absent-reference-identity-view.yaml` (new)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` (written by the re-pin command only)
- `crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv` (written by the re-pin command only)
- `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv` (written by the re-pin command only)

Must not touch:

- `crates/verify/ess-conformance/src/witness.rs`, `src/witness_memo.rs`, `src/synthesize/caller.rs`: a parallel performance track owns them.
- `crates/verify/ess-conformance/src/synthesize/made.rs`, `src/synthesize/subject_fact.rs`: the 0.97.4 memo (`row_picks`, `RowPicks`).
- `crates/verify/ess-conformance/src/synthesize/row_set.rs`: the sibling story on row-set refusals beside a creator that needs a related row.
- `crates/verify/ess-conformance/src/synthesize/existence.rs`, `src/synthesize/repeated_identity.rs`, `src/synthesize.rs` `fn fresh_identity` (line 18577), `fn own_identity` (line 9448) and the help text at line 1523.
- `crates/verify/ess-conformance/tests/arranged_identities_stay_distinct.rs`, `tests/fixtures/synthesis-component-suites-table.tsv`.
- `CHANGELOG.md`, `website/`, `docs/`, `AGENTS.md`, `Taskfile.yml`, `.github/workflows/*`, `crates/edge/ess-xtask/tests/ci_lanes.rs`, `README.md`.

## For the coordinator at integration

- `CHANGELOG.md`, under `## [Unreleased]`, `### Fixed` (the section exists), append:

  ```markdown
  - `ess verify conform synthesize` witnesses a creating branch guarded on the absence of the row it creates (`when_related: {via: input.<id>, exists: false}` beside `creates:` with `instance: <id>`), where it withdrew that branch's scenario, and every scenario whose arrangement needed two such rows, as `ESS-SYNTH-001` "names one row per value": each row the branch creates for an arrangement carries its own identity. The further run that leaves an Optional reference out sends an identity no row of its scenario holds, also beside a view filtered on the created row's identity. Every other suite is byte for byte unchanged.
  ```

- `website/docs/reference/predicates.md`, in the paragraph that starts `Synthesis sends the `exists: false` branch an identity no row carries` (line 1373), after its first sentence (`… and with an input an accepting `when:` branch would take.`, ending on line 1374), insert:

  ```markdown
  Where that branch `creates:` the entity its `via:` names and takes the new row's identity from the same input (`instance:`), the guard is the creation's own existence test: every row synthesis creates through the branch for an arrangement carries an identity of its own, and the default branch is sent the identity of such a row, as beside `existing_instance:`.
  ```

- The three pin tables: a unit that adds or changes any YAML model in this wave also re-pins them. Merge the units one after another and re-run the three write commands of `## Commands` 5-6 after the last merge, then the `comm` proof against the wave base.

## Out of scope

- Witnessing the default branch exactly as `existing_instance:` is (snapshot, the identity sent again with other values, no event, row unchanged). The story's design lists it; this unit keeps the default branch's present-row witness, which the fixture shows passing. It is a follow-up.
- A help line naming `existing_instance:` on the remaining `names one row per value` refusal. After commit A that refusal is left only where a guard on the identity leaves one value, which `existing_instance:` does not cure either. Keep the help text at `synthesize.rs` line 1523 as it is.
- A validation warning or refusal for the guard form (story decision D1, options b and c). No new `ValidationCode`, no format version.
- Changing `fresh_identity`, `without_row_each`, `without_row`, `missing_sends`, `arranged_at` or the scenario's own send. The identity no row carries is right for the `exists: false` send itself.
- Any change to `Beside::next`, `arrange_matching` or `earlier_runs`; the companion numbering stays as it is.
- Moving the absent-reference run where it does not repeat an identity: that would move committed suites.
- `ABSENT_REFERENCE_WITNESS` / `Witness::RelatedValueAbsent` (`{related:}` copies): a different run, not involved.
- Adding a new test binary; editing the tables by hand; running the whole workspace or `task check`.

## Acceptance

- Test 1 (`a_creation_guarded_on_its_own_rows_absence_is_witnessed`) passes: fixture A synthesizes with 0 refusals, holds `demo.depot.OpenShelf/outcome/opened` and `/outcome/shelf-exists`, and every scenario passes against the interpreter (Commands 2).
- Test 2 passes: no scenario of fixture A sends one literal `shelf_id` to two `opened` creations; the `opened` scenario creates three shelves under three identities (Commands 2).
- Test 3 passes: fixture B synthesizes with 0 refusals; `demo.depot.FileLabel/outcome/filed` sends at least three distinct literal `label_id`s, one of them without `shelf_id`; every scenario passes against the interpreter (Commands 12).
- `arranged_identities_stay_distinct` passes unchanged (Commands 3, 13).
- The `comm` proof shows no moved row in any of the three tables; each gained exactly the lines of the two fixtures (Commands 7, 14).
- `synthesis_suite_bytes_table_cli_read`, `adversary_related_values_all_bytes`, `external_beside_held_guard`, ess-synth `served_codec_names` and `go`, and ess-cli `tutorial_page` pass with no re-pin (Commands 8, 15, 16).
- Clippy on `ess-conformance` and `ess-cli` is clean and `slot task fmt-check` is clean (Commands 9, 17).
- No format version changes.

## Depends on

nothing. It is written against `16cde5949b` (0.97.4 with the content-keyed witness memo merged).
