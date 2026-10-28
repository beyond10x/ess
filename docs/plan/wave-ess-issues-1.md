# Wave: open issues, batch 1

Integration branch `integration/ess-issues-1`, cut from `origin/main` `085f893e8` (0.74.0).
One pull request to `main`, merged after the `0.75.0` tag exists. Skill: `aep:implementing` 0.24.2,
wave mode, non-interactive (the operator approved waves for this work in advance; the record is
`approval-record:ess-issues-batch-1`).

## Triage of the 41 issues open on 2026-10-28

Reproduced with the 0.74.0 release binary at `ess/32`.

- Fixed by merged changes and closed with evidence: #623 (0.68.0, `ess/30`), #635 (0.69.0,
  13ebe723e), #655 (0.69.0, c1fdc2af8).
- Duplicates: none.
- Needs a decision first: #650 (WebSocket half), #663, #674 (no ADR); #666 (which side sends the
  subset, asked of the reporter).
- Remaining size: S 10, M 14, L 14.

| # | verdict | size | crates | format change | artifact(s) | shares with |
|---|---|---|---|---|---|---|
| 498 | still valid | L | ess-domain, ess-conformance | ess/33, suite pair | story:stored-field-equals-returned-response-value (draft) | 678 |
| 554 | partly fixed (umbrella G4): #642, the Initialize hint and two synthesis defects remain | L | ess-domain, ess-conformance | ess/33 for #642 | epic:generic-guard-precedence | 642, 643, 644, 645 |
| 560 | partly fixed (umbrella G1) | L | ess-primitives, ess-domain, ess-conformance, ess-synth | per slice | epic:generic-expression-language | 561, 564, 674 |
| 561 | still valid (umbrella G3) | L | ess-domain, ess-conformance, ess-synth, ess-diff | ess/N, suite pair | epic:generic-aggregates | 560, 564 |
| 564 | partly fixed (umbrella G2) | L | ess-domain, ess-conformance, ess-synth | per slice | epic:generic-row-queries | 560, 561 |
| 623 | fixed 0.68.0, closed | | | | story:a-race-above-a-held-state-branch-is-witnessed | |
| 635 | fixed 0.69.0, closed | | | | story:a-first-match-chain-is-synthesized-within-a-work-bound | |
| 636 | still valid | M | ess-domain, ess-conformance | ess/33 | story:caller-attributes-are-readable-in-a-when-related-predicate | 640 |
| 637 | still valid; content-key half waits on a design page | L | ess-domain, ess-gen, ess-synth, ess-conformance | ess/33 | story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key | 644 |
| 638 | still valid; accepted ADR chooses `unique:` first | L | ess-domain, ess-compiler, ess-conformance, ess-synth | ess/33 | story:an-identity-may-be-a-composite-of-fields | 636, 640 |
| 640 | still valid | M | ess-domain, ess-compiler, ess-conformance | ess/33 | story:via-and-instance-may-name-a-member-of-a-struct-input | 636 |
| 642 | still valid (ESS-COMMAND-009 and synthesis) | M | ess-domain, ess-conformance | ess/33 | none yet | 554, 643 |
| 643 | still valid | S | ess-domain | none | story:when-subject-reads-the-addressed-identity | 644 |
| 644 | still valid | S | ess-domain | none or ess/33 | story:defined-and-not-defined-branches-are-exhaustive | 643 |
| 645 | still valid | S | ess-conformance | none | none yet | 693 |
| 646 | partly fixed (unreached Binary64, 0.70.0) | L | ess-conformance, Go/TS runtimes | suite pair | story:a-binary64-value-round-trips-through-a-conformance-suite | 680 |
| 647 | still valid; decided (approval record, option A) | M | ess-openapi, ess-cli | import envelope | story:openapi-import-admits-open-objects-as-json | 648 |
| 648 | still valid | M | schema-contract, ess-openapi | none | story:an-imported-integer-format-realizes-at-its-native-width | 647, 661 |
| 650 | WebSocket half needs a decision; JSON-RPC half decided, design page draft | L | ess-http, ess-gen, ess-synth | new binding format | story:http-binding-carries-transport-framing | 652, 653, 673, 681 |
| 651 | still valid (command routes) | M | ess-gen, ess-http | none (bytes) | story:a-component-not-reached-over-the-network-has-no-http-route | 652, 653, 673 |
| 652 | still valid | M | ess-http, ess-gen, ess-synth, ess-conformance | additive ess-http/1 | story:two-outcomes-answered-alike-may-share-a-status-and-body | 653, 673 |
| 653 | still valid | M | ess-http, ess-gen | additive ess-http/1 | story:an-http-binding-names-operation-ids-and-servers | 652, 673, 681 |
| 655 | fixed 0.69.0, closed | | | | story:a-denied-grant-over-a-command-that-writes-nothing-is-not-an-error | |
| 659 | still valid | S | ess-synth | none | story:a-command-input-and-response-field-may-share-a-name-in-rust | 661 |
| 661 | still valid | M | schema-contract | none (bytes) | story:generated-fields-keep-declaration-order-and-box-only-recursion | 648, 686 |
| 663 | needs an ADR | L | ess-domain, ess-compiler, ess-cli | ess/N | story:a-specification-imports-another-specifications-types | 664 |
| 664 | still valid | M | ess-composition | ess-composition/5 | story:a-composition-conformance-may-name-an-event-or-command | 665, 671 |
| 665 | still valid | S | ess-composition | none | story:reference-outside-component-names-its-remedy | 664 |
| 666 | follows the documented rule; reporter asked which side sends the subset | S/M | ess-composition | /5 for a caller stance | story:role-input-drift-tells-a-caller-from-an-implementation | 664, 671 |
| 671 | still valid (documented limit) | M | ess-composition | ess-composition/5 | under story:a-composition-conformance-may-name-an-event-or-command | 664 |
| 673 | still valid (all four forms) | M | ess-http, ess-gen, ess-synth, ess-conformance | none (relaxation) | story:ess-http-admits-text-bodies-route-refusals-empty-bodies-and-array-views | 652, 653 |
| 674 | needs an ADR | L | ess-domain, ess-conformance, ess-synth, ess-http | ess/N, suite | story:a-view-may-answer-values-no-row-holds | 560 |
| 678 | partly fixed (wire half 0.69.0); keep-versus-clear remains, ADR accepted | L | ess-domain, ess-compiler, ess-conformance, ess-synth, ess-diff | ess/33 | story:an-input-keeps-missing-and-null-apart | 498 |
| 680 | partly fixed (hint 0.69.0); row-shape check remains | L | ess-conformance, Go/TS runtimes | suite pair | story:a-view-read-checks-every-rows-shape | 646 |
| 681 | still valid, unblocked (ADR accepted, option B) | L | ess-http, ess-gen, ess-synth, ess-conformance | additive ess-http/1 | ADR a-binding-authenticates-the-caller-through-a-port | 653 |
| 685 | still valid | S | ess-gen | none (bytes) | none yet | 686, 687, 688 |
| 686 | still valid | M | ess-gen, schema-contract | none (bytes) | none yet | 685, 687, 688, 661 |
| 687 | still valid | S | ess-gen | none (bytes) | none yet | 685, 686, 688 |
| 688 | still valid | S | ess-gen | none (bytes) | none yet | 685, 686, 687 |
| 693 | still valid | S | ess-conformance | none | none yet | 645 |
| 694 | still valid; gap 3 is by design (`resolve.rs:368`) | M | ess-cli-contract, ess-cli-project | ess-cli/3 | none yet | |

## Batches

| batch | issues | why together |
|---|---|---|
| 1 | S and M with no source-format change: #643, #644, #645, #647, #648, #651, #652, #653, #659, #661, #664, #665, #671, #673, #685, #686, #687, #688, #693, #694 | no `ess/N` bump |
| 2 | the `ess/33` language batch: #636, #640, #642, #498, #678, #637 (tag half), #638 | one source format opened once |
| 3 | suite formats and HTTP authentication: #646, #680, #681 | new suite pair; ADR accepted |
| 4+ | the epic remainders: #554, #560, #561, #564 | decomposed per story |

A unit of batch 1 that turns out to need `ess/33` leaves batch 1 for batch 2.

## Units of batch 1

| unit | issues | stories | crates | shares a file with |
|---|---|---|---|---|
| U1 schema-invariants | #688, #687, #685, #686 | new | ess-gen (`types.rs`), schema-contract (`realize.rs`, #686) | U7, U8 (`realize.rs`) |
| U2 subject-guards | #643, #644 | when-subject-reads-the-addressed-identity; defined-and-not-defined-branches-are-exhaustive | ess-domain (`command/subject_fact.rs`, `command/finite.rs`) | none |
| U3 composition | #665, #664, #671 | reference-outside-component-names-its-remedy; a-composition-conformance-may-name-an-event-or-command | ess-composition | none |
| U4 http-presentation | #653, #651 | an-http-binding-names-operation-ids-and-servers; a-component-not-reached-over-the-network-has-no-http-route | ess-http, ess-gen (`openapi.rs`, `http.rs`) | U5 |
| U5 http-answers | #652, #673 | two-outcomes-answered-alike-may-share-a-status-and-body; ess-http-admits-text-bodies-route-refusals-empty-bodies-and-array-views | ess-http, ess-gen, ess-synth, ess-conformance | U4 |
| U6 openapi-import | #647, #648 | openapi-import-admits-open-objects-as-json; an-imported-integer-format-realizes-at-its-native-width | ess-openapi, schema-contract | U1, U7 |
| U7 rust-layout | #661 | generated-fields-keep-declaration-order-and-box-only-recursion | schema-contract realizers | U1, U6 |
| U8 synth-field-names | #659 | a-command-input-and-response-field-may-share-a-name-in-rust | ess-synth | none |
| U9 synthesis-order | #645, #693 | new | ess-conformance synthesis | orchestrator 0.75.0 (`synthesize.rs`) |
| U10 cli-flags | #694 | new | ess-cli-contract, ess-cli-project | none |

## Builds

One shared `CARGO_TARGET_DIR` (`~/.cache/ess-batches-target`), `CARGO_PROFILE_DEV_DEBUG=line-tables-only`,
`CARGO_INCREMENTAL=0`, every cargo call serialised by one lock (`ess-cargo` wrapper) so a test never
runs a binary another tree just rebuilt. No build under 40 GB free while another ESS tree builds;
none under 25 GB.

## Commits this wave makes

One commit per unit on its branch (made by the coordinator through `b10x-gates bot`), the merges
into `integration/ess-issues-1`, the store commits, the pull request and its merge through the App.

## Unit state

Stopped on 2026-10-28 before dispatch: the steward handed ESS steering to one session. No unit
tree exists, no implementor ran, the planning store is unchanged. Fit reviews and scopes were
derived read-only for U2, U3, U5, U6, U7, U8, U9 and U10 (summaries below); U1 and U4 were not
returned.

| unit | stage | scoping result |
|---|---|---|
| U1 schema-invariants | not started | not returned |
| U2 subject-guards | not started | #643 decline with the idiom (drop `when_subject:`, keep `unknown_instance: true`; hint in ESS-COMMAND-003, no duplicate 005); #644 accept, redesigned, no format: a `defined`/`not defined` pair over one Optional path is proved exhaustive, held-state partitions excluded |
| U3 composition | not started | #665 accept, redesigned (hint per kind, no format); #664 and #671 accept, redesigned, both at `ess-composition/5`, one unit, #664 first; #671 gets its own story |
| U4 http-presentation | not started | not returned |
| U5 http-answers | not started | split into U5a (#652 `alike` groups, N21 `encoding: text`) then U5b (N26 route `refusals`, N27 literal body on `empty`, N29 `body: array`); all `ess-http/1` additions; after U4 |
| U6 openapi-import | not started | #647 accept, redesigned, stays `ess-openapi-import/1`: empty schemas import as `json`, objects gain `additional_properties`, a nullable `anyOf`/`oneOf` imports as its schema with the null gap (not `Optional<X>` as the 2026-10-23 approval says: confirm); #648 accept, redesigned: bundle `int32`/`int64` realize at native width (breaking for generated bundle libraries) |
| U7 rust-layout | not started | #661 accept, redesigned, no format: declaration order via a side table from `ModelTypes`, Box only on cycle-closing references, Go keeps pointers; Rust key order on the wire changes; textual conflict with U6 in `realize.rs` `kind()` |
| U8 synth-field-names | not started | #659 accept, redesigned: response fields get their own allocation scope in Rust and web; field `naming.code` becomes a follow-up story |
| U9 synthesis-order | not started | #645 accept as proposed, edits `synthesize.rs` `arranged` (collides with 0.75.0); #693 accept as proposed, stays in `synthesize/related_guard.rs` |
| U10 cli-flags | not started | #694 gaps 1-2 accept, redesigned: `flag` over a required Boolean, `repeated` over a required `List<T>`, in ess-cli/1 and /2 with no format bump; gap 3 decline with the idiom plus a clearer refusal |

## Decisions for Timo

### #650 WebSocket upgrade binding

The JSON-RPC half is decided (ADR `transport-and-idl-compile-targets` § Decision 2,
`docs/design/rpc-bindings.md:19-48`). Only the WebSocket half is open; `101` stays refused as an
outcome (`http-presentation-binding.md:337`).

| option | what it does | cost |
|---|---|---|
| A | Sibling `ess-websocket/1`, shaped like `ess-jsonrpc/1`: pins the specification and lists sockets (path, subprotocol, opening command, commands the client sends, events the server sends); compiles to `ess-websocket-ir/1`, projected into AsyncAPI 3's `ws` binding | ADR amendment 2; new format and crate; extends `ess-gen/src/asyncapi.rs`; size M; breaks nobody |
| B | `ess-transport/3` with `brokers[].protocol: websocket` | a format change; broker and stream vocabulary does not fit a point-to-point socket; leans to the ADR's rejected generic binding; size M |
| C | `ess-http/2` route key `upgrade: {websocket: …}` | moves `ess-http/2` and `ess-http-ir/2`; messages need AsyncAPI from the HTTP crate; against the ADR's sibling pattern; size M-L |

Recommendation: A, because it follows the accepted ADR's sibling-document pattern and needs no
`ess/N` change.

### #663 Types across specifications

On 0.74.0 a reference to another specification's type is ESS-TYPE-001, and a copy under the owner's
names is ESS-SPEC-004.

| option | what it does | cost |
|---|---|---|
| A | Source import in `system.yaml` (`imports:` with system, version, `source_digest`, vendored IR, types); imported names keep their owner | new `ess/N` and IR shape, every IR reader changes; exceptions to SPEC-004 and SPEC-006; size L |
| B | `ess specify vendor --from <owner ir> --type <T> --into <domain>` generates the renamed copy, its nested types and the `conformances:` entry, with the owner's digest in a header | one CLI subcommand, no format; generated code still has two types; size S-M |
| C | Decline with the idiom: hand copy plus `compose conformances:`, written up on the design page | docs only, size S; copies stay hand-written |

Recommendation: B, because it removes the hand copying without changing ownership or any format;
A stays possible later.

### #674 A view answering values no row holds

On 0.74.0 a constant `expr:` on a view with a source validates; a view without a source does not
(`ViewSpec.source` is required). Components already declare `settings:` with an optional fixed
`value:`; the ADR for guards reading a setting is accepted but not built.

| option | what it does | cost |
|---|---|---|
| A | Extend that ADR to views: a field may read `setting.<name>`, and `source:` becomes optional when every field is a literal or a setting; first cut reads only settings with a fixed `value:` | next `ess/N`; optional source in the view IR across synthesis, interpreter, generators and `ess-http`; ADR amendment; size L (first cut about B) |
| B | Constants only: `source:` optional when every field is a literal | next `ess/N`, same IR change without settings; size M |
| C | Decline with the idiom: constant `expr:` on a sourced view; documented in `read-api-view-idioms.md` | docs only, size S; no limits endpoint, the constant repeats per row |

Recommendation: A, staged, because a limit already has a typed home in `settings:`.

### #666 role_input_drift on a producer sending a subset

On 0.74.0 a local command standing `implements` toward a contract command with more inputs reports
`role_input_drift` even on an `Optional` contract input; `stands: calls` does not exist.

| option | what it does | cost |
|---|---|---|
| A | Keep the rule; document that a caller is not a role entry; pin the behaviour with a fixture | docs and one fixture, size S |
| B | Add `stands: calls`: every local input is a contract input, every required contract input is a local input | `ess-composition/5` (or `/6` after #664 and #671); size M; breaks nobody |
| C | Relax `implements` for `Optional` contract inputs behind `reader: true` | `ess-composition/5`; size S-M; sound only for decoders that ignore unknown keys |

Recommendation: A now, then B or C on the reporter's answer to which side sends the subset. The
question is not yet posted in the issue thread.
