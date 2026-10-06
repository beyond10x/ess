---
format: aep.planning-md/3
id: review-result:ess-054-w2-3-adversary-1
kind: review-result
status: active
title: Adversary pass 1, unit W2-3 parse refusals and case-record diagnostics
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-448
- reviews: story:feature-request-426b
revision: 1
---
I added 8 cases and 7 of them fail. W2-3 still holds up on everything you asked me to compare. Two of the failures add misleading follow-on refusals, and two show the new `diagnostics.md` sentence claims more than the code does.

```
unit: W2-3 precedence-diagnostics-448-426, commit e770d88a4 (range 6192d9be25..e770d88a4) plus my uncommitted test file
verdict: NEEDS-CHANGE
cases: executed 454→462, red 7
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths (part 6)
needs-coordinator: whether a declaration hiding its own other refusals (finding 4) is accepted design, which needs one doc caveat, or needs a code change
```

**1. `git --no-pager diff --stat`** is empty, because my only change is a new untracked file. `git status --short`:
```
?? crates/specify/ess-domain/tests/adversary_w2_3_pass1.rs
```
It is a test file. No implementation file was touched.

**2. Cases added** in `crates/specify/ess-domain/tests/adversary_w2_3_pass1.rs`. Each one first checks that the unbroken version validates on its own.

| Case | Asserts | Now |
|---|---|---|
| `adv_aggregate_where_that_does_not_parse_does_not_stop_the_file` | a bad view filter plus a bad aggregate `where:` give 2 `unparsable_predicate` refusals | red |
| `adv_instances_where_that_does_not_parse_does_not_stop_the_file` | a bad `instances: {where}` is refused at its outcome, and the file is not stopped | red |
| `adv_affects_where_that_does_not_parse_does_not_stop_the_file` | the same for `affects[].where` | red |
| `adv_withheld_view_is_not_refused_again_by_a_command_tree` | a bad filter on a view that a component's `cli.views` reads gives exactly 1 refusal | red |
| `adv_unparsable_guard_does_not_hide_a_sibling_branchs_refusal` | another outcome's refusal in the same command is still reported | red |
| `adv_unparsable_filter_does_not_hide_the_views_own_refusal` | the view's own undeclared-field refusal is still reported | red |
| `adv_boolean_variant_is_refused_once` | `variants: [plain, fancy, True]` gives only the `types.<T>.variants` refusal | red |
| `adv_newtype_invariant_that_does_not_parse_is_refused_once` | a bad newtype invariant gives 1 refusal, at `types.probe.item.OwnerId.invariants[0]` | green |

Red output from the first run, which was this file alone (`cargo test -p ess-domain --test adversary_w2_3_pass1`), quoted verbatim:
```
---- adv_boolean_variant_is_refused_once stdout ----
assertion `left == right` failed: only the variant's refusal; nothing declared with the type is refused for it:
TypeMismatch types.probe.item.Kind.variants: `true` is a YAML boolean, not a variant name: ...
UndeclaredReference command.probe.item.Create.input[2].type: `probe.item.Kind` is not a declared type
UndeclaredReference event.probe.item.Created.fields[1].type: `probe.item.Kind` is not a declared type
UnobservableFact binding.close-fancy.when.where: `event.kind` does not resolve through declared types
  left: 4
 right: 1
---- adv_affects_where_that_does_not_parse_does_not_stop_the_file stdout ----
the reader stopped the file, so nothing beside it is reported: cannot parse predicate "team ~= subject.team": invalid fact path identifier ...
---- adv_unparsable_filter_does_not_hide_the_views_own_refusal stdout ----
the view's own refusal `UndeclaredReference view.probe.item.Labelled.fields[2]` is still reported beside the parse refusal:
UnparsablePredicate view.probe.item.Labelled.filter: cannot parse predicate "label ~= \"\"": ...
---- adv_aggregate_where_that_does_not_parse_does_not_stop_the_file stdout ----
the reader stopped the file, so nothing beside it is reported: `where`: cannot parse predicate "label ~= \"\"": ...
---- adv_unparsable_guard_does_not_hide_a_sibling_branchs_refusal stdout ----
the sibling's refusal `UnobservableFact command.probe.item.Create.outcomes.unlabelled-owner.when_related` is still reported beside the parse refusal:
UnparsablePredicate command.probe.item.Create.outcomes.empty-label.when: cannot parse predicate "input.label ~= \"\"": ...
---- adv_instances_where_that_does_not_parse_does_not_stop_the_file stdout ----
the reader stopped the file, so nothing beside it is reported: cannot parse predicate "team ~= input.team": ...
---- adv_withheld_view_is_not_refused_again_by_a_command_tree stdout ----
assertion `left == right` failed: only the filter's refusal; the tree reading the view is not refused for it:
UnparsablePredicate view.probe.item.Labelled.filter: cannot parse predicate "label ~= \"\"": ...
UndeclaredReference component reader: the command tree reads `probe.item.Labelled`, which no domain `reader` owns projects
  left: 2
 right: 1
test result: FAILED. 1 passed; 7 failed
```
Two later edits to my own file: an `assert_ne!` form so stable clippy passes, and `rustfmt` on this file only. I re-ran the file alone afterwards and the same 7 are red. Stable clippy on this target is clean, and `cargo fmt -p ess-domain --check` exits 0.

**3. Suite run**, after the cases existed:
`cargo test --locked --offline -p ess-domain --lib --test adversary_w2_3_pass1 --test enum_refusal_shapes --test generated_case_record_idioms --test adversary_e_u1_resolver --test calendar_window --test expression_a1 --test expression_a2 --test expression_utf8_bytes --test stored_field_guards --test typed_text_operands --no-fail-fast` → exit 101.
- Every target passed except mine:
  - lib 361
  - `adversary_e_u1_resolver` 5
  - `calendar_window` 6
  - `enum_refusal_shapes` 10
  - `expression_a1` 15
  - `expression_a2` 12
  - `expression_utf8_bytes` 9
  - `generated_case_record_idioms` 8
  - `stored_field_guards` 18
  - `typed_text_operands` 10
- `adversary_w2_3_pass1`: 1 passed, 7 failed.
- The 454 "before" figure is this same run's per-target totals minus my file's 8. It is not a separate deselected run.
- Not run: the ess-cli `validate_parse_refusals` target (it needs the whole `ess` binary built) and `cargo xtask site-data --check`.

**4. Findings** (they cover e770d88a4)

| # | file:line | What breaks | Case | Verdict / origin |
|---|---|---|---|---|
| 1 | `website/docs/reference/diagnostics.md:50` | The new sentence says a predicate that does not parse does not stop the file. Three positions still parse inside serde and stop it: aggregate `where` (`view.rs:594`), `instances.where` (`set_effects.rs:49`) and `affects[].where` (`set_effects.rs:60`). Selection `where` (`selection.rs:81`) probably does too; I did not test it. Story 448's fit review lists aggregate `where` among the positions that move. **Reached by:** any `ess/22` aggregate measure, or any set-effect outcome with a typo. **Fix:** move these fields to `WrittenPredicate`, or have the sentence name the positions that still stop. | aggregate / instances / affects | NEEDS-CHANGE / introduced (the base stops too, but the claim is new) |
| 2 | `crates/specify/ess-domain/src/spec.rs:584` | A view held back for a bad filter disappears from the per-domain view map. A component tree that reads it then gets a false `UndeclaredReference`. `Refused.views` covers actor grants but not components. **Reached by:** a component with `cli.views` (a documented surface) and a typo in a filter. **Fix:** count refused views as projected by their domain in `validate_components`. | `adv_withheld_view_…` | NEEDS-CHANGE / introduced (this path was unreachable before) |
| 3 | `crates/specify/ess-domain/src/types.rs:1569` (refused at `:1735`) | A boolean variant refuses the whole type. Every input, event field and guard declared with it then gets "`probe.item.Kind` is not a declared type" or `unobservable_fact`: 3 extra refusals from one mistake. There is no `Refused.types`. **Reached by:** issue #426's own `variants: [True, False, Unknown]` used as a field type. **Fix:** report the refusal and keep the type with the boolean entries left out, the same way `withhold_unparsed_invariants` does. | `adv_boolean_variant_is_refused_once` | NEEDS-CHANGE / introduced |
| 4 | `crates/specify/ess-domain/src/command.rs:6394`, `view.rs:1875` | An unparsable predicate holds back its whole command, view or binding. That declaration's other refusals stay hidden: a sibling outcome's `unobservable_fact`, the view's own undeclared field. Both the 448 decision and `diagnostics.md:50` say the file's other refusals are still reported. Between declarations it works. **Reached by:** two mistakes in one command or view. **Fix:** at least a doc caveat. | sibling / view-own | CONFIRMED / introduced (doc claim; the base hid everything) |

**5. What I attacked and could not break**
- **The 12 re-pinned assertions:** none is weaker. Six are still exact string matches: `adversary_e_u1_resolver`, both in `expression_a1`, `expression_a2`, and the two `types.rs` ones, which now also pin the location. Five now pin the code as well as the message: `stored_field_guards`, `typed_text_operands`, `synthesis` (null guard), `expression_validation`, `utf8`. In `calendar_window`, the `refused()` helper asserts at least one match, so `.all()` cannot pass on an empty list.
- **IR and suite bytes:** `WrittenPredicate` reads the same `Node` through `Predicate::from_node` under the same reader format. It serializes the parsed predicate, and its schema reuses `Predicate`'s (and `RawEnumVariant`'s reuses `EnumVariant`'s). I checked `finite.rs` arm by arm: every new `Ok`/`Err` matches the old `Some`/`None`, including the state and field caps and the choice between booleans and enum-only. This is code reading only; I compiled no older example at base.
- **Documents that validated before:** none found that is now refused. `scalar_representation` refuses the same set as before, and a boolean read as an `EnumVariant` was already refused.
- **Codes:** `bridge` (`resolve.rs:979-999`) is the only place a family is computed, and it forces both new codes to `SPEC` whatever the site.
- **Others:** invariant indices keep their place, so lexical spellings stay aligned. A held-back command is still counted as declared by actors, components, bindings and lifecycle moves. A newtype invariant is held back once (green case).

**6. Paths written outside the worktree**
- `~/.cache/ess-054-wave/W2-3/adv1/run1.log`
- `~/.cache/ess-054-wave/W2-3/adv1/run2.log`
- `~/.cache/ess-054-wave/W2-3/adv1/suite.log`
- `~/.cache/ess-054-wave/W2-3/adv1/tmp/`
- The build dir `~/.cache/b10x-target/ess-054-W2-3` was created by my build and then removed with `cargo clean`.

**7. Findings block**
```findings
- file: website/docs/reference/diagnostics.md
  line: 50
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'the SPEC row says an unparsable predicate does not stop the file, but aggregate where, instances.where and affects[].where still parse inside serde and stop it'
- file: crates/specify/ess-domain/src/spec.rs
  line: 584
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'a view withheld for an unparsable filter drops out of the domain view map, so a component cli tree reading it gets a false UndeclaredReference'
- file: crates/specify/ess-domain/src/types.rs
  line: 1569
  category: judgement
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'a YAML boolean enum variant refuses the whole type, adding three false not-a-declared-type and unobservable refusals for every input, event field and guard using it'
- file: crates/specify/ess-domain/src/command.rs
  line: 6394
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'an unparsable predicate withholds its whole command or view, hiding that declaration''s other semantic refusals, contrary to the 448 decision and diagnostics.md:50'
```
