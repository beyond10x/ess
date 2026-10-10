---
format: aep.planning-md/3
id: review-result:selection-plan-w4-u2-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 4 unit U2 (synthesis bytes pin)
relations:
- reviews: story:synthesis-reads-selection-plan
revision: 1
---
```
unit: story:synthesis-reads-selection-plan unit 1 (U2 synthesis bytes pin), uncommitted tree at base 373df4ecba
verdict: NEEDS-CHANGE
cases: executed 1→5, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/ess-selection-plan/w4-u2-scratch/adversary-1/ (2.1G)
needs-coordinator: yes — whether the table must also pin in-test models before units 2-5 rely on it
```

Added `synthesis_suite_bytes_table_claim_search.rs` (red: no table line pins the #464 door model, `suite=9 ec61136c…:41109`) and `synthesis_suite_bytes_table_cli_read.rs` (green, 46/46 system.yaml directories equal the CLI read).

Mutants in a scratch copy: `sibling_refusals` all (red, 5 lines), `later_refusals` empty (red, 3), `earlier_accepting_branches` without take_while (red, 26) or without the external check (red, 2), `subject_fact` last-declared (red, 1) and no refusal-first retain (red, 1). Green: `held_state_claims` ignoring the sibling guard (`synthesize.rs:3935`; `unclaimed_external_witnesses_keep_their_bytes` and `adversary_464_…_base_bytes` go red), `related_guard` rfind and forced `orders_present_related_refusal`, three `row_set` order flips, `authored::not_taken` early return.

Not broken: walker (193 `format: ess/` files + 46 directories = 239), the 53 ASSEMBLE rows, parse vs parse_all, digest coverage (full serde output), table integrity (missing/extra/duplicate/header drift fail; sorted write order).

```findings
[
  {"file": "crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs", "line": 11, "category": "mutant", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a held_state_claims mutant (synthesize.rs:3935) moves the #464 door model's bytes and two existing tests catch it, but the table stays green because that model is built in-test and never pinned"},
  {"file": "crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs", "line": 2, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "the outside= column is always 0 under whole-system synthesis, and authored::not_taken, synthesize_for and synthesize_with_seeds are unpinned, although U2 edits authored.rs"},
  {"file": "crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs", "line": 48, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the excluded conditional-measures model is pinned only by an ignored slow probe that the unit 2-5 gates do not run"}
]
```
