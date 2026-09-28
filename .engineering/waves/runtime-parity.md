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
| go | `src/go/runtime.go`, `replay.go`, `predicate.go`, `reading.go`, `response.go`; Go parity tests | `ess-rp-go` | `impl/runtime-parity-go` | `~/.cache/b10x-target/ess-rp-go` | `~/.cache/ess-wave-rp/go` | dispatched |
| ts | `src/ts/runtime.ts`, `runtime.test.ts`, `predicate.ts`, `reading.ts`, `response.ts`, `presence.test.ts`; TS parity tests | `ess-rp-ts` | `impl/runtime-parity-ts` | `~/.cache/b10x-target/ess-rp-ts` | `~/.cache/ess-wave-rp/ts` | dispatched |

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
