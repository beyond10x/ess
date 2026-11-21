---
format: aep.planning-md/3
id: story:each-code-target-admits-binary64-through-a-switch-of-its-own
kind: story
status: active
title: Each code target admits a finite Binary64 through a switch of its own, and guards, filters and orders read one as a number
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:every-code-target-carries-a-finite-binary64
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/binary64_publication.rs
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: cited
  path: crates/generate/ess-synth/src/failure.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/binary64.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/mod.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/binary64.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/view_query.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/web/binary64.rs
- confidence: cited
  path: crates/generate/ess-synth/src/web/mod.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/binary64_plan.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/binary64_served/support.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/bounded_accessor.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/clock_reading.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/feasibility.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/fixtures/binary64-served/model.yaml.in
- confidence: cited
  path: crates/generate/ess-synth/tests/go_owner.rs
revision: 20
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T23:43:17Z", actor: "human:timo", revision: 19}
- {from: "proposed", to: "active", at: "2026-11-21T23:43:18Z", actor: "human:timo", revision: 20}
---
## Outcome

- `failure::binary64` reads `CARRIED` in new `src/{rust,go,web}/binary64.rs` (all `false`); a `true`
  target is refused only for an uncarried Binary64, with a new detail.
- Guards, `filter:` and `order_by:` over a `Binary64` are determined; membership stays owed.
- Tests that used a Binary64 as a refusal trigger use a Binary64 identity, which stays refused.
- The fixture model and the shared served-test module (`support.rs`) for B2 and B3 exist and compile.
- No target's output changes. No format version changes.

## What to change

Line numbers are at `origin/main` `f3e4477974` (release 0.97.1). All paths are under
`crates/generate/ess-synth/` unless written in full.

1. **New file `src/rust/binary64.rs`**, exactly:

   ```rust
   //! How the Rust target carries a finite `Binary64`.

   /// `true` once this target's codec carries a finite `Binary64` in every carried position
   /// (`crate::failure::binary64` reads it). Flipped by the unit that adds the codec, never before
   /// the generated code compiles.
   pub(crate) const CARRIED: bool = false;
   ```

2. **New file `src/go/binary64.rs`**: the same text with `Rust` replaced by `Go` in the first line.
   **New file `src/web/binary64.rs`**: the same, first line `//! How the browser target carries a
   finite \`Binary64\`.`, and the doc of `CARRIED` adding one sentence: `The browser target carries
   one only when the Rust target does too: its bridge compiles that target's wire module.`

3. **`src/rust/mod.rs`**, module list at lines 29-50: add `pub(crate) mod binary64;` between
   `mod actor;` (line 30) and `pub(crate) mod behaviour;` (line 31).
   **`src/go/mod.rs`**, module list at lines 40-58: add `pub(crate) mod binary64;` between
   `mod accessor;` (40) and `mod behaviour;` (41).
   **`src/web/mod.rs`**, module list at lines 31-35: add `pub(crate) mod binary64;` before
   `mod bridge;` (31).
   It must be `pub(crate)`: `failure.rs` sits at the crate root and reads the constants. Touch
   nothing else in these three files.

4. **`src/failure.rs`**, `pub(crate) fn binary64` at lines 156-176. Today it makes one cause per
   `ess_compiler::binary64::locations(ir)` entry with the detail `this target has no qualified
   finite Binary64 codec`, for every target. Replace lines 156-176 with:

   ```rust
   /// The detail of a `Binary64` position a target with no codec for it refuses.
   const NO_BINARY64_CODEC: &str = "this target has no qualified finite Binary64 codec";

   /// The detail of a `Binary64` position a target with a codec still does not carry.
   const UNCARRIED_BINARY64: &str = "a Binary64 here is a key, a lookup or a delivery route, which \
       this target does not carry; it carries one in entity fields, command input and response, \
       event and error fields, and view fields and row";

   /// Whether `target` carries a finite `Binary64`: each target's own switch, so the unit that
   /// lands a codec flips one constant in a file it owns.
   fn carries_binary64(target: Target) -> bool {
       match target {
           Target::Rust => crate::rust::binary64::CARRIED,
           Target::Go => crate::go::binary64::CARRIED,
           Target::Web => crate::rust::binary64::CARRIED && crate::web::binary64::CARRIED,
           Target::Clap => false,
       }
   }

   /// One cause per `Binary64` position the target refuses: every position where it has no codec,
   /// and where it has one, every position the suite contract does not carry either
   /// (`ess_compiler::binary64::uncarried_uses`).
   fn binary64_causes(ir: &ess_compiler::EssIr, carried: bool) -> Vec<TargetFailureCause> {
       let (locations, detail): (Vec<String>, &str) = if carried {
           (
               ess_compiler::binary64::uncarried_uses(ir).into_keys().collect(),
               UNCARRIED_BINARY64,
           )
       } else {
           (
               ess_compiler::binary64::locations(ir).into_iter().collect(),
               NO_BINARY64_CODEC,
           )
       };
       locations
           .into_iter()
           .map(|at| {
               TargetFailureCause::new(
                   TargetFailureCode::MissingRepresentation,
                   vec![at],
                   detail.to_owned(),
               )
           })
           .collect()
   }

   pub(crate) fn binary64(
       ir: &ess_compiler::EssIr,
       plan: &SynthesisPlan,
       target: Target,
   ) -> Result<(), TargetFailure> {
       let causes = binary64_causes(ir, carries_binary64(target));
       if causes.is_empty() {
           Ok(())
       } else {
           Err(TargetFailure::new(ir, target, plan, causes))
       }
   }
   ```

   The `string` continuation (`\` + newline + indentation) collapses to single spaces; the detail
   must read exactly: `a Binary64 here is a key, a lookup or a delivery route, which this target
   does not carry; it carries one in entity fields, command input and response, event and error
   fields, and view fields and row`. The callers (`lib.rs:488`, `rust/mod.rs:192`, `go/mod.rs:319`,
   `web/mod.rs:300`, `clap/mod.rs:63`) do not change. `Target` is already imported (line 7). If
   `ess_compiler::EssIr` does not name the type, use the path the old function used.

5. **`src/failure.rs`**, end of file (after line 364): add a unit-test module (see Tests first).

6. **`src/determined.rs`**:
   - `enum Kind`, variant doc at line 892: `/// An \`Integer\` or \`Decimal\` (or a count), compared
     by value.` becomes `/// An \`Integer\`, \`Decimal\` or \`Binary64\` (or a count), compared by
     value.`
   - Primitive mapping at line 1140: `Primitive::Integer | Primitive::Decimal => Kind::Number(*name),`
     becomes `Primitive::Integer | Primitive::Decimal | Primitive::Binary64 => Kind::Number(*name),`.
     Today a `Binary64` falls to the `Opaque` arm below, so every guard, filter and order over one
     is owed ("a literal of another kind than the value it is compared with").
   - `Predicate::AnyOf { path, values } | Predicate::NoneOf { path, values }` at lines 1327-1332.
     Today: `let kind = resolve(ir, env, path)?.kind;` then the `literal_matches` loop. Insert after
     the `let kind` line:

     ```rust
     // A generated guard tests membership by the text it reads, and `1` and `1.0` name one
     // binary64: no membership over one is generated.
     if kind == Kind::Number(Primitive::Binary64) {
         return Err(format!(
             "`{predicate}`, membership over a Binary64, which no generated guard compares"
         ));
     }
     ```
     Do not touch the coalesce arms at lines 1186-1196 (a `Binary64` default stays owed).

7. **`src/view_query.rs`**, `enum Values`, doc of `Numeric` at line 342: `/// \`Integer\` or
   \`Decimal\`: by exact numeric value.` becomes `/// \`Integer\`, \`Decimal\` or \`Binary64\`: by
   exact numeric value.` Nothing else: `values()` (line 353) already maps every `Kind::Number(_)`
   to `Values::Numeric`, and `query_scalar` (line 204) keeps `Binary64` unqueryable as a parameter.

8. **New file `tests/fixtures/binary64-served/model.yaml.in`** (`.in` so the repository's model
   walkers skip it), exactly:

   ```yaml
   format: ess/37
   system: demo
   version: v1
   domain: demo.search
   summary: Documents with finite scores in every value position.
   types:
     - {name: demo.search.DocId, kind: newtype, of: Uuid}
     - {name: demo.search.Ratio, kind: newtype, of: Binary64, invariants: ["value >= 0", "value <= 1"]}
     - name: demo.search.Hit
       kind: struct
       fields:
         - {name: label, type: String}
         - {name: score, type: Binary64}
         - {name: maybe, type: Optional<Binary64>}
         - {name: many, type: List<Binary64>}
         - {name: by_name, type: "Map<String, Binary64>"}
   entities:
     - name: demo.search.Doc
       identity: {name: doc_id, type: demo.search.DocId}
       fields:
         - {name: added, type: Binary64}
         - {name: ratio, type: demo.search.Ratio}
         - {name: hit, type: demo.search.Hit}
       lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
   actors:
     - {name: demo.search.Clerk, may: [demo.search.AddDoc]}
   commands:
     - name: demo.search.AddDoc
       input:
         - {name: added, type: Binary64}
         - {name: ratio, type: demo.search.Ratio}
         - {name: hit, type: demo.search.Hit}
       outcomes:
         - name: added
           when: "input.added >= 0.5"
           creates: demo.search.Doc
           instance: doc_id
           sets: {added: input.added, ratio: input.ratio, hit: input.hit}
           emits: [demo.search.DocAdded]
           payload:
             demo.search.DocAdded: {doc_id: {generated: true}, added: input.added}
         - name: rejected
           error: demo.search.TooLow
           payload:
             demo.search.TooLow: {added: input.added}
   errors:
     - name: demo.search.TooLow
       fields:
         - {name: added, type: Binary64}
   events:
     - name: demo.search.DocAdded
       fields:
         - {name: doc_id, type: demo.search.DocId}
         - {name: added, type: Binary64}
   views:
     - name: demo.search.DocsByAdded
       source: demo.search.Doc
       consistency: read_your_writes
       filter: "added > 0.75"
       order_by: [added desc, doc_id asc]
       fields:
         - {name: doc_id, type: demo.search.DocId}
         - {name: added, type: Binary64}
         - {name: ratio, type: demo.search.Ratio}
         - {name: hit, type: demo.search.Hit}
   components:
     - component: search-service
       reached_by: network
       owns: {domains: [demo.search]}
       accepts: {commands: [demo.search.AddDoc]}
       publishes: {events: [demo.search.DocAdded]}
   ```

   Checked with `ess` 0.97.1: valid; its suite has exactly the scenarios
   `demo.search.AddDoc/outcome/added` and `demo.search.AddDoc/outcome/rejected`; Clap refuses it at
   `command.demo.search.AddDoc.input.added`, `entity demo.search.Doc.fields.added`,
   `error.demo.search.TooLow.fields.added`, `event.demo.search.DocAdded.fields.added`,
   `types.demo.search.Hit.fields.by_name.value`, `types.demo.search.Hit.fields.many.of`,
   `types.demo.search.Hit.fields.maybe.of`, `types.demo.search.Hit.fields.score`,
   `types.demo.search.Ratio.of`, `view.demo.search.DocsByAdded.fields.added`. The same model with
   `Binary64` replaced by `Decimal` plans 14 of 14 capabilities generated.

9. **New file `tests/binary64_served/support.rs`**: the cases units B2 (Rust server) and B3 (Go
   server) run, included by their tests with `#[path = "binary64_served/support.rs"] mod support;`.
   Cargo does not build a file in a subdirectory of `tests/` as a target. It is modelled on
   `tests/integer_spellings.rs` (lines 83-106 `model`/`routes`, 233-308 `Served`, 420-557 the
   `ConformanceTarget` impl) and on `tests/generated_view_queries.rs:1383-1400` (a declared error
   read from `refusal`). Write it as below; keep every name, constant and literal.

   ```rust
   //! The cases a generated server carrying a finite `Binary64` is held to, shared by
   //! `binary64_rust.rs` and `binary64_go.rs`: the fixture model, the harnesses' line protocol, the
   //! wire table and the synthesized suite.
   //!
   //! A harness speaks one JSON object per line each way, as `integer_spellings.rs`' harnesses do:
   //! `{"op":"reset"}`, `{"op":"command","command":…,"actor":…,"body":…}` and
   //! `{"op":"view","view":…,"query":…}`, each answered `{"status":N,"body":…}` with the served
   //! body verbatim. The wire cases read the raw answer line, never a value a JSON parser rebuilt,
   //! so a token is checked as the server wrote it.

   #![allow(dead_code)]

   use std::cell::RefCell;
   use std::io::{BufRead as _, BufReader, Write as _};
   use std::path::Path;
   use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
   use std::time::{Duration, Instant};

   use ess_compiler::ir::EssIr;
   use ess_compiler::resolve::compile_locating;
   use ess_compiler::source::SourceMap;
   use ess_conformance::report::Status;
   use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
   use ess_conformance::target::{
       ConformanceTarget, DeclaredErrorValue, EventObservationRequest, ExternalOutcomeControl,
       ImplementationIdentity, ObservedEvent, RedeliveryRequest, ScenarioContext,
       SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
       TargetError, ViewRow,
   };
   use ess_conformance::{AdmittedSuite, Runner};
   use ess_domain::spec::{RawSpecFile, Specification};
   use ess_domain::system::Source;
   use ess_primitives::consistency::ConsistencyToken;
   use ess_primitives::node::Node;
   use serde_json::{json, Value};

   /// The model, read by `task binary64-harness` too, which generates the Rust server from it.
   pub const MODEL: &str = include_str!("../fixtures/binary64-served/model.yaml.in");
   pub const ADD: &str = "demo.search.AddDoc";
   pub const DOCS: &str = "demo.search.DocsByAdded";
   pub const CLERK: &str = "demo.search.Clerk";

   /// Each score token sent, and the token the server must write back.
   pub const WRITTEN: [(&str, &str); 15] = [
       ("0.73", "0.73"),
       ("-0.0", "-0.0"),
       ("-0", "-0.0"),
       ("3", "3.0"),
       ("1e16", "1e16"),
       ("1e15", "1000000000000000.0"),
       ("0.00001", "0.00001"),
       ("0.000001", "1e-6"),
       ("9007199254740993", "9007199254740992.0"),
       ("5e-324", "5e-324"),
       ("1e-400", "0.0"),
       ("-1e-400", "-0.0"),
       ("1.7976931348623157e308", "1.7976931348623157e308"),
       ("123.456", "123.456"),
       ("1.5e300", "1.5e300"),
   ];

   /// Each score a server must refuse with `400`, and what the refusal says it found.
   pub const REFUSED: [(&str, &str); 6] = [
       ("1e400", "the number 1e400"),
       ("-1e400", "the number -1e400"),
       ("1.7976931348623159e308", "the number 1.7976931348623159e308"),
       ("1e999999999", "the number 1e999999999"),
       ("\"0.73\"", "a string"),
       ("true", "a boolean"),
   ];
   ```

   then these items, each with a one-line doc comment:

   - `pub fn compile(source: &str) -> EssIr`: the body of `integer_spellings.rs::model` (lines
     83-91) over `source`, label `model.yaml`.
   - `pub fn model() -> EssIr { compile(MODEL) }`.
   - `pub fn routes(ir: &EssIr) -> Vec<String>`: `integer_spellings.rs` lines 94-106, verbatim.
   - `pub struct Served` with the fields of `integer_spellings.rs` lines 233-239, and
     `impl Served` with: `pub fn start(binary: &Path, ir: &EssIr) -> Self` (lines 242-259
     verbatim); `pub fn raw(&self, request: &Value) -> String` (the body of `ask`, lines 261-270,
     returning `line.trim_end().to_owned()`); `pub fn ask(&self, request: &Value) -> (u64, Value)`
     (parses `self.raw(request)` as lines 271-275 do); `pub fn reset(&self)` (lines 278-283);
     `pub fn add(&self, body: &str) -> (u64, Value)` sending
     `json!({"op": "command", "command": ADD, "actor": CLERK, "body": body})`;
     `pub fn docs_line(&self) -> String` returning
     `self.raw(&json!({"op": "view", "view": DOCS, "query": ""}))`; `fn tick(&self) -> u64` (lines
     296-301). `impl Drop for Served` as lines 303-308.
   - `pub fn body(added: &str, ratio: &str, hit: &str) -> String` returning
     `format!(r#"{{"added":{added},"ratio":{ratio},"hit":{{"label":"a",{hit}}}}}"#)`.
   - `pub fn tokens(line: &str, name: &str) -> Vec<String>`: for every occurrence of
     `"{name}":` in `line`, the text after it up to the next `,`, `}` or `]`
     (`rest.find([',', '}', ']'])`), in order.
   - `pub fn writes_every_token(language: &str, binary: &Path)`: starts one `Served`; for each
     `(sent, written)` of `WRITTEN`: `reset`, then
     `add(&body("1.5", "0.5", &format!(r#""score":{sent},"many":[],"by_name":{{}}"#)))`, asserts
     `answer["outcome"] == "added"`, and asserts `tokens(&served.docs_line(), "score") ==
     vec![written.to_owned()]`, each message naming `language`, `sent` and the line. Then `reset`,
     `add(&body("1.5", "0.5", r#""score":1.5,"maybe":1e16,"many":[-0,2.5e-7],"by_name":{"k":3}"#))`,
     outcome `added`, and the docs line contains each of `"maybe":1e16`, `"many":[-0.0,2.5e-7]`,
     `"by_name":{"k":3.0}`.
   - `pub fn refuses_what_no_binary64_holds(language: &str, binary: &Path)`: for each
     `(sent, found)` of `REFUSED`: `reset`; time the `add` of
     `body("1.5", "0.5", &format!(r#""score":{sent},"many":[],"by_name":{{}}"#))`; assert status
     `400`; assert `answer.to_string()` contains
     `format!("hit.score: expected a finite Binary64 number, found {found}")`; assert it took under
     2 s; assert `tokens(&served.docs_line(), "score")` is empty. Then `reset` and add
     `body("1.5", "1.0000000000000002", r#""score":1.5,"many":[],"by_name":{}"#)`: status `400`
     (the `Ratio` invariant `value <= 1`), and the answer text contains `ratio`.
   - `pub fn compares_by_value(language: &str, binary: &Path)`: `reset`; for `added` in
     `["2.5", "10.25", "0.75", "0.7500000000000001", "1e16"]` add
     `body(added, "0.5", r#""score":1.5,"many":[],"by_name":{}"#)`, each outcome `added`; assert
     `tokens(&served.docs_line(), "added") == ["1e16", "10.25", "2.5", "0.7500000000000001"]`
     (the filter `added > 0.75` drops `0.75`; the order is numeric, descending). Then `reset`; the
     same body with `added` `0.5` answers outcome `added`; with `0.49999999999999994` (the largest
     binary64 below 0.5) answers outcome `rejected`, and the raw answer line of that request
     (`served.raw(...)` with the same command request) contains `"added":0.49999999999999994`.
   - `impl ConformanceTarget for Served`: `integer_spellings.rs` lines 429-557 with three changes:
     identity `ImplementationIdentity::new("binary64-served", env!("CARGO_PKG_VERSION"))`;
     `execute_command` fills `error` from the answer's `refusal` member exactly as
     `generated_view_queries.rs:1383-1400` does (`DeclaredErrorValue::new(name)` plus every
     non-null payload member through `node_of`) and passes it in `SemanticCommandResult { error,
     .. }`; `query_view` inserts no `page` member (delete line 518). Copy `failure` and `node_of`
     (lines 420-427) beside it.
   - `pub fn passes_the_synthesized_suite(language: &str, binary: &Path)`:
     `let ir = model(); let suite = ess_conformance::synthesize(&ir).suite;`, admit it with
     `AdmittedSuite::from_suite(&suite)`, run `Runner::for_suite(&suite).run_admitted(&admitted,
     &Served::start(binary, &ir)).into_report()`, collect `(scenario.to_string(), status)` per
     result, assert the sorted ids are exactly `["demo.search.AddDoc/outcome/added",
     "demo.search.AddDoc/outcome/rejected"]` and every status is `Status::Passed`, naming
     `language` and the verdicts.

## Tests first

All fast: no generated code is compiled, every model is under 80 lines.

1. **`src/failure.rs`, new `#[cfg(test)] mod tests`** (red before step 4: `binary64_causes` does
   not exist). Build the IR as `src/rust/invariant.rs:1448-1457` does, from

   `"format: ess/2\nsystem: demo\nversion: v1\ndomains: [demo.core]\ndomain: demo.core\ntypes:\n  - {name: demo.core.Ratio, kind: newtype, of: Binary64}\nentities:\n  - name: demo.core.Gauge\n    identity: {name: gauge_id, type: Binary64}\n    fields:\n      - {name: level, type: demo.core.Ratio}\n    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}\n"`

   (valid under 0.97.1). Test `a_carrying_target_refuses_only_the_uncarried_binary64`:
   `binary64_causes(&ir, true)` has one cause, sources `["entity demo.core.Gauge.identity"]`,
   detail `UNCARRIED_BINARY64`, code `MissingRepresentation`. Test
   `a_target_without_a_codec_refuses_every_binary64`: `binary64_causes(&ir, false)` sources, in
   order, `["entity demo.core.Gauge.identity", "types.demo.core.Ratio.of"]`, each detail
   `NO_BINARY64_CODEC` (0.97.1's Clap names exactly these two).

2. **New `tests/binary64_plan.rs`**, starting with `#[path = "binary64_served/support.rs"] mod support;`
   (this also compiles the shared module):
   - `a_guard_a_filter_and_an_order_over_a_binary64_are_generated`: `SynthesisPlan::of(&support::model())`
     `.is_generated(CapabilityKind::CommandBehavior, support::ADD)` and
     `.is_generated(CapabilityKind::ViewQuery, support::DOCS)` are both true; the message prints
     `plan.to_canonical_json()`. **Red before step 6**: both are obligations ("a literal of another
     kind than the value it is compared with").
   - `membership_over_a_binary64_stays_owed`: source =
     `support::MODEL.replace("when: \"input.added >= 0.5\"", "when: {added: [0.5, 1.5]}")`
     (assert it changed); `plan.disposition_of(CapabilityKind::CommandBehavior, support::ADD)` is
     `Some(SynthesisDisposition::Obligation(owed))` and `serde_json::to_string(owed)` contains
     `membership over a Binary64`. Red before step 6 (the reason is the `Opaque` one).
   - `clap_refuses_the_fixture_at_every_binary64_location_with_the_earlier_detail`:
     `synthesize_for(&support::model(), Target::Clap)` is `Err`; its causes' sources are the ten
     locations listed in step 8 (compare sorted); every detail is `this target has no qualified finite
     Binary64 codec`. Green before and after (it guards the Clap arm).

3. **Restated refusal triggers.** Each test keeps its assertions on format, code and
   `detail().contains("Binary64")` and must pass both now and after units B2-B4 flip their switch,
   because a Binary64 identity is never carried.
   - `tests/feasibility.rs:51-68` `binary64_is_refused_even_when_no_component_consumes_the_type`:
     append to the model string after `of: Binary64}\n` the text
     `entities:\n  - name: demo.core.Gauge\n    identity: {name: gauge_id, type: demo.core.Ratio}\n    fields: []\n    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}\n`.
     Expected unchanged: one cause on all four targets (`types.demo.core.Ratio.of`), formats Rust
     and Web `ess-target-failure/1`, Go and Clap `/2` (checked with 0.97.1). Rename the test
     `binary64_is_refused_where_an_identity_reaches_it`.
   - `tests/go_owner.rs:124-143` `ownerless_binary64_keeps_the_earlier_refusal`: second `compile`
     argument becomes
     `"domain: demo.core\nentities:\n  - name: demo.core.Gauge\n    identity: {name: gauge_id, type: demo.Ratio}\n    fields: []\n    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}\n"`;
     replace the exact-detail `assert_eq!` (lines 139-142) with
     `assert!(facade.causes()[0].detail().contains("Binary64"));`. One cause at
     `types.demo.Ratio.of` (checked with 0.97.1).
   - `tests/bounded_accessor.rs:100-104`: replace the `replacen("name: result\n        type: String",
     "name: result\n        type: Binary64", 1)` with
     `.replacen("\nevents:\n", "\nentities:\n  - name: projection.core.Gauge\n    identity: {name: gauge_id, type: Binary64}\n    fields: []\n    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}\nevents:\n", 1)`
     and assert the text changed. Checked with 0.97.1: `/3` and source
     `entity projection.core.Gauge.identity` on all four targets; the part before `bindings:` gives
     `/1` for Rust and Web and `/2` for Go and Clap, as the test already expects.
   - `tests/clock_reading.rs:95-98`: replace the `unsupported, type: Binary64` insertion with the
     same entity insertion named `chronology.reading.Gauge` before `\nevents:\n` (assert changed).
     Checked: `/3`, `missing-representation`, on Rust and Go.
   - `crates/edge/ess-cli/tests/binary64_publication.rs:56-59`: the model becomes
     `NOTES.replace("{input}", REACHED).replace("{name: demo.notes.NoteId, kind: newtype, of: Uuid}", "{name: demo.notes.NoteId, kind: newtype, of: Binary64}")`;
     the assertion at line 108 checks `demo.notes.NoteId` instead of `demo.notes.Rate`; rewrite the
     comment at lines 61-64 to say the identity `NoteId` is a Binary64 no route carries, so every
     publication route still refuses it. Checked with 0.97.1: all five operations refuse and name
     `demo.notes.NoteId`; formats unchanged. The other tests in that file stay as they are.

## Commands

Run in the worktree root, in order:

1. `cargo test -p ess-synth --locked --lib failure::tests` — 2 passed.
2. `cargo test -p ess-synth --locked --test binary64_plan` — 3 passed.
3. `cargo test -p ess-synth --locked --test feasibility` — all pass.
4. `cargo test -p ess-synth --locked --test go_owner` — all pass.
5. `cargo test -p ess-synth --locked --test bounded_accessor` — all pass.
6. `cargo test -p ess-synth --locked --test clock_reading` — all pass.
7. `cargo test -p ess-synth --locked --test generated_view_queries` — all pass (reads
   `determined.rs`/`view_query.rs`).
8. Byte pins this change could move (`determined.rs` feeds every generated guard):
   `cargo test -p ess-synth --locked --test adversary_related_guard_behaviour_pass1`,
   `--test http_binding_rust`, `--test served_codec_names`, `--test go` — each one command, all pass,
   no committed file changes (`git status` shows only this unit's files).
9. `cargo test -p ess-cli --locked --test binary64_publication` and
   `cargo test -p ess-cli --locked --test target_failure` — all pass.
10. `cargo clippy -p ess-synth --all-targets --locked -- -D warnings` and
    `cargo clippy -p ess-cli --all-targets --locked -- -D warnings` — clean.
11. `slot task fmt-check` — clean.

## Files this unit owns

- `crates/generate/ess-synth/src/failure.rs`
- `crates/generate/ess-synth/src/determined.rs`
- `crates/generate/ess-synth/src/view_query.rs`
- `crates/generate/ess-synth/src/rust/mod.rs` (one `mod` line only)
- `crates/generate/ess-synth/src/go/mod.rs` (one `mod` line only)
- `crates/generate/ess-synth/src/web/mod.rs` (one `mod` line only)
- `crates/generate/ess-synth/src/rust/binary64.rs` (new)
- `crates/generate/ess-synth/src/go/binary64.rs` (new)
- `crates/generate/ess-synth/src/web/binary64.rs` (new)
- `crates/generate/ess-synth/tests/feasibility.rs`
- `crates/generate/ess-synth/tests/go_owner.rs`
- `crates/generate/ess-synth/tests/bounded_accessor.rs`
- `crates/generate/ess-synth/tests/clock_reading.rs`
- `crates/edge/ess-cli/tests/binary64_publication.rs`
- `crates/generate/ess-synth/tests/binary64_plan.rs` (new)
- `crates/generate/ess-synth/tests/binary64_served/support.rs` (new)
- `crates/generate/ess-synth/tests/fixtures/binary64-served/model.yaml.in` (new)

Must not touch: every file under `crates/specify/ess-domain/src`, `crates/verify/ess-conformance/src/**`,
`crates/edge/ess-xtask/**`, `crates/generate/ess-synth/tests/external_request_binding.rs`,
`.config/nextest.toml`, `Taskfile.yml`, `.github/workflows/*`, `rust-toolchain.toml`, any other
`crates/edge/ess-cli` file, and every other file under `ess-synth/src/rust`, `src/go`, `src/web`
(units B2, B3, B4 own them).

## For the coordinator at integration

- No `Taskfile.yml`, workflow, `ci_lanes.rs`, `README.md` or `website/` change for this unit.
- `CHANGELOG.md`: no bullet of its own; unit B4 carries the release bullet for the whole feature.
- `docs/`: none for this unit (the docs edits are listed in units B2 and B4).

## Out of scope

- Flipping any `CARRIED` to `true`, and any codec arm in `rust/`, `go/`, `web/` (units B2-B4).
- Changing the 16 `unreachable!` Binary64 arms or their messages.
- A `Binary64` default in `coalesce` (stays owed), a `Binary64` view parameter (stays refused).
- Membership (`any_of`/`none_of`) over a `Binary64` (owed by design here).
- Any change to `ess-conformance`, `ess-domain`, or the format of any document. No format version changes.

## Acceptance

- `failure::tests` passes: a carrying target refuses only `entity demo.core.Gauge.identity` with the
  uncarried detail; a target without a codec refuses every location (command 1).
- `binary64_plan` passes: `CommandBehavior` and `ViewQuery` of the fixture are generated, membership
  is owed, Clap refuses the ten locations (command 2).
- The four restated `ess-synth` tests and `binary64_publication` pass (commands 3-6, 9).
- The byte-pin targets pass and no committed file changes (command 8).
- Clippy and fmt are clean (commands 10, 11).

## Depends on

nothing.

Provides to B2, B3, B4: `pub(crate) const CARRIED: bool` in `src/rust/binary64.rs`,
`src/go/binary64.rs`, `src/web/binary64.rs` (each module `pub(crate) mod binary64;` in its
target's `mod.rs`); the fixture `tests/fixtures/binary64-served/model.yaml.in` (content in step 8);
`tests/binary64_served/support.rs` with `pub fn model() -> EssIr`, `pub fn compile(&str) -> EssIr`,
`pub fn routes(&EssIr) -> Vec<String>`, `pub const MODEL/ADD/DOCS/CLERK/WRITTEN/REFUSED`,
`pub struct Served` (`start(&Path, &EssIr)`, `raw`, `ask`, `reset`, `add(&str)`, `docs_line()`),
`pub fn body(&str, &str, &str) -> String`, `pub fn tokens(&str, &str) -> Vec<String>`,
`pub fn writes_every_token(&str, &Path)`, `pub fn refuses_what_no_binary64_holds(&str, &Path)`,
`pub fn compares_by_value(&str, &Path)`, `pub fn passes_the_synthesized_suite(&str, &Path)`.
