---
format: aep.planning-md/3
id: review-result:zi-t2-uitest-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:a-ui-test-declares-how-each-read-and-command-is-answered
relations:
- reviews: story:a-ui-test-declares-how-each-read-and-command-is-answered
revision: 1
---
unit: story:a-ui-test-declares-how-each-read-and-command-is-answered (commit e65d9d65cd plus my one untracked test file)
verdict: broken
new failing cases: 2 (adversary_t2_fixture_answers_attack.rs::an_unknown_key_in_a_list_answer_is_refused, adversary_t2_fixture_answers_attack.rs::a_list_answer_whose_by_params_is_not_a_map_is_refused)
mutants survived: 3, found by reading the code and not run: in `parity::answers`, removing the `fails` check, the `view_latency` check or the `lists` check each leaves `fixture_answers` green, because its one spec test combines a list and a command
cases: executed 9→17, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths (listed below)
needs-coordinator: none

1. `git status --short` shows only `?? crates/ui/ess-ui-test/tests/adversary_t2_fixture_answers_attack.rs`, and `git diff --stat` is empty. Every path I touched is a test file.

2. **Blocker: a misspelled key inside a list answer is silently accepted.** In `crates/ui/ess-ui-test/src/spec.rs:412`, each list answer is only checked for `rows`. A typo such as `{by_param: {order: o-3}, rows: […]}` passes validation, so the keyed answer becomes the fallback and answers every read. The test then runs against the wrong data and nothing says so. The acceptance says an unknown key under `fixtures` is still refused, so this is in scope.
   - Red output (run1.txt): `` `by_param` in a list answer was accepted `` at the new test file, line 82.
3. **Warning: `by_params` that is not a map is also accepted.** `by_params: o-3` is read as no `by_params` at all (`crates/ui/ess-ui-tui/src/data.rs:386`), so that answer also becomes the fallback for every read.
   - Red output (run1.txt): `` `by_params: o-3` was accepted and answers every read `` at line 101.
   - Fix for both: in `parse_fixtures`, apply `known(item, &["by_params","rows","total"])` to each list item and require `by_params` to be a map. This is a suggestion; I did not change any implementation file.
4. **Weak unit test.** The name check in `fixture_answers.rs:186`, `contains("odd")`, is already satisfied by the file name `odd.test.yaml`, so it cannot catch a missing test name. My command-key case uses distinct file and test names, and it passes.
5. Suite run (run3.txt), command `test -p ess-ui-test --locked --no-fail-fast --test fixture_answers --test adversary_t2_fixture_answers_attack`:
   - My file: `test result: FAILED. 6 passed; 2 failed`.
   - `fixture_answers`: `test result: ok. 9 passed`.
   - `exit 101`, `slot B waited 20s ran 15s`.
   - run2.txt is the same command without `--no-fail-fast`. It stopped after my file and never ran `fixture_answers`.
6. These attacks did not break anything (all green in my file, and they now pin behaviour the suite did not):
   - **by_params:** numbers and strings compare as text; an answer keyed on a param the read does not ask is not chosen; when two answers match, the first wins.
   - **`fails`:** it fails two sections reading the same view and also a placeholder.
   - **`refuses`:** the command is still logged, and the refusal plus its payload show in the record where the user acted.
   - **Latency per view:** a view stays loading through a partial `advance`, an unnamed view answers at once, and a placeholder can have its own latency.
   - **Playwright fixme:** each new key on its own gives exactly one `test.fixme` with the right reason.
7. **Playwright output and the 0.92.0 map form, checked by reading only.** For a test without the new keys, the output should be byte-identical to the base:
   - The only change in `emit_test` is an early return that fires only when `answers()` finds a new key.
   - The new key shapes were refused when parsed at the base.

   The 0.92.0 map form reads as before, apart from the placeholder override the acceptance asks for. One small exception: a document whose fixture index lists a file without rows for a view that a test replaces now fails at load. Before, the replacement meant that file was never read. I found no realistic setup that does this.
8. Paths written outside the worktree: `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/a-ui-test-declares-how-each-read-and-command-is-answered/adv/{run1.txt,run2.txt,run3.txt,tmp/}`.

```findings
- file: crates/ui/ess-ui-test/src/spec.rs
  line: 412
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: an unknown key inside a list answer (e.g. `by_param`) is accepted and silently turns the keyed answer into the fallback for every read, while the acceptance says unknown keys under fixtures are refused with file, test and key
- file: crates/ui/ess-ui-tui/src/data.rs
  line: 386
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a list answer whose `by_params` is not a map passes validation and is read as having no by_params, so it answers every read
- file: crates/ui/ess-ui-test/tests/fixture_answers.rs
  line: 186
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the unit's suite would stay green if the error dropped the test name (contains("odd") is met by the file name) or if parity::answers dropped its fails, view_latency or lists check, since its only spec case combines a list and a command
```
