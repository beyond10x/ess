---
format: aep.planning-md/1
id: review-result:entity-runtime-lowering-source-pass-1
kind: review-result
status: active
title: 'Complete ESS lowerer source examination: literal preservation and response presence'
relations:
- reviews: story:entity-runtime-service-lowering
revision: 1
---
# Independent whole lowerer source examination — pass 1 of 2

```text
unit: story:entity-runtime-service-lowering, first whole source examination
candidate: a8ec43feb2364819941ce0cdc1fdb1290983c041
tree: de5130aec2ad639afb4f57ca2632da3e88f53ad7
base: be604d874ee9e567ae104e565e7cbddf953885a9
target: entity-runtime 250f6993181822ab1e36c17d38dbc909d084d423
verdict: NEEDS-CHANGE — 2 introduced blockers
reviewer cases: 2 compiled, 2 behavioral failures
production edits: none
reviewer addition: 1 dedicated test file, 156 lines
```

## Scope and source identity

I examined the complete 14-path candidate diff: workspace/changelog wiring, the new crate manifest,
all 2,735 lines of `src/lib.rs`, all 971 lines of the author integration target, all five focused
fixture files, the admitted design, and both diagnostic model files. I also read the original
implementation contract, author report, requirement map, and source manifest. The complete billing
and gatepass fixtures and their existing tests remain unchanged.

The submitted source manifest rechecked all 14 paths at exit 0. Its SHA-256 is
`8fcf0c5f33f59e6275cc473816ef50fa24a9f48d12620cbc5e0f32f2e0e2f025`; the author report is
`a2d55950e1b9f56e1433812a2296f5387ccdb1fbac3ca22ada4991eb62081d24`. The checkout remained at
candidate `a8ec43f` and tree `de5130a`. The production crate does not exist at the base, so both
findings below are introduced by this candidate.

The only source-tree addition is the untracked dedicated reviewer target
`crates/generate/ess-entity-runtime/tests/reviewer_source_review_1.rs`, SHA-256
`2027faa1b837a598430f396196ae939a7de07dfc3c5ca0460309afc69a755c01`. No production file,
existing test, fixture, design, manifest, lockfile, changelog, or planning artifact was edited.

## Findings

### F1 — canonical slot renumbering rewrites authored string values

`remap_outcome_slots` serializes an entire `OutcomeDefinition` to untyped JSON and
`remap_slot_strings` recursively treats **every** string equal to an old slot argument/template as
a reference (`src/lib.rs:2290-2327`). This includes authored literals in `set`, event payloads,
conditions, refusal text, and logical identity values. A literal is data and has no reference tag,
so its spelling cannot safely select this rewrite.

The reviewer case at `reviewer_source_review_1.rs:94` starts from the admitted focused service and
changes only the existing `marker` literal to `$args.bound.b00000002`. Existing identity,
undetermined, event, response, and external slots make canonical numbering move old slot 2. The
lowerer succeeds, but the emitted event contains `$args.bound.b00000005` rather than the authored
literal. The isolated behavioral run and the final two-case run both exit 101 with that exact
left/right result (`logs/06-literal-regression.log`, `logs/07-final-focused-reviewer-tests.log`).

This is reachable from an ordinary accepted ESS string literal. It breaks exact values, event
payloads, and the promise that canonicalization changes references without changing source data.

**Verdict `NEEDS-CHANGE`, origin `introduced`, severity `blocker`.**

### F2 — response reuse derives presence and type from the destination rather than the response

For a direct `ResolvedPayloadValue::ResponseField`, `mapping_value` ignores the source `type_ref`
and constructs the semantic response slot from `mapping.target_type` (`src/lib.rs:1894-1912`). A
required source is assignable to an optional target under the admitted ESS type rules, so the
destination's optional wrapper cannot define the response value's presence. The later response
materialization correctly requests the same key as a required response slot. `SlotBook::allocate`
then finds two unequal descriptions for one semantic value and panics at `src/lib.rs:2154` under
debug assertions. With assertions disabled it retains the first optional slot, leaving a required
response backed by an optional argument leaf.

The reviewer case at `reviewer_source_review_1.rs:117` makes the existing `optional_receipt`
response required while retaining its reuse in an optional event field. The ESS compiler admits the
required-to-optional assignment. Lowering panics with the exact `Optional<String>/Optional` versus
`String/Required` `BoundValue` mismatch. The final behavioral target exits 101 at the production
assertion (`logs/07-final-focused-reviewer-tests.log`).

This violates the normative rule that the source mapping and semantic reuse own one value and one
presence choice; destination optionality alone cannot make a required response optional.

**Verdict `NEEDS-CHANGE`, origin `introduced`, severity `blocker`.**

## Complete ten-dimension examination

| Dimension | Examination and disposition |
|---|---|
| Creation | Traced observed identity selection, one-create enforcement, required/optional undetermined fields, conditional insertion, event identity reuse, and `IdentitySupplied`. Existing real billing/gatepass creation tests remain intact. No additional finding. |
| Updates | Traced exact `Updates` effect, accepting-branch detection, explicit mappings, and the complete per-field fulfillment walk. Every omitted declared non-identity field receives a typed requirement; no implicit `Preserve` is emitted. No additional finding. |
| Transitions | Traced exact ordered `from` collection and `to` state, state/predicate selection, wrong-state normalization, and post-selection fulfillment. The author runtime cases cover IssueInvoice, PayInvoice, and AdmitVisitor state effects. No additional finding. |
| Predicates | Examined all predicate variants, empty truth tables, operator translation, input/entity/nominal/bound path roots, and quantifier scoping. No additional finding. |
| Invariants | Examined entity rules and recursive newtype/struct expansion at every use, optional guards, collection quantification, union tag/content guards, and active recursion handling. No additional finding. |
| Identities | Examined required identity schemas, supplied versus observed instances, literal/input/generated/response/conversion identities, immutable operation identity, shared creation/event template, and host address obligation. No additional finding. |
| Relations and closure | Traced owned entities, transitive outgoing targets, incoming `Owns` declarers to fixpoint, exact relation kind/target/cardinality/via, three graph obligations, individual validation, and complete-registry validation. No additional finding. |
| Exact values and fields | Examined every primitive and declared structural shape, outer optionality, nullable collection refusal, recursion refusal, exact scalar decoding, typed bound schemas, and operation action algebra. F1 corrupts authored string values during canonicalization; F2 assigns response presence from a destination. |
| Outcomes and commands | Examined target/entrypoint diagnostics, exact input/response schemas, source outcome order with default/wrong-state normalization, refusals/errors, external evidence, service/1-/2-/3 selection, no-partial-output behavior, and all operation fulfillment coordinates. F2 reaches an otherwise admitted accepting outcome. |
| Event order and multiplicity | Traced zero/one/many and duplicate occurrences, occurrence-specific omitted slots, exact event names, closed payloads, optional insertion, response reuse, and empty payloads. F1 changes an exact event literal; F2 makes shared event/response presence inconsistent. |

The broader selected-closure, source preservation, target validation, deterministic second build,
source/synthesis digests, typed obligation inventory, and capability-order paths were also examined.
The author requirement map overstates what its `complete_real_fixture_inventory...` body directly
asserts for fields, relations, invariants, and scales, but the corresponding source paths are typed,
passed complete registry validation in retained author evidence, and yielded no third reachable
counterexample in this pass. The two findings above occur in behaviors the existing tests do name:
exact literals, canonical slots, and shared optional response reuse.

## Commands and results

The focused commands used Rust 1.91.0, one Cargo job, locked/offline mode, empty wrappers, `lld`,
debug info 0, incremental 0, an isolated target, and an owned evidence `TMPDIR`. Preflight measured
27 GiB free disk and about 45 GiB available RAM.

| Log | Exit | Meaning |
|---|---:|---|
| `01-author-source-manifest-check.log` | 0 | all 14 submitted source rows match |
| `02-focused-capacity.log` | 0 | capacity and tool versions |
| `03-focused-reviewer-tests.log` | 101 | reviewer-test compilation only: attempted `contains_key` directly on `serde_json::Value`; production not exercised |
| `04-focused-rerun-capacity.log` | 0 | capacity before corrected rerun |
| `05-focused-reviewer-tests-rerun.log` | 101 | F2 reached production and panicked; F1 had a reviewer-only wrong normalized outcome index |
| `06-literal-regression.log` | 101 | corrected isolated F1 behavioral failure, expected slot-like literal 2 and observed rewritten literal 5 |
| `07-final-focused-reviewer-tests.log` | 101 | final compiled two-case target; both failures are production behavior |
| `08-reviewer-rustfmt-check.log` | 1 | formatting-only diff in the new reviewer helper |
| `09-reviewer-rustfmt-restored.log` | 0 | final reviewer file is rustfmt-clean |
| `10-origin-status-diffcheck.log` | 0 group | exact candidate/tree; base-path query 128 proves crate absent at base; tracked diff check 0 |

No full package or workspace gate was run: the granted lane was explicitly bounded to the dedicated
reviewer target, and two blockers were already reproduced. The author's retained full crate,
Clippy, rustdoc, and Rust 1.85 evidence is not relabeled as independent reviewer execution.

## Evidence hashes and residual limits

- `manifests/candidate-source.sha256`: copied exact 14-row manifest, SHA-256
  `8fcf0c5f33f59e6275cc473816ef50fa24a9f48d12620cbc5e0f32f2e0e2f025`.
- `manifests/reviewer-tests.sha256`: final test row, manifest SHA-256
  `fb7556e7f8324a81dc80cf4122fd91a7174722c691d5ad03bf3d78a3ad405bda`.
- `manifests/logs.sha256`: ten raw log rows, manifest SHA-256
  `634f4198c0fcfdf3e4ebda8005b1055ff587c8c10b690ea4d9190506192d0f16`.

This pure review used no provider, store, executor, Eventlog, filesystem/network runtime path, or
external system. It makes no SDK `/4`, persistence, administration, publication, or deployment
claim. The isolated reproducible Cargo target remains at
`home-path:sha256:9ca5027fffe05fd3ed210e4c57e20ad60f93c1556e195ef119453f768085b96e`; evidence and owned `TMPDIR` remain under
the assigned evidence directory. There are no live commands.

## Findings block

```findings
- file: crates/generate/ess-entity-runtime/src/lib.rs
  line: 2300
  category: value-integrity
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: canonical slot renumbering recursively rewrites every matching JSON string, so an admitted authored literal equal to an old slot template is silently changed in the lowered event payload
- file: crates/generate/ess-entity-runtime/src/lib.rs
  line: 1896
  category: binding-presence
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a reused response slot derives its type and requiredness from the optional destination instead of the required response source, causing an admitted required-response reuse to panic or retain the wrong optional presence
```
