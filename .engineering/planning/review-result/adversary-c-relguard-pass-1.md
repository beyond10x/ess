---
format: aep.planning-md/3
id: review-result:adversary-c-relguard-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.41 unit related-guard
relations:
- reviews: story:a-guard-reads-another-entity-by-identity
revision: 1
---
unit: related-guard (beyond10x/ess#211), working tree ~/.local/state/worktree/trees/b10x/ess/ess-c-relguard on b7ecd8d30e (uncommitted)
verdict: NEEDS-CHANGE
cases: executed 2569→2575, red 6
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9 paths, all under ~/.cache/ess-wave-n2/relguard/adv1/
needs-coordinator: whether the command-level narrowing (finding 5) stands for 0.41 or is lifted for `existing_instance`

## 1. Diff stat

`git --no-pager diff --stat`: `25 files changed, 581 insertions(+), 83 deletions(-)`. That is the implementor's tracked diff, unchanged. My additions are two new untracked test files and nothing else:

- crates/specify/ess-domain/tests/adversary_related_guard_pass1.rs
- crates/verify/ess-conformance/tests/adversary_related_guard_pass1.rs

No implementation file was touched.

## 2. Cases added (each run alone before the suite)

| # | case | asserts | now |
|---|---|---|---|
| D1 | `adversary_pass1_exists_false_with_an_input_guard_leaves_a_missing_row_unanswered` | an `exists: false` branch that also has `when: client == "console"` leaves a missing row unanswered, so the domain must refuse it with `non_exhaustive_branches` | red |
| D2 | `adversary_pass1_an_accepting_input_branch_overlapping_exists_false_is_refused` | an accepting `when:` creation next to `exists: false` overlaps on a missing row, so the domain must refuse it with `conflicting_declaration` | red |
| D3 | `adversary_pass1_a_supplied_identity_create_with_a_duplicate_refusal_takes_a_related_guard` | a create with a caller-supplied id, with both `existing_instance` and `when_related {exists:false}`, is admitted | red |
| C1 | `adversary_pass1_a_predicate_over_a_related_enum_field_is_witnessed_on_both_sides` | `predicate: plan == Basic` over the related row is witnessed both true and false | red |
| C2 | `adversary_pass1_a_folder_inside_an_existing_folder_is_witnessed` | a folder created in a parent folder that must exist (root via `CreateRoot`) is synthesized | red (stack overflow) |
| C3 | `..._with_the_root_creator_first` | the same as C2 with `CreateRoot` declared first | red (stack overflow) |

Red output, verbatim. D1 and D2 first failed on my own spelling (`input.client` inside `when:`), which was a typo and not a finding. They were fixed and rerun. The lines below are from that rerun.

```
a supplied-identity create cannot declare both its duplicate refusal and a guard on the related customer:
[conflicting_declaration] command.demo.orders.PlaceOrder.outcomes: `demo.orders.PlaceOrder` selects on a related row (`when_related`) and on an `existing_instance` branch; which of the two answers first is not stated (hint: guard the command on the related row alone, or split the other guard into a command of its own)
admitted: a missing Configuration with `client != "console"` selects no branch (the predicate is Unknown, the default is never taken for a missing row, and `no-configuration` is refuted by its `when:`)
admitted: a missing Configuration with `client == "console"` selects both `fast-path` and `no-configuration`, and nothing states which answers
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
```
`plan == Basic` over the related row must be witnessed true (no-redirect-entry: true) and false (initiated: false); refusals: [
    "ESS-SYNTH-003: no candidate of the 4 tried satisfies ``initiated` selected on a row of `demo.signin.Configuration` that the input names`",
]
```
```
thread 'adversary_pass1_a_folder_inside_an_existing_folder_is_witnessed' (3762153) has overflowed its stack
fatal runtime error: stack overflow, aborting
  process didn't exit successfully: `.../adversary_related_guard_pass1-975a80cc1e956b0b` (signal: 6, SIGABRT: process abort signal)
```
With `RUST_MIN_STACK=1073741824` (1 GiB), C2 still overflows (`timeout: the monitored command dumped core`, EXIT=134). The recursion is unbounded, not merely deep. C3 overflows the same way, so declaration order does not avoid it.

## 3. Suite runs (after the cases existed)

- `cargo test -p ess-domain --no-fail-fast`: EXIT=101. passed=1026, failed=3. The 3 failures are D1 to D3. `error: 1 target failed: -p ess-domain --test adversary_related_guard_pass1`.
- `cargo test -p ess-conformance --no-fail-fast`: EXIT=101. passed=1543, failed=0 in every other binary. `adversary_related_guard_pass1` aborted with SIGABRT (`...has overflowed its stack`) after `running 3 tests`.
- `cargo xtask generate --check`: `projections are up to date`, EXIT=0.
- Before count: 2569 = 1026 (ess-domain) + 1543 (ess-conformance), from the implementor's gate logs `gate-ess-domain.log` and `gate-ess-conformance.log`. After count: 2575.
- Not rerun: the ess-compiler, ess-entity-runtime and ess-xtask suites. I added no cases there.

## 4. Findings

1. **blocker, NEEDS-CHANGE, introduced**: `crates/verify/ess-conformance/src/synthesize/related_guard.rs:250`.
   - `row_at` calls `arrange_first(..., &[])` and drops the `arranging` chain that `arrange` uses to stop cycles.
   - When the related entity's own creator reads a related row of that entity, the path `drive → with_row → row_at → arrange_first → drive` recurses without bound, and synthesis aborts the process.
   - What was measured: C2 and C3, SIGABRT, even with a 1 GiB stack.
   - What reaches it: any model whose create is guarded on an existing row of the same entity (a folder in its parent, a reply to a comment, a child task) and that has a second, unguarded creator. `ess verify conform synthesize` would crash on it.
   - Fix to name: thread `arranging` through `drive`/`with_row`/`row_at`, and refuse with a named cause on a cycle.
2. **blocker, NEEDS-CHANGE, introduced**: `crates/verify/ess-conformance/src/synthesize/related_guard.rs:396`.
   - Candidate rows are arranged at `BLOCK * (base + 8n)` with `BLOCK = 27720`, so every row sits at a distinction that is 0 mod every k ≤ 12. Every row therefore gets the same enum variant (and the same bool).
   - The row's stored fields are never searched toward a goal. The scope recommended the `subject_fact` goal search; it is not used.
   - So a predicate over a related field alone is witnessable on one side only. The issue's own "Expected" asks for "arranges the other record present ... with the field set".
   - What was measured: C1, `ESS-SYNTH-003` on `initiated`.
   - What reaches it: any `when_related` predicate that does not compare with the input. That is the plain form. The fixture only covers `redirect_client != input.client`, which the input grounding happens to rescue.
   - The mod-27720 cause is inferred from the constants, not traced.
3. **warning, NEEDS-CHANGE, introduced**: `crates/specify/ess-domain/src/command/related_guard.rs:594`.
   - `validate_partition` leaves out the `exists: false` branch, and nothing checks the missing-row side.
   - So an `exists: false` branch with a `when:` leaves a missing row with the refuted input unanswered, and the model is admitted.
   - What was measured: D1.
   - What reaches it: `when:` beside `when_related`, which `alone` explicitly admits.
4. **warning, NEEDS-CHANGE, introduced**: `crates/specify/ess-domain/src/command/related_guard.rs:579`.
   - On a missing row, a plain `when:` accepting branch and `exists: false` both hold.
   - Synthesis sidesteps this (`selects` returns none). Only refusals have a stated precedence (#178), so a target may answer either branch, and the model is admitted.
   - What was measured: D2.
   - What reaches it: a creating command with an input-guarded variant creation next to the related refusal.
5. **warning, CONFIRMED, introduced**: `crates/specify/ess-domain/src/command/related_guard.rs:242`.
   - The command-level narrowing refuses `existing_instance` next to `when_related`. So a create with a caller-supplied id cannot declare both "id taken" and "customer missing".
   - The same narrowing blocks create-or-update (`unknown_instance` on a creation) and `unknown_instance` on a move or update guarded by a related row.
   - The issue's own shape (generated `SignIn` id) does not need it. The decision says "usable on any branch including `creates:`".
   - What was measured: D3.
   - What reaches it: any ess/16 supplied-identity create. That is common, but the decision does not name it explicitly, hence a warning.
6. **note, CONFIRMED, introduced**: `crates/verify/ess-conformance/src/synthesize/related_guard.rs:255`. The doc paragraph on `prepare` appears twice (lines 255 and 261).

## 5. Attacked, not broken

- `cargo xtask generate --check`: up to date, so there is no drift in the generated outputs or the schema.
- Optional, nested or non-identity `via`: read, not run. It is refused by the `TypeRef::Primitive|Named` match, by `input_field`, and by `Referenced::NoEntity` in `related_guard.rs` `entity_or_refusal`.
- Existing committed suites: 1026 + 1543 still pass with my files present, so there are no regressions there.
- Not attacked: canonical IR bytes beyond `generate --check`, executed mutants, and running the Entity Runtime and HTTP 409 projection. For those I only read the code.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/relguard/adv1/review.md
- ~/.cache/ess-wave-n2/relguard/adv1/red-domain.log
- ~/.cache/ess-wave-n2/relguard/adv1/red-conformance.log
- ~/.cache/ess-wave-n2/relguard/adv1/red-conformance-enum.log
- ~/.cache/ess-wave-n2/relguard/adv1/red-conformance-folder-1g.log
- ~/.cache/ess-wave-n2/relguard/adv1/red-conformance-folder-rootfirst.log
- ~/.cache/ess-wave-n2/relguard/adv1/suite-ess-domain.log
- ~/.cache/ess-wave-n2/relguard/adv1/suite-ess-conformance.log
- ~/.cache/ess-wave-n2/relguard/adv1/generate-check.log
- Build output went into the assigned ~/.cache/b10x-target/ess-c-relguard. The C2 abort with a 1 GiB stack reported `dumped core`, which systemd-coredump normally keeps.

## 7. Findings block

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 250, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "row_at drops the arranging chain, so a create guarded on a row of its own entity recurses without bound and synthesis aborts with a stack overflow"},
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 396, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "every candidate related row is arranged at a distinction that is 0 mod 27720 with no goal search, so a predicate over a related enum field is witnessable on one side only (ESS-SYNTH-003 on the default)"},
  {"file": "crates/specify/ess-domain/src/command/related_guard.rs", "line": 594, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "an exists:false branch carrying a when: leaves a missing row with the refuted input unanswered and the model is admitted"},
  {"file": "crates/specify/ess-domain/src/command/related_guard.rs", "line": 579, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the missing-row side is never partitioned, so an accepting when: branch overlapping exists:false is admitted with no stated precedence"},
  {"file": "crates/specify/ess-domain/src/command/related_guard.rs", "line": 242, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "the command-level narrowing refuses existing_instance beside when_related, so a supplied-identity create cannot declare both its duplicate refusal and a related guard"},
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 255, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the doc comment on prepare is duplicated verbatim"}
]
```
