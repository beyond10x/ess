---
format: aep.planning-md/3
id: review-result:adversary-c1-onename-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave correctness-1 unit onename
relations:
- reviews: story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts
revision: 1
---
unit: story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts
verdict: red
cases: executed 926→935, red 4
origin: introduced 1, pre-existing 2, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-c1/onename/adv1/ (red-alone.log, suite.log, suite-compiler.log, base-run.log, base/ + base-target/, fx/a, fx/b)
needs-coordinator: yes (two findings are pre-existing on the base — route: back to this unit or out as own story)

Adversary pass 1 (`aep:adversary`), 2026-09-28, on head 8c34e8a77 + untracked `crates/specify/ess-domain/tests/adversary_one_name_pass1.rs`.

| case | now | base b5c53e8a1 |
|---|---|---|
| `two_sound_copies_of_one_kind_are_refused_once` | RED (2 vs 1) | RED 2 |
| `two_sound_actors_of_one_name_are_refused_once` | RED (2 vs 1) | RED 2 |
| `a_name_is_refused_when_one_copy_sits_in_a_file_with_no_domain` | RED (0 vs 1) | RED 0 |
| `an_error_and_a_view_..._whichever_copy_converts` | RED (locations `domain shop.cart.errors` / `views` ×3) | not reported on base |
| sound after broken of own kind; broken domainless; same short name in two domains ×4; four broken kinds both orders; type+entity ×4 ×2 | green | 4 of 5 red |

`cargo test -p ess-domain --no-fail-fast`: exit 101, passed 931, failed 4 (exactly these cases), ignored 2. `cargo test -p ess-compiler --no-fail-fast`: exit 0, 208 passed.

- F1 (pre-existing, NEEDS-CHANGE): two sound same-kind copies are refused twice — `declare` and `Assembly::claim` (`system.rs:1093`); contradicts the doc at `spec.rs:~857` and "no double report". Reached by any name written twice, both sound. Fix: claim skips same-kind duplicates, or absorb does not push when `first` is false.
- F2 (pre-existing, NEEDS-CHANGE): `converted` means "reaches Assembly::claim" (`spec.rs:886`), but a converted member in a file with no `domain:` never reaches it (`spec.rs:997`): sound domainless event + sound domain command of one name → 0 refusals. Same masking class.
- F3 (introduced, CONFIRMED, note): `Collected` writes errors before views (`spec.rs:983`), `Assembly` claims views before errors (`system.rs:1057`); location and kind order flip with which copy converts, contrary to `Claim::refuse`'s doc.

Could not break: same short name in two domains; 3–4 copies; type+entity; parse-failing names (whole file refused); n log n BTreeMap; `ess specify validate` refuses each fixture once (cites `<document>`, as the base locator does for Assembly refusals); ess-compiler green.

```findings
[{"file":"crates/specify/ess-domain/src/system.rs","line":1093,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"Two sound same-kind copies of one name are refused twice, once by declare and once by Assembly::claim, contradicting Collected::write's doc and the story's no-double-report claim."},{"file":"crates/specify/ess-domain/src/spec.rs","line":886,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"A sound copy in a file with no domain: never reaches Assembly::claim, yet write treats it as converted, so a name held by it and a domain member is never refused while the broken variant is."},{"file":"crates/specify/ess-domain/src/spec.rs","line":983,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"Collected writes errors before views while Assembly claims views before errors, so an error/view name clash is refused at a different location and kind order depending on which copy converts, contrary to Claim::refuse's one-sentence doc."}]
```

Coordinator routing: F1 and F2 are pre-existing but inside this story's claim (a name held twice is refused exactly once); the coordinator takes them into this unit.
