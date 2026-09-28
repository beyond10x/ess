---
format: aep.planning-md/3
id: review-result:adversary-c1-trailing-pass-2
kind: review-result
status: active
title: Adversary pass 2, wave correctness-1 unit trailing
relations:
- reviews: story:a-wrong-trailing-key-guess-is-reported-as-a-line
revision: 1
---
unit: story:a-wrong-trailing-key-guess-is-reported-as-a-line
verdict: red
cases: executed 215→218, red 3
origin: introduced 2, pre-existing 1, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-c1/trailing/adv2/ (base export, surveys, logs)
needs-coordinator: no

Adversary pass 2 (`aep:adversary`), 2026-09-28, head 4602cc32c + untracked `crates/specify/ess-compiler/tests/trailing_key_guess_adversary_pass2.rs`. Pass-1 findings fixed: all 5 pass-1 cases green.

| case | asserts | now | base b5c53e8a1 |
|---|---|---|---|
| `a_precondition_input_key_is_cited_where_it_is_written` | `system.preconditions[0].input.colour` cited at outcome-shapes.yaml:7 | red: `("<document>", None)` | passes |
| `a_relation_refusal_is_not_cited_in_a_file_that_holds_no_refusal` | billing relation renamed + undeclared target not cited in components.yaml | red: components.yaml:61:7 | fails the same way |
| `a_right_key_above_the_declaration_in_a_bare_dash_item_is_still_cited` | bare `-` item, key above `id:` cited at line 4 | red: 5 vs 4 | passes |

`cargo test -p ess-compiler --no-fail-fast`: 215 passed, 3 failed, 2 ignored, EXIT=101.

- F1 introduced (resolve.rs:949 NAMED_LISTS): `Precondition.input` is a map (`system.rs:296`), keys written `field: value`; the guess is dropped, refusal falls to `<document>`. Reached by 4 refusal paths in outcome_shapes.rs:687,706,722,737; fixtures outcome-shapes.yaml, explore-preconditions.yaml.
- F2 pre-existing (resolve.rs:1044): `relations` in neither NAMED_LISTS nor STRUCTURAL; entity relation refusal cited first-unique in a file holding no refusal (billing components.yaml:61). Same shape untested: `attributes` (actor), `response` (command).
- F3 introduced note (resolve.rs:586-597 encloses): only `- ` with a space opens an item; bare `-` item coarsens a key above the declaration.

Could not break: one-line-edit survey over examples and 110 fixtures (10,206 citations): 18 + 34 differences, all wrong base citations now on the refused declaration, except two precondition deletes (`<document>` more honest). Topology workload keys and sibling outcome payload keys fixed generally. input_absent, fixture_inputs, input.<f>.example, variants.<k> keep matchable guesses. Flow items, nested/compact lists, CRLF, comments.

```findings
[{"file":"crates/specify/ess-compiler/src/resolve.rs","line":949,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"NAMED_LISTS drops the guess after every input key, but a system precondition input is a map written <field>: <value>, so system.preconditions[i].input.<field> refusals the base cited at their key now fall to <document>"},{"file":"crates/specify/ess-compiler/src/resolve.rs","line":1044,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"relations is in neither NAMED_LISTS nor STRUCTURAL, so an entity relation refusal has no matchable declaration needle and its never-matching <relation>: guess is cited first-unique in a file holding no refusal (billing: components.yaml:61)"},{"file":"crates/specify/ess-compiler/src/resolve.rs","line":586,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"encloses only recognises an item opener written - with a trailing space, so in a bare - item a correct key written above the declaration key is coarsened to the declaration line"}]
```

Trend: pass 1 → 3 findings, pass 2 → 3 findings, 0 carried (all pass-1 fixed). Coordinator routing: correction 2 to the same implementor; the coordinator verifies that correction by reading the diff (no third attack).
