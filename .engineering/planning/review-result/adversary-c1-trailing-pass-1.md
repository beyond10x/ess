---
format: aep.planning-md/3
id: review-result:adversary-c1-trailing-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave correctness-1 unit trailing
relations:
- reviews: story:a-wrong-trailing-key-guess-is-reported-as-a-line
revision: 1
---
unit: story:a-wrong-trailing-key-guess-is-reported-as-a-line (tree ess-c1-trailing, head 08241ecc6 + 1 untracked test file)
verdict: red
cases: executed 210→215, red 4
origin: introduced 3, pre-existing 1, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-c1/trailing/adv1/suite.log
needs-coordinator: no

Adversary pass 1 (`aep:adversary`), 2026-09-28. Cases in `crates/specify/ess-compiler/tests/trailing_key_guess_adversary.rs`.

| case | asserts | now |
|---|---|---|
| `a_stateful_workload_in_the_billing_example_is_cited_at_its_workload_key` | billing with `stateless: false` on `invoice-service` cited at `topology.yaml:7` | red: `("<document>", None)` |
| `a_workload_naming_an_undeclared_component_is_cited_at_its_workload_key` | billing with `email-service:` → `mail-service:` cited at `topology.yaml:15` | red: `("<document>", None)` |
| `an_outcome_refusal_is_not_cited_at_a_sibling_outcomes_payload_key` | `command.shop.probe.Doit.outcomes.filed` not cited at outcome `other`'s payload key | red: cited `a.yaml:28` |
| `a_right_key_written_above_the_declaration_key_is_still_cited` | binding item `- mapping: {recipient:…}` with `id:` below cited at `recipient:` | red: line 4 vs 3 |
| `crlf_comments_and_blank_lines_keep_the_block_and_the_next_item_closes_it` | CRLF, column-0 comment, blank line keep the block | green |

`cargo test -p ess-compiler --no-fail-fast`: 211 passed, 4 failed, 2 ignored, EXIT=101.

- F1 introduced, NEEDS-CHANGE, blocker (`resolve.rs:520`): `topology.workloads.<component>` has only the `<component>:` needle (no declaration needle, `workloads` is structural); the base cited it; now `<document>`. Reached from examples/billing with one-line edits. Suggested: with no declaration needle, keep first-unique; drop a guess only when a declaration was located and the guess is outside its block.
- F2 pre-existing, NEEDS-CHANGE (`resolve.rs:520`, `encloses` :561): an `outcomes.<name>` guess `filed:` matches a sibling outcome's payload key inside the same command and is still reported, contrary to the story's second acceptance clause. Suggested: emit no `<last>:` guess after `outcomes` (outcomes are `- name:`).
- F3 introduced, CONFIRMED, note (`resolve.rs:564`): `at.line <= declared.line` coarsens a correct key written above `id:`/`name:` in the same list item to the declaration line.

Attacked, not broken: CRLF/comments/blank lines; tab lines (YAML refuses); column-0 vs nested declarations; sibling commands; same outcome name across files; `component <name>` guess.

```findings
[{"file":"crates/specify/ess-compiler/src/resolve.rs","line":520,"category":"correctness","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"topology.workloads.<component> refusals have no declaration needle, so the correct unique workload-key guess the base cited is now dropped to <document>; reachable from examples/billing with one-line edits"},{"file":"crates/specify/ess-compiler/src/resolve.rs","line":520,"category":"correctness","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"an outcome-name guess that cannot match its target is still reported when its unique match is a sibling outcome's payload key inside the refused command's block, contrary to the story's acceptance"},{"file":"crates/specify/ess-compiler/src/resolve.rs","line":564,"category":"correctness","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"a correct key written above the declaration key in the same list item is coarsened to the declaration line"}]
```

Coordinator note: the findings block was converted from the adversary brief's field names (`summary`) to the store's (`message`, plus `category` and `severity` assigned by the coordinator); the brief named the wrong schema.
