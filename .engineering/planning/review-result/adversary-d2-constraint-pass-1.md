---
format: aep.planning-md/1
id: review-result:adversary-d2-constraint-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: D-2 home page claims more enforcement than the tree ships'
summary: 'Red: 4 introduced findings, 2 blockers; the acceptance is unmet by one word in the shipped check'
relations:
- reviews: story:d2-constraint-has-a-home
revision: 1
---
# Adversary pass 1 — story:d2-constraint-has-a-home

Worktree `mdi1-d2-constraint-has-a-home`, branch `impl/d2-constraint-has-a-home`, uncommitted, base
`1a2effd6`. Verdict **red**. Cases executed 138 → 141, 3 red. Origin: introduced 4, pre-existing 1,
undecided 0.

## What it attacked and could not break

Link correctness of all four new links; the "says it is gone" disclaimer on all five surviving
`gap-register.md` references, in rendered text and within the window; alternative spellings of the
register reference (no bare `gap-register`, no `../plan/` form, no case variant, no URL-encoding, no
symlink); both D-2 clauses present on the new home page; `file!()` self-exclusion working; and the
`docs/`-only scope of the restatement scan, which it judged defensible for a linker implementing the
rule and not defensible for two Markdown pages, which became finding 4.

## Cases added

`crates/edge/ess-xtask/tests/d2_constraint_home_adversary.rs`, three cases, all red at exit 101:
`the_home_page_names_only_linkers_that_ship_the_tests_it_claims`,
`d2_is_stated_once_under_docs_however_the_second_clause_is_worded`,
`every_markdown_page_stating_d2_in_full_links_to_its_home`.

## Coordinator note recorded with the pass

The adversary's gate line in my brief was wrong: `-p ess-xtask` requires the pinned profile
environment at `Taskfile.yml:46-54` (`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=2`,
`RUSTFLAGS='-C link-arg=-fuse-ld=lld'`, cleared wrappers), which
`crates/edge/ess-xtask/src/consumer_coverage/mod.rs:152-163` asserts exactly. Without it
`consumer_coverage::metadata::tests::current_compiled_provider_executes_one_guard_and_binds_its_opaque_proof_to_this_run`
fails `unsupported measured compiled profile DEBUG`. That red is a harness artifact of my brief, is
pre-existing, and touches no line this unit changed. `task doc-check` was not run, stated as a
deviation, because it is workspace-scope `cargo doc` on a disk that had twice gone under 25 GiB
today and it cannot compile an integration-test target anyway.

```findings
- file: docs/design/linker-never-chooses.md
  line: 48
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the new home page's "Where it is enforced" section says each named linker ships a test for the zero case and a test for the many case, but examples/gatepass-go-realization/ contains no _test.go file and no go test is invoked anywhere in the repository
- file: docs/design/ess-model-driven-interpretation-design-v0.1.md
  line: 59
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: D-2 is still stated in full a second time under docs/ ("an unsatisfied obligation; two remains an ambiguity naming both") and the shipped check misses it only because D2_CLAUSES requires the literal spelling "ambiguity error"
- file: docs/design/ess-model-driven-interpretation-design-v0.1.md
  line: 76
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the sentence this unit added claims the page "cites it rather than holding the second-best copy of it" while a full copy of both clauses remains seventeen lines above
- file: docs/design/linker-never-chooses.md
  line: 3
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the page claims to be the only full statement of D-2 with every other page linking here, but generated/rust/README.md and website/blog/2026-08-20-2316-structural-synthesis.md state it in full and link nowhere
- file: crates/edge/ess-xtask/tests/d2_constraint_home.rs
  line: 41
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the doc comment says the exclusion list mirrors tests/layout.rs, which it does not — layout.rs also excludes docs/design/ and docs/reviews/ and asks git ls-files instead of walking the filesystem
- file: crates/edge/ess-xtask/tests/d2_constraint_home.rs
  line: 43
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: UNSCANNED_PREFIXES matches root-relative only, so the gitignored build trees generated/rust/*/target/, examples/billing-web/target/ and website/node_modules are walked and read, making the scan's input depend on local build state; none are present in this tree today so no failure was observed
```
