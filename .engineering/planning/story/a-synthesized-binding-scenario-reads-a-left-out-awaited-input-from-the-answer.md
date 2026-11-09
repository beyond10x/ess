---
format: aep.planning-md/3
id: story:a-synthesized-binding-scenario-reads-a-left-out-awaited-input-from-the-answer
kind: story
status: draft
title: A synthesized binding scenario reads a left-out awaited input from the answer
summary: The binding scenario for an awaited Optional input passes against a target that leaves it out, without a hand edit.
tags:
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

Found while landing `story:an-optional-input-a-system-command-awaits-is-compared-only-when-issued`
(0.69.0), and still present on 0.92.0. An awaited system command copies its `Optional` input into
the row and publishes it in an event, and a binding relays the event field. For that command, the
binding scenario synthesis generates reads the input from the observed event (`Finished`), not from
the awaited answer. When the target leaves the input out, which the specification allows, the
scenario errors with ESS-CF-SUITE "`jobs.job.Finished` carried no field `note`" instead of comparing
the row as empty. The unit's test rewrote that one value to read the answer, so that the invocation
path is exercised (`crates/verify/ess-conformance/tests/system_issued_interpreter.rs`,
`suite_keeping`, which still does that hand edit).

Minimal reproduction: the `relaying_model()` of that test file, as YAML. `Finish(job_id, note:
Optional<Note>)` is held by a `system: true` actor, `sets: {note: input.note}`, and publishes
`Finished {job_id, note: input.note}`. The binding is `archive-on-finished`
(`when: {event: Finished}`, `invoke: {command: Archive}`, `mapping: {job_id: event.job_id, note:
event.note}`). On 0.92.0:

```text
$ ess verify conform synthesize --path relaying.yaml --out suite.json
11 scenario(s) (0 authored), 1 refusal(s), written to suite.json
$ ess verify conform run --target interpreted --path relaying.yaml --suite suite.json --report-format 2
jobs v1 against interpreted 0.92.0 — error
  error archive-on-finished/binding/mapping
      ESS-CF-SUITE: `jobs.job.Finished` carried no field `note`
  11 scenarios: 10 passed, 0 failed, 1 error, 0 unsupported
```

Cause, from reading: the system-issued rewrite pass turns an invocation value into
`{kind: awaited}` only where it equals a literal synthesis invented for the await
(`crates/verify/ess-conformance/src/synthesize/system_issued.rs:625`, `relay`). A value relayed
through an event field stays `{kind: observed, event: Finished, field: note}`.

## Acceptance

- The synthesized binding scenario for that fixture passes against a target that leaves the
  `Optional` input out and against one that issues it, with no hand edit of the suite. The
  reproduction above reports `11 scenarios: 10 passed, 0 failed, 0 error` (the refusal stays), and
  `suite_keeping` in `system_issued_interpreter.rs` no longer rewrites the value.
- A target that publishes a value it never issued still fails (`Lie::InvokedNote`,
  `Lie::PublishedNote`).
- The Rust, Go and TypeScript runners give the same verdicts; one test replays the suite through all
  three.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/system_issued.rs
- cited: crates/verify/ess-conformance/tests/system_issued_interpreter.rs
- inferred: crates/verify/ess-conformance/src/runner.rs
- inferred: crates/verify/ess-conformance/src/go/runtime.go
- inferred: crates/verify/ess-conformance/src/ts/runtime.ts
- inferred: CHANGELOG.md

confidence: medium
