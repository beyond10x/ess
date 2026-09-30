---
format: aep.planning-md/3
id: review-result:adversary-ui-schema-pass-1
kind: review-result
status: active
title: Adversary pass 1, ess-ui wave unit ui-spec-schema
relations:
- reviews: story:ui-spec-schema
revision: 1
---
unit: story:ui-spec-schema, working tree ~/.local/state/worktree/trees/b10x/ess/ess-ui-schema (uncommitted over c2df36f75)
verdict: NEEDS-CHANGE
cases: executed 17→34, red 17
origin: introduced 17 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (this file; build dir $HOME/.cache/b10x-target/ess-ui-schema, already assigned)
needs-coordinator: route the 6 cases in adversary_pass1_checks_scope.rs (keep here or move to story:ui-spec-checks)

## 1. Diff

`git --no-pager diff --stat` shows only Cargo.lock/Cargo.toml (the implementor's; the crate is untracked).
Adversary additions, both test files, both untracked:
- crates/ui/ess-ui/tests/adversary_pass1.rs (11 cases)
- crates/ui/ess-ui/tests/adversary_pass1_checks_scope.rs (6 cases)
No implementation file touched. One probe file (adversary_pass1_probe.rs) was written, run, and deleted.

## 2. Cases (all red, each run alone before the suite ran)

`cargo test -p ess-ui --test adversary_pass1`: 0 passed; 11 failed
| case | red output |
|---|---|
| two_page_sections_sharing_an_inherited_name_are_refused | accepted; sections kept: filters, list (second `list` dropped) |
| a_page_overrides_an_inherited_choices_options_with_the_shorthand | refused: …/filters/choices/stage: invalid type: string "x", expected struct ChoiceOption |
| the_composite_shorthand_applies_to_a_forms_record | refused: pages/p/sections/form: invalid type: string "record", expected a YAML mapping |
| an_action_in_a_tabs_form_derives_its_name | refused: pages/p/sections/summary: data did not match any variant of untagged enum TabForm |
| a_reads_map_passed_as_a_widget_arg_becomes_the_bodys_read | refused: pages/p/sections/feed/body/list: invalid type: map, expected a string |
| an_unmapped_marker_is_accepted_by_an_enum_typed_field | refused: unknown variant `UNMAPPED: …`, expected one of `eager`, `on_visible`, `on_demand` |
| a_node_name_holding_the_path_separator_is_refused | accepted; a section now sits at pages/p/sections/summary/fields/tier |
| a_derived_action_name_is_a_valid_path_segment | `row.first + row.last` in pages/p/sections/summary/actions/row.first + row.last is not a node name |
| a_widget_named_like_a_member_of_the_union_is_refused | a widget named `metric` was accepted and is unreachable |
| an_omitted_optional_arg_is_not_reported_as_an_unknown_param | `args.hidden` names no param of widget `pill` |
| a_placeholder_read_without_a_fixture_in_an_overlay_is_refused | an overlay's placeholder read without a fixture was accepted |

`cargo test -p ess-ui --test adversary_pass1_checks_scope`: 0 passed; 6 failed
| case | red output |
|---|---|
| sensitive_state_in_the_url_or_browser_storage_is_refused | sensitive state in `url` was accepted |
| a_credential_in_local_storage_is_refused | a credential in local_storage was accepted |
| a_type_written_as_a_string_expression_is_refused | `type: 'list<PartnerId>'` was accepted |
| a_widget_that_contains_itself_is_refused_even_before_it_is_used | a self-containing widget was accepted |
| a_degrades_key_that_names_no_capability_is_refused | `no_chartz` was accepted as a capability |
| a_link_with_both_to_and_href_is_refused | a link with both `to` and `href` was accepted |

## 3. Suite

`cargo test -p ess-ui --no-fail-fast` (after the cases existed): example 8 ok, schema 6 ok, shorthands 3 ok,
adversary_pass1 0/11, adversary_pass1_checks_scope 0/6; "error: 2 targets failed"; exit non-zero.
Before = 17 (8+6+3, the three original targets, adversary files excluded).

## 4. Findings (tree above; all origin introduced: the crate does not exist at c2df36f75)

| # | file:line | verdict | what reaches it |
|---|---|---|---|
| 1 | src/expand.rs:636 | NEEDS-CHANGE | merge_named skips every page entry whose name is inherited, so a duplicate is dropped silently and names_unique never sees it. Reached by any page of a kind with a typo'd duplicate |
| 2 | src/expand.rs:300 | NEEDS-CHANGE | choice `options` shorthand runs before kind merge and keys on `component: choice` in the same map; a page overriding an inherited choice writes no component. Reached by the documented named-list merge |
| 3 | src/expand.rs:52 | NEEDS-CHANGE | NODE_KEYS omits `record` (form.record is typed Node) and Tab.form's Node arm |
| 4 | src/expand.rs:326 | NEEDS-CHANGE | Action shorthands run only under ACTION_LISTS and `action`; Tab.form (one_of Node, Action) is missed |
| 5 | src/expand.rs:330 | NEEDS-CHANGE | pass 1 expands `reads: args.x` to `{view: args.x}` before pass 4 substitutes; a map arg lands as `view: {…}`. Schema says args may be maps |
| 6 | src/model.rs:730 | NEEDS-CHANGE | unmapped_marker.accepted_by "any field, whatever its declared type"; enum-typed fields refuse it. Reached by every retrofit that uses the marker |
| 7 | src/path.rs:370 | NEEDS-CHANGE | segment_pattern not enforced; a name with `/` forges another node's path |
| 8 | src/expand.rs:479 | CONFIRMED | the schema's own first_present derives a name from `copy` (an expr) and produces segments outside the pattern; schema and loader both need a call |
| 9 | src/expand.rs:786 | CONFIRMED | a widget named like a union member is kept and unreachable |
| 10 | src/expand.rs:866 | CONFIRMED | an omitted optional param without default makes `not args.x` fail as "names no param" (false message) |
| 11 | src/path.rs:382 | NEEDS-CHANGE | check_reads runs for Section and Node only; overlay (and inline-confirm) reads skip the unit's own exactly_one_of check |
| 12 | tests/example.rs:133 | CONFIRMED | constructs_used pre-seeds Degrades, StateClass, Store, Navigation, Composite, Node, NodePath, and inserts Reads unconditionally (:247); dropping them from the example keeps the test green (constant pin). The example uses degrades today (4 occurrences) |
| 13–17 | tests/adversary_pass1_checks_scope.rs:29,58,66,78,89 | CONFIRMED | sensitive/credential placement refusals, string type expressions, a self-containing widget nobody uses yet, unknown degrades capability, link to+href: all declared as refusals in the schema and accepted by the loader. They may belong to story:ui-spec-checks |

## 5. Attacked and not broken

component typo, `component: header`/`overlay`, primitive as a section, a property from another member on a section/overlay/node: all refused with a path. Kind cycles, self-extends, unknown kind, same_as cycle, `remove: true` naming nothing: all refused. Unknown widget arg, mutual widget recursion when used, duplicate fields/actions: all refused. Placeholder with fixture loads (no warnings API exists; warnings are ui-spec-checks). Every shorthand template is read from `shorthands.index` (`Schema::template` panics on a missing entry); which keys *trigger* each shorthand is hand-coded (findings 2–5). Brand grep (babel, manager v2, common SaaS names) over schema, crate, example: no match. `type: "` / `type: '` / `<` in the example: none.

## 6. Paths written outside the worktree

- ~/.cache/ess-ui-wave/schema/adv1-review.md
- ~/.cache/b10x-target/ess-ui-schema (the assigned build dir; reused, not created)

## 7. Findings block

```findings
- {file: crates/ui/ess-ui/src/expand.rs, line: 636, category: acceptance, severity: blocker, verdict: NEEDS-CHANGE, origin: introduced, message: "a page section whose name repeats an inherited one is dropped silently during kind merge instead of refused by names_unique"}
- {file: crates/ui/ess-ui/src/expand.rs, line: 300, category: contract-drift, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "choice options shorthand is skipped for a page entry that overrides an inherited choice by name, because expansion precedes kind merge and keys on component"}
- {file: crates/ui/ess-ui/src/expand.rs, line: 52, category: contract-drift, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "the Composite shorthand is not applied at form.record, a Node-typed position"}
- {file: crates/ui/ess-ui/src/expand.rs, line: 326, category: contract-drift, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "an action in Tab.form gets no derived name, so the tab is refused"}
- {file: crates/ui/ess-ui/src/expand.rs, line: 330, category: boundary, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "a Reads map passed as a widget arg is wrapped as view: {map} because the reads shorthand runs before substitution"}
- {file: crates/ui/ess-ui/src/model.rs, line: 730, category: contract-drift, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "enum-typed fields refuse the UNMAPPED marker that the schema says any field accepts"}
- {file: crates/ui/ess-ui/src/path.rs, line: 370, category: acceptance, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "node names are not held to segment_pattern, so a name containing / forges another node's canonical path"}
- {file: crates/ui/ess-ui/src/expand.rs, line: 479, category: contract-drift, severity: note, verdict: CONFIRMED, origin: introduced, message: "the schema's first_present derives an action name from copy, an expression, yielding path segments outside segment_pattern"}
- {file: crates/ui/ess-ui/src/expand.rs, line: 786, category: boundary, severity: warning, verdict: CONFIRMED, origin: introduced, message: "a widget declared under a union member's name is accepted and can never be used"}
- {file: crates/ui/ess-ui/src/expand.rs, line: 866, category: boundary, severity: note, verdict: CONFIRMED, origin: introduced, message: "an omitted optional param without default inside an expression is refused as naming no param of the widget"}
- {file: crates/ui/ess-ui/src/path.rs, line: 382, category: mutant, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "the Reads exactly_one_of and fixture check skips overlays, so an overlay placeholder without a fixture loads"}
- {file: crates/ui/ess-ui/tests/example.rs, line: 133, category: mutant, severity: note, verdict: CONFIRMED, origin: introduced, message: "the uses-every-construct test pre-seeds eight constructs and Reads, so it cannot fail if the example stops using them"}
- {file: crates/ui/ess-ui/tests/adversary_pass1_checks_scope.rs, line: 29, category: acceptance, severity: note, verdict: CONFIRMED, origin: introduced, message: "sensitive state in url/session_storage/local_storage and a credential in local_storage load despite PlacementProfile.resolution.refusals"}
- {file: crates/ui/ess-ui/tests/adversary_pass1_checks_scope.rs, line: 58, category: acceptance, severity: note, verdict: CONFIRMED, origin: introduced, message: "a type written as a string expression such as list<PartnerId> loads as a named type"}
- {file: crates/ui/ess-ui/tests/adversary_pass1_checks_scope.rs, line: 66, category: boundary, severity: note, verdict: CONFIRMED, origin: introduced, message: "a widget that contains itself loads while no page uses it"}
- {file: crates/ui/ess-ui/tests/adversary_pass1_checks_scope.rs, line: 78, category: boundary, severity: note, verdict: CONFIRMED, origin: introduced, message: "a degrades key naming no capability loads"}
- {file: crates/ui/ess-ui/tests/adversary_pass1_checks_scope.rs, line: 89, category: boundary, severity: note, verdict: CONFIRMED, origin: introduced, message: "a link with both to and href loads despite link.exactly_one_of"}
```
