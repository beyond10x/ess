---
format: aep.planning-md/3
id: review-result:selection-plan-w3-u2-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 3 unit U2 (Rust and Go emitters read the plan)
relations:
- reviews: story:generated-behaviour-reads-selection-plan
revision: 1
---
unit: `story:generated-behaviour-reads-selection-plan`, U2 working tree on base `1046621a8`, with the implementation uncommitted
verdict: CONFIRMED
cases: executed 611→614, red 1
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 21 paths under `~/.cache/ess-selection-plan/w3-u2-scratch/` (part 6)
needs-coordinator: decide what proves byte identity for steps 2/3 and 5/6: my base-against-unit comparison of 10,284 generated models, or a new probe fixture (F1)

I found no model where the unit's Rust or Go answers differently from the base emitter. Two gaps remain: the byte probes the acceptance relies on cannot see two of the reorders, and no test checks that `precheck` follows the plan.

**1. Diff stat**
```
 crates/generate/ess-synth/src/determined.rs     |  43 --
 crates/generate/ess-synth/src/go/behaviour.rs   | 473 ++++++++++++++--------
 crates/generate/ess-synth/src/plan.rs           |  22 +-
 crates/generate/ess-synth/src/rust/behaviour.rs | 509 ++++++++++++++++--------
 4 files changed, 674 insertions(+), 373 deletions(-)
?? crates/generate/ess-synth/tests/adversary_selection_plan_w3_u2_pass1.rs   (mine)
?? crates/generate/ess-synth/tests/selection_plan_emitters.rs                (the implementor's)
```
The four tracked files are the unit's own uncommitted work. Their stat (674+/373−) is the same now as when I started, so I changed none of them. My only path in the tree is the untracked test file.

**2. Cases added** (`crates/generate/ess-synth/tests/adversary_selection_plan_w3_u2_pass1.rs`)

| case | asserts | now |
|---|---|---|
| `adv_w3u2_p1_the_byte_probes_see_every_reorder_of_the_rewired_phases` | the test swaps two phases with `with_phase_order`. Each swap first changes an inline witness model (validates, Rust and Go generate it). It then checks that at least one probe model (every repository YAML, as written and relabelled to `ess/22`) changes too. | red |
| `adv_w3u2_p1_precheck_follows_the_phase_order_{rust,go}` | with Accepting and Default swapped, the stored-reference `precheck` block changes | green on the unit, red on a mutant |

Red output of the first case, run alone (`adv1-red-3.log`):
```
input_refusal <-> existence: 0 of 217 probe models change (180 generate a behaviour)
present_related <-> accepting: 0 of 217 probe models change (180 generate a behaviour)
...panicked at crates/generate/ess-synth/tests/adversary_selection_plan_w3_u2_pass1.rs:410:5:
the byte probes (adversary_e_u2_bytes, adversary_e_u6_bytes) hash no model whose generated Rust or Go behaviour changes under these reorders, each of which changes a validating, generated witness: ["input_refusal <-> existence", "present_related <-> accepting"]
test result: FAILED. 0 passed; 1 failed
```
- **Earlier draft, discarded:** cases for `existing_instance:` beside a stored reference or a re-key. Their models failed validation (`keep existing_instance: on a command that only creates`), so they were typos, not findings (`adv1-red-1.log`).
- **At base:** the same case is red at `:385` instead, because the base emitters ignore the seam (`adv1-pass1-at-base.log`).
- **Mutant:** in a scratch copy, `precheck` iterates `command.outcomes` (declared branches, default last), not `looking_ahead()`. On it both precheck cases fail with `assertion left != right` (`adv1-precheck-mutant.log`). The unit's own `selection_plan_emitters` stays green on that mutant. No other ess-synth test uses `with_phase_order`.

**3. Suite run** (after all three cases existed)

`CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --no-fail-fast` → 613 passed, 1 failed, 3 ignored. Only `-p ess-synth --test adversary_selection_plan_w3_u2_pass1` failed. `EXIT=101`. The baseline of 611 comes from the implementor's `r2-suite.log`.

**4. Findings** (they cover the working tree above)

| # | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| F1 | `tests/adversary_selection_plan_w3_u2_pass1.rs:410` | CONFIRMED | introduced | No probe model changes under InputRefusal↔Existence or PresentRelated↔Accepting, though both swaps change a generated witness. So "309 of 309 identical" says nothing about those two reorders. | Acceptance bullet 2 and the unit's report. A real reorder in either emitter, or a later change to the plan, would pass the probes. The YAML set itself is unchanged, but only this unit's emitters follow the plan. |
| F2 | `src/rust/behaviour.rs:1719`, `src/go/behaviour.rs:2509` | CONFIRMED | introduced | Scope decision 2 ("precheck reads the same phases") has no test. The mutant above passes the unit's seam test. | Only the seam, or a future `Phase::PRECEDENCE` change. In the default order the mutant writes identical bytes. |

Suggested fixes:
- **F1:** add the `WITNESS` model from my case as a repository YAML that the probes read, then rerun both probes base against unit. Or record my comparison (part 5) as the evidence.
- **F2:** keep my two precheck cases.

**5. Attacked, could not break**
- **Base-against-unit comparison:** I ran the same test in an export of `1046621a8` and in the unit. It covers 18 families × every declaration order:
  - input refusals, related rows read through the input or a stored field, refusals ordered by `wrong_state:`, accepting `when_related:`;
  - `existing_instance:` (creation-only and beside an input-read related row), held-state guards, `unknown_instance:`, external and `external` + `when:`, a re-key collision.
  - Result: 9,720 Rust and 9,600 Go generated models. `behaviour.rs`, `behaviour.go`, `PLAN.md` and `plan.json` are identical on 9,720 of 9,720 lines.
- **`when: true` refusals (#489):** no panic in 564 Rust and 444 Go models. Every one of them changes bytes, as the decision expects. I did not look at its answer further, per the brief.
- **`existing_instance:` moving to step 1 on a stored reference or a re-key:** unreachable, because validation allows `existing_instance:` only on commands that only create.
- **Held-state branch beside an ordered `when_related:` refusal:** validation refuses it (`related_guard::other_authority`).
- **Go `RelatedSet` `unreachable!`:** Go refuses every model with an identity write before writing anything (`go/mod.rs:299`).
- **Removed helpers:** `git grep` finds none of the three in ess-synth.
- **Emitted comments:** the only new comment text (`UNGUARDED_REFUSAL`) is reached only by the #489 shape.

**6. Paths written outside the worktree** (all under `~/.cache/ess-selection-plan/w3-u2-scratch/`)
- Logs and listings:
  - `adv1-red-1.log`, `adv1-red-2.log`, `adv1-red-3.log`, `adv1-suite.log`, `adv1-pass1-at-base.log`
  - `adv1-precheck-unit.log`, `adv1-precheck-mutant.log`, `adv1-go.diff`
  - `adv1-diff-unit.log`, `adv1-diff-base.log`
  - `adv1-diff-{unit,base}.txt`, `adv1-diff-{unit,base}-2.txt`, `adv1-diff-{unit,base}-3.txt`
- Comparison test and output: `adv1-differential-probe.rs`, which was removed from the tree, and `adv1-dump/` (3 files).
- `base-tree/`: `git archive 1046621a8` plus two adversary test files, with its own `target/`. 1.1 G.
- `mutant-tree/`: a copy of `base-tree` with the unit's sources overlaid and the precheck mutant, with its own `target/`. 1.2 G.

I released the lease `w3-u2-adversary-1`.

**7. Findings block**
```findings
- file: crates/generate/ess-synth/tests/adversary_selection_plan_w3_u2_pass1.rs
  line: 410
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: The byte probes the acceptance relies on hash no model that changes under InputRefusal<->Existence or PresentRelated<->Accepting, so an identical base/unit listing cannot catch either reorder.
- file: crates/generate/ess-synth/src/rust/behaviour.rs
  line: 1719
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: precheck reading the plan (scope decision 2) is untested; a mutant iterating command.outcomes in Rust and Go keeps the unit's seam test green and fails only the added precheck cases.
```
