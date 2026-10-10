---
format: aep.planning-md/3
id: review-result:selection-plan-w3-u3-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, wave 3 unit U3 (validation reads the plan)
relations:
- reviews: story:validation-reads-selection-plan
revision: 1
---
unit: story:validation-reads-selection-plan, uncommitted working tree on 1046621a8 (round-1 correction, same tree the implementor measured)
verdict: INFEASIBLE (1 red case; it needs the `with_phase_order` test seam)
cases: executed 1448→1449, red 1
origin: introduced 0 / pre-existing 1 / undecided 0
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: yes, for routing. My case is red at base too, but it breaks the per-pair claim this correction makes (detail under part 4).

**Verdict:** at the default order, `refusal_settles` gives the base verdict and base diagnostics on every model `assemble` can reach. Under an exchanged order it admits one shape it should refuse.

**1. Diff stat**
`git --no-pager diff --stat`: 3 files changed, 244 insertions(+), 96 deletions(-). That is the unit's own diff, unchanged during my pass. My only addition is the untracked `crates/specify/ess-domain/tests/adversary_validation_selection_plan_pass2.rs`. I changed no implementation file.

**2. Case**
`an_input_refusal_read_after_two_conflicting_held_state_refusals_does_not_hide_their_conflict`: red.
- Setup: RotateSecret at ess/18 with two held-state refusals, `locked-a` and `locked-b`, both on `Configured` and `mode == Freeze`, plus the input refusal `frozen` on `mode == Freeze`.
- Control 1: without `frozen`, the two locked refusals conflict. Holds.
- Control 2: at default, `frozen` is read first and the model is admitted. Holds.
- Under `exchanged(InputRefusal, HeldState)` the model must be refused for the locked conflict. It is admitted instead. Red output, run alone:
  `panicked at …pass2.rs:111:22: admitted: `frozen` is read after `locked-a` and `locked-b`, which conflict on `Configured` and `mode = Freeze`, and `refusal_settles` let `frozen` answer alone`

**3. Suite**
`CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --no-fail-fast` gave EXIT=101 across 147 test binaries: 1448 passed, 1 failed, 2 ignored. Cargo printed `error: 1 target failed: -p ess-domain --test adversary_validation_selection_plan_pass2`. The "before" count of 1448 comes from `r1-domain.log`. My file passes `cargo clippy` and `rustfmt --check` on its own; I formatted only that file with `rustfmt`. I did not run ess-compiler, because nothing I added touches it.

**4. Findings**

| # | file:line | finding | what reaches it | verdict / origin |
|---|---|---|---|---|
| G1 | `command.rs:261` (`other.is_refusal() ||`), used at `subject_state.rs:383` | Each tie-break asks `refusal_settles` about one neighbour at a time. When two refusals are read before the refusal under test, it settles each pair, so the refusal answers alone and their conflict is hidden. Fix: let the earliest-read selected branches answer, and count those. My pass-1 case 2 still passes under that fix, because there an earlier lone input refusal answers | only `with_phase_order`; the default order is unaffected | INFEASIBLE / pre-existing |

On G1's origin: the case fails the same way against a base copy (`adv-p2-base-run.log`, :110 before formatting) because the base never reads the order. So it is pre-existing by definition. But it breaks the per-pair claim made in this correction's `refusal_settles` doc, so routing it is your call.

**5. Attacked and could not break**
- **Default verdicts:** `refusal_settles` differs from base only where a non-refusal from an earlier phase is in the partition. There are three such branches:
  - an accepting `exists: false` on an input row;
  - a `when_subject_state` acceptance;
  - an accepting `unknown_instance:`.

  `related_guard::other_authority` refuses all three at conversion (`CommandSpec::try_from` calls `validate_shape(None)`), so the partitions never run on them. Three cases built for these classes were green: only the shape refusal is reported. I removed them, as you asked for failing cases only.
- **The 0-difference measurement:** it is sound for what it covers.
  - The measured diff is byte-identical to the current tree (`cmp` against `unit-r1-restored.diff`, and md5 matches `unit-clean.md5`).
  - Its corpus (the ess-domain and ess-compiler suites) contains none of the three classes, so on its own it says nothing about them; the conversion refusal above closes that gap.
  - It records the default order only.
- **Several rows and present-related refusals:** `several_selected` and `selected_count` match base on every reachable shape.
- **Masking at default in the related partition:** two unguarded refusals read before a related refusal also conflict where the related refusal is not selected. So the model is refused anyway and no verdict changes.
- **Undecidable refusals:** `analyze_partition` drops one only where every other branch is a refusal or read later, which keeps the proof sound.
- **R2 and pass-1 cases:** still green, unchanged.

**6. Paths written outside the worktree**
- `~/.cache/ess-selection-plan/w3-u3-scratch/adv-p2-base/`: base 1046621a8 sources plus its own `target/`, 575M. I left it for you to delete.
- `~/.cache/ess-selection-plan/w3-u3-scratch/adv-p2-base-run.log`
- `~/.cache/ess-selection-plan/w3-u3-scratch/adv-p2-domain.log`

My worktree lease is released.

**7.**
```findings
[
  {
    "file": "crates/specify/ess-domain/src/command.rs",
    "line": 261,
    "category": "property",
    "severity": "warning",
    "verdict": "INFEASIBLE",
    "origin": "pre-existing",
    "message": "refusal_settles treats any refusing neighbour as settled pair by pair, so under exchanged(InputRefusal, HeldState) an input refusal read after two conflicting held-state refusals answers alone and their conflict is admitted; red case, reachable only through the with_phase_order test seam, also red at base, which never reads the order"
  }
]
```
