---
format: aep.planning-md/1
id: story:binding-mapping-bounded-accessor
kind: story
status: proposed
title: A binding mapping may read one bounded path into the event, so an envelope-shaped event can drive a flat-input command
summary: 'mapping: <input>: event.data.<field> — segments resolved against declared types, Optional/union segments refused unless the target is Optional, bounded depth, no list traversal. Measured need: 17 commands and 0 bindings in babelconnect because its push events are wire envelopes.'
relations:
- serves: vision:O2
- depends_on: story:docs-literal-mapping-claims-unchecked
scope:
- confidence: cited
  path: babelconnect-specs
- confidence: cited
  path: crates/edge/ess-cli/src/coverage.rs
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/release_evidence.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/support.rs
- confidence: cited
  path: crates/generate/ess-gen/src/asyncapi.rs
- confidence: cited
  path: crates/generate/ess-gen/src/docs.rs
- confidence: cited
  path: crates/generate/ess-gen/src/graph.rs
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/system.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/feasibility.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/system.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
- confidence: cited
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/error.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/counts.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/evidence.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/report.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/web_replay.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: docs/design/binding-mapping-bounded-accessor.md
revision: 6
---
## Why

A binding's `mapping:` reads one flat field of the triggering event; a path is refused as
`unsupported_construct` — the rule at `crates/specify/ess-domain/src/binding.rs:32`, the reasoning under the
module-doc heading *One field, not a path* at `:120`, the refusal emitted at `:718-723`
("`event.<field>` reads a path, and this build maps one field of an event onto one input of a command").
The refusal's own hint calls the construct "unsupported here rather than wrong", and the module doc offers
two repairs: map the whole value, or add a field to the event that carries it.

In a system whose events are wire messages, neither repair is available. babelconnect's backend push events
are `{event: String, data: <payload struct>}` — the Bayeux envelope, byte for byte what the pusher sends.
Every server-internal command they drive takes flat scalar inputs. So:

- **map the whole value** works only when the command input's type *is* the payload struct. None of the 17 is.
- **add a field to the event** makes the declaration stop describing the wire.

The second repair was tried and reverted (`downstream-adopter/babelconnect-app`, wave 3, 2026-09-11: 77 fields across
11 events), and the two reasons it failed are the argument for this story.

**It was invisible to the gate.** That repository compares every push payload field against the
specification that owns it (`check_push_types.py`). A field `invented_not_on_the_wire: String` added to a push
event passed it with `11 event(s), 307 field(s) checked`, exit 0 — the check walks the `data` subtree, so it
is silent about every field beside it. 77 projection fields sat under a line that reads as agreement.

**It bought a false statement, which is the stronger half.** With the fields in place a row *was* written:
`CallUpdated → BackendCallEvent`, nine inputs, all read off the envelope leg. The reducer does not work that
way. `/agent/{id}/calls` pushes a call with both legs (`child`/`parent` in the `'current'` map), and
`pickLegs` (`babelconnect-server/internal/grpcsvc/backendstate.go:1074-1090`) walks the `legs` slice and
selects **two different elements by predicate** — `Domain == "internal" || Source == "webrtc"` for the agent
leg, `Domain == "external"` for the other. `mergeCall` then keys the call on `agent.ID` (`:1035`) while
`From`, `To` and `Anonymous` come off `external` (`:1046-1055`), and `recording` is
`len(agent.Recordings) > 0 || len(external.Recordings) > 0` (`:1033`) — a derivation over both. Every inbound
queue call takes that path. The projection did not produce a specification that was merely unchecked; it
produced one that was **wrong**, and no gate could tell, because a flat field is exactly as checkable as a
correct one.

The measured cost of having neither repair: **17 server-internal commands, 0 bindings.** The causation the
system does have — "this push invokes that command, at most once, dropping on a parse failure" — cannot be
stated at all, so `ess verify conform synthesize` emits no flow scenario for any of them, and
`interactions.md` reads "This system declares no bindings".

## What would settle it

A **bounded accessor** in `mapping:`, not a general path expression:

```yaml
mapping:
  status: event.data.status              # one dotted path into the event's declared structure
  last_reason: event.data.last_reason
```

Bounded by the same argument the current refusal makes — a projection must not silently turn a value that is
present into one that may be absent:

- each segment resolves against a **declared** type of this model, so a typo is `UnobservableFact` exactly as
  a flat field is today;
- traversal through `Optional<T>` or a union is **refused** unless the target input is `Optional` too; a terminal Optional is copied whole and may use an existing exact host conversion,
  which is the whole content of "nothing in the model says a projection is total";
- depth is bounded (three segments after `event` covers the measured `data.body.text` path) so the accessor stays a projection and does
  not become an expression language;
- `List<T>` is not traversable — selecting an element is a different construct (see *Out of scope*).

That is the checkable half of what a reader means by "JSON path", and it keeps every property the refusal
protects.

## Acceptance

- [ ] `mapping: <input>: event.<field>.<field>` validates when every segment is declared and non-`Optional`,
      or the target is `Optional`.
- [ ] Traversal through `Optional`/a union into a required input is refused, with a distinct code and the two repairs. A terminal Optional retains its declared whole-value conversion contract.
- [ ] A segment naming nothing is `UnobservableFact`, as a flat field is.
- [ ] The projections (docs, OpenAPI, AsyncAPI, graph, synth) print the path; the conformance scenario reads
      the same value the mapping does.
- [ ] `babelconnect-specs` writes **4 of its 5 measured rows** against the released version with no field
      added to any push event, and `check_push_types.py` still reports every push event as exactly
      `{event, data}`. The fifth, `/agent/{id}/calls → BackendCallEvent`, stays blocked: it additionally
      needs the list selection this story excludes — `pickLegs` walks a slice and picks two elements by
      predicate (`backendstate.go:1074-1090`), and the row's inputs are read from both. **This story does not
      promise that row.** The honest four is the number to weigh the construct by.

## Out of scope

Selecting an element of a `List<T>` (babelconnect needs it for a conference member fan-out and for the
per-leg selection in `/agent/{id}/calls` — `pickLegs`, `backendstate.go:1074-1095`), and any guard on a
binding. Both are separate constructs; this story is the accessor only.

## Provenance

Filed 2026-09-11 by the babelconnect-app wave-3 coordinator, from measurements in that repository (adversary
passes 1 and 2 on `story:backend-bindings`, `review-result:adversary-backend-bindings-pass-{1,2}`). Its
counterpart there is `dependency-blocker:ess-binding-path-mapping-absent` (filed by the
specs-authority session). Operator: Timo, who asked that the gap be fixed upstream rather than worked around.

## Wave source provenance

Imported through the AEP CLI from the operator-selected primary-checkout draft, revision 3. Original bytes retained as local-evidence:ess-evolution-20260910/priority-source-binding-mapping-bounded-accessor.md (SHA256 27a06004bd43ee6fc6ea3978308b8b5f2de3c8dfe8857690f0a0d43ef5f161c3). This branch starts a truthful import record; it does not invent the source draft's earlier command history. The primary draft and journal remain untouched.

## Scope

Derived 2026-09-11 by aep-drive:story-scoper at main 6b666e58.

- Primary surface: crates/specify/ess-domain/src/binding.rs (MappingSource, shape validation and check_entry) — cited.
- Compiler: crates/specify/ess-compiler/src/{resolve,ir}.rs (mapped_field, ResolvedMappingValue) — cited.
- Generated descriptions: crates/generate/ess-gen/src/{docs,asyncapi,openapi,graph}.rs — cited.
- Executable synthesis: crates/generate/ess-synth/src/plan.rs, src/rust/{system,feasibility}.rs and src/go/system.rs — cited.
- Conformance: crates/verify/ess-conformance/src/{scenario,synthesize,runner}.rs — cited; Observed currently reads a single flat field.
- Semantic diff: crates/verify/ess-diff/src/diff.rs — cited.
- Design: docs/design/binding-mapping-bounded-accessor.md — inferred; settle optional/union/depth/wire-name/conversion and format compatibility before code.
- Consumer: babelconnect-specs, four bindings with original envelopes — cited by story; exact local checkout and rows still require discovery.
- Confidence: high for ESS paths inspected in the tree — cited.
- Collisions: literal-doc changes in docs.rs and ir.rs require sequential integration — cited.

## Wave implementation contract

The operator selected this story with the literal-doc correction and xattr reconciliation, excluded crosswalk, and authorized sub-agents plus a new release after main integration. Complete the existing literal-doc correction first so the expanded binding renderer and resolved mapping documentation build on truthful literal guarantees. This is an explicit integration dependency over their shared files, not an excuse to omit accessor semantics.

Write and review the design before code. Settle depth after event, Optional absence versus null, union traversal and terminal union values, source and wire names, newtype wrappers, conversion behavior, malformed events, old reader refusal and preservation of old flat-mapping bytes. Keep the four-consumer-binding acceptance; discover the real declarations rather than inventing them. List selection and guards remain excluded. Do not invent ess-ir/2 or weaken unsupported-target refusals. The operator excludes full local workspace/ownership gates; use focused source, generator, conformance, compatibility and actual consumer checks with explicit scope. Required remote merge/release checks are authorized for the requested release and separately tracked.

## Public import correction

The first unpublished import was refused by the coordinated private-identifier check. Its exact rejected patch is retained privately in the local wave evidence. This replacement was created through AEP from the same source snapshot with the private organization identifier generalized before any journal event was written. Source acceptance and source snapshot hashes are preserved; no scanner policy or exception changed.


## Design correction and observed adoption limits

The design at docs/design/binding-mapping-bounded-accessor.md now states the complete source, resolved-plan, native generation, conformance and report contracts. First review review-result:priority-accessor-design-pass1-20260911 found unbounded finite branching, missing report compatibility paths, and ambiguous nested Optional construction. Correction digest d950020e8d3f778f221fb1ff83a13aeb9c933c1de444b2b69e778ab2db7875f0 answers those classes with a shared bounded DAG, explicit existing report/2 routing for suite/6-/7, and equality-first typed Optional lifting. Second design review is pending; no accessor source implementation is claimed.

Three segments after event are required by the measured notification body path. Source ess/3 and distinct resolved EventAccessor/ObservedAccessor variants preserve old flat bytes; ordinary suite/6 and coverage suite/7 carry new semantics. Existing report/2 retains its exact-suite meaning. Report/1 incompatibility must be refused before target execution. Resource limits govern the new capability without weakening existing scanner, gate or source policies.

The four adopter rows remain acceptance obligations, not an accessor-only completion claim. Each requires authoritative session identity absent from the binding source vocabulary. The status payload has an id, but its equivalence to authenticated session identity is unproven; the other three payloads lack recipient identity. Existing exact host conversions can declare whole-Optional or whole-struct crossings, but declarations do not execute reducer algorithms. Actual context, conversion and released-pin adoption evidence remains required. No invented event fields or optionalized required inputs may manufacture the four-row count. Typed context is a separate follow-up contract, not accessor syntax admitted here.
