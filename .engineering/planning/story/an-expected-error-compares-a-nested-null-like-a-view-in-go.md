---
format: aep.planning-md/3
id: story:an-expected-error-compares-a-nested-null-like-a-view-in-go
kind: story
status: rejected
title: An expected error compares a nested null member like a view assertion in the generated Go runner
summary: Go's expectError uses exact equal through payloadCarries; Rust and TypeScript treat a nested absent member and null as one value (row 48 follow-up, 0.70.0).
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T00:15:33Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
- {from: "proposed", to: "rejected", at: "2026-10-23T00:15:33Z", actor: "human:timo", revision: 4, correlation: "ess-0690"}
---
## Need

Left open by `story:a-null-member-matches-an-absent-optional-in-a-nested-assertion` (row 48,
0.69.0). That story makes an absent struct member and an explicit `null` one value at any depth in
view assertions, in the Rust runner (`same_value` in `matches` and `mismatch`), the Go runner
(`sameValue` in `matches`) and the TypeScript runner (`sameValue` in `matches`).

The generated Go runner still differs on error payloads: `expectError` (`go/runtime.go`, about line
2617) checks fields through `payloadCarries`, which uses the exact `equal`. The Rust error check
reaches `same_value` through `mismatch`, and TypeScript uses `matches`. So an error-payload field
written `null` inside a nested struct passes in Rust and TypeScript and fails in Go when the
target omits it. The difference predates row 48 for dotted `null` leaves.

`payloadCarries` is shared with the event-payload checks, which keep an exact `null` on purpose.

## Acceptance

- An expected error whose payload holds a nested struct member written `null` gets the same
  verdict from the Rust, Go and TypeScript runners against a target that omits the member, and
  against one that carries `null`; a test replays one authored suite through all three.
- Event-payload checks keep requiring an exact `null`; a test asserts it in Go.
- No format change.

## Decision

Folded into `story:a-null-member-matches-an-absent-optional-in-a-nested-assertion` (unit U28, 0.69.0) after its adversary pass: expected errors follow the view rule in all three runners there, so this story is not needed.
