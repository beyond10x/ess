---
format: aep.planning-md/3
id: review-result:adversary-d343-u4-pass-1
kind: review-result
status: active
title: 'D343 U4 adversary pass 1: short names keep a body root''s owner'
relations:
- reviews: story:short-names-combine-with-all-commands-and-all-views
revision: 1
---
unit: story:short-names-combine-with-all-commands-and-all-views, tree ess-d343-u4 at c612f4dc7 (plus my untracked test file)
verdict: NEEDS-CHANGE
cases: executed 286→291, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: none

**1. `git --no-pager diff --stat`**: empty. The tree has no changes to tracked files. `git status` shows one untracked file, `?? crates/generate/schema-contract/tests/adversary_d343_u4_pass1.rs`, which is a test file.

**2. Cases** in `crates/generate/schema-contract/tests/adversary_d343_u4_pass1.rs`:
- `a_declared_type_named_row_keeps_its_last_segment_under_all_types`: **red**. With every type selected, the type `demo.notes.row` should keep `Row`. The parser accepts lower-case name segments (`ess_domain/src/name.rs:22`).
- `a_declared_type_is_named_alike_whether_selected_or_reached`: **red**. The same type gets two different names, depending on whether it is selected or only reached from another selected type.
- `an_event_named_input_keeps_its_last_segment_under_all_events`: **red**. The event `demo.notes.input` should keep `Input`.
- `a_body_root_shadowing_a_declared_type_is_refused_naming_both`: green. My first version was red for a reason in my own fixture: both definitions were in one domain, so the qualified names `DemoNotesCreateInput` already collided in `from_model`. That is not a finding. I moved the type to `demo.other` and the case is now green.
- `every_target_declares_body_roots_of_admitted_owner_spellings`: green. Owners `archive-note` and `by_owner` become `ArchiveNoteInput`, `ArchiveNoteResponse` and `ByOwnerRow`, declared as `pub struct` (Rust), `type … struct` (Go) and `export type … = {` (TypeScript). None of the three outputs contains `DemoNotes`.

Red output from running this file alone, before the suite (`cargo test -p schema-contract --locked --test adversary_d343_u4_pass1`, EXIT=101):
```
a_declared_type_named_row_keeps_its_last_segment_under_all_types  (:73)  left: "NotesRow"  right: "Row"
an_event_named_input_keeps_its_last_segment_under_all_events      (:107) left: "NotesInput" right: "Input"
a_declared_type_is_named_alike_whether_selected_or_reached        (:93)  left: "Row"  right: "NotesRow"
test result: FAILED. 1 passed; 4 failed   (4th = shadowing fixture error, fixed above)
```

**3. Suite** (run after the cases existed): `build-slot.sh cargo test -p schema-contract --locked --no-fail-fast` gave EXIT=101 with 288 passed and 3 failed. The only failing target is `-p schema-contract --test adversary_d343_u4_pass1` (2 passed, 3 failed). The before-count of 286 is the 291 executed cases minus my 5, all from this same run.

**4. Findings** (they cover c612f4dc7):
- **F1** `crates/generate/schema-contract/src/realize.rs:2060`: NEEDS-CHANGE, introduced, in scope. `short_name` decides a definition is a body root from two facts: it is a selected root, and its last segment is `input`, `response` or `row`. But `ModelTypes::select` resolves a declared type or event before it tries a body root (`ess-gen/src/model_types.rs` `select_root`). So under `--all-types` or `--all-events`, a type or event with a lower-case concept segment gets the owner prefix. This breaks the decision that type and event short names are unchanged. The base computed `declaration_name(last)` for every source (`git show 1628ffb34`, line 886), so the defect is new. What reaches it: `--names short --all-types` or `--all-events`, or `--root` naming such a type. Suggested fix: have `ModelTypes` expose the body roots it actually resolved (the `bodies` keys), and key `short_name` on that set instead of on the suffix.
- **F2** `crates/edge/ess-cli/src/model_types.rs:59`: NEEDS-CHANGE, introduced, in scope. The `Names::Short` help text still says "The last segment", which is no longer true for body roots. The story's scope already flagged this line as "inferred". It is the only copy of that text in the repo.
- **F3** `crates/generate/schema-contract/tests/short_body_root_names.rs` (TypeScript checks in `all_commands_with_short_names_generates`): CONFIRMED, introduced, in scope, note. `typescript.contains("CreateResponse")` and `contains("SummariesRow")` would also pass with qualified names, because `DemoNotesCreateResponse` contains `CreateResponse`. So that test does not prove TypeScript gets short names. My green case `every_target_declares_body_roots_of_admitted_owner_spellings` asserts the exact `export type` lines and the absence of `DemoNotes`.

**5. Attacked, could not break**: a body root that shadows a declared type in another domain is refused, naming both. Hyphenated, underscored and lower-case owners give valid names in Rust, Go and TypeScript. The qualified default is unchanged, and the unit's tests already cover the same command name in two domains. The `ess generate client` publisher selects only events, so it is untouched.

**6. Paths written outside the worktree**: the unit's adversary scratch directory (`case-alone.log`, `suite.log`). The build wrote only to the tree's own `target/`. I took a session lease (`d343-u4-adversary-pass1`) and released it.

```findings
[
  {"file": "crates/generate/schema-contract/src/realize.rs", "line": 2060, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "F1 (in scope): short_name treats any selected root ending in input/response/row as a body root, so under --all-types/--all-events a declared type demo.notes.row becomes NotesRow and an event demo.notes.input becomes NotesInput, and one type gets two names depending on whether it is selected"},
  {"file": "crates/edge/ess-cli/src/model_types.rs", "line": 59, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "F2 (in scope): the --names short help still promises the last segment, which body roots no longer get"},
  {"file": "crates/generate/schema-contract/tests/short_body_root_names.rs", "line": 79, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "F3 (in scope): the TypeScript assertions are substring checks that also pass for qualified names such as DemoNotesCreateResponse, so they cannot fail if TypeScript ignores short names"}
]
```
