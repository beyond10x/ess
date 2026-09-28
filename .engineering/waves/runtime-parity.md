# runtime-parity

beyond10x/ess#188 (operator, 2026-09-28): the Go and TypeScript runtimes that 0.38.0 generates
admit only `ess-conformance/2`–`/21`, while the same release synthesizes `/22`–`/27`. A downstream
adopter (session `bc-impl`) holds its ESS pin for the fix and asked for a release. One story,
`story:generated-runtimes-run-every-emitted-suite-version`, two units split by language. Runs beside
wave `correctness-1` (no shared files) and the concurrent-history branch (which edits `go/mod.rs`,
`ts/mod.rs`, `go/explore.go`, `ts/explore.ts` — not assigned here).

Skill: aep 0.16.0 (`aep:implementing`, wave mode). Dispatch types: `aep:implementor`,
`aep:adversary`. N = 2.

## Commits this wave makes

One opening store commit (story, scope, this page), one commit per unit plus corrections, the unit
merges into `integrate/runtime-parity`, one closing store commit, the bot-published branch, one PR
to `main` and its bot merge once CI is green. The release that follows is the operator's standing
rule ("ready means ship") and the request in #188; it follows `AGENTS.md` and is not part of the
wave.

## Integration

| | |
|---|---|
| branch | `integrate/runtime-parity` |
| base | `4626606d4` (origin/main after #187) |
| worktree | managed `ess-wave-rp` |
| build dir | `~/.cache/b10x-target/ess-rp-int` |
| scratch | `~/.cache/ess-wave-rp/int` |

## Units

| unit | files | worktree (managed id) | branch | build dir | scratch | stage |
|---|---|---|---|---|---|---|
| go | `src/go/runtime.go`, `replay.go`, `predicate.go`, `reading.go`, `response.go`; Go parity tests | `ess-rp-go` | `impl/runtime-parity-go` | `~/.cache/b10x-target/ess-rp-go` | `~/.cache/ess-wave-rp/go` | correction 1 |
| ts | `src/ts/runtime.ts`, `runtime.test.ts`, `predicate.ts`, `reading.ts`, `response.ts`, `presence.test.ts`; TS parity tests | `ess-rp-ts` | `impl/runtime-parity-ts` | `~/.cache/b10x-target/ess-rp-ts` | `~/.cache/ess-wave-rp/ts` | adversary 2 pending (head cde14a7bb) |

The admission guard test (every version synthesize can write is admitted by both runtimes) is the
go unit's, as a Rust test under `crates/verify/ess-conformance/tests/`.

## Pre-flight (2026-09-28)

| check | value |
|---|---|
| free disk `/` | 97G (after the concurrent-history session removed two build dirs) |
| compiler cache | sccache; `RUSTC_WRAPPER=sccache` per unit |
| concurrent agents | 3 (wave correctness-1) + 2 here |

## Decisions taken by the coordinator

- Reverses the-5-waves decision that the generated runtimes refuse `/22`+ by version (#188).
- Priority inside each unit: `/26` dotted-leaf expectations first, then the rest of `/26`, `/27`,
  then `/22`–`/25`. A construct a runtime cannot execute refuses that scenario with a named reason,
  never the suite.

## Log
- 13:30 operator: one shared integration branch. integrate/runtime-parity merged into integrate/ess-next (2e0960e7f); the go and ts units merge into integrate/ess-next.
- 12:55 main moved: PR #189 (concurrent-history) merged as aa82c4de2; its release comes next. This wave merges main before its PR.
- 13:09 go: implementor green (ess-conformance 1158→1184; TS half of runtime_suite_admission red until ts lands), committed 7b21e59d1; adversary pass 1 dispatched. Follow-ups: ~/.cache/ess-wave-rp/go/stale-docs.md (20 lines saying Go/TS refuse /22–/27), changelog.md; older-vocabulary parity gaps (expect_error fields, holds on list/map/union, eventually_event payload, response wire/display/summary) → story at close.
- 13:12 ts: implementor green once its two coordinator patches applied (fixtures.test.ts, aggregate_views_mutants.rs flip pinned refusals to admission); coordinator applied them, typescript_runtime 4/4, typescript_suite_versions 29/29, aggregate_views_mutants 3/3; committed 810ba9ae3; adversary pass 1 dispatched.
- 13:20 ts: adversary pass 1 red (5 findings: undefined-as-present blocker, exponent decimals refuse a whole suite, big integer literals, vocabulary guard too loose); recorded review-result:adversary-rp-ts-pass-1; correction 1 to the same implementor (undecided decided in scope: undefined = absent, literals exact).
- 13:21 go: adversary pass 1 red (F1 blocker: eventuallyEvent ignores payload/shape — reachable from every synthesized binding flow/delivery at /26; F2 event null leaf parity), recorded review-result:adversary-rp-go-pass-1; correction 1 to the same implementor.
- 13:33 From now on every store write for runtime-parity goes to the shared tree (ess-wave-c1); integrate/runtime-parity is frozen.
