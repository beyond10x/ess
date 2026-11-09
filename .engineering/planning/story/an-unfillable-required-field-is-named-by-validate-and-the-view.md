---
format: aep.planning-md/3
id: story:an-unfillable-required-field-is-named-by-validate-and-the-view
kind: story
status: draft
title: The interpreted view names the required field it cannot fill
summary: 71 unsupported scenarios name no view or field when a required field is never set (adopter item 5, beyond10x/a2a); defect, 0.69.0.
tags:
- adopter-report
- defect
- ess-0.70.0
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/interpreted_views.rs
revision: 13
---
## Need

A required entity field that no creating command sets has no stored value, so the interpreter
cannot fill a view that publishes it. The run reports every scenario that reads such a view as
unsupported, with one shared line that names neither the view nor the field:

```text
ESS-CF-TARGET: cannot expose interpreted view: the model does not determine a complete typed view observation
```

One adopter's specification lost 71 scenarios to a single missing assignment (a required
`List<…>` field). A second lost one scenario: a creator left a required list unset, assuming the
target gives an unset `List` the empty list. Neither could tell from the message which field was
at fault.

Minimal reproduction (still present on 0.92.0): an entity `Job { job_id, label: String, tags:
List<Tag> }` (`Tag` a struct of one `text: String`), whose only creator `OpenJob` sets `label` and
not `tags`, and a read-your-writes view `Jobs` publishing `job_id`, `label` and `tags`.

```text
$ ess specify validate --path .                → valid, exit 0
$ ess verify conform run --path . --target interpreted --report-format 2
  unsupported shop.jobs.OpenJob/outcome/opened
      ESS-CF-TARGET: cannot expose interpreted view: the model does not determine a complete typed view observation
  1 scenarios: 0 passed, 0 failed, 0 error, 1 unsupported      (exit 1)
```

## Where it lives

- `crates/verify/ess-conformance/src/interpret/views.rs`: three sites return the shared
  `unsupported()` (`views.rs:11-16`) for a required view field with no stored value, each with the
  view and the field in scope: an aggregation group key (return at `:394`), an aggregate's input
  field (return at `:412`) and a plain view field (return at `:460`).
- Rendering: `crates/verify/ess-conformance/src/target.rs:1323` (`cannot expose {observation}: {why}`).

## Fit review

1. **Need.** The author must learn which field makes a view unobservable. No syntax proposed.
2. **Class: defect (diagnostic).** A diagnostic that names neither the view nor the field gives the
   author nothing to act on.
3. **Already expressible?** Not applicable; the specification is wrong, ESS just does not say where.
4. **Fit.** No authored surface: the interpreter's unsupported cause names the view and the field,
   which are in scope at each site.
5. **Second adopter.** Any entity that gains a required field after its creator was written.
6. **Cost.** None to formats; verdicts unchanged.
7. **Alternatives.** (a) Change nothing. (b) This story: name them at run time. (c) Refuse at
   validate, which is `story:a-required-field-no-creator-sets-is-refused-by-validate`.

## Decisions

Accept. This story is the run-time diagnostic only. Validation refusing a required field no
creator sets is `story:a-required-field-no-creator-sets-is-refused-by-validate`, a separate unit on
`ess-domain`/`ess-compiler` with a new `ValidationCode`. No format change.

## Acceptance

Fixture: the minimal reproduction above, written by the test, plus one variant per site.

- `an_unfillable_required_field_is_named_in_the_unsupported_cause`
  (`tests/interpreted_views.rs`): the run still reports the scenario unsupported, and its cause
  names the view `shop.jobs.Jobs` and the field `tags`. Before the fix the cause names neither.
- `an_unfilled_group_key_is_named`, `an_unfilled_aggregate_input_is_named`,
  `an_unfilled_view_field_is_named`: the same unfilled field reached through each of the three
  sites; each cause names its view and field. Reverting any one site to the bare `unsupported()`
  fails its case.
- Verdicts unchanged: the reproduction's counts stay the same (unsupported, never passed or
  failed); every existing case in `tests/interpreted_views.rs` keeps its verdict; suites are
  byte-identical.

## Scope

- cited: `crates/verify/ess-conformance/src/interpret/views.rs` (`unsupported` :11, returns :394, :412, :460)
- cited: `crates/verify/ess-conformance/src/target.rs` (rendering :1323)
- cited: `crates/verify/ess-conformance/tests/interpreted_views.rs`

confidence: high
