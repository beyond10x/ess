---
format: aep.planning-md/1
id: review-result:priority-accessor-design-pass1-20260911
kind: review-result
status: active
title: 'Bounded accessor design review: expansion and report compatibility'
relations:
- reviews: story:binding-mapping-bounded-accessor
revision: 1
---
unit: bounded accessor design at 59afcf8caec5230a88aadfd7c590a703e0b5500b plus untracked design SHA256 9af5c2f3de5f991e0579e0f13d873bafef33ddde8b8449a3d3a267b21ab04ace
verdict: NEEDS-CHANGE (design review only)
cases: executed 0→0, red 0 (no implementation or tests executed)
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 assigned report, identified by local-evidence alias below
needs-coordinator: resolve two design blockers and one Optional clarification; implementation and four-row adoption remain unverified
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 574340783ca69038357215b751527f6a32a3075a6d66f237a3fe934365263056, retained as local-evidence:runtime-gaps/publication-replay/snapshots/574340783ca69038357215b751527f6a32a3075a6d66f237a3fe934365263056.md. Source creation recorded at 2026-09-10T23:54:24Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 20f18d83063afef9bcfdc0a3cfc4d2c7c3de61654f8a76f99244e7cbc9b95f03, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/20f18d83063afef9bcfdc0a3cfc4d2c7c3de61654f8a76f99244e7cbc9b95f03-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

1. `git --no-pager diff --stat`

```text
(empty output)
```

`git status --short`:

```text
?? docs/design/binding-mapping-bounded-accessor.md
```

The untracked design was the assigned input, not a reviewer edit. No source, design, planning, or test file was changed. The report is a read-only design review under the coordinator's specific charter; the general adversary's runtime-test procedure was not performed or represented as completed.

2. Cases added

None. The counterexamples below are static design examples, not executed failing cases. There is no runtime green verdict.

3. Suite run

None, as required by the design-only charter. No builds, test reruns, or expensive gates were run.

4. Judgement findings

| Source | Verdict / origin / severity | Finding |
| --- | --- | --- |
| `docs/design/binding-mapping-bounded-accessor.md:61` | NEEDS-CHANGE / introduced / blocker | The path-depth limit and active-stack cycle check do not bound acyclic union expansion or transparent-chain depth, so the proposed closed plan can grow exponentially or recurse arbitrarily deeply for a short valid accessor. |
| `docs/design/binding-mapping-bounded-accessor.md:214` | NEEDS-CHANGE / introduced / blocker | The suite 6/7 migration omits a compatible report-production contract, although the ordinary CLI report path and report/1 reader currently require suite majors 1–4. |
| `docs/design/binding-mapping-bounded-accessor.md:123` | NEEDS-CHANGE / introduced / warning | The instruction to lift a projected value once does not define the representation for nested Optional targets that the explicitly reused assignability rule accepts. |

**Finding 1 — finite source graph is not a bounded expanded plan.**

What was measured: source inspection only. The design bounds author-written segments at three (`:22–32`), refuses active-stack repetition (`:61–64`), resolves every union branch (`:81–93`), and serializes ordered branch plans (`:168–172`). None bounds expansion after a completed branch leaves the active stack. A family `U0 = union {a: U1, b: U1}`, through `Un = struct {status: String}`, uses O(n) declarations and the two-segment path `event.data.status`, but a recursively embedded branch plan has O(2^n) leaves. Memoizing resolution without sharing the persisted/emitted plan still duplicates those leaves. A long acyclic newtype chain also never repeats an active pair and consumes no further authored segment.

What reaches it: union payloads are arbitrary type references in `crates/specify/ess-domain/src/types.rs:408–414`; distinct variants may reference the same declared type. The inhabitation check accepts finite constructible unions and newtypes (`crates/specify/ess-domain/src/system.rs:546–559`). `MAX_TYPE_DEPTH` bounds inline generic nesting, not the number of named declarations (`crates/specify/ess-domain/src/types.rs:141–154`). This is a family admitted by the existing type model, not a claim that a new accessor implementation has been run.

Required outcome: bound both construction and every emitted/serialized representation; a short path must not imply unbounded transparent recursion or exponential output. Proposed repair: memoize type/suffix resolution and retain shared plan nodes with deterministic references, or impose an explicit work/node/depth budget with deterministic unsupported-accessor refusal before expansion. Sharing alone must also bound traversal stack depth. Include an acyclic repeated-diamond example and a long transparent chain in the implementation's negative/resource tests. The design owns choosing the precise representation or budget.

**Finding 2 — suite admission is not the entire version migration.**

What was measured: source inspection only. `ConformanceReport::standalone` asserts majors 1–4 (`crates/verify/ess-conformance/src/evidence.rs:138–142`); the report/1 reader independently rejects other suite versions (`:79–89`). The CLI calls that method for `--report-out` on its ordinary report path (`crates/edge/ess-cli/src/main.rs:2868–2871`). Its current pre-execution report guard only catches coverage-bearing suites (`crates/edge/ess-cli/src/coverage.rs:145–154`), so merely admitting ordinary suite/6 would leave the new run on a path whose report contract cannot represent it. The Go ordinary writer labels its output report/1 and copies the suite version (`crates/verify/ess-conformance/src/go/runtime.go:1631–1670`), which would likewise fail the current closed report reader. No proposed implementation was executed, so these are uncovered migration obligations, not an observed panic in new code.

What reaches it: the design explicitly emits ordinary suite/6 and promises both runners (`:203–218`, `:341–343`); ordinary CLI report output and generated Go `ESS_REPORT_OUT` are existing public paths. The consumer table lists suite readers, runners, and generic CLI inspection, but makes no decision about report/1, report/2, defaults, refusal timing, or old report readers.

Required outcome: decide how ordinary suite/6 and coverage suite/7 runs publish readable, accurately versioned evidence, with refusal before execution when a requested report mode cannot represent the run. Proposed repair: explicitly route supported new runs to an appropriate existing report contract if its semantics suffice, or justify a coordinated report extension; do not reflexively create another report version. Preserve legacy report bytes and check exact old-reader behavior. Include Rust CLI report output and generated Go report output in the migration matrix. CountReport already works from admitted exact suites (`crates/verify/ess-conformance/src/counts.rs:149–215`); this finding does not claim report/2 is intrinsically limited to suite/5. Also audit the suite/5-specific parent-reference check at `crates/verify/ess-conformance/src/coverage.rs:457–466` when implementing selection; the design already requires that broader coverage migration.

**Finding 3 — target wrapping is separate from traversal flattening.**

What was measured: source inspection only. The design says to lift once (`:123–124`) while retaining exact terminal types and applying the existing assignability function (`:104–109`, `:144–149`). That function recursively peels target Optionals (`crates/specify/ess-domain/src/types.rs:969–976`): both `String -> Optional<Optional<String>>` and `Optional<String> -> Optional<Optional<String>>` are assignable. One unconditional lift does not describe both cases, especially a terminal empty Optional versus traversal unavailability. Existing synthesis already records additional-wrapper refusal in `crates/generate/ess-synth/tests/feasibility.rs:612–618`, backed by its representation guard at `crates/generate/ess-synth/src/rust/feasibility.rs:738–750`; that old flat limitation is not introduced by this unit.

What reaches it: nested Optional is valid source syntax, and the new accessor explicitly preserves its terminal type while reusing this assignment rule. The new design's representation promise must define its treatment; it cannot assume the target has exactly one Optional layer.

Required outcome: distinguish path-presence flattening from the number of wrappers required to inhabit the declared target type. Proposed repair: derive an explicit assignment/lift plan from the source and target types, or explicitly preserve a synthesis refusal for unsupported wrapper shapes without claiming them implemented. Include required and Optional leaves into nested Optional targets, with unavailable traversal and terminal empty values tested separately. Do not alter old flat semantics merely to settle this new construct.

5. Attacks without additional findings

- Source versioning has a concrete boundary: current supported formats are exactly 1 and 2 (`crates/specify/ess-domain/src/system.rs:53–58`), so adding the accessor under ess/3 can preserve old successful documents and let exact old compiler admission refuse it. This still needs the promised previous-reader probe; serde parsing of a version number alone is not admission. Binary64 is separately refused only under ess/1 (`crates/specify/ess-domain/src/primitive_admission.rs:15–20`), so ess/3 need not erase ess/2's numeric capability or relax synthesis/conformance Binary64 refusals.
- Separate suite/6 ordinary and suite/7 coverage are justified by the existing exact coverage-envelope equivalence (`crates/verify/ess-conformance/src/admission.rs:177–190`). New ObservedAccessor vocabulary deserves new admission; another ess-ir envelope is not justified. Report routing remains the missing part identified above.
- The design preserves legacy EventField/Observed variants, distinguishes terminal Optional copying from traversal absence, refuses heterogeneous union leaves and list/map traversal, and specifies malformed union input as error. No additional contradiction was established in those rules by this read-only pass.
- The four real adopter rows remain explicitly open pending typed authenticated context and verified conversions (`docs/design/binding-mapping-bounded-accessor.md:279–325`). This preserves the story's acceptance obligation; an accessor-only release would not prove all four rows complete. This review did not independently re-audit private adopter source.
- Compile-time shared semantics, wire/source-name separation, conversion refusal, absence assertions, generated Rust/Go execution, compatibility probes, mutation checks, and feature-preservation accounts are implementation obligations, all unexecuted here. Silence is not a correctness or release verdict.

6. Outside-worktree output

Only `local-evidence:ess-evolution-20260910/priority-wave/accessor/design-adversary-report.md`, the exact report location assigned in the coordinator's private brief. This public-safe alias intentionally replaces the local home-directory prefix. No additional scratch, logs, build outputs, or private-source inventory were written. The reviewer's own worktree lease is released on returning the report; the coordinator retains lifecycle ownership.

7. Machine-readable findings

```findings
- file: docs/design/binding-mapping-bounded-accessor.md
  line: 61
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The path-depth limit and active-stack cycle check do not bound acyclic union expansion or transparent-chain depth, so the proposed closed plan can grow exponentially or recurse arbitrarily deeply for a short valid accessor.
- file: docs/design/binding-mapping-bounded-accessor.md
  line: 214
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The suite 6/7 migration omits a compatible report-production contract, although the ordinary CLI report path and report/1 reader currently require suite majors 1–4.
- file: docs/design/binding-mapping-bounded-accessor.md
  line: 123
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The instruction to lift a projected value once does not define the representation for nested Optional targets that the explicitly reused assignability rule accepts.
```