---
format: aep.planning-md/3
id: review-result:adversary-c1-onename-pass-2
kind: review-result
status: active
title: Adversary pass 2, wave correctness-1 unit onename
relations:
- reviews: story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts
revision: 1
---
unit: story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts
verdict: red
cases: executed 936→943, red 3
origin: introduced 2, pre-existing 1, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-c1/onename/adv2/ (logs, probe_dump.rs)
needs-coordinator: yes

Adversary pass 2 (`aep:adversary`), 2026-09-28, head 8ce4a54c2 + untracked `crates/specify/ess-domain/tests/adversary_one_name_pass2.rs`. Pass-1 F1–F3 fixed (9 pass-1 cases green); the mechanism does not change the assembled model.

| case | now | base |
|---|---|---|
| `a_same_kind_copy_filed_under_another_domain_names_that_domain` :55 | RED | green |
| `a_misplaced_copy_is_refused_for_its_domain_even_when_it_repeats_a_name` :79 | RED | RED |
| `the_first_file_named_in_a_hint_is_the_first_to_declare_the_name` :161 | RED | green |
| file-order ×6, third copy told first file, domainless type + member, domainless first copy | green | third-copy red (hint new) |

`cargo test -p ess-domain --no-fail-fast`: 940 passed, 3 failed, EXIT=101. `cargo test -p ess-compiler`: 208 passed.

- N1 introduced (spec.rs:912 `keep`): a same-kind copy under another domain — the suppressed Assembly::claim refusal was the only sentence naming the second domain.
- N2 introduced (spec.rs:653): declare is keyed by kind; "X declares it first" names the first file of that kind, false when an earlier file holds the name as another kind (command a.yaml, event b.yaml, event c.yaml → c.yaml told b.yaml).
- N3 pre-existing note (system.rs:1077, spec.rs:912): a duplicate copy filed under a domain that cannot hold its name is dropped before DomainSpec::validate; misplacement surfaces only after the duplicate is fixed. Base drops it the same way.
- N4 introduced note (spec.rs:1034): views-before-errors reorders diagnostics of specs with no duplicate (broken error + broken view in one file).

Could not break: model content identical base vs head; Specification::assemble is the only production path (SystemSpec::merge, Manifest have no external callers); no test/doc matches the old hint; file order ×6 stable.

```findings
[{"file":"crates/specify/ess-domain/src/spec.rs","line":912,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Suppressing the Assembly::claim refusal of a same-kind copy filed under another domain removes the only sentence that named that second domain; declare refusal names files only."},{"file":"crates/specify/ess-domain/src/spec.rs","line":653,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"declare new hint names the first file of the same kind as the one that declares the name first, which is false when an earlier file holds the name as another kind."},{"file":"crates/specify/ess-domain/src/system.rs","line":1077,"category":"acceptance","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"A duplicate copy filed under a domain that cannot hold its name is dropped before DomainSpec::validate, so its misplacement is reported only after the duplicate is fixed."},{"file":"crates/specify/ess-domain/src/spec.rs","line":1034,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"Reading views before errors reorders the conversion diagnostics of specs with no duplicate at all, putting a broken view refusal ahead of a broken error refusal."}]
```

Trend: pass 1 → 3, pass 2 → 4, carried 0. Coordinator routing: N1, N2, N4 → correction 2 (N4: restore Collected order, move the ordering change to Assembly::claim, which has no production caller besides assemble); N3 → correction 2 if local, otherwise escalated as its own story. Coordinator verifies correction 2 by diff read.
