---
format: aep.planning-md/3
id: review-result:adversary-n-docdrift-pass-1
kind: review-result
status: active
title: Adversary pass 1, ess-next unit docdrift
relations:
- reviews: story:generated-suite-docs-say-what-the-runner-does
revision: 1
---
unit: story:generated-suite-docs-say-what-the-runner-does
verdict: red
cases: executed 6→12, red 4
origin: introduced 3, pre-existing 3, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-c1/docdrift/adv1/ (logs, tree/, target/ 490M, pt/)
needs-coordinator: yes

Adversary pass 1 (`aep:adversary`), 2026-09-28, head 86ce9f8cd + `crates/verify/ess-conformance/tests/adversary_generated_docs.rs`. README matched real runs at /4, /5, /10, /21 on both targets. Integration simulation with the runtime-parity runtimes: 5 of 6 adversary cases and all 4 unit cases fail (`adv1/integ2.log`).

| case | now |
|---|---|
| `adversary_typescript_readme_never_tells_a_bare_npm_test_the_runner_refuses` | red (two `## Running it`, first says bare npm test) |
| `adversary_go_readme_is_not_emitted_for_a_suite_its_runner_refuses` | red (Go /22 refused, README gives run instructions) |
| `adversary_typescript_readme_is_not_emitted_for_a_suite_its_runner_refuses` | red (TS /12) |
| `adversary_verify_conformance_guide_run_step_agrees_with_the_generated_readme` | red |
| `adversary_go_readme_run_instructions_match_what_run_does` | green (/4, /5, /10, /21, /26) |
| `adversary_typescript_readme_run_instructions_match_what_run_does` | green (/4, /5, /10, /12, /21, /26) |

```findings
[{"file":"crates/verify/ess-conformance/src/ts/mod.rs","line":512,"category":"acceptance","severity":"blocker","verdict":"CONFIRMED","origin":"introduced","message":"TS README gains a second Running it section; the first still instructs bare npm test, which the runner refuses for /5-/11 and /18-/21."},{"file":"crates/verify/ess-conformance/src/go/mod.rs","line":355,"category":"contract-drift","severity":"blocker","verdict":"CONFIRMED","origin":"introduced","message":"README requirement sets are hard-coded and the test literal-based gate parser reads the runtime-parity >= 8 gate as {5,6,7}, so after #188 Go /22-/27 and TS /12-/17,/22-/27 READMEs revert to the pre-#186 text."},{"file":"crates/verify/ess-conformance/src/go/mod.rs","line":350,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"Packages are emitted with run instructions for suite versions their own runner refuses at admission (Go /22-/29, TS /12-/17 and /22-/29 on base)."},{"file":"website/docs/guides/verify-conformance.md","line":260,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"Guide run step omits ESS_REPORT_FORMAT=2, contradicting the generated README for /5-/21 suites."},{"file":"crates/edge/ess-cli/src/main.rs","line":777,"category":"judgement","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Help says suite_version is chosen by constructs only, while --suite-format 5 (declared coverage) also selects it."},{"file":"crates/verify/ess-conformance/src/ts/runtime.ts","line":2362,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"TS refusal for /10-/21 says suite/8 and /9 require...; the README quotes it elided, so it is not false there."}]
```

Coordinator routing: all six into correction 1, run on top of integrate/ess-next after the runtime-parity go unit merges (the README must follow the merged runtimes): one source of truth for the ESS_REPORT_FORMAT threshold shared by runtime text and README; no run instructions for a version the runtime refuses (the emitter refuses instead); guide and help fixed; TS message fixed.
