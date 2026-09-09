---
format: aep.planning-md/1
id: review-result:typed-diagnostics-adversary-wave21-pass1
kind: review-result
status: active
title: Typed diagnostics adversary, wave 21, pass 1
relations:
- reviews: story:review-typed-diagnostics
revision: 1
---
unit: story:review-typed-diagnostics — worktree ess-typed-diagnostics-wave21 at HEAD 36fa1df (base 2900f628)
verdict: red
cases: executed 513→517, red 3
origin: introduced 6, pre-existing 0, undecided 0
wrote-outside-worktree: none
needs-coordinator: no

Recorded by the wave-21 coordinator from the aep-drive:adversary 0.8.1 (Opus) report as returned, pass 1, 2026-09-09. Harness accounting: 135,367 sub-agent tokens, 47 tool uses, 23.3 min. One transformation by the coordinator: the `message` values in the closing findings block are single-quoted so the block parses as YAML (the returned block carried an unquoted `located: None`); the wording is unchanged.

## 1. What I touched

```
$ git --no-pager diff --stat          # empty: my file is untracked
$ git status --short
?? crates/specify/ess-compiler/tests/adversary_typed_diagnostics_pass1.rs
```
One path, a test file. No implementation file, no doc, no planning file.

## 2. Cases added — `crates/specify/ess-compiler/tests/adversary_typed_diagnostics_pass1.rs`

Each case compiles a document through `Specification::assemble` + `diagnose_locating`, then bridges the *same* `location` string a second time through `ValidationError::new` — that arm is the base algorithm, byte-identical at the base commit (`needles_for`/`STRUCTURAL`/`family_of` untouched by the diff, verified by diffing `git show <base>:resolve.rs`).

Red output of the three cases, run alone, before any suite run:

```
$ TMPDIR=… cargo test --locked -p ess-compiler --test adversary_typed_diagnostics_pass1
running 3 tests
test a_sited_payload_refusal_cites_the_same_line_as_its_own_location_string ... FAILED
test a_sited_refusal_whose_name_is_a_stop_list_word_cites_the_same_line_as_its_location_string ... FAILED
test the_repeated_names_fixture_reports_an_unlocated_refusal_as_its_design_page_says ... FAILED

---- a_sited_payload_refusal_cites_the_same_line_as_its_own_location_string stdout ----
panicked at crates/specify/ess-compiler/tests/adversary_typed_diagnostics_pass1.rs:176:5:
assertion `left == right` failed: the typed needles and the string needles cite different lines for one refusal
  left: Some(Location { line: 23, column: 11 })
 right: None

---- a_sited_refusal_whose_name_is_a_stop_list_word_cites_the_same_line_as_its_location_string stdout ----
panicked at crates/specify/ess-compiler/tests/adversary_typed_diagnostics_pass1.rs:193:5:
assertion `left == right` failed: the typed needles walked into a structural key the stop-list exists to skip
  left: Some(Location { line: 17, column: 9 })
 right: Some(Location { line: 10, column: 5 })

---- the_repeated_names_fixture_reports_an_unlocated_refusal_as_its_design_page_says stdout ----
panicked at crates/specify/ess-compiler/tests/adversary_typed_diagnostics_pass1.rs:217:5:
the design page says this fixture asserts `located: None`; every refusal it produces is located: [("command.shop.repeat.FileOne.input[1]", Some(Location { line: 12, column: 5 })), ("command.shop.repeat.FileOne.outcomes.filed", Some(Location { line: 12, column: 5 })), ("command.shop.repeat.FileOne.outcomes", Some(Location { line: 12, column: 5 }))]

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
```

A fourth case was appended afterwards and is **green from its first run**: `a_validation_error_stays_under_the_result_large_err_threshold` pins `size_of::<ValidationError>() <= 128`, the bound the design page rests the `Box` on and which nothing else in the tree records (brief item 6). Re-run of the file alone after adding it: `1 passed; 3 failed`.

## 3. The gate, after the cases existed

| command | exit | result |
|---|---|---|
| `cargo fmt -p ess-primitives -p ess-domain -p ess-compiler -- --check` | 0 | — |
| `cargo clippy --locked -p ess-primitives -p ess-domain -p ess-compiler --all-targets -- -D warnings` | 0 | — |
| `cargo test --locked --no-fail-fast -p ess-primitives -p ess-domain -p ess-compiler` | **101** | 17 targets ok; `adversary_typed_diagnostics_pass1`: `test result: FAILED. 1 passed; 3 failed` |
| `cargo test --locked -p ess-cli --test command_surface` | 0 | `ok. 6 passed` |
| `cargo xtask schema --check` | 0 | `schemas/generated/ess.schema.json: current` |
| `task consumer-check` | 0 | `Refused:0, Supported:54, executed_cases:22` |

517 executed after, 513 before (the implementing state's own `gate-test.txt` in scratch sums to 513 across 17 `test result:` lines; my run adds exactly my target's 4).

## 4. Findings

| ID | file:line | category | severity | verdict | origin | finding |
|---|---|---|---|---|---|---|
| F1 | `crates/specify/ess-compiler/src/resolve.rs:632` | contract-drift | warning | CONFIRMED | introduced | The page (`docs/design/review-typed-diagnostics.md:93`) says `needles_of_site` "derives the same needles the heuristic derives". It does not. **Measured:** case at `adversary_typed_diagnostics_pass1.rs:176`, sited `Some(23,11)` vs string `None`, exit 101. **Reaches it:** any document with a `payload:` block for an unemitted event (`command.rs:1233`, migrated) — the trailing `Segment::Name` is a *qualified* event name, which starts lowercase, so the guard at `resolve.rs:634` never fires for it and `shop.tail.Missing:` is pushed as a needle. The page's justification at `:97` ("`InvoiceCreated` … is not a YAML key in the documents this repository has") is false twice over: the guard does not see `InvoiceCreated`, it sees `shop.tail.InvoiceCreated`, and every `payload:` block writes exactly that key (`nested.yaml:43`, `cross_file_a.yaml:16`, `repeated_names.yaml:33`). **Fix:** either drop the lowercase test's stated rationale and document the move, or key the needle on the last dotted component. |
| F2 | `crates/specify/ess-compiler/src/resolve.rs:634` | boundary | warning | CONFIRMED | introduced | The `Key`/`Name` split does *not* replace the `STRUCTURAL` stop-list (`resolve.rs:729`) for `Name` segments, and an author-chosen name may be one of its 50 words. **Measured:** case at `:193`; an outcome named `error` refused by `command.rs:1132` is cited at `Some(17,9)` — the **sibling** outcome's `error: shop.stop.Refused` key — where the string path cites `Some(10,5)`, the command's own `name:` line. That is the confidently-wrong line the stop-list exists to prevent, and which the unit's own test comment calls "worse than none". **Reaches it:** ordinary YAML through `Specification::assemble`; no document in this repo's corpus names an outcome or field with a stop-list word, so no pinned output moves today. **Fix:** consult `STRUCTURAL` for the trailing `Name` needle too, or say in the page that the guard was deliberately dropped. |
| F3 | `docs/design/review-typed-diagnostics.md:161` | contract-drift | warning | NEEDS-CHANGE | introduced | The page says `repeated_names.yaml` asserts "the honest `located: None` … rather than a confidently wrong line". Every refusal that fixture produces is located at 12:5, and the shipped test pins all three there (`typed_diagnostics.rs:181`). **Measured:** case at `:217`, exit 101. One of the two must change; the cheap one is the doc line. |
| F4 | `crates/specify/ess-compiler/tests/typed_diagnostics.rs:179` | judgement | note | CONFIRMED | introduced | The comment says "`filed:` appears three times, so it locates nothing". `filed:` appears **zero** times in `repeated_names.yaml` — the file writes `- name: filed`. The line the test pins is right; the reason given for it is not, and the next reader will trust the reason. |
| F5 | `docs/design/review-typed-diagnostics.md:134` | judgement | note | CONFIRMED | introduced | The inventory row for `Outcome::try_from`/`subject_of` says 7 sites; the tree has 8 (`command.rs:2168, 2268, 2295, 2311, 2353, 2377, 2392, 2419`). Whole-file check: `grep -c ValidationError::new command.rs` = 11 and `::at` = 28 (39 at base, all `new`); the inventory's command.rs rows account for 2+1+7 = 10. The rest of the counts reconcile (the payload row's "7" is right: `refuse` at `:1876` is called twice, `:1900` and `:1906`). |
| F6 | `crates/specify/ess-compiler/src/resolve.rs:607` | judgement | note | CONFIRMED | introduced | `family_of_kind` is documented as "a total match on the kind, with no … unrecognised head to fall back from", but it ends in `_ => codes::family::SPEC` and never names `ConstructKind::Specification`. On a `#[non_exhaustive]` enum that is the same silent fallback the page says the typed path removed: a variant added later becomes `SPEC` with nothing failing. Nothing reaches it today — only `ConstructKind::Command` is produced. |

## 5. Attacked and could not break

- **The acceptance statement itself.** Rewriting a sited error's `location` changes neither the emitted `Code` nor `span.located` nor `span.path` for migrated rules — `family_of_kind` and `needles_of_site` never read the string. The migrated path is sound.
- **`location == site.construct.render()`.** The only writer of `.location` outside tests is `command.rs:2444` (the `Outcome::try_from` rebase), and every error reaching it is built by `ValidationError::new` with no site, so the invariant holds in-tree today. `location` being `pub` leaves it unguarded for the next unit.
- **Rendered-string parity.** All 28 `ValidationError::at` sites render byte-identically to the `format!` they replaced; the guide sample and the 48 in-crate `.location` assertions are untouched.
- **`serde(skip)` round-trip.** A round-tripped `ValidationError` loses its site and compares unequal to the original, but nothing in the workspace deserializes one (`from_str::<Validation…>`/`from_value::<Validation…>`: 0 hits; the only cross-crate use, `ess-diff/tests/canonical.rs:1009`, constructs and never round-trips). INFEASIBLE, not raised.
- **`SyntaxSpan` honoured verbatim; index/empty-name boundaries in `render`; `size_of::<ValidationError>()` under the 128-byte lint bound** — all hold.
- **`task consumer-check` mutant** (removing a classification entry): not probed. The file is an implementation-owned data file and the checker reads it from the crate path, so probing it would have required mutating a file under attack.

## 6. Paths written outside the worktree

None. One file was written inside the assigned scratch directory: `target/review-boundaries-21/scratch/resolve-base.rs` (the base copy of `resolve.rs`, used to prove `needles_for`/`STRUCTURAL` are unchanged).

```findings
- file: crates/specify/ess-compiler/src/resolve.rs
  line: 632
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'the design page claims needles_of_site derives the same needles as needles_for, but a qualified event name starts lowercase so the typed path pushes a "shop.x.Event:" needle the string path never pushes, and the two cite different lines for one refusal'
- file: crates/specify/ess-compiler/src/resolve.rs
  line: 634
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'the Key/Name split does not replace the STRUCTURAL stop-list for Name segments, so an outcome named with one of the fifty words is cited at a sibling outcome''s structural key instead of the command''s own declaration line'
- file: docs/design/review-typed-diagnostics.md
  line: 161
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'the page says repeated_names.yaml asserts the honest `located: None`, and every refusal that fixture produces is located at 12:5 with the shipped test pinning it there'
- file: crates/specify/ess-compiler/tests/typed_diagnostics.rs
  line: 179
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'the comment justifying the pinned line says "`filed:` appears three times" and that string appears zero times in the fixture'
- file: docs/design/review-typed-diagnostics.md
  line: 134
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'the inventory row for Outcome::try_from/subject_of counts 7 remaining string sites where command.rs has 8, leaving the file''s eleven ValidationError::new sites accounted as ten'
- file: crates/specify/ess-compiler/src/resolve.rs
  line: 607
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'family_of_kind is documented as a total match with no unrecognised fallback but ends in `_ => SPEC` and never names ConstructKind::Specification, so a future variant silently becomes SPEC'
```
