---
format: aep.planning-md/3
id: review-result:zi-t2-datetime-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:the-date-time-agreement-oracle-refuses-non-digits
relations:
- reviews: story:the-date-time-agreement-oracle-refuses-non-digits
revision: 1
---
```
unit: story:the-date-time-agreement-oracle-refuses-non-digits (tree zi-t2-9, HEAD f4bc600ce9, plus my one uncommitted test file)
verdict: weak
new failing cases: none
mutants survived: 7 (month `_ => return false` -> 31; day `1..=last` -> `0..=last`; `minute > 59` -> `> 60`; `second > 60` -> `> 61`; century rule `year % 100 != 0` -> `true`; `offset_hour > 23` -> `> 24`; `offset_minute > 59` -> `> 60`)
```
`git diff --stat` is empty because my file is untracked. `git status` shows `?? crates/generate/schema-contract/tests/adversary_t2_datetime_rfc3339.rs` (mine, a test file) and `security_t2_datetime_conformance.rs` (the security reviewer's, not touched). No non-test path was changed.

1. **No red case; the checker holds.** A first version of my test compared the checker with jsonschema 0.52.1's own `date-time` check restricted to ASCII digits (`adv/run1-case.txt`, 2 passed, exit 0). The candidates were ten valid seeds, each with every one-byte replace, delete, insert and truncation (multibyte and NUL bytes included), plus full sweeps of every field's value range. The two matched exactly with no panic. So the checker still accepts everything the dependency accepted that is genuinely valid: leap seconds with ±offsets, fractions, lowercase t/z, ±23:59.
2. **That version tripped an existing guard, not the code under attack.** `schema_unique_items.rs:347` forbids tests that build the dependency's own validator (`adv/run5-suite.txt`, exit 101). I saved it as `adv/adversary_t2_datetime_rfc3339.v1-upstream-oracle.rs` and rewrote it to use an RFC 3339 reference written in the test instead. The kept file also checks the `uniqueness::options()` validator against that reference.
3. **The kept file passes and pins the surviving mutants.** I ran 7 mutants on copies of `date_time.rs` with plain rustc (`adv/mutants.sh`). The unit's own test at `date_time.rs:116` stays green under all 7. Nothing else in the suite tests those ranges either: `date_time_digits` and the closure test bodies don't. My literal case `the_field_ranges_are_rfc_3339s` kills each one (`adv/kill.sh`, for example `2026-13-01T00:00:00Z admitted`). This is next to the acceptance, not in it: a backlog item, not a blocker.
4. **Harness case: passed.** Ran in this order:
   - Built `ess` with `build.sh … build --workspace --exclude ess-xtask --locked --bin ess` (`adv/run2-ess.txt`, exit 0).
   - Generated the server and built the closure harness with a direct cargo build, target dir under `adv/` (`adv/harness.sh`, `adv/run3-harness.txt`, exit 0).
   - Ran `ESS_JSON_BODY_CLOSURE_HARNESS=adv/json-body-closure-harness build.sh … test -p ess-synth --locked --test json_body_closure` (`adv/run4-closure.txt`): 4 passed, including `the_server_refuses_exactly_what_its_served_openapi_refuses`, exit 0.
5. **Final suite run: green.** `test -p schema-contract --locked --no-fail-fast --lib --test date_time_digits --test schema_unique_items --test schema_unique_items_adversary --test bundle --test adversary_t2_datetime_rfc3339` (`adv/run6-suite.txt`): 28+2+17+1+6+10 passed, exit 0, slot A. Cases executed 62→64 (before = this run minus my file), red 0. Six cargo runs used in total.
6. **No emitted bytes change.** The emitted normalization runtimes (`rust_runtime.rs.txt` and the legacy ones) build their own options with formats off. `bundle.rs` and `source.rs` also have formats off. Only `validate.rs` asserts formats. The generated server's `timestamp_at` (`ess-synth/src/rust/json.rs:839`) uses the same rules as the new checker.
7. **Couldn't break:** panics on short, odd or multibyte input; the leap-second offset arithmetic; the `T` split position; non-strings (still pass); wiring through `options()`.
8. **Files written outside the worktree** (all under the scratch dir `the-date-time-agreement-oracle-refuses-non-digits/`):
   - `adv/` with `mut/`, `harness/` (generated server and its `target/`), `tmp/`
   - `adv/ess`, `adv/json-body-closure-harness`
   - `adv/run{1..6}*.txt` and `adv/*.touch`
   - `adv/*.sh`, `adv/rewrite.py`, `adv/*.pid`, the v1 test copy

```findings
- file: crates/generate/schema-contract/src/date_time.rs
  line: 116
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the checker's only unit test names no month, day-zero, minute, second-61, century-leap or offset-range input, so seven range mutants survive the unit's whole suite; adversary_t2_datetime_rfc3339.rs::the_field_ranges_are_rfc_3339s now kills each.
```
