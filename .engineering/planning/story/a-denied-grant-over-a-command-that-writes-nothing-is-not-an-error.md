---
format: aep.planning-md/3
id: story:a-denied-grant-over-a-command-that-writes-nothing-is-not-an-error
kind: story
status: implemented
title: A denied grant over a command that writes nothing is not an error
summary: 'Adopter report S7: grant/denied over a command that writes nothing errors (ess#655).'
refs:
- provider: github
  reference: beyond10x/ess#655
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:53Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:53Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-26T13:37:59Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Problem

For a command with `accepts: nothing` beside a `read_your_writes` view, the synthesized `grant/denied` scenario emits `expect_view_unchanged` with no consistent query before it, and the run reports `ESS-CF-SUITE` errors (3 of 169 in an adopter's model). Adopter report S7, ess 0.68.0. Recheck against the merged denied-grant read-token fix (commit 9bd955683) before starting.

## Acceptance

Each line names a case in an existing test binary. The cases pass on main as written. A mutant that
lets a refused send overwrite `Run::written` (the 0.68.0 rule) turns each one red, and that is
checked once by hand before merging.

- `crates/verify/ess-conformance/tests/grant_denied.rs`:
  `a_denied_grant_over_a_command_writing_nothing_passes`: a model with `Check` (`accepts: nothing`)
  denied to one actor, beside a `read_your_writes` view. Its `Check/grant/denied` holds no
  `execute_command` before the refusal, reads the view before and after it, and passes on the
  interpreted target. The whole run reports 0 errors.
- `crates/verify/ess-conformance/tests/grant_denied.rs`:
  `a_denied_grant_over_a_command_writing_nothing_reads_alike_in_go_and_typescript`: the same
  scenario through `support_go::assert_parity` and the TypeScript package gives `passed` in both,
  and `failed` against `WritesOnRefusal::new()`.
- `crates/verify/ess-conformance/tests/grant_denied.rs`:
  `every_denied_scenario_queries_before_it_compares`: in every `/grant/denied` scenario synthesized
  from the `desk()`, `JOBS` and nothing-writing models, each `snapshot_view` and
  `expect_view_unchanged` is preceded by a `query_view` of the same view.

## Fit review

Checked 2026-10-26 against main `a3bc2df3d` (0.69.2) with a minimal reproduction: one domain
reached over the network, view `Notes` at `read_your_writes`, `Check` with `accepts: nothing`
granted to one of two actors (scratch `fit/repro-s7/`, outside the repository).

1. **Need.** A caller denied a command that writes nothing must be refused, and the suite must say
   so without erroring. Requester's syntax: none; the issue (`issue-S7.md`) asks for "the refusal
   alone (or reads a view the scenario arranges first)".
2. **Class.** Defect: the suite errored on a well-formed model (`ESS-CF-SUITE: no consistent view
   query preceded \`expect_view_unchanged\``, `runner.rs:2981`).
3. **Already fixed?** Yes. `c1fdc2af8` (merged `9bd955683`, shipped 0.69.0): a send refused as not
   granted keeps the earlier read token (`runner.rs:1028`, `Run::written` at `runner.rs:3202`).
   `ess verify conform run` on the repro: 0.68.0 gives `4 scenarios: 3 passed, 0 failed, 1 error`;
   0.69.0 and 0.69.2 give `4 passed, 0 failed, 0 error`. Both releases synthesize the same suite
   bytes (`diff` empty), so the fix is in the runner alone.
4. **Fit.** No grid cell moves. The denied scenario already reads the view before and after
   (`query_view`, `snapshot_view`, send, `expect_not_granted`, `query_view`,
   `expect_view_unchanged`, observed in the 0.69.2 suite); synthesis (`synthesize/grant.rs:47`)
   needs no change.
5. **Second adopter.** A `Ping`/`Validate` command that only answers, beside any
   `read_your_writes` view, denied to an auditor role.
6. **Cost.** None in the language. One regression case per runner. Gap: every pinned case writes
   before the refusal (`tests/grant_denied.rs` asserts "a write precedes the refused send" in
   `denied_get_job_suite`; `tests/adversary_u9b_pass1.rs` fixtures all start with `OpenJob`). The
   no-write-in-scenario shape, which is this report, is not pinned.
7. **Alternatives.** (a) Archive as a duplicate of
   `story:a-refused-send-keeps-the-earlier-read-token-for-a-denied-grant`: it loses the uncovered
   shape. (b) The requester's "refusal alone": it drops the check that a denied send wrote nothing.
   (c) Pin the shape as a regression in the three runners (chosen).

## Decisions

- Accept, redesigned: the defect is already fixed on main by `c1fdc2af8` (0.69.0). No synthesis
  change and no runner change. The story shrinks to pinning the `accepts: nothing` shape, with no
  write before the refusal, in the Rust, Go and TypeScript runners.
- The requester's "assert the refusal alone" is refused: the before-and-after read is what fails a
  target that writes on a refused send (`a_refused_send_that_writes_fails_the_denied_snapshot`).
- Reply to the requester: fixed in 0.69.0; upgrade; the 3 errors go to passes.
- No source, suite or IR format moves.

## Scope

| unit | files | confidence |
|---|---|---|
| fix (done) | `crates/verify/ess-conformance/src/runner.rs` (`:1028`, `unread_for_want_of_a_token` `:2750`, `Run::written` `:3202`), `src/go/runtime.go`, `src/ts/runtime.ts` (commit `c1fdc2af8`) | cited |
| tests | `crates/verify/ess-conformance/tests/grant_denied.rs` (inline models `desk()`, `JOBS`; helpers `WritesOnRefusal`, `support_go`, `support_ts`) | cited |
| synthesis | `crates/verify/ess-conformance/src/synthesize/grant.rs`: no change | cited |

Crate: ess-conformance (tests only). Format gate: none (no `ess/32`, no suite major). Test binary:
`grant_denied`. Overlaps `story:a-refused-send-keeps-the-earlier-read-token-for-a-denied-grant`
(same file; that story is active and owns the fix). Sizing: S.

confidence: high
