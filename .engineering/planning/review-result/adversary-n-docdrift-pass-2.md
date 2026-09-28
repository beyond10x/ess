---
format: aep.planning-md/3
id: review-result:adversary-n-docdrift-pass-2
kind: review-result
status: active
title: Adversary pass 2, ess-next unit docdrift
relations:
- reviews: story:generated-suite-docs-say-what-the-runner-does
revision: 1
---
unit: story:generated-suite-docs-say-what-the-runner-does
verdict: green
cases: executed 11→14, red 0
origin: introduced 1, pre-existing 0, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-c1/docdrift/adv2/ (sweep, logs)
needs-coordinator: no

Adversary pass 2 (`aep:adversary`), 2026-09-28, head ff62941c5 + `crates/verify/ess-conformance/tests/adversary_generated_docs_pass2.rs` (3 cases, green; the first killed a mutant that swaps the two emit refusals).

Suite: generated_docs 5, adversary_generated_docs 6, pass2 3, direct_returns 19, runtime_suite_admission 3, typescript_suite_versions 29; exit 0. CLI sweep over 87 spec paths: every /4–/27 suite emits on Go and TS. README correct for every major 1–27 on both targets.

```findings
[{"file":"crates/verify/ess-conformance/tests/direct_returns.rs","line":490,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"Both emitter refusals share UnsupportedTarget and the existing cases check only the reason, so moving refuse_unadmitted ahead of refuse_generation survives the suite; adversary2_direct_return_suite_meets_the_direct_return_refusal_first now kills it."}]
```

Coordinator routing: the note is closed by the adversary's own case, committed with the unit (outcome fixed).
