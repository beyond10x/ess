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
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/interpreted_views.rs
revision: 10
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (item 5 of 9, 2026-10-22).
Making `Task.artifacts: List<Artifact>` non-optional, with no creator setting it, validates
cleanly; the interpreted run then reports 49 passed and 71 unsupported, each with the same line
naming no view and no field:

```text
ESS-CF-TARGET: cannot expose interpreted view: the model does not determine a complete typed view observation
```

Minimal reproduction (brand-free): an entity `Job { id, tags: List<Tag> }` whose only creator does
not set `tags`; a view over `Job`; synthesize and run against the interpreted target.

## Where it lives

- `crates/verify/ess-conformance/src/interpret/views.rs`: three sites return the shared
  `unsupported()` (`views.rs:11-16`) for a required view field with no stored value, each with the
  view and the field in scope: an aggregation group key at `:411` (return at `:413`), an aggregate's input field at `:432` (return at `:434`) and a plain view field at `:471`
  (return at `:476`).
- Rendering at `crates/verify/ess-conformance/src/target.rs:1301-1302`.

## 0.68.0

No. Rerun on 0.67.0 and 0.68.0: the same 71 unsupported. `views.rs` changed in range only for the
`now` filter (6d0f10ba8), not these lines.

## Cause

Verified by run plus code. A required field no creator sets has no stored value; the interpreter
cannot fill the view and reports a generic unsupported with no field. Whether `ess specify
validate` should refuse the unset required field is a design call (inferred, see Decisions).

## Fit review

1. **Need.** The author must learn which field makes a view unobservable, and preferably before
   running a suite. No syntax proposed.
2. **Class: defect (diagnostic).** A diagnostic that names neither the view nor the field gives
   the author nothing to act on; 71 identical lines for one missing assignment.
3. **Already expressible?** Not applicable; the specification is wrong, ESS just does not say where.
4. **Fit.** No authored surface. Two parts: (a) the interpreter's unsupported cause names the view
   and the field (`views.rs:471-476` has both in scope); (b) validate refuses a required,
   non-defaulted entity field that no creating command sets, the same family as other
   creator-completeness checks (I don't know whether one exists for scalars; the implementor
   cites it or adds the code).
5. **Second adopter.** Any entity that gains a required list field after its creator was written.
6. **Cost.** (a) none. (b) a new validation code, which moves the pins in `macro-guards.json` and
   `review-typed-diagnostics.md` (AGENTS.md § A new ValidationCode); if it would reject documents
   valid today, it is gated on the next format version instead of applied retroactively.
7. **Alternatives.** (a) Change nothing. (b) Only (a): fixes the message, still found at run
   time. (c) Chosen: both, with (b) gated if it breaks existing documents.

## Decisions

**accept.** The interpreted view's unsupported cause names the view and the field it cannot fill.

2026-10-22: split. This story is part (a) only. Part (b), validate refusing a required field no
creator sets with a new `ValidationCode`, is
`story:a-required-field-no-creator-sets-is-refused-by-validate`, a separate unit: it moves the
pins in `crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json` and
`docs/design/review-typed-diagnostics.md`, and it lands on `ess-domain`/`ess-compiler`, not here.
This story shares `target.rs` with unit A2 and is dispatched after it in the same slot. No format
change.

## Acceptance

Fixture: the minimal reproduction above (`Job { id, tags: List<Tag> }`, a creator that does not
set `tags`, a view over `Job`), written by the test, plus one variant per site below.

- **The cause names the view and the field.** Transition: the interpreted target answers the
  view for a `Job` row with no stored `tags`. Before: unsupported with `ESS-CF-TARGET: cannot
  expose interpreted view: the model does not determine a complete typed view observation`. After:
  still unsupported, and the cause names the view and `tags`. Test:
  `crates/verify/ess-conformance/tests/interpreted_views.rs`, case
  `an_unfillable_required_field_is_named_in_the_unsupported_cause`. Failing before: the cause
  carries neither name on the current tree.
- **Each site names its field.** Transition: the same unfilled field reached through each of the
  three sites: an aggregation group key (`views.rs:411`), an aggregate's input field (`:432`) and a plain view field (`:471`). After: each cause names its view and field. Same
  file, cases `an_unfilled_group_key_is_named`, `an_unfilled_aggregate_input_is_named` and
  `an_unfilled_view_field_is_named`. Mutation check: reverting any one site to the bare
  `unsupported()` fails its case.
- **Verdicts unchanged.** The reproduction's counts stay the same (unsupported, never passed or
  failed); every existing case in `tests/interpreted_views.rs` keeps its verdict, and suites are
  byte-identical.

## Schedule

Release 0.70.0 (defect, wave A unit A9, after A2 in its dispatch slot). The release tag on the
artifact marks it. Part (b) is scheduled on its own story.

## Further report

Second adopter: public repository beyond10x/inference, commit 8e72194, `ess/domains/graph.yaml:953`. `inference.graph.AddSubgraphNode/added` creates an `inference.graph.Node` without setting its required `requires: List<inference.model.Capability>`, which the view `inference.graph.Nodes` publishes; the specification's comment assumes the target gives an unset List the empty list. `ess verify conform run --target interpreted --report-format 2` on 0.67.0, 0.68.0 and the 0.69.0 integration build (commit 52bd110dd):

```
unsupported inference.graph.AddSubgraphNode/outcome/added
    ESS-CF-TARGET: cannot expose interpreted view: the model does not determine a complete typed view observation
469 scenarios: 468 passed, 0 failed, 0 error, 1 unsupported
```

and the run exits 1. Probe (0.69.0 build, verified): with `requires` removed from `inference.graph.Nodes` only, 469 passed, 0 unsupported, exit 0. Same site, `crates/verify/ess-conformance/src/interpret/views.rs:476`; validate exits 0, so part (b), `story:a-required-field-no-creator-sets-is-refused-by-validate`, would have refused this one too.

## Downstream case 2026-10-29

On 2026-10-29 (orchestrator D194) a downstream adopter's suite still has one `unsupported`
scenario on 0.76.0 and on the 0.77.0 head (84671d6b8), the outcome of a command that adds a node
to a planned subgraph:

```text
unsupported inference.graph.AddSubgraphNode/outcome/added
    ESS-CF-TARGET: cannot expose interpreted view: the model does not determine a complete typed view observation
```

Its message names no view and no field, which is the defect this story fixes. Once the field is
named, the adopter can tell whether its model leaves a required view field unset or the
interpreter cannot read a set one; the second would be its own story. The reproduction is kept
out of this repository.
