---
format: aep.planning-md/3
id: epic:downstream-reported-gaps
kind: epic
status: active
title: Downstream-reported synthesis, binding and grant gaps are closed
relations:
- serves: vision:O2
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:18Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-30T13:04:29Z", actor: "human:timo", revision: 3}
---
## Outcome

A downstream specification synthesizes, runs and records conformance without refusals that come from ESS rather than from the model: related-row guards and values, aggregate group keys, bindings that move state, authorization, and the binding language gaps it reported are closed.

## Scope

Issues beyond10x/ess#229 (state inside when_related), #257, #265, #266, #267, #268, #269, #270, #271, #272. Each story names its conformance scenarios; the downstream mini specifications under the reporter's scratch reproduce #270–#272 and are not copied into this repository.

## Order

Scheduled by `aep plan artifact waves` from each story's scope; the synthesis stories that share `synthesize/aggregate.rs` or the relation-guard arrangement run in sequence.

## Downstream report 2026-10-19 (triage)

Eighteen findings from one downstream specification (ess 0.60.0; reported 2026-10-19), triaged on
0.60.0, 0.62.0 and an integration-branch build (0.62.0 plus the 2026-10-13 port). Neutral
reproductions under `.engineering/repro/`. Where each finding went:

| # | finding (short) | answer |
|---|---|---|
| 1 | `affects:` cascade onto rows whose creator has a guarded sibling: ESS-SYNTH-001 | `story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed` |
| 2 | `--synthesis-seed` cannot reach those rows | same story, Decisions (seeds stay subject-only) |
| 3 | authored scenarios do not discharge obligations | `specification:authored-scenarios-are-selected-by-scenarios-and-discharge-no-obligation` |
| 4 | `ess-inputs.yaml` `scenarios:` not read by synthesize/run | same specification (documented; `--scenarios <model-dir>`) |
| 5 | row-set `when_related` beside identity-addressed: ESS-COMMAND-009 | `epic:generic-row-queries` (wave 4 item 4, #473) |
| 6 | two related rows of one entity: every outcome ESS-SYNTH-003 | needs the reporter's failing command (below) |
| 7 | owned row's stored `via` compared with an input | `story:an-owner-link-guard-beside-an-owner-existence-guard-is-witnessed` |
| 8 | `{related:}` copy read later | row-set half fixed by #601 (`tests/row_set_related_copy.rs`); `affects:` half in finding 1's story; stored list / four records: needs repro |
| 9 | related row's `state` as a value | `story:row-queries-navigate-a-reference` |
| 10 | `when_related` one hop; `subject.` forms | hint: `story:a-subject-prefixed-via-hint-names-the-bare-field`; row-set: `story:a-related-selector-compares-a-row-field-with-a-subject-field`; depth: `story:row-queries-navigate-a-reference` |
| 11 | two identity-addressed rows in one branch | `story:row-queries-navigate-a-reference` |
| 12 | refusals answered before a narrower success branch | `story:declaration-order-agrees-with-every-step-pair` |
| 13 | `state` in an `affects:` `where:` | `epic:generic-row-queries` |
| 14 | `<Command>/outcome/wrong-state` id disappears | `specification:the-wrong-state-scenario-id-is-the-unknown-identity-fallback` |
| 15 | every entity needs a lifecycle | `specification:a-stateless-record-declares-a-one-state-lifecycle` |
| 16 | validate admits a write breaking a field invariant | `story:an-authored-text-literal-outside-its-alphabet-is-refused` (entity-invariant literals) |
| 17 | Integer literals written as floats | `story:an-integer-literal-is-an-integer-token-in-a-suite` |
| 18 | `ess verify diff` rates most changes unknown | `story:verify-diff-decides-what-the-model-settles` |

Needs the reporter's reproduction:

- **6.** Two related rows of one entity with existence and direction guards on both
  (`.engineering/repro/two-related-rows/system.yaml`, `ported.yaml`) synthesize with 0 refusals on
  all three binaries. Missing: the exact failing command (its guards and the refusal text); the
  report's own wording ("modality, encoding and pipeline membership … compare two rows") suggests a
  cross-row comparison, which is finding 11.
- **8, second half.** "A guard over a stored list" and "a guard over four related records": the
  report gives neither the command nor the refusal text.

## a2a wave 1 (2026-10-22)

The orchestrator relayed a2a wave 1's 23 ESS gaps from the adopter session specifying A2A in
beyond10x/a2a (ESS 0.67.0, a2a main 3a18050): 7 streaming (G1-G7), 7 JSON-RPC (J1-J7) and 9 model
(M1-M8 with M4b). They are filed as draft stories for 0.69.0 beside adopter items 1-9, duplicates
merged into the story that already holds the need. They do not delay 0.68.0. Each gap is cited as
"a2a wave-1 gaps list, line N".

| gaps | story |
|---|---|
| G1 | `story:an-event-channel-may-be-bound-to-server-sent-events` (ADR-blocked) |
| G2 | `story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant` (item 4) |
| G3 | `story:a-payload-may-leave-an-optional-event-field-absent` |
| G4 | `story:an-outcome-that-keeps-its-subject-may-emit` |
| G5 | `story:a-response-stream-follows-one-row-until-a-terminal-state` (ADR-blocked) |
| G6 | `story:an-authored-claim-may-compare-a-nested-field` |
| G7 | `story:an-asyncapi-send-operation-names-every-emitter` |
| J1, J2, J3, J7 | `story:a-command-binds-to-json-rpc-and-grpc-beside-http` (item 8) |
| J4, J6 | `story:an-authored-scenario-claims-a-binding-error-code` (ADR-blocked) |
| J5 | `story:view-declares-its-answer-for-an-unknown-identity` |
| M1 | `story:an-identity-may-be-a-composite-of-fields` |
| M2, M4, M4b | `story:a-field-or-view-may-be-internal-to-the-model` |
| M3 | `story:a-delete-may-have-a-no-op-branch-that-emits-nothing` |
| M5 | `story:a-view-may-scope-rows-to-the-reading-actor` |
| M6 | `story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain` |
| M7 | `story:response-field-declares-its-value-source` |
| M8 | split: 12 errors `story:an-optional-input-a-system-command-awaits-is-compared-only-when-issued` (item 1), 4 errors `story:a-refused-send-keeps-the-earlier-read-token-for-a-denied-grant` (item 2) |

The relay also carried one question for the A2A specification itself (a second push-notification
config with the same id under the same task); it is not an ESS gap and nothing is filed for it.

## Adopter binding defects (2026-10-22)

The orchestrator relayed three binding-scenario synthesis defects from an adopter, with the instruction: reproduce #1 and #2
against the 0.68.0 build; fix a 0.68.0 regression before the tag, otherwise file all three for 0.69.0 ahead of the a2a gaps.

| defect | story | 0.68.0 regression |
|---|---|---|
| 1 | story:a-binding-trigger-acts-on-the-row-its-setup-arranged | no; 0.67.0 suite byte-identical, entered 0.65.0 (`9d0531a4f`) |
| 2 | story:a-refusal-policy-scenario-is-not-set-up-through-a-trigger-that-needs-the-row | no; same |
| 3 | story:an-unbounded-retry-binding-leaves-no-trigger-scenario-unsupported | no; present since 0.65.0 |

None blocks 0.68.0. The three carry the tag `ess-0.69.0-first`: 0.69.0 takes them before the a2a wave-1 gaps.

## Adopter defects, second batch (2026-10-22)

Items 4 to 10 of the adopter report relayed on 2026-10-22, each reproduced on 0.68.0 and on 0.67.0 with a neutral fixture.

| item | story | 0.68.0 regression |
|---|---|---|
| 4 | `story:an-input-of-list-type-may-carry-an-example` (gap) | no |
| 5 | `story:the-interpreter-mints-a-newtype-under-a-disjunction-of-prefixes` (defect) | no |
| 6 | `story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view` (gap) | no |
| 7 | `story:a-guard-matches-an-input-against-a-stored-wildcard-pattern` (gap; fit review against `docs/design/stored-rules-boundary.md`) | no |
| 8 | `story:a-command-response-may-be-declared-optional` (gap) | no |
| 9 | `story:a-protocol-timer-takes-its-duration-from-a-message-field` (gap) | no |
| 10 | `story:go-synthesis-repairs-a-served-component-named-server` (existing; report appended) | no |

Not filed: the report's excluded items (#597, #550, #598, #531, #594, #473, #505, #554, and ESS-SYNTH-013 for a command with no
view, which is by design) and its not-expanded items (a second team's limit file of 19 items, and two synthesis refusals
recorded on 0.56.0 without a reproduction).

## 0.69.0 batch (Timo, 2026-10-22)

The adopter and a2a defects ship as one batch: every fix lands on one integration branch, `integrate/ess-batch-0690`, cut from
`main` after 0.68.0 merges. Each unit is reviewed (adversary pass), the full gate runs on the integrated branch, and one pull
request against `main` releases 0.69.0.

In the batch: the stories tagged `ess-0.69.0` (the ten tagged `ess-0.69.0-first` go first), adopter items 3, 4 and 6, and the
units already moved to 0.69.0 (row-set on ess/31, aggregate measures #561, aggregate group keys, derived measures, browser fixture).

Not in the batch until their ADR is accepted: the stories blocked by
architecture-decision-record:transport-and-idl-compile-targets (items 7 and 8, server-sent events, row-keyed response streams,
binding error codes). The stored-wildcard-pattern story enters only if its fit review does not decline it.

## a2a wave 2, HTTP binding (2026-10-22)

The orchestrator relayed 7 `ess-http/1` gaps (H1-H7) from the adopter session specifying A2A in
beyond10x/a2a (a2a wave 2, story http-binding, ESS 0.67.0). Each is cited as "a2a wave-2 HTTP
binding gaps list, line N". They are filed as draft stories for 0.69.0, after the
`ess-0.69.0-first` set and beside the wave-1 gaps; they carry `ess-0.69.0`, not
`ess-0.69.0-first`. No existing story held any of them, so nothing was merged:
`story:a-command-binds-to-a-protocol-or-deployment-http-route` (the `ess-http/1` umbrella) informs
each, and `story:a-command-binds-to-json-rpc-and-grpc-beside-http` is about other transports. H1,
H3 and H5 reopen decisions in `docs/design/http-presentation-binding.md`, so each needs a fit review
before dispatch. 0.68.0 is not held for them.

| gap | line | story |
|---|---|---|
| H1 | 8 | `story:a-path-segment-may-be-a-variable-with-a-verb-suffix` |
| H2 | 9 | `story:an-operation-may-have-additional-http-routes` |
| H3 | 10 | `story:an-optional-input-may-select-between-an-operations-routes` (depends on H2) |
| H4 | 11 | `story:a-delete-command-may-take-its-input-from-the-query-string` |
| H5 | 12 | `story:a-literal-path-segment-may-take-precedence-over-a-variable` |
| H6 | 13 | `story:an-outcome-may-answer-a-response-field-bare-and-errors-share-a-default-status` |
| H7 | 14 | `story:a-view-parameter-carries-its-wire-name-on-every-http-reader` (`naming.wire` may already cover it) |

Not filed: the list's upstream notes (lines 16-17), on disagreements between the A2A specification
and its protobuf definitions; they are about A2A, not ESS.

## Adopter defects, recheck batch (2026-10-22)

Recheck of the adopter report against the 0.68.0 integration build (runtime identical to 52bd110dd) and 0.67.0, each item with a
neutral fixture.

- Item 7: `story:a-guard-matches-an-input-against-a-stored-wildcard-pattern` declined as by design and moved to `rejected`: a
  pattern held as data is the system's evaluator, an `external:` outcome (`docs/design/stored-rules-boundary.md`, "Patterns held
  as data"; beyond10x/ess#449).
- Item 5: `story:the-interpreter-mints-a-newtype-under-a-disjunction-of-prefixes` narrowed and retitled. 0.68.0 fixed the
  response path (`262c66ca8`); what remains is the value minted for an entity field or event payload, which satisfies no
  positive text invariant (`starts_with`, `contains`, an `any:` of them) its base text misses. Not a regression.

| item | story | 0.68.0 regression |
|---|---|---|
| 11 | `story:an-update-may-fall-back-to-the-stored-value-when-an-optional-input-is-absent` (new, gap) | no |
| 12 | `story:accepts-nothing-may-be-declared-under-a-when-related-guard` (new, gap; related beyond10x/ess#554) | no |
| 13 | `story:an-input-guard-may-apply-only-on-the-create-path-of-create-or-update` (new, gap; beyond10x/ess#554) | no |
| 14 | `story:a-selection-delete-may-bound-how-many-rows-one-call-removes` (new, gap) | no |
| 15 | `story:a-view-may-declare-a-fixed-server-side-row-cap` (new, gap) | no |
| 16 | `story:expression-arithmetic-and-time` (existing, beyond10x/ess#560; report appended) | no |
| 17 | `story:a-periodic-binding-may-run-on-a-wall-clock-schedule` (new, gap) | no |
| 18 | `story:an-update-addresses-its-record-by-an-optional-identity-or-a-unique-field` (new, gap; beyond10x/ess#564) | no |
| 19 | `story:a-list-input-key-conflict-is-refused-after-the-existence-check` (new, gap; related beyond10x/ess#554) | no |
| 20 | `story:an-input-path-reads-a-list-element-and-a-fallback-chain-tries-several-places` (new, gap) | no |

Every item gives identical refusals, and byte-identical suites where synthesis runs, on 0.67.0 and 0.68.0. None blocks 0.68.0.

## Scope 2026-10-23: one pull request

The operator: "make sure ess knows that we expect all of those fixed efficiently at once via one PR". All 41 items of the adopter gap report (G1-G6, R1-R12, S1-S7, N1-N16) land in pull request 633 with the 0.69.0 wave, grouped into one unit per shared cause. An item whose fix needs an unaccepted ADR stays a row and waits on that ADR. By design, with a reason on its issue or here: R6 (ess#93), R12, S4, N6.

| unit | ids | cause | size | ADR blocker | priority |
|---|---|---|---|---|---|
| G-U15 | N1 | generate types has no command input, response or view row root | L | none | 1 |
| G-U16 | N15,N16 | several-row related arrangement creates the related owner apart from the subject (inferred) | L | none | 2 |
| G-U17 | N5 | generated manifest pins exact versions | S | none | 3 |
| G-U18 | N2,N13,N14 | Rust realizer takes no generator options | M | none | 3 |
| G-U19 | N3 | Rust realizer reads schema values, declared variant names are gone | M | none | 4 |
| G-U20 | N8 | Go synthesis allocates no identifier for a dotted name | S | none | 4 |
| G-U9 | S1 | no WebSocket transport binding | L | transport-and-idl-compile-targets (proposed; needs WebSocket amendment) | 5 |
| G-U10 | S2 | routes projected regardless of reached_by | M | none | 5 |
| G-U11 | S5 | ess-http/1 lacks operation_id and servers | M | none | 5 |
| U54 | N4 | input and response fields share one allocation namespace | M | none | 6 |
| G-U13 | S6 | types generator lacks the shared --check path | S | none | 7 |
| G-U22 | N9 | no cross-specification type import | L | new ADR needed | 8 |
| G-U23 | N10,N11 | composition reference admission: kinds a conformance end may name, and its refusal hints | M | none | 8 |
| G-U24 | N12 | implements rule vs a caller sending a subset (repro pending) | M | none | 8 |
| U16 | R1,G6 | create-or-update beside a related-row refusal | M | none | 9 |
| U19 | G4 | lifecycle states have no wire name | L | none | 9 |
| U31 | R3,R4 | subject identity and defined coverage in guards | M | none | 9 |
| U32 | G5 | via and instance cannot name a struct member | M | none | 9 |
| U33 | R2,R7 | precedence of a related row against input guards (#554) | L | none | 9 |
| U41 | R5 | sets cannot read a response value | L | none | 9 |
| G-U1 | G1 | caller attributes not readable in when_related | M | none | 9 |
| G-U2 | G2 | union tag unreadable; content key fixed | M | none | 9 |
| G-U3 | G3 | no composite identity | L | a-compound-key-is-a-uniqueness-constraint-first (proposed) | 9 |
| G-U4 | R8 | Binary64 refused at admission; suite cannot carry it | L | none | 9 |
| G-U5 | R9 | OpenAPI import refuses open objects | L | ess-evolution-08-reviewable-source-candidates (proposed) | 9 |
| G-U6 | R10 | imported integer formats not mapped to widths | M | none | 9 |
| G-U7 | R11 | empty-struct refusal ignores openness | S | none | 9 |
| G-U8 | R6,R12,S4,N6 | by design; answers documented on issues | S | none | 9 |
| G-U12 | S3 | ESS-HTTP-025 has no declared-alike exception | M | none | 9 |
| G-U14 | S7 | grant/denied asserts an unqueried view | S | none | 9 |
| G-U21 | N7 | field order lost in the BTreeMap plan; Box on every reference | M | none | 9 |

## Approval 2026-10-23

- The 31 units run in the priority order above, G-U15 (N1) first: it edits ess-gen model_types.rs and U55 edits ess-gen types.rs, so the two do not share a file.
- G-U3, G-U5, G-U9 and G-U22 wait on their ADRs. Without a decision they leave pull request 633 and their issues stay open; the rest of the wave does not wait for them.
- One tree builds at a time; each unit tree discards its build cache when the unit merges.
