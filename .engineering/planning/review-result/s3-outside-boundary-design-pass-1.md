---
format: aep.planning-md/1
id: review-result:s3-outside-boundary-design-pass-1
kind: review-result
status: active
title: First independent examination of the approved finite boundary amendment
relations:
- informed_by: task:consumer-accounting-authored-boundaries
- reviews: task:consumer-accounting-authored-boundaries
revision: 1
---
# S3 outside-consumer-boundary amendment — first independent design examination

Verdict: **needs-revision**. Three blocking findings, three advisory.
Reviewer: fresh Opus, first of at most two substantive design passes. Read-only. No children, no
build, no source edit, no SQL, no lease. Nothing adopted, nothing qualified, no gate run.

The approved policy is not re-litigated here. The decision at `s3-reconciliation/decision.md` is
taken as given; this pass asks only whether the proposal implements it faithfully and can be
implemented without weakening an existing gate.

## Frozen proposal tree — hashes recomputed

| Path | Recomputed SHA256 | Brief |
| --- | --- | --- |
| `docs/design/consumer-outside-boundary-accounting.md` | `f00dc712fc14e8e251d7c7474d864ad6d8f7551343a4b8e9c929060fd7db9186` | match |
| `ess/consumer-outside-boundary/system.yaml` | `55cf51f5120a58a67a801afefe2631420859cce5270424417d2a416275e523d9` | match |
| `ess/consumer-outside-boundary/domains/outside_boundary.yaml` | `1c4a4fc420edd10fcc6b136624490d5391d61ada649b0631f6e7f845c2856f57` | match |

Recomputed in `~/.local/state/worktree/trees/b10x/ess/ess-evolution-s3-authored-boundaries-20260915`
(`sha256sum`, exit 0). All three equal the brief and `s3-boundary-amendment/RESULT.md`. Inherited
earlier changes in that tree were excluded from review; canonical source was read from the read-only
sibling `ess-evolution-scope-20260915` throughout.

Model validation reproduced independently: `ess --version` → `ess 0.24.0`;
`ess specify validate --path ess/consumer-outside-boundary` → `consumer v1 — 2 file(s), valid`,
exit 0.

---

## Blocking findings

### B1 — the plan-insertion instruction and the do-not-extend-the-shared-enum instruction cannot both hold

`docs/design/consumer-outside-boundary-accounting.md:258-261` is explicit and correct:

> **Do not extend the shared enum.** `DispositionV2` is currently shared by `ExecutionPlanV2`, `V3`
> and `V4`; adding a variant there would retroactively admit `OutsideConsumerBoundary` into a v4
> plan […] Add `DispositionV5`, `PlannedCellV5` and `ExecutionPlanV5` beside the existing types.

The sharing is real. `enforce.rs:125`, `enforce.rs:165` and `enforce.rs:201` all declare
`cells: Vec<PlannedCellV2>`, and `PlannedCellV2.disposition` is `DispositionV2` (`enforce.rs:84-90`).
`read_plan_v4` re-tags to v2 and delegates to `read_plan_versioned` (`enforce.rs:1245-1253`), which
deserializes `ExecutionPlanV2` (`enforce.rs:1126`). A sixth `DispositionV2` variant is therefore
readable by the v4 reader, and would additionally be *counted* by `qualify_v3` and `qualify_v4`,
whose match arms over `DispositionV2` are exhaustive (`enforce.rs:1586-1590`, `enforce.rs:1668-1672`)
and feed a conservation assertion (`enforce.rs:1511-1513`).

Two other instructions in the same document require exactly that extension:

- `:288-290` — "Outside-boundary cells are inserted after the aggregate claims and before the
  unaccounted sweep, **through the existing `insert_v2`**". `insert_v2` takes
  `&mut BTreeMap<(String,String), PlannedCellV2>` and a `PlannedCellV2` (`enforce.rs:464-466`). A
  cell passed through it carries a `DispositionV2` and nothing else.
- `:282` — "`plan_versioned` gains an `OutsideBoundaryPolicy` parameter **in the same shape as**
  `AggregatePolicy`". `AggregatePolicy` (`enforce.rs:503-506`) selects between two claim functions
  and two case functions (`enforce.rs:514-529`); both branches end at one construction,
  `DispositionV2::AggregateClosure` (`enforce.rs:651`). A parameter of that shape cannot change the
  cell type, the map type, or the returned `ExecutionPlanV2` (`enforce.rs:714-742`).

**Reachability.** An implementor works the "Exact Rust implementation surfaces" table
(`:271-285`), which is the operative instruction list. Following it produces
`DispositionV2::OutsideConsumerBoundary`, because `insert_v2` and `plan_versioned` admit no other
value. The v5 types at `:260-261` are then unreachable decoration.

**Effect.** The implementation lands the precise defect the document names, and
`consumer-accounting-applicability.md:144` forbids ("A shared enum must not silently extend v1"): an
`ess-consumer-accounting/4` plan that parses, counts and qualifies a disposition no v4 review ever
covered. The old-reader rejection cases at `:265-269` would fail to go red for the
"`OutsideConsumerBoundary` disposition inside a v2/v3/v4 plan" case, because the shared enum accepts
it — so the amendment's own guard against this is the test that cannot pass.

**What the revision must supply.** The v5 plan path, concretely: either `plan_versioned` is
generified over the cell/disposition type (and `insert_v2` with it), or a `plan_v5` builds its own
`BTreeMap<_, PlannedCellV5>` and the outside-boundary cells never enter the v2 map. Naming
`insert_v2` and an `AggregatePolicy`-shaped parameter is not compatible with either.

### B2 — `replacement_matches` cannot carry the new claim kind without the same extension

`:283` requires `reconciliation.rs` to gain "v2 claim enum with `OutsideConsumerBoundary { boundary }`,
its `validate` arm, and the matching `enforce::replacement_matches` arm".

`replacement_matches` is `fn(&ReplacementClaim, &DispositionV2) -> bool` (`enforce.rs:904-907`). Its
only call site is `enforce.rs:663`, passing `cell.disposition` from the v2 cell map. An arm matching
a new claim kind must pair it with a `DispositionV2` variant; there is no other operand.

**Reachability.** This is not avoidable by ordering. The 27 changed frozen pairs are genuine
`Action::Replace` decisions — `reconciliation::resolve` reaches the `Replace` arm for every frozen
tuple whose current shape differs (`reconciliation.rs:245-296`), and `ReplacementClaim` currently
admits only `Supported`, `Refused`, `AggregateClosure` (`reconciliation.rs:76-89`). Confirmed in the
evidence: `exact-boundary-conflict.json` carries exactly 27 of 168 rows with a non-null `old_shape`
(9 models × 3 profiles).

**Effect.** Either `DispositionV2` is extended (B1's defect), or the 27 pairs have no path to a
matched replacement and `plan_versioned` bails at `enforce.rs:665-668` with "reconciliation
replacement lacks its exact current behavior" for all 27. The document treats this as a solved point
(`RESULT.md` finding 1) when the solution it names is the one it forbids.

### B3 — G4 has zero satisfiable downstream cells at current evidence, and the specified controls require a green re-run

G4 (`:187-208`) requires, for each of the 168 pairs, that "the same execution plan must carry a
`Supported` or `Refused` cell at `(model, shape, downstream_consumer, downstream_profile)` whose
cases are all in this run's executed case union", and that "a missing, stale or unexecuted
downstream cell fails the check".

Measured against the canonical authority
`crates/edge/ess-xtask/src/consumer_coverage/reviewed-model-behavior.json`: 2,061 claims total,
distributed across exactly three consumers — `acquisition-specification-direct-file` (687),
`acquisition-specification-legacy-directory` (687), `acquisition-specification-manifest` (687).
Claims at any of the seven compiler profiles G4 names: **0**. Of the 56 models in scope, number
covered by at least one of the seven: **0**.

The seven profiles themselves check out — `profiles.json` contains exactly seven `model-consumer`
profiles with an `ess_compiler` entrypoint in their declared set, and their ids match the document's
table at `:194-201` one for one. The eligibility argument is sound. The evidence is absent.

**Reachability.** Every one of the 168 G4 checks fails on the first run after implementation.

**Effect.** Two specific instructions become unexecutable as written:

- `:309` and `:310` both end "restore the exact manifest and lock bytes and re-run green" /
  "restore exact bytes and re-run green". The restored state is not green: G4 refuses 168 times.
  The two experiments the document itself calls "the two that matter" (`:316-319`) have no valid
  control state to return to.
- The "Exact Rust implementation surfaces" table (`:274-285`) and the "Decisive tests" table
  (`:298-314`) name no work item for authoring and executing 168 downstream behavior claims at
  compiler profiles. That work is the larger half of the amendment and is currently invisible in its
  scope contract.

This is faithful to constraint 4 of the decision ("Missing real-consumer evidence still fails
complete accounting") — the *policy* is implemented correctly. What is missing is the statement that
implementing this amendment converts 168 `UNPROVED_NO_ALLOWED_DISPOSITION` refusals into 168 G4
refusals until that separate body of work exists, and that the amendment's own control experiments
cannot be run before then. A reader of `:309-319` would reasonably expect otherwise.

---

## Advisory findings

### A1 — the cited precedent is the opposite of the prescription (`:260-261`)

"Add `DispositionV5`, `PlannedCellV5` and `ExecutionPlanV5` beside the existing types, **the way
`ExecutionPlanV3` and `V4` were added beside `V2`**."

`ExecutionPlanV3` (`enforce.rs:159-165`) and `ExecutionPlanV4` (`enforce.rs:195-201`) were added
beside `V2` by *reusing* `PlannedCellV2` and `DispositionV2`. They got no cell type of their own.
The precedent that actually matches the prescription is the `AggregateClosure` addition:
`Disposition` (4 variants, `enforce.rs:62-80`) was left closed and `DispositionV2` /
`PlannedCellV2` were added beside it (`enforce.rs:82-115`).

An implementor copying the named precedent reproduces the shared-enum reuse the same paragraph
forbids. This compounds B1 rather than mitigating it. Cite the `Disposition` → `DispositionV2`
precedent instead.

### A2 — the insertion window at `:288-290` spans the replacement check

"Outside-boundary cells are inserted after the aggregate claims and before the unaccounted sweep."

Between the aggregate insertion loop (ends `enforce.rs:657`) and the unaccounted sweep
(`enforce.rs:681-687`) sit two other passes: the reconciliation replacement check
(`enforce.rs:659-670`) and the stale shape/profile check (`enforce.rs:671-680`). Only insertion
*before* line 659 works — the 27 replacement claims are matched by looking the cell up in the map
(`enforce.rs:661-663`), so a cell inserted after that pass is invisible to it and all 27 bail.

The window as stated admits a wrong answer. Name the exact point: immediately after
`required_cases.extend(aggregate_cases(...))` at `enforce.rs:658`.

The duplicate-refusal claim in the same paragraph is correct: `insert_v2` pushes a problem on any
second write to a `(model, consumer)` key (`enforce.rs:470-475`), so there is no last-writer-wins
path. The `PreferExactCurrentBehavior` claim is also correct: `current_behavior_identities` is built
only from `behavior_cells` (`enforce.rs:578-586`), so it gains no outside-boundary member.

### A3 — the document's description of the typed model contradicts the model

`:326` — "A pair leaves `Declared` only through `ProvePair`".

`ess/consumer-outside-boundary/domains/outside_boundary.yaml:165-167` declares
`invalidate` `from: [Declared, OutsideBoundary]`. A pair leaves `Declared` through either command.
The model is the better of the two — invalidation from `Declared` is what a stale reviewed row needs
— so correct the prose, not the model.

---

## Verified without finding

Every specific below was read from the canonical read-only tree
`ess-evolution-scope-20260915` or from the retained evidence, not inferred.

| Claim | Location | Result |
| --- | --- | --- |
| `spec.rs` digest `c4a73f9c…f332a977` | `crates/specify/ess-domain/src/spec.rs` | match (`sha256sum`) |
| `parse`/`assemble`/`validate` at lines 151 / 219 / 266 | same file | match |
| `baseline_sha256` `3dd8dff5…374a47de` | `consumer_coverage/initial-baseline.json` | match (`sha256sum`) |
| frozen file: 1,813 models, 87 groups, 157,677 eligible pairs | same | match; `mandatory_pairs_excluded` 54 |
| three authored groups carry all 1,813 models, owner `ess-domain`, follow-up `epic:qualify-initial-consumer-baseline` | same | match, all three |
| authored-admission profile `07e86fbe…` | `exact-boundary-conflict.json`, frozen group | match |
| 56 models / 168 pairs / 56 per profile / all `rust:ess_compiler::*` | `exact-boundary-conflict.json` | match |
| 141 with null `old_shape`, 27 with a shape | same | match |
| 47 unfrozen models need no reconciliation decision | `reconciliation.rs:245` iterates frozen `old_cells` only | correct |
| seven `model-consumer` profiles with an `ess_compiler` entrypoint, ids as tabled | `consumer_coverage/profiles.json` | exactly seven, ids match |
| G2's six checks mirror `model_behavior::candidate` | `model_behavior.rs:376-402` | field-for-field: `profile_sha256`, one-element `definition.entrypoints`, `[entry]` destructure for absent/ambiguous, `entry["source"]`, `declaration_sha256`, `source_item_sha256`, `sources.get(entrypoint_source)` |
| private qualified-proof pattern | `model_behavior.rs:284-312` | `VerifiedExecution` is `pub(super)` with private fields and `pub(super)` accessors; `qualify_v4` takes it by reference (`enforce.rs:1628`). `VerifiedOutsideBoundary` mirrors it, including `payload()`/`digest()` used at `enforce.rs:1647` |
| the existing extraction call is `cargo metadata --locked --offline --no-deps --format-version 1` | `mod.rs:86-94`, cargo path pinned from `build["tools"]["CARGO"]["path"]` at `mod.rs:83` | match; dropping `--no-deps` is the correct way to obtain `resolve` |
| `check_at` ordering: aggregate verify → qualify | `mod.rs:697` executed union, `mod.rs:705` `aggregate::verify`, `mod.rs:712` `qualify_v4` | the proposed insertion point is available |
| `read_reviewed`, `plan_extraction`, `check_at` exist | `mod.rs:427`, `510`, `620` | match |
| every named old reader exists | `account.rs:39`, `115`, `146`, `178`; `enforce.rs:1118`, `1224`, `1245` | match |
| `aggregate_cells` needs a v5 tag arm | `enforce.rs:1390-1395` | correct — the `_` arm falls through to `read_plan_v2`, which would misread a v5 plan |
| the two quotations from `consumer-accounting-applicability.md` | lines 57 and 144 | both verbatim |
| Rust inventory gaining `obligation_owner` has no further format consequence | see below | **confirmed** |
| the real made-reachable experiment can reach G1 | see below | **confirmed** |

### The Rust inventory ownership claim holds

The document claims three version steps "and nothing else" (`:243-255`) while also adding an
`obligation_owner` map to the Rust inventory (`:182-184`, `:284`). I traced every consumer of that
file. It is produced at `mod.rs:105` and written at `mod.rs:115` as an untagged object
(`rust.rs:630`: `{obligations, references, macro_definitions, diagnostic_macros}`). It is read only
inside `mod.rs` — at `:108` (`diagnostic_macros`), `:123` (`obligations`), `:403-408`
(`extracted_models`), `:546` and `:704` (`aggregate::capture`). Every read is a `Value` index; no
struct with `deny_unknown_fields` deserializes the inventory, so a new top-level key is inert.

The one place it could have leaked into a digest is the aggregate structural receipt.
`aggregate::structure` rebuilds a fresh object from `rust["obligations"]` and `rust["references"]`
only and returns `{obligations, references}` (`aggregate.rs:39-69`); `fresh_current_structure_sha256`
(`aggregate.rs:1086`, `:1461`) hashes that reconstruction, not the inventory file. The other
candidate, `provider_profile_sha256` (`model_behavior.rs:412`), hashes `source-profile.json`
(`mod.rs:110`), which contains source digests and build identity, not the inventory.

No fourth format step is required. The claim is correct.

The supporting claim at `:180-181` is also correct with one nuance worth recording: `rust::extract`
reads exactly three crate roots and prefixes each declaration with the crate it walked
(`rust.rs:69-83`, prefix `name.replace('-', "_")`), so the crate segment is extractor output. The
nuance is that the prefix is a Rust module name (`ess_compiler`) and `model_package` is a Cargo
package name (`ess-compiler`); the underscore-to-hyphen mapping is a convention, which is a further
reason to emit `obligation_owner` explicitly as the document already proposes.

### The real made-reachable experiment reaches its guard

`:309` specifies adding `ess-compiler` to `crates/specify/ess-domain/Cargo.toml` `[dev-dependencies]`
with a test naming `ess_compiler::ir::ResolvedType`, built and run through the ordinary gate.

- The type exists and is reachable: `pub struct ResolvedType` at
  `crates/specify/ess-compiler/src/ir.rs:335`, under `pub mod ir` at
  `crates/specify/ess-compiler/src/lib.rs:50`.
- It is one of the 56 in-scope models (`rust:ess_compiler::ir::ResolvedType` appears in
  `exact-boundary-conflict.json`), so the experiment is decisive for this amendment rather than
  incidental.
- The dev-dependency cycle is legal: `ess-compiler` depends on `ess-domain` normally, and Cargo
  permits a dev-dependency back-edge. `ess-domain` already carries a `[dev-dependencies]` section
  (`Cargo.toml:23-24`).
- G1 traverses `resolve.nodes` over every `dep_kinds` entry including dev (`:112-117`), so the new
  edge is reached and the guard refuses.

I specifically checked whether the experiment would instead die earlier, at the stage-1 consumer
inventory: `consumer::extract` does walk `test` targets (`consumer.rs:63-74`, `:85`), and `classify`
refuses any unclassified entry in both directions (`proposal.rs:493-501`), which would have made the
observed refusal `classify`'s rather than G1's. It does not: every entry-emitting arm in
`consumer.rs:246-313` is guarded by `if !test`, including the module-boundary entry at
`consumer.rs:246-247`, and `review_cases` iterates the reviewed set only, so an extra actual case is
not refused (`proposal.rs:254-257`). A test target adds cases, never entries. The experiment reaches
G1.

---

## Sources inspected

Canonical read-only tree `~/.local/state/worktree/trees/b10x/ess/ess-evolution-scope-20260915`:
`crates/edge/ess-xtask/src/consumer_coverage/{enforce,reconciliation,model_behavior,mod,proposal,consumer,rust,aggregate,account,preservation}.rs`;
`consumer_coverage/{initial-baseline.json,profiles.json,reviewed-model-behavior.json}`;
`crates/specify/ess-domain/src/spec.rs`; `crates/specify/ess-domain/Cargo.toml`;
`crates/specify/ess-compiler/src/{lib.rs,ir.rs,resolve.rs,refs.rs,expression.rs}`;
`docs/design/consumer-accounting-applicability.md`.

Frozen proposal tree: the three hashed paths, plus `ess/consumer-outside-boundary/system.yaml`.

Evidence: `s3-boundary-amendment-brief.md`, `s3-reconciliation/decision.md`,
`s3-reconciliation/exact-boundary-conflict.json`, `s3-boundary-amendment/RESULT.md`.

Commands run: `sha256sum` (4 invocations), `ess --version`, `ess specify validate --path
ess/consumer-outside-boundary`, `git log`/`git status` on both trees, and read-only `grep`/`python3`
inspection of the JSON authorities. No build, no test, no mutation, no network, no cleanup.

## What could not be established

- **Whether `task consumer-check` is green today.** No build token; the gate was not run. B3's
  "re-run green" analysis rests on G4's specified semantics and the measured absence of compiler-
  profile claims, not on an observed exit code. If the gate is already red for these 168 cells, B3's
  practical effect is narrower than stated — but the control experiments at `:309-310` are still
  unexecutable as written, and that part stands regardless.
- **Whether `cargo metadata --locked --offline` without `--no-deps` succeeds in this checkout.** Not
  run (build prohibition). It requires every dependency manifest to be present in the local registry
  cache. The retained `s3-reconciliation/cargo-metadata.json` shows a prior offline resolve
  succeeded (exit 0, 91/107 node readings), which is supporting but not current evidence.
- **Whether `--locked` tolerates the dev-dependency edge without a lock regeneration.** Not run. The
  document anticipates a lock change and requires restoring its exact bytes, so this is a procedural
  detail rather than an open question.
- **Whether the 168 downstream obligations in B3 are achievable at all** — that is, whether the
  seven compiler profiles can carry executed `Supported`/`Refused` claims for these 56 models. That
  is a question about the native-case authority's admission rules, which the brief places outside
  this examination. B3 reports only that zero such claims exist now.

---

```yaml
findings:
  - file: docs/design/consumer-outside-boundary-accounting.md
    line: 290
    category: implementability-contradiction
    severity: blocker
    verdict: CONFIRMED
    origin: crates/edge/ess-xtask/src/consumer_coverage/enforce.rs:464
    message: >-
      Inserting outside-boundary cells "through the existing insert_v2" requires a
      DispositionV2::OutsideConsumerBoundary variant, because insert_v2 takes PlannedCellV2
      (enforce.rs:464-466) whose disposition field is DispositionV2 (enforce.rs:84-90). That
      contradicts the same document's instruction at :258-261 not to extend the shared enum, and
      produces exactly the retroactive v4 admission it names: read_plan_v4 delegates to
      read_plan_versioned over ExecutionPlanV2 (enforce.rs:1245-1253, :1126), and qualify_v3/v4
      count DispositionV2 exhaustively (enforce.rs:1586-1590, :1668-1672). The v5 plan path is
      unspecified.
  - file: docs/design/consumer-outside-boundary-accounting.md
    line: 282
    category: implementability-contradiction
    severity: blocker
    verdict: CONFIRMED
    origin: crates/edge/ess-xtask/src/consumer_coverage/enforce.rs:503
    message: >-
      An OutsideBoundaryPolicy parameter "in the same shape as AggregatePolicy" cannot carry a new
      disposition. AggregatePolicy (enforce.rs:503-506) only selects between claim and case
      functions (enforce.rs:514-529) that both construct DispositionV2::AggregateClosure
      (enforce.rs:651); plan_versioned's map, cell type and returned ExecutionPlanV2 are fixed
      (enforce.rs:535-742). Same root cause as the finding at line 290.
  - file: docs/design/consumer-outside-boundary-accounting.md
    line: 283
    category: implementability-contradiction
    severity: blocker
    verdict: CONFIRMED
    origin: crates/edge/ess-xtask/src/consumer_coverage/enforce.rs:904
    message: >-
      "the matching enforce::replacement_matches arm" has no valid operand without extending the
      shared enum. replacement_matches is fn(&ReplacementClaim, &DispositionV2) (enforce.rs:904-907)
      and its only call site passes a v2 cell's disposition (enforce.rs:663). Without it, all 27
      changed frozen pairs bail at enforce.rs:665-668; ReplacementClaim currently admits only
      Supported, Refused and AggregateClosure (reconciliation.rs:76-89), and the 27 are genuine
      Action::Replace decisions (reconciliation.rs:275-294).
  - file: docs/design/consumer-outside-boundary-accounting.md
    line: 205
    category: unsatisfiable-guard
    severity: blocker
    verdict: CONFIRMED
    origin: crates/edge/ess-xtask/src/consumer_coverage/reviewed-model-behavior.json
    message: >-
      G4 requires 168 downstream Supported/Refused cells with executed cases, and zero exist.
      reviewed-model-behavior.json holds 2061 claims, all at three acquisition-specification-*
      profiles; claims at any of the seven compiler profiles G4 names: 0; of the 56 in-scope models,
      0 are covered. Every G4 check fails on the first run after implementation. The controls at
      :309-310 both require "re-run green" after restoration, which is therefore unreachable, and
      neither the surfaces table (:271-285) nor the tests table (:298-314) names the work to author
      those 168 downstream cells.
  - file: docs/design/consumer-outside-boundary-accounting.md
    line: 261
    category: miscited-precedent
    severity: advisory
    verdict: CONFIRMED
    origin: crates/edge/ess-xtask/src/consumer_coverage/enforce.rs:159
    message: >-
      "the way ExecutionPlanV3 and V4 were added beside V2" is the opposite of the prescription:
      both reuse cells Vec<PlannedCellV2> (enforce.rs:165, :201) and got no cell type of their own.
      The precedent that matches is Disposition -> DispositionV2 with PlannedCell -> PlannedCellV2
      (enforce.rs:62-115). An implementor copying the named precedent reproduces the shared-enum
      reuse the same paragraph forbids.
  - file: docs/design/consumer-outside-boundary-accounting.md
    line: 288
    category: ordering-ambiguity
    severity: advisory
    verdict: CONFIRMED
    origin: crates/edge/ess-xtask/src/consumer_coverage/enforce.rs:659
    message: >-
      "after the aggregate claims and before the unaccounted sweep" spans the reconciliation
      replacement check (enforce.rs:659-670) and the stale shape check (enforce.rs:671-680). Only
      insertion before line 659 works; cells inserted after it are invisible to the replacement
      lookup at enforce.rs:661-663 and all 27 replacements bail. Name the exact point: immediately
      after enforce.rs:658.
  - file: docs/design/consumer-outside-boundary-accounting.md
    line: 326
    category: doc-model-mismatch
    severity: advisory
    verdict: CONFIRMED
    origin: ess/consumer-outside-boundary/domains/outside_boundary.yaml:165
    message: >-
      "A pair leaves Declared only through ProvePair" contradicts the model it describes:
      outside_boundary.yaml:165-167 declares invalidate from [Declared, OutsideBoundary]. The model
      is correct; the prose should follow it.
```

---

Stopping here. No repair of the proposal, no reviewer or implementor dispatch, no continuation into
implementation. Root routes what follows.
