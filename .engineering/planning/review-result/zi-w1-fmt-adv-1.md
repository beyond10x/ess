---
format: aep.planning-md/3
id: review-result:zi-w1-fmt-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
relations:
- reviews: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
revision: 2
---
unit: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs (tree zi-t1-2 at HEAD 9f4f4e8935, plus 2 untracked test files)
verdict: weak
new failing cases: 1 (crates/edge/ess-cli/tests/adversary_t1_epic_formats.rs::examples_billing_redeclared_at_ess_37_validates)
mutants survived: 1 (removing `65` from `COVERAGE_MAJORS` in src/ts/runtime.ts left the unit's suite green; my new case now catches it)
cases: executed 3→5 (CLI: 3 is the implementor's epic.txt, my 2 added; run 3 stopped at my red target before epic_formats_open ran); new ess-conformance target 0→3; red 1
origin: introduced 0 / pre-existing 1 / undecided 0
wrote-outside-worktree: 8 paths (part 6)
needs-coordinator: yes. Decide whether the acceptance example (examples/billing) is replaced by the fixture the implementor used.

**1. `git --no-pager diff --stat`:** empty, so no tracked file changed. `git status --short` shows only my two test files:
```
?? crates/edge/ess-cli/tests/adversary_t1_epic_formats.rs
?? crates/verify/ess-conformance/tests/adversary_t1_pair_64_65.rs
```

**2. The cases I added, each run alone first**
- `adversary_t1_epic_formats.rs::examples_billing_redeclared_at_ess_37_validates` is **red**. It asserts the acceptance as written: `ess specify validate` exits 0 on examples/billing re-declared at `ess/37`. Output in `adv/run1-cli.txt`, exit 101:
  `it exits Some(1): ... [missing_declaration] command.billing.email.SendEmail.outcomes.sent.payload.billing.email.EmailSent.message_id: emitted payload field has no source` (and the same for `InvoiceCreated.invoice_id`).
- `adversary_t1_epic_formats.rs::examples_billing_at_ess_37_is_judged_exactly_as_at_ess_36` is green. Billing gets the same exit code and the same diagnostics at `ess/36` and `ess/37`. This is what the acceptance means, and it was never run on billing.
- `adversary_t1_pair_64_65.rs` has 3 cases, all green (`adv/run2-conf.txt`, exit 0):
  - `an_explicit_selection_of_a_coverage_suite_65_is_admitted_as_one_of_63_is`: the Rust coverage check `coverage.rs:50` had no test at /65.
  - `the_typescript_runtime_reads_64_and_65_where_it_reads_62_and_63`: TypeScript accepts /62–/65 and a selected /65 input, and refuses /64 with coverage, /65 without it, and /66.
  - `probe_the_case_above_fails_when_65_leaves_the_typescript_coverage_table`: runs the same check against an edited copy of the emitted package with `65` removed from `COVERAGE_MAJORS`, and the verdicts differ. This proves the case can fail. Delete the probe if you don't want it kept.

**3. Suite runs after the cases existed**
- `build.sh adv/run3-cli-suite.txt <tree> test -p ess-cli --locked --test epic_formats_open --test adversary_t1_epic_formats`: `test result: FAILED. 1 passed; 1 failed`, exit 101. Cargo stopped at the first failing target, so epic_formats_open did not run here. Its green count is from the implementor's epic.txt (3 passed).
- `build.sh adv/run4-admission.txt <tree> test -p ess-conformance --locked --test runtime_suite_admission`: `test result: ok. 2 passed`, exit 0. Go and TypeScript accept every major up to /65 and refuse /66. This target is not in the unit's gate.

**4. Findings**
- **Acceptance example is unreachable.** `examples/billing` declares `ess/1`. Re-declared at `ess/36` it is already refused with ESS-COMMAND-005: I checked this with a slot binary from another tree on `adv/billing36`. So "exit 0 at ess/37" cannot hold. The implementor silently used `aggregate-measures-over-expressions.yaml` instead.
  - Verdict: INFEASIBLE, pre-existing, note.
  - What reaches it: only the story text.
  - Fix: reword the acceptance to "billing at ess/37 is judged as at ess/36" (my green case holds that), or accept the substituted fixture.
- **Mutant that survived.** Removing `65` from TypeScript's `COVERAGE_MAJORS` (`src/ts/runtime.ts:7193`) left every unit test green. It would make the generated TypeScript runner refuse every /65 coverage suite.
  - Verdict: CONFIRMED, introduced (a gap in the unit's tests, not in its code), warning.
  - Now pinned by `the_typescript_runtime_reads_64_and_65_where_it_reads_62_and_63`.
  - The Rust counterpart, `"ess-conformance/65"` in `coverage.rs:50`, is now covered by the selection case. I did not run a mutant against it.

**5. Attacked and could not break**
- Every check keyed on a version number in Rust, Go and TypeScript, including the 62/63 lists. All are `>=`/`<` floors or were updated.
- Go and TypeScript agree with Rust on /64, /65 and /66.
- `ess/37` gates: every `V36` comparison is `>=`.
- ess-diff /24: the chosen version is the highest change minimum, raw.rs and delta.rs check generically, and the CLI writes nothing at /24.
- `spec-versions.md`: the hand-written rows are outside the generated region, the same as earlier format openings.
- The browser bundle's coverage-admission.js lists only 5, 9 and 35. That predates this unit.

**6. Paths written outside the worktree** (all under `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/ess-37-and-conformance-64-65-are-open-for-the-epics-constructs/adv/`):
- `billing36/`
- `run1-cli.txt`, `run1-cli.txt.touch`
- `run2-conf.txt`, `run2-conf.txt.touch`
- `run3-cli-suite.txt`
- `run4-admission.txt`, `run4-admission.txt.touch`

Builds also went into build.sh's shared slot target dirs A and B.

**7.**
```findings
- file: crates/edge/ess-cli/tests/adversary_t1_epic_formats.rs
  line: 58
  category: acceptance
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: the acceptance names examples/billing validating at ess/37, but billing is already refused (ESS-COMMAND-005) at ess/36, so the example cannot hold and the unit substituted another fixture without saying so
- file: crates/verify/ess-conformance/src/ts/runtime.ts
  line: 7193
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: dropping 65 from the TypeScript COVERAGE_MAJORS table left the unit's suite green, so nothing held the generated TypeScript runner to admitting a /65 coverage suite until adversary_t1_pair_64_65.rs
```
