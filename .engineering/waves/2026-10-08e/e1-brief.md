# Unit brief e1, wave 2026-10-08e

## Identity

```
story:  story:response-string-newtype-constraints-checked-not-refused   (https://github.com/beyond10x/ess/issues/499)
branch: unit/ess-wave-20261008e-e1   forked from integrate/ess-wave-20261008e
```

Read the story: `aep plan artifact show story:response-string-newtype-constraints-checked-not-refused`
(from the integration tree `~/.local/state/worktree/trees/b10x/ess/ess-wave-20261008e`). Its
`## Decisions`, `## Acceptance` and `## Scope` sections are the contract. Reproducers:
`.engineering/repro/499/`.

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261008e-e1
build dir: <worktree>/target
scratch:   the TMPDIR named in the go message (outside every Git checkout)
```

## The file assignment

| | |
|---|---|
| **yours** | the files the story's `## Scope` lists under `crates/verify/ess-conformance/src/` (typed_fields.rs, direct_response.rs, response.rs, one_time_response.rs, runner.rs, runner/disclosure.rs, scenario.rs, admission.rs, go/**, ts/**, web_replay.rs), new test files under `crates/verify/ess-conformance/tests/`, `docs/design/typed-response-outcome-payloads.md`, the format tables a new suite format must be registered in (find them by where `ess-conformance/44` and `/45` are named; `cargo xtask docs` regenerates generated pages), and the diagnostics catalogue if a refusal code changes |
| **not yours** | everything else. `CHANGELOG.md` and `.engineering/**` are the coordinator's. `crates/verify/ess-conformance/src/synthesize.rs` and `src/synthesize/**` are held by another session: if the misleading ESS-SYNTH-001 help line lives there, drop that part and say so |

Invariants: `.engineering/waves/2026-10-08e/invariants.md` in the integration tree. Read it first.

Decided by the coordinator:
- The suite format pair is `ess-conformance/46` (ordinary) and `/47` (coverage), emitted only when a
  reachable response type carries a String-newtype constraint. Every suite that synthesizes today
  keeps its bytes and format; the existing snapshot tests must stay unchanged.
- Reuse `one_time_response::StringConstraints`; no second carrier, no observer-side predicate list.
- A predicate that does not decide over a lone `value` text fact is refused at synthesis by name.

Spec first: model the change in this repository's ESS specification, validate it with the
newest `ess`, regenerate, then implement against the generated code. If the specification cannot
express it, stop and report that; do not hand-write a parallel model.

(No authored-language change is expected: the story amends the design page first, then the suite
format. Say in the report whether `models/` declares anything this touches.)

Reproduce first on the installed `ess` (`ess verify conform synthesize` on
`.engineering/repro/499/catalog.yaml`, quote the ESS-SYNTH-001 line). Then the red test run
(quote it), implement, green run, `cargo fmt -p ess-conformance`, then
`CARGO_INCREMENTAL=0 cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`.

**Build slot.** Every `cargo test -p ess-conformance` run waits for the coordinator's go: send a
message naming the `--test` targets you want to run and wait. Run one `--test` at a time under
`timeout 1800`, `CARGO_INCREMENTAL=0`, and stop below 45G free on `/`. `cargo check` and clippy need
no go.

## Report header

```
unit: story:response-string-newtype-constraints-checked-not-refused
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: https://github.com/beyond10x/ess/issues/499
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
changelog: <the [Unreleased] lines you would write>
```
