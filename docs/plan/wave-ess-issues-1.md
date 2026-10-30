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

A wave is at most four units (the orchestrator's process rule), so the S and M issues with no
`ess/N` change run as three batches of at most four units each (revised 2026-10-28):

| batch | units | issues | why together |
|---|---|---|---|
| 1 | U1, U2, U3, U4 | #685, #686, #687, #688, #643, #644, #664, #665, #671, #666 (option A fixture), #651, #653 | no two share a source file; U1 and U4 share regenerated projections (U1 merges first, the integration branch regenerates on conflict), U1 and U2 share a guide page, and the integration branch writes every `CHANGELOG.md` entry; decided by D133, D137, D138 |
| 2 | U5, U6, U7, U8, U9, U10 | #652, #673, #647, #648, #661, #659, #703, #702 | U5 needs U4; U6 and U7 need U1 (`realize.rs`); D135 may take #648 or #661 out; U9 and U10 are two downstream defects D186 added over the four-unit cap |
| 3 | U13, U14, U15, U11 | #645, #693, and four downstream defects without an issue | D214: consumer-blocked defects first; U12 (#694) and the two #554 synthesis stories move to a later batch |
| 4 | exchange wave 1: the exchange in the specification, its projections, its conformance, the WebSocket binding | #650 | `epic:messages-are-exchanged-independent-of-the-wire`; moves `ess/N` |
| 5 | exchange wave 2: HTTP upgrade key, broker binding, JSON-RPC binding; the sourceless view (first cut) | #650, #674 | the same `ess/N` as batch 4 when both land in one release |
| 6 | the language batch | #636, #640, #642, #498, #678, #637 (tag half), #638 | one source format opened once |
| 7 | suite formats and HTTP authentication | #646, #680, #681 | new suite pair; ADR accepted |
| later | #663 (D132, `ess specify vendor`) | | sized for later |
| 8+ | the epic remainders | #554, #560, #561, #564 | decomposed per story |

A unit of batches 1-3 that turns out to need a new `ess/N` leaves for a format batch.

Revised 2026-10-29 by D220 (`approval-record:d220-batch-4-after-0780`). Batch 4, which starts after
`0.78.0`, holds four units: U16 (`service_forward` over commands and views, plus the Timestamp,
Decimal and Uuid CLI primitives), U17 (components sharing a CLI binary), U18 (required form inputs)
and U19 (a record action's row scope). D225 (`approval-record:d225-batch-5-and-6-after-0790`) sets
the batches after it. Batch 5 (`0.80.0`) has four units: React tokens and themes; badge tone with
account menu `sets` and `navigate`; the row confirm key; the Uuid path segment. Batch 6 holds the
minor findings. The stories carry the tags `ess-batch-5` and `ess-batch-6`.

Two synthesis findings filed on 2026-10-29 were placed by D229 (`approval-record:d229-u20-in-batch-4-and-705-in-batch-7`):
- #706 is a defect. It joins batch 4 as U20, its fifth unit, over the cap as D186 allowed for batch 2.
  Batch 4 is the earliest batch not yet gated.
- #707 (D232, `approval-record:d232-707-is-u21-in-batch-4`) is a defect: the generated Rust server admits
  bodies its served OpenAPI refuses. It joins batch 4 as U21, its sixth unit.
- #708 to #711 (D234, `approval-record:d234-mutate-limits-in-batch-7`) are four mutation-audit gaps: the
  order-flip tie, error-swap payloads, actor grants and sites left unmutated. They join batch 7.
- #712 (D236, a defect found while reproducing #710: grant/denied withheld beside a ranked view) joins
  batch 5 (`0.80.0`), moved there from batch 7 by D237 (`approval-record:d237-712-moves-to-batch-5`).
- #713 (D242, `approval-record:d242-713-in-batch-5`) is a gap an adopter is blocked on: `ess generate ui` takes no
  ess-http binding. It joins batch 5 (`0.80.0`) as a sixth unit.
- #705 is a gap. It joins batch 7 beside D226's obligation-contract story. The exchange, language and suite-format batches listed above as 4 to 7 follow after
it, in the same order.

## Units (batches 1-3)

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
| U9 precedence-swap | #703 | precedence-swap-of-disjoint-guards-is-equivalent | ess-conformance (`mutate.rs`), ess-diff | none |
| U10 unreachable-branch | #702 | validate-refuses-a-branch-implied-by-an-earlier-input-refusal | ess-domain (`command.rs`) | U9 (tests in ess-conformance) |
| U11 synthesis-order | #645, #693 | new | ess-conformance synthesis | orchestrator 0.75.0 (`synthesize.rs`) |
| U12 cli-flags | #694 | new | ess-cli-contract, ess-cli-project | none |
| U13 parent-row-guard | none (downstream) | a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children | ess-conformance synthesis | U11, U14 (synthesis arrangement) |
| U14 cascade-witness | none (downstream) | a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed | ess-conformance synthesis | U11, U13 |
| U15 scenario-reading | none (downstream) | a-scenario-with-a-repeated-key-is-refused; an-integer-literal-is-written-into-the-suite-as-an-integer | authored scenario reader, ess-synth | none |

## Builds

Every unit tree of a batch builds into one shared `CARGO_TARGET_DIR` for that wave, `~/.cache/ess-issues-1-target` for batch 1 (operator rule, 2026-10-28). Builds run one at a time under one lock, with `CARGO_INCREMENTAL=0` and no debug info (`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, disk steward request, 2026-10-28). Tests still write scratch output under each tree's own `target/`.

The trees share one layout, so cargo names a workspace crate's artifacts the same in every tree and judges freshness by mtime: a sibling tree's newer build of a crate counts as current in another tree (observed 2026-10-28, units U1 and U4). The wave wrapper therefore touches every tracked file of a tree when the previous build came from a different tree, so the workspace crates are rebuilt from that tree's sources; registry dependencies stay shared. Results produced in the shared target before that guard are re-run.

`cargo test` names each `--test` target as its own argument. Builds start only at 40 GB free or more; a build stops when the shared target passes 30 GB or `/` drops under 25 GB free. When a unit merges, the coordinator runs `cargo clean -p` for the crates it changed and `worktree finish --discard-cache` on its tree.

From batch 3 on (operator rule, 2026-10-29), every tree builds into its own `target/` inside the checkout and no target is shared between trees. One ESS tree builds at a time under one lock; a build does not start under 25 GB free and stops when its tree's target passes 20 GB or `/` drops under 25 GB free. A unit tree is finished with `worktree finish --discard-cache --archive` as soon as it merges.

## Commits this wave makes

One commit per unit on its branch (made by the coordinator through `b10x-gates bot`), the merges
into `integration/ess-issues-1`, the store commits, the pull request and its merge through the App.

## Unit state

Stopped on 2026-10-28 before dispatch, then taken over by the orchestrator's batch lead the same
day. Every unit has a fit review and a scope; batch 1's stories carry them in the store. No unit
tree exists and no implementor has run (nothing builds before the `0.75.0` tag).

| unit | stage | scoping result |
|---|---|---|
| U1 schema-invariants | released in 0.76.0 (batch 1) | #688 accept as proposed (`x-ess-invariants`, integer bounds); #687, #685, #686 accept, redesigned (D138); list counts deferred to their own story |
| U2 subject-guards | released in 0.76.0 (batch 1) | #643 decline with the idiom (drop `when_subject:`, keep `unknown_instance: true`; hint in ESS-COMMAND-003, no duplicate 005); #644 accept, redesigned, no format: a `defined`/`not defined` pair over one Optional path is proved exhaustive, held-state partitions excluded |
| U3 composition | released in 0.76.0 (batch 1) | #665 accept, redesigned (hint per kind, no format); #664 and #671 accept, redesigned, both at `ess-composition/5`, one unit, #664 first; #671 gets its own story |
| U4 http-presentation | released in 0.76.0 (batch 1) | #653 accept, additive `operation_id` and `servers` (ESS-HTTP-032/033); #651 accept as one `info.description` sentence, routes stay (D137) |
| U5 http-answers | released in 0.77.0 (batch 2) | split into U5a (#652 `alike` groups, N21 `encoding: text`) then U5b (N26 route `refusals`, N27 literal body on `empty`, N29 `body: array`); all `ess-http/1` additions; after U4 |
| U6 openapi-import | released in 0.77.0 (batch 2) | #647 accept, redesigned, stays `ess-openapi-import/1`: empty schemas import as `json`, objects gain `additional_properties`, a nullable `anyOf`/`oneOf` imports as `Optional<X>`, as the operator approved on 2026-10-23 (D135; the scoper's null-gap variant is dropped); #648 accept, redesigned: bundle `int32`/`int64` realize at native width only under the opt-in `types-bundle --integer-format-widths`, so the default output keeps its bytes (D135) |
| U7 rust-layout | released in 0.77.0 (batch 2) | #661 accept, redesigned, no format: declaration order via a side table from `ModelTypes` under the opt-in `--declared-field-order`, Box only on cycle-closing references under the opt-in `--rust-box-only-recursion`, Go keeps pointers; the default output keeps its bytes (D135); no edit in `realize.rs` `kind()` |
| U8 synth-field-names | released in 0.77.0 (batch 2) | #659 accept, redesigned: response fields get their own allocation scope in Rust and web; field `naming.code` becomes the draft follow-up `story:a-field-naming-code-chooses-its-identifier-in-rust-and-go` |
| U9 precedence-swap | released in 0.77.0 (batch 2) | #703 accept (D186): a precedence swap of guards no input selects together is `equivalent` (`unsatisfiable_guard`), naming the contradicting part; an enum compared only by equality or membership is decided over its named variants plus one stand-in; downstream shapes S1-S4 decided, S5-S7 (equivalence under declared invariants) a possible follow-up |
| U10 unreachable-branch | released in 0.77.0 (batch 2) | #702 accept (D186): validate refuses a `when:` branch implied by an earlier input-guarded refusal as `unreachable_branch`; the D177 replay found no adopter refused, so it ships as a refusal |
| U11 synthesis-order | batch 3 (D214) | #645 accept as proposed, edits `synthesize.rs` `arranged` (collides with 0.75.0); #693 accept as proposed, stays in `synthesize/related_guard.rs` |
| U12 cli-flags | not started | #694 gaps 1-2 accept, redesigned: `flag` over a required Boolean, `repeated` over a required `List<T>`, in ess-cli/1 and /2 with no format bump; gap 3 decline with the idiom plus a clearer refusal |
| U13 parent-row-guard | batch 3 (D214), prep | downstream: a tenant guard on the input-addressed parent row is unmapped or starves the reader's `found`; synthesis arranges child rows under tenants no parent row was created under |
| U14 cascade-witness | batch 3 (D214), prep | downstream: a cascade onto rows whose creator needs the subject active is refused ESS-SYNTH-001 |
| U15 scenario-reading | batch 3 (D214), prep | downstream: a repeated key in an authored scenario is accepted, last value wins; an authored Integer literal reaches the suite as `1.0` |

## Decisions for the operator

Outcomes, 2026-10-28 (records in `.engineering/planning/`):

- #650: the operator first chose A, then the same day asked for a transport-neutral message
  exchange with A, B and C all possible. `architecture-decision-record:messages-are-exchanged-independent-of-the-wire`
  was proposed and accepted the same day with forks F1 (a), F2 (a), F3 (a)
  (`approval-record:exchange-adr-accepted-as-recommended`).
- #663: B (D132, `approval-record:d132-types-across-specifications-by-vendor`).
- #674: A, staged (`approval-record:a-view-reads-a-setting-first-cut`; Amendment 1 of
  `a-command-guard-reads-a-component-setting`).
- #666: A now (D133); the reporter is asked which side sends the subset.

The option tables below are kept as they were put to the operator.

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
