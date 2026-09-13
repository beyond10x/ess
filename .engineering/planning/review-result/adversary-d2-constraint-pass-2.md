---
format: aep.planning-md/1
id: review-result:adversary-d2-constraint-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: the home page''s completeness claims outrun the tree'
summary: 'Red: 6 introduced, 1 pre-existing; pass-1 fixes held, new claims introduced new falsehoods'
relations:
- reviews: story:d2-constraint-has-a-home
revision: 1
---
# Adversary pass 2 — story:d2-constraint-has-a-home

Verdict **red**. Cases executed 143 → 146, 3 red. Origin: introduced 6, pre-existing 1, undecided 0.
All three pass-1 cases stayed green, so the pass-1 corrections held.

## Confirmed by re-measurement

`generated/rust/README.md` is hand-written, verified three ways: the only `Artifact::new("README.md")`
is `ess-synth/src/web/mod.rs:329` landing at `generated/web/billing/README.md`; the Rust emitter
emits no README; `cargo xtask synth` writes one level below the edited file; `PROJECTION_EXCLUSIONS`
at `ess-xtask/src/main.rs:29` contains `"rust"`. The implementor's edit survives generate and synth.

The `#![allow(clippy::case_sensitive_file_extension_comparisons)]` weakening is **accepted**: the
file was diffed against what the adversary wrote and every assertion is present with identical text
and semantics. `Path::extension()` on `linker.go` returns `go`, so the lint's rewrite would match
every Go file rather than `_test.go`. Caveat recorded: module scope also silences a future genuine
`.MD`-style bug in that file.

The coordinator's relayed "assertion count 11" could not be reproduced — the file held 5 macros and
4 `.expect` guards before pass 2, 11 and 6 after three cases were added. Nothing was weakened; the
number was wrong.

```findings
- file: docs/design/linker-never-chooses.md
  line: 4
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the page claims every document stating the rule in full links here, but website/blog/2026-08-20-2316-structural-synthesis.md names the home as inline code and carries no link, so the sentence is false and the story's acceptance is unmet outside docs/
- file: docs/design/linker-never-chooses.md
  line: 4
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the page says code copies of D-2 are the ones it names and are checked by the tests it names, but examples/billing-web/src/lib.rs states the rule in full, is not a linker, and is named by neither the page nor any test it lists
- file: examples/gatepass-go-realization/linker.go
  line: 14
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the doc comment names TestTheLinkersObligationListIsExactlyThePlans as holding the obligation list equal to the plan, and that test is defined nowhere in the repository, which the unit's new no-Go-test-lane paragraph now directly contradicts
- file: docs/design/linker-never-chooses.md
  line: 58
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the Go copy is said to be evidenced by the wave 7 cross-language comparison, but Link() offers exactly one claimant per obligation and linker.go:123 states the resolution cannot fail for those offers, so neither refusal branch is ever executed
- file: crates/edge/ess-xtask/tests/d2_constraint_home.rs
  line: 66
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: ZERO_CLAUSE remains a single literal spelling while TWO_CLAUSE_SPELLINGS grew to three, so the one-spelling-is-not-a-definition defect is closed on only one half of a two-half rule
- file: crates/edge/ess-xtask/tests/d2_constraint_home.rs
  line: 187
  category: property
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: states_d2_in_full requires no locality, so a file carrying the zero clause and a two clause in unrelated sections reads as a restatement; no such file exists today so it could only be shown by construction
- file: crates/edge/ess-xtask/tests/d2_constraint_home.rs
  line: 163
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: paths from git ls-files are decoded with from_utf8_lossy, so a non-UTF-8 path would be mangled and then skipped silently rather than reported; no such path and no submodule exists in this repository
```
