---
format: aep.planning-md/3
id: review-result:read-your-writes-parity-312-20261004-r1
kind: review-result
status: active
title: Generated read-your-writes parity independent review round 1
relations:
- reviews: story:feature-request-312
revision: 1
---
needs-revision

# Story 312 generated missing-token read-your-writes parity — independent whole-unit review round 1

Candidate `48fdc424f0cdee1b101fd168fb82177caef0f3e4` over base `0b98f1bb60fd28b9d9e1a75e8fa8b8f157a0a8de` has one correctness finding.

After a command returns no consistency token, both generated runners correctly suppress an immediate read-your-writes query and retain the command/view cause. A later successful `establish_entity` invalidates the prior observation: the native runner clears both `last_view` and `unreadable`, but generated Go clears only `queried`/`lastView` at `runtime.go:2461-2463`, and generated TypeScript does the same at `runtime.ts:3871-3873`. Entity-setup admission permits setup after earlier assertions and requires a later assertion, so the sequence tokenless command → suppressed query → successful entity setup → expectation of the same view is admitted. Native reports a suite Error because no post-setup view was read; generated Go and TypeScript incorrectly retain the pre-setup cause and report Failed. Clear both generated unreadable fields on successful setup and add a native/Go/TypeScript regression scenario for the transition.

The rest of the reviewed change is coherent. The six scenarios distinguish missing-token immediate expectation, missing-token snapshot, exact-token propagation, no-prior-write Current, eventual Current, and stale-view suppression. Current `/34` with typed Empty provenance and historical `/32` are both exercised. The Go and TypeScript helper bridges preserve exact tokens, and the drop/change mutations remain decisive. The callback trace distinguishes Current from exact `AtLeast(token)`.

Reviewer execution and compilation were not performed. The reviewer independently inspected the complete seven-path source diff, checked admission and native/generated state transitions, streamed and hash-verified the six archived test binaries, and audited the author's raw evidence. That evidence records the initial semantic red, final 5/5 focused pass, execution 12/12, faults 20/20, TypeScript parity 28/28, upsert 1/1, corrected Go parity 23/23, strict scoped Clippy, and formatting checks. The final `clone_into` test-parser edit changes allocation style only and was compiled by strict Clippy after runtime execution; the focused runtime binary is therefore evidence for the immediately preceding behavioral source, not a claimed execution of that lint-only final test line. Full refreshed package and browser/final-combined obligations remain outstanding as the story states.

The retained patch artifact has SHA-256 `1032adc6995166399c4cd8ec73c952c91df8ec8409f1cc90e7399e5841609b01`. A newly rendered canonical `git diff` has SHA-256 `1b675892d6cf100c9bf1b7e9e2996d99153da3ad50fb4436857165dbfd5e9fd9` because its file hunks are ordered differently. The candidate tree, seven changed paths, and every source blob match the retained source manifest; this artifact-order difference does not alter the source finding or candidate identity.

```findings
[
  {
    "file": "crates/verify/ess-conformance/src/go/runtime.go",
    "line": 2461,
    "category": "correctness",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "successful establish_entity clears queried/lastView but leaves unreadableView/unreadableCommand from a preceding tokenless read-your-writes query; the TypeScript runtime has the same omission at runtime.ts:3871, while the native runner clears both last_view and unreadable at runner.rs:2027. An admitted tokenless-command -> suppressed-query -> establish-entity -> expect-view sequence therefore reports Failed in generated Go/TypeScript but Error ('no view had been read') natively. Clear both generated unreadable markers on successful setup and add native/Go/TypeScript regression coverage for this transition."
  }
]
```
