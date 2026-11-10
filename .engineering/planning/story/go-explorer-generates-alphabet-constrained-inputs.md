---
format: aep.planning-md/3
id: story:go-explorer-generates-alphabet-constrained-inputs
kind: story
status: active
title: The Go explorer generates inputs whose type declares an alphabet
tags:
- adopter-report
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: inferred
  path: crates/verify/ess-conformance/tests/explore_constrained_inputs.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/explore-constrained-inputs.yaml
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:44:55Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-11-10T00:44:56Z", actor: "human:timo", revision: 8}
---
## Outcome

The generated Go explorer draws random sequences for a command whose input type constrains its
values with `alphabet:`, generating values the alphabet admits, instead of excluding the command.

## Evidence

An adopter on ess 0.56.0: the explorer excludes a command with "input `code_verifier` is
`demo.CodeVerifier`, which constrains its values". https://github.com/beyond10x/ess/issues/512
covers List inputs and related guards only; this is a third exclusion reason. Effect: a whole
endpoint gets no random sequences.

## Evidence (second adopter report)

A second adopter hardening OAuth specifications on ess 0.56.0: four specifications had commands
excluded for the same reason, including input types with `invariants:` (not only `alphabet:`) and
inputs that carry an `example:`. The sequential explorer excludes them without consulting
`example:`; the concurrent explorer is exempt.

## Reproduction on main (4ab2ea1449)

A read of `crates/verify/ess-conformance/src/go/explore.go` on `main` at 4ab2ea1449, in
`exploreResolveAs`:

```go
if !concurrent && (len(exploreList(body["invariants"])) > 0 || body["alphabet"] != nil) {
	return exploreKind{kind: "unsupported", why: fmt.Sprintf("`%s`, which constrains its values", name)}
}
```

(line 468), so any declared input type with `alphabet:` or `invariants:` still sends its command to
the explorer's `excluded` list with that reason, and `example:` is never read on this path. The
string draw (`exploreDraw`, `case "string"`, line 2458) picks from the input's text pool
(`exploreFieldPools`, line 736) with no check against the type. The TypeScript explorer has the
same exclusion (`crates/verify/ess-conformance/src/ts/explore.ts:423`); it is not part of this story.

## Acceptance

The fixture declares `demo.CodeVerifier`, a newtype of `String` with
`alphabet: "abcdefghijklmnopqrstuvwxyz0123456789-"`, invariants `value.utf8_bytes >= 43` and
`value.utf8_bytes <= 128`, and an `example:` that satisfies them, read as input `code_verifier`
by command `demo.Exchange`. Every bullet is a case of
`cargo test -p ess-conformance --locked --test explore_constrained_inputs`, which generates the Go
explorer for the fixture and runs it with a fixed seed.

- The sequential explorer's report has no `excluded` entry for `demo.Exchange`, and its sequences
  send `demo.Exchange` at least once.
- Every `code_verifier` value the sequential explorer sends uses only alphabet characters and is
  43 to 128 bytes long.
- The `example:` value is among the `code_verifier` values the sequential explorer sends.
- The concurrent explorer's report has no `excluded` entry for `demo.Exchange` either, so both
  explorers explore the same commands of the fixture.
- A second type `demo.Impossible` with the same alphabet and invariants `value.utf8_bytes >= 2` and
  `value.utf8_bytes <= 1` still excludes its command from the sequential explorer, with an
  `excluded` reason naming `demo.Impossible` and `value.utf8_bytes >= 2`.

## Scope

- cited: crates/verify/ess-conformance/src/go/explore.go (`exploreResolveAs`, line 468; `exploreFieldPools`, line 736; `exploreDraw`, line 2428)
- inferred: crates/verify/ess-conformance/tests/explore_constrained_inputs.rs (new, shaped like the Go lane of `tests/explore.rs`)
- inferred: crates/verify/ess-conformance/tests/fixtures/explore-constrained-inputs.yaml (new)
confidence: medium

Shared surfaces moved: proposes its `CHANGELOG.md` sentence; no format row, no byte-pin row.
