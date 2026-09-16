---
format: aep.planning-md/1
id: story:a-skip-says-why-the-target-could-not-answer
kind: story
status: active
title: A skip says why the target could not answer
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
- serves: vision:O2
revision: 3
---

# Story: a skip says why the target could not answer

## Outcome

Somebody reading a skipped scenario learns which question the target declined and why, from the run
itself, without patching the generated runner first.

## Context

The emitted Go runner throws away the only sentence that explains a skip.

```go
// crates/generate/ess-synth/src/go/… → essconform/runtime.go:1855
if errors.Is(err, ErrUnsupported) {
    r.skip("step %d: the target does not expose `%s`", index, step.Command)
    return false
}
```

`err` is in scope and is discarded. A target that returns a carefully argued refusal — and two
adopters write exactly those — has that argument deleted before anybody sees it.

Measured 2026-09-16 on two real targets of one adopter:

- the first answered *"<command> accepted, and <command> declares 2 non-error outcomes told apart by
  a guard over the input rather than by anything the server reports"*. What the run printed was
  `the target does not expose <command>`, which reads as an unimplemented method and is not one.
- the second answered *"this target maps no surface for command <command>"*, which is a different
  fact with a different fix — and rendered as the same sentence.

Both were only readable after patching that line locally. Across 41 skips in one run, the printed
text distinguished nothing: three separate causes rendered identically.

This is not cosmetic. The runner's own contract is that a skip is an honest answer rather than a
pass, and `report.rs:563` makes every unsupported scenario fail the run. A verdict somebody is
expected to act on that deletes its own reason makes the next step guesswork.

The same pattern is at `runtime.go:2032`, `:2062` and `:2213` — views and binding invocations.

## Acceptance

- A scenario skipped because `ExecuteCommand` returned `ErrUnsupported` prints the target's error,
  not only the command name.
- The same holds for a skipped view read and a skipped binding invocation.
- The `ess-conformance-report` entry for that scenario carries the same sentence, so the document
  and the test log still cannot disagree — the invariant `skip()` exists for.
- A test asserts a target's wrapped message survives into both, because a message nobody asserts is
  a message the next refactor drops again.

## Out of Scope

The Rust runner, unless the same discard is there. Changing when a target *should* return
`ErrUnsupported`. The exit-code defect — `go test` exiting 0 over a run the report calls
`inconclusive` — which is its own story.

## Ambiguities

- `inferable` — whether the report entry already carries the cause: `failed_scenarios` holds
  `"skipped <id>"` strings only (read from a 137-scenario report, 2026-09-16), so it does not.
- `inferable` — the emitter templates that own these lines are under
  `crates/generate/ess-synth/src/go/`; the four call sites are named above by their line numbers in
  the emitted file.

## Open Questions

None.
