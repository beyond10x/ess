---
format: aep.planning-md/1
id: review-result:priority-accessor-design-pass2-20260911
kind: review-result
status: active
title: Bounded accessor second design review
relations:
- reviews: story:binding-mapping-bounded-accessor
revision: 1
---
unit: bounded accessor design at 59afcf8caec5230a88aadfd7c590a703e0b5500b plus untracked design SHA256 d950020e8d3f778f221fb1ff83a13aeb9c933c1de444b2b69e778ab2db7875f0
verdict: NEEDS-CHANGE (final full design attack; one bounded correction remains)
cases: executed 0→0, red 0 (design-only; no implementation or tests executed)
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 assigned report, identified by local-evidence alias below
needs-coordinator: settle nested Optional observation semantics locally; retain implementation and four-row adoption obligations
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 828a21bde43839f39caebb71ee24adb6ba386c4e6fd0997fb447366199c4424b, retained as local-evidence:runtime-gaps/publication-replay/snapshots/828a21bde43839f39caebb71ee24adb6ba386c4e6fd0997fb447366199c4424b.md. Source creation recorded at 2026-09-11T00:06:06Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 b97fb592ec712d6bca57a9dc9ba4b78be232a898a1a9d77813595e668dd6f2a8, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/b97fb592ec712d6bca57a9dc9ba4b78be232a898a1a9d77813595e668dd6f2a8-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

1. `git --no-pager diff --stat`

```text
(empty output)
```

`git status --short`:

```text
?? docs/design/binding-mapping-bounded-accessor.md
```

The untracked design is the assigned input, unchanged by this reviewer. Its SHA256 matches the assignment. No source, design, test, planning, or Git-state writes were made. Only the reviewer's own worktree lease was acquired and released through the lifecycle CLI.

2. Cases added

None. The counterexample is a static evaluation of the proposed contract and existing native emitter; it is not represented as an executed runtime failure.

3. Suite run

None, under the design-only charter. No builds, tests, or gates were run.

4. Judgement findings

| Source | Verdict / origin / severity | Finding |
| --- | --- | --- |
| `docs/design/binding-mapping-bounded-accessor.md:297` | NEEDS-CHANGE / introduced / blocker | Computing conformance assignment from a decoded terminal Optional loses an observable native Some(None) state and can demand an omitted input where the specified identity mapping produces present-null. |

**What was measured:** source inspection and contract evaluation only. The correction now defines native assignment precisely: equal source/target types use Identity (`:184–192`), and whole terminal values are preserved (`:120–125`). It also explicitly acknowledges that the existing decoder collapses a null Optional member to outer None (`:207–214`). Yet the generic projection rule still turns terminal JSON null into `Projected(None)` (`:165–170`), and the conformance plan computes the expected target member from that assignment (`:295–298`). Those rules disagree for an ordinary representable native source value.

Consider `event.data.status`, where the terminal field and command input are both `Optional<Optional<String>>`, and a native event contains `status = Some(None)`. There is no Optional traversal before the terminal field and no conversion. The designed native mapping uses Identity, so the command receives `Some(None)` and its wire member is present with null. The event's own observation also carries `status: null`. Following the proposed terminal-JSON rule reconstructs outer None, then Identity predicts an omitted command member. That prediction rejects the native transformation the design expressly requires. This is not repaired by saying hidden wrapper states cannot be recovered: the resulting target presence difference is observable and is asserted by the new conformance contract.

**What reaches it:** generated transformations accept an actual typed event reference, with no mandatory decode/encode normalization (`crates/generate/ess-synth/src/rust/system.rs:314–336`). Native structs expose their declared fields (`crates/generate/ess-synth/src/rust/items.rs:61–86`), so a host can supply nested Optional states. The existing wire encoder omits outer None but encodes inner None as null (`crates/generate/ess-synth/src/rust/wire.rs:476–526`); its decoder collapses missing/null at an Optional field (`:607–613`). Conformance receives only a Node payload map and Node invocation input map (`crates/verify/ess-conformance/src/target.rs:497–502`, `:737–744`), not the original typed wrapper stack. The proposed two-segment accessor adds a path to that existing reachable representation; no runtime implementation of the accessor was executed.

**Required outcome:** define a conformance observation/assignment rule that agrees with the specified native mapping for terminal nested Optional sources, without recovering typed information the observation does not contain. Merely covering nested targets from a single-Optional source leaves this source-side case open.

**Bounded repair direction:** keep the typed native assignment. Define wire-observable projection separately, retaining terminal member presence and raw observable null/value where that suffices to determine the target's wire result. For cases where different admitted source interpretations yield different target observations, require an explicit capability refusal or additional authoritative observation contract; do not fabricate one expected value or make the target under test supply its own oracle. Alternatively establish and enforce a common normalization boundary before native binding execution, but that would be a semantic choice requiring explicit treatment of the existing whole-value promise. Add terminal nested-Optional source cases with equal and deeper Optional targets, preserving the distinction between native proof and wire-observation proof. This is a bounded correction of the Optional class, not a request for another full design-review cycle.

5. Full design review outcome and resolved findings

- **First finding resolved at the design level:** shared DAG nodes and stable references survive serialization and native helper generation; the resource table bounds depth, nodes, edges, escaped bytes, aggregate accounts, and output. Completed-node depth checking defeats memoization shortcuts. The specified diamond, chain, width, repetition, boundary, and expansion-mutation cases address the whole failure class. They remain unexecuted implementation obligations.
- **Second finding resolved at the design level:** the explicit suite/report routing table preserves report/1 and uses existing report/2 for suites 6/7. Both CLI and Go refuse incompatible requests before target creation; exact suite bytes, unknown ordinary coverage, coverage lineage, filtered parent versions, release-evidence input, and browser carriers are accounted for. Current `CountReport` exact-input checks support this choice (`crates/verify/ess-conformance/src/counts.rs:238–260`), and current Go configuration/admission provides the stated pre-target boundary (`crates/verify/ess-conformance/src/go/runtime.go:1498–1523`). No report/3 or weakened report/1 contract is needed.
- **Third finding partly resolved:** the assignment algebra now correctly states every target wrapper and conversion call boundary. The remaining finding above is the unresolved interaction with source observation, not a request to undo that native assignment rule.
- Source versions 1/2 and flat EventField/Observed bytes remain protected; ess/3 and suite 6/7 have specific new-capability boundaries, with exact previous-reader probes required. No ess-ir/2 or duplicate general semantic engine is introduced.
- Bounded syntax, declared names versus wire names, required/Optional traversal, union branch absence versus malformed data, nominal newtypes, whole collection leaves, exact conversion permissions, and opaque-conversion coverage refusal were read in full. No further design finding was established in those areas.
- The four adopter rows remain open pending authenticated typed context, actual host conversions, and verification against a published release. The design does not invent payload fields or imply accessor syntax alone completes those rows. Crosswalk remains excluded.
- Design review cannot prove implementation, generated Rust/Go behavior, conformance agreement, format compatibility, actual adoption, merge checks, or release artifacts. Those explicitly retained obligations require their own evidence. This report makes no implementation or release approval claim.

6. Outside-worktree output

Only `local-evidence:ess-evolution-20260910/priority-wave/accessor/design-adversary-pass2-report.md`, the exact assigned report location. The alias omits the private local home-directory prefix. No other scratch files, logs, tests, or build outputs were written. The coordinator retains worktree lifecycle ownership after the reviewer's own lease ends.

7. Machine-readable findings

```findings
- file: docs/design/binding-mapping-bounded-accessor.md
  line: 297
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Computing conformance assignment from a decoded terminal Optional loses an observable native Some(None) state and can demand an omitted input where the specified identity mapping produces present-null.
```