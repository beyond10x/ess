---
format: aep.planning-md/1
id: review-result:typed-diagnostics-adversary-wave21-pass2
kind: review-result
status: active
title: Typed diagnostics adversary, wave 21, pass 2
relations:
- reviews: story:review-typed-diagnostics
revision: 1
---
unit: story:review-typed-diagnostics — worktree ess-typed-diagnostics-wave21, HEAD 1b0368b + one untracked test file, base 2900f628
verdict: red
cases: executed 520→525, red 5
origin: introduced 5, pre-existing 0, undecided 0
wrote-outside-worktree: none
needs-coordinator: no

Recorded by the wave-21 coordinator from the aep-drive:adversary 0.8.1 (Opus) report as returned, pass 2, 2026-09-09. Harness accounting: 160,136 sub-agent tokens, 52 tool uses, 14.0 min. Workstation path prefixes removed; nothing else changed.

## 1. `git --no-pager diff --stat`

```
$ git --no-pager diff --stat HEAD
(empty)
$ git status --short
?? crates/specify/ess-compiler/tests/adversary_typed_diagnostics_pass2.rs
```

One path, untracked (the coordinator commits): a test file. No implementation file, no fixture, no `docs/`, no `.engineering/planning/` file touched. `cargo fmt -p ess-primitives -p ess-domain -p ess-compiler -- --check` exit 0; `cargo clippy --locked -p ess-primitives -p ess-domain -p ess-compiler --all-targets -- -D warnings` exit 0.

## 2. The cases (written first, each run alone, red output verbatim)

All five are in `crates/specify/ess-compiler/tests/adversary_typed_diagnostics_pass2.rs`. All five are red now.

**C1 `every_command_family_refusal_carries_a_site_as_the_migration_table_says`** — the page says the `command` family is migrated; every `command.…` refusal must carry a typed site.
```
test every_command_family_refusal_carries_a_site_as_the_migration_table_says ... FAILED
panicked at crates/specify/ess-compiler/tests/adversary_typed_diagnostics_pass2.rs:149:5:
the design page says the `command` family is migrated; these `command.…` refusals carry no typed site and are still bridged from their string: ["command.shop.wrong.Touch.outcomes.wrong-state"]
test result: FAILED. 0 passed; 1 failed; 4 filtered out.   exit=101
```

**C2 `rewording_the_path_of_a_command_family_refusal_does_not_move_its_code`** — the unit's own transformation (`resolve.rs::a_sited_refusal_takes_its_family_from_the_construct_not_the_location_head` rewrites the location head to `reworded.`), applied to a `command`-family refusal.
```
test rewording_the_path_of_a_command_family_refusal_does_not_move_its_code ... FAILED
panicked at ...pass2.rs:176:5:
assertion `left == right` failed: rewording the human-facing path changed the machine code of a `command`-family refusal
  left: "ESS-SPEC-012"
 right: "ESS-COMMAND-012"
test result: FAILED. 0 passed; 1 failed; 4 filtered out.   exit=101
```

**C3 `a_deferred_family_location_is_renderable_by_a_construct_reference`**
```
test a_deferred_family_location_is_renderable_by_a_construct_reference ... FAILED
panicked at ...pass2.rs:204:5:
assertion `left == right` failed: `entity.rs` is inventoried as the same shape as `command.rs`, and no `ConstructRef` renders the string it writes
  left: "entity.shop.wrong.Order.transitions[0]"
 right: "entity shop.wrong.Order.transitions[0]"
test result: FAILED. 0 passed; 1 failed; 4 filtered out.   exit=101
```

**C4 `the_page_and_the_source_agree_on_whether_construct_kind_is_non_exhaustive`**
```
test the_page_and_the_source_agree_on_whether_construct_kind_is_non_exhaustive ... FAILED
panicked at ...pass2.rs:240:5:
assertion `left == right` failed: the design page's `ConstructKind` row says `#[non_exhaustive]` is true and `error.rs` declares it false
  left: true
 right: false
test result: FAILED. 0 passed; 1 failed; 4 filtered out.   exit=101
```

**C5 `the_repeated_name_fixture_locates_at_least_one_of_its_refusals`**
```
test the_repeated_name_fixture_locates_at_least_one_of_its_refusals ... FAILED
panicked at ...pass2.rs:267:5:
no refusal from the repeated-name fixture carries a source location, so the suite checks no location for the hazard the story names: [("command.shop.repeat.File.input[1]", None), ("command.shop.repeat.File.outcomes.filed", None), ("command.shop.repeat.File.outcomes", None)]
test result: FAILED. 0 passed; 1 failed; 4 filtered out.   exit=101
```

## 3. The suite, after the cases existed

```
$ TMPDIR=... cargo test --locked --no-fail-fast -p ess-primitives -p ess-domain -p ess-compiler
...
failures:
    a_deferred_family_location_is_renderable_by_a_construct_reference
    every_command_family_refusal_carries_a_site_as_the_migration_table_says
    rewording_the_path_of_a_command_family_refusal_does_not_move_its_code
    the_page_and_the_source_agree_on_whether_construct_kind_is_non_exhaustive
    the_repeated_name_fixture_locates_at_least_one_of_its_refusals
test result: FAILED. 0 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out
exit=101
```
Summed over all 19 `test result:` lines: **520 passed, 5 failed, 525 executed**. `<before>` = 520 is the implementor's own count from `target/review-boundaries-21/scratch/c1-test.txt` (18 `test result:` lines, 520 passed); the suite was not run before my cases existed.

`--no-fail-fast` is required: without it cargo stops at my binary and only 41 cases execute.

## 4. Findings

| ID | file:line | category | severity | verdict | origin | finding |
|---|---|---|---|---|---|---|
| F1 | `docs/design/review-typed-diagnostics.md:126` | contract-drift | blocker | CONFIRMED | introduced | "**Migrated (this wave):** the `command` family" is false. `entity.rs:935`, `:1072`, `:1126` write `command.…` locations with `ValidationError::new`; the inventory books all 20 `entity.rs` sites as entity-family work (`:166`), so these three have neither a typed replacement nor the explicit unsupported result the story's compatibility clause requires. **Measured:** `pass2.rs:149`, exit 101 — `ESS-COMMAND-012` at `command.shop.wrong.Touch.outcomes.wrong-state`, `site()` is `None`. **What reaches it:** an ordinary specification — a command with a `wrong_state:` branch and no `moves:`; reproduced through the shipped CLI (`ess validate`) before any test was written. |
| F2 | `crates/specify/ess-domain/src/entity.rs:1072` | acceptance | blocker | CONFIRMED | introduced | The acceptance statement does not hold for that refusal: rewriting the human-facing path's head moves the emitted code `ESS-COMMAND-012` → `ESS-SPEC-012`. The `format!` at `:1072` is the wording, and `family_of` still parses it. **Measured:** `pass2.rs:176`, exit 101. **What reaches it:** editing the path prose at `entity.rs:1072` — exactly the edit F14 says must be safe. The refusal itself is pre-existing at the base; what is introduced is the page declaring its family migrated. **Fix (not applied):** migrate `entity.rs:935/:1072/:1126` to `CommandSpec::site()`, or move them to a `command`-family row of the inventory with an explicit result. |
| F3 | `docs/design/review-typed-diagnostics.md:166` | contract-drift | warning | CONFIRMED | introduced | The deferred families are recorded as "Same shape as `command.rs`". `entity.rs` writes `entity <name>` (`:830`, `:1004`, `:1215`, `:1490`) and `component.rs` writes `component <name>` (11 sites) — a space, which `ConstructRef::render` cannot produce. That is the class the page's own `validate_sets` row (`:162`) calls "a location change, not a wording change ... outside this story's acceptance", so at least 35 of the 102 deferred sites carry the wrong result. **Measured:** `pass2.rs:204`, exit 101. **What reaches it:** the next unit of the migration, which takes this row as its brief. |
| F4 | `docs/design/review-typed-diagnostics.md:34` | contract-drift | warning | CONFIRMED | introduced | The type table says `ConstructKind` is `#[non_exhaustive]`; `error.rs:~360` declares it without, and `:89` plus the type's doc comment say the omission is deliberate and give the reason (a downstream `_` arm is how a kind reaches `family::SPEC` unnoticed). The summary row is what a reader adding a kind reads first, and following it reintroduces the defect the correction commit removed (pass 1 F6). **Measured:** `pass2.rs:240`, exit 101. **What reaches it:** any reader of the page. |
| F5 | `crates/specify/ess-compiler/tests/fixtures/typed_diagnostics/repeated_names.yaml:12` | acceptance | warning | CONFIRMED | introduced | The correction answered pass 1 F3 by editing the evidence: it renamed the fixture's command `shop.repeat.FileOne` → `shop.repeat.File`, making the fallback needle a substring of `shop.repeat.FileTwo` and `shop.repeat.Filed`, so all three refusals became `located: None`. The suite now pins **no** source location for a repeated-name defect anywhere, while the story's Validation clause requires repeated names to "retain correct codes/locations" and its Implementation boundary requires syntax spans "for repeated-name and multi-file cases". **Measured:** `pass2.rs:267`, exit 101. **What reaches it:** the story's own close condition. **Fix (not applied):** keep the unlocated fixture and add a second repeated-name fixture whose refusal is located, restoring the assertion the rename deleted. |
| F6 | `crates/specify/ess-primitives/src/error.rs:628` | judgement | note | INFEASIBLE | introduced | `location` stays `pub` while `site` is private, and nothing keeps them in step: `every_site_renders_its_own_location` is a test helper, not a guard. `command.rs:2444` already mutates `error.location` by string concatenation on every error from `Outcome::try_from`, so the first migrated admission site — which the page names as "the next unit of this migration" — desynchronises the printed location from the cited path silently. **What reaches it:** nothing today (all 8 admission sites are `ValidationError::new`); I constructed the state, so this is a forward hazard, not a live defect. |
| F7 | `crates/specify/ess-compiler/src/resolve.rs:647` | mutant | note | INFEASIBLE | introduced | The `Segment::Index` arm of `needles_of_site` cannot affect any needle for a construct a producer builds: a digit is never key-like, and every producer places a `STRUCTURAL` key (`input`, `outcomes`) before the index, so `declared` stops before it. Deleting the arm leaves `needles_of_site(c) == needles_for(&c.render())` true and the suite green. The positional fact the typed site added is discarded before the line is chosen. Read, not run. |
| F8 | `crates/specify/ess-primitives/src/error.rs:695` | judgement | note | CONFIRMED | introduced | `with_span` silently drops the span on an unsited error. It is `#[must_use]` and returns `Self`, so `ValidationError::new(..).with_span(s)` compiles, type-checks and loses the parser position with no diagnostic — the one API the page says exists so "the first producer able to supply a position needs no further change". No caller does this today. |

## 5. Attacked and could not break

- **The single needle derivation (brief line 1).** `needles_of_site(c) == needles_for(&c.render())` holds by construction for every input, including index segments, empty names, names containing `.`/`[`/`]`/space, a `Key` that is a stop-list word, and a qualified name with a capitalised last component — `split_path` and `needles_for`'s tokenizer are the same character set and both drop empties, and `render` inserts only those characters. No divergence exists to find.
- **The inventory test (brief line 2).** A file listed with `0 0` fails (it is excluded from `measured`), a renamed file fails, and `matches()` vs `grep -c` agree on every one of the 14 files. `ess-domain/src` has no subdirectories, so the non-recursive `read_dir` has no live gap.
- **`ConstructKind::ALL` / `family_of_kind` (brief line 3).** Not vacuous: `error.rs` pins `ALL.len() == 13`, the match is exhaustive so a new variant is a compile error, and the both-ways family check catches a duplicated entry in `ALL`.
- **Dropping `#[non_exhaustive]` (brief line 4).** The only downstream matches are `ConstructKind::as_str` and `resolve.rs::family_of_kind`, both in-repo and both exhaustive. Nothing warns or errors.
- **`location` byte-identity across the 28 `at` sites (brief line 7).** Every migrated site renders exactly the string its `format!` produced at 2900f628; I checked all twelve functions against `git show 2900f628:command.rs`.
- **Locator wrong-file citation.** `scan` requires global uniqueness across every searched file and returns `None` on a second hit, so a cross-file needle cannot pick the wrong file.
- **Serialization.** `#[serde(skip)]` on `site` keeps bytes and schema identical; the existing case covers it.

## 6. Paths written outside the worktree

None. `TMPDIR` was used for every cargo command and is empty. Scratch (inside the worktree, under the assigned root): `target/review-boundaries-21/scratch/adv2/` — `p1/`, `p2/` (probe specifications run through `./target/debug/ess validate`), `suite.txt`, `suite-nff.txt`.

## 7. Findings block

```findings
- file: docs/design/review-typed-diagnostics.md
  line: 126
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'the page declares the command family migrated while entity.rs:935, :1072 and :1126 still write command.… locations with ValidationError::new, booked under the entity.rs inventory row, so three command-family paths have neither a typed replacement nor an explicit unsupported result'
- file: crates/specify/ess-domain/src/entity.rs
  line: 1072
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'rewording the human-facing path of this command-family refusal moves its machine code from ESS-COMMAND-012 to ESS-SPEC-012, which is the acceptance statement failing on a family the design page says was migrated'
- file: docs/design/review-typed-diagnostics.md
  line: 166
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'the deferred families are recorded as the same shape as command.rs, but entity.rs writes "entity <name>" and component.rs writes "component <name>" with a space, which ConstructRef::render cannot produce — the class the page itself calls a location change and puts outside this story acceptance'
- file: docs/design/review-typed-diagnostics.md
  line: 34
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'the type table row says ConstructKind is #[non_exhaustive] while the source declares it exhaustive on purpose and the page prose at :89 says so, and following the row reintroduces the silent family::SPEC fallback the correction commit removed'
- file: crates/specify/ess-compiler/tests/fixtures/typed_diagnostics/repeated_names.yaml
  line: 12
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'the correction renamed the fixture command from shop.repeat.FileOne to shop.repeat.File so every refusal became unlocated, leaving no assertion anywhere in the suite that pins a source location for a repeated-name defect, which the story Validation clause requires'
- file: crates/specify/ess-primitives/src/error.rs
  line: 628
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: 'location stays pub with no guard keeping it in step with the private site, and command.rs:2444 already mutates it, so the first migrated admission site desynchronises the printed location from the cited path; no caller reaches that state today'
- file: crates/specify/ess-compiler/src/resolve.rs
  line: 647
  category: mutant
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: 'the Segment::Index arm of needles_of_site can be deleted with the suite still green because a digit is never key-like and every producer places a STRUCTURAL key before the index, so the positional fact the typed site adds is discarded before the line is chosen'
- file: crates/specify/ess-primitives/src/error.rs
  line: 695
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'with_span silently discards the parser position on an unsited error, so ValidationError::new(..).with_span(s) compiles and loses the one fact the type was added to carry'
```
