---
format: aep.planning-md/3
id: review-result:zi-t2-covers-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:an-authored-scenario-retires-the-refusal-it-covers
relations:
- reviews: story:an-authored-scenario-retires-the-refusal-it-covers
revision: 1
---
unit: story:an-authored-scenario-retires-the-refusal-it-covers
verdict: weak
new failing cases: 3 (adversary_t2_covers_readers.rs::the_go_reader_agrees_with_the_rust_reader_on_covered_by, ::the_typescript_reader_agrees_with_the_rust_reader_on_covered_by, ::the_browser_reader_agrees_with_the_rust_reader_on_covered_by_at_35)
mutants survived: none run (the unit's own tests never run the Go, TypeScript or browser reader on a suite with `covered_by`, and never hand Rust admission a `covered_by` below `/65`; the new file now covers both)

`git diff --stat`: empty. The only change is one untracked test file, `crates/verify/ess-conformance/tests/adversary_t2_covers_readers.rs`, uncommitted. No implementation file was touched.

1. **Blocker? No, backlog.** The Go, TypeScript and browser readers have no `/65` gate for `covered_by`. They admit it on `ess-conformance/35` and `/53` suites, which Rust refuses with "a covered refusal requires suite/65". Go and TypeScript then also call those suites complete.
   - Command: `build.sh adv/r2.txt <tree> test -p ess-conformance --locked --test adversary_t2_covers_readers` (exit 101).
   - Failing assertion, `Go (left) and Rust (right)`: `35-covered-candidate_not_emitted.json reader=admitted-complete rust=refused`, and the same for `53-`. TypeScript fails identically. The browser fails the same way on `35-` (`admitted` vs `refused`).
   - Where: `go/runtime.go:329`, `ts/runtime.ts:1193`, `assets/coverage-admission.js:511`.
   - What reaches it: no ess writer, because the writer always chooses `/65`. Only a hand-edited or foreign-produced suite gets there. The design page says "a reader of `/65` admits the field only…", so this is drift the unit introduced. At the base, all three readers refused the key outright.
   - Fix: in each of the three readers, require the suite major to be at least 65 when `covered_by` is present.
2. **Judgement, doc only.** The browser admission reads only `/5`, `/9` and `/35` (`coverage-admission.js:187,547`), so it can never legitimately see a `covered_by` suite. Its state label (`:635`) also ignores `covered_by`. The design sentence at `review-conformance-coverage.md:185`, that the browser "appl[ies] the same completeness rule", is not true.
3. **Agreement holds where the formats match.** On the unit's real synthesized `/65` suite, all three generated runtimes (Rust, Go, TypeScript) say admitted-complete. On a `select` child input that leaves the covering scenario out, all three say admitted-incomplete. Refusals agree where `covered_by` has a null scenario or names an authored id that isn't there.
4. **Pinned, passing, kept.** `the_rust_reader_admits_covered_by_only_from_65` passes. It pins the Rust `/65` admission gate (`coverage.rs:485`), which no test pinned before.
5. **Suite run** (`adv/r4.txt`, `--no-fail-fast`, exit 101):
   - adversary file: 1 passed, 3 failed
   - `authored_covers`: 6 passed
   - `coverage_refusal_catalogue`: 4 passed
   - executed 10→14, red 3
6. **Attacked and could not break:**
   - A covering scenario that is skipped, unsupported or errored never yields Passed (`counts.rs:465`). Known-failing never makes a run pass.
   - Two scenarios covering one id are refused with `covers.repeated-scenario`, as the design says. One scenario covering several ids marks every occurrence.
   - A `/6` document holding `covers:` is refused.
   - The ess-conformance byte pins were green in the implementor's gate. No row moves without `covers:`.
7. **Paths written outside the worktree**, all under `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/an-authored-scenario-retires-the-refusal-it-covers/adv/`: `r1.txt` to `r4.txt`, `cmp.py`, `tmp/`. The build itself used the shared slot target dirs A and B under `.../ess/targets/`.

```findings
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 329
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: Go admits covered_by on /35 and /53 suites and calls them complete, where Rust refuses them as needing suite/65.
- file: crates/verify/ess-conformance/src/ts/runtime.ts
  line: 1193
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: TypeScript admits covered_by on /35 and /53 suites and calls them complete, where Rust refuses them as needing suite/65.
- file: crates/verify/ess-conformance/assets/coverage-admission.js
  line: 511
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: The browser admission admits covered_by on a /35 suite, the only coverage version it reads, which Rust refuses.
- file: docs/design/review-conformance-coverage.md
  line: 185
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: The page says the browser admission applies the same completeness rule, but it cannot read /65 and its state label at coverage-admission.js:635 ignores covered_by.
```
