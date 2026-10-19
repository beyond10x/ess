---
format: aep.planning-md/3
id: story:a-subject-prefixed-via-hint-names-the-bare-field
kind: story
status: draft
title: 'A when_related via: subject.<field> refusal names the bare stored-field form in its hint'
tags:
- adopter-report
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/related_guard.rs
revision: 2
---
## Outcome

From `ess/22`, a `when_related` whose `via:` is `subject.<field>` (or any other non-input path) is
refused with a hint that names both accepted forms, `via: input.<field>` and the bare stored field
`via: <field>`, instead of only the input form.

## Fit review

Finding covered: downstream specification report of 2026-10-19, finding 10, the one-hop part.

1. **Need.** An author wants a related guard over the row the subject's stored reference names and
   writes `via: subject.node_id`. The refusal (`ESS-COMMAND-002`, `type_mismatch`) says "`when_related`
   reads the row an input field names, one hop (hint: write `via: input.<field>` …)". The form the
   language accepts for this since `ess/22` — `via: node_id`, a bare stored field of the subject — is
   not named, so the author concludes the guard cannot be written. Reproduction:
   `.engineering/repro/related-from-subject-field/system.yaml` (refused, integration build) and
   `bare-via.yaml` (the same guards with `via: node_id`: 9 scenarios, 0 refusals, integration build).
2. **Class.** Defect in a diagnostic: the hint constant is documented as the hint "through ess/21"
   (`crates/specify/ess-domain/src/command/related_guard.rs:327-329`) and is given at every format
   (`related_guard.rs:249-257`), while `website/docs/reference/predicates.md:27` documents the bare
   stored field from `ess/22`.
3. **Already expressible?** Yes, `via: <field>` (`bare-via.yaml`). That is the idiom; only the
   hint is wrong.
4. **Fit.** No new surface. The refusal keeps its code and message; the hint gains the bare form
   where the document's format admits it (the format gate already exists for the bare form).
5. **Second adopter.** Any author of a guard over a parent row named by a stored reference.
6. **Cost.** One hint text per format band; diagnostic pins (`task test-xtask`) move.
7. **Alternatives.** (a) Change nothing: authors keep concluding the guard is impossible. (b) Accept
   `subject.<field>` as a synonym of the bare field: two spellings for one idea (refused, see the
   external-request review's red flags). (c) Fix the hint (chosen).

## Decisions

Accept as a defect (diagnostic). The requester's wider need — more than one hop — folds into
`story:row-queries-navigate-a-reference`; the row-set `subject.` form folds into
`story:a-related-selector-compares-a-row-field-with-a-subject-field`.

## Acceptance

- `a_subject_prefixed_via_is_refused_with_the_bare_field_hint`: at `ess/22` and later the refusal of
  `via: subject.node_id` names `via: node_id`; at `ess/21` the hint is unchanged.

## Scope

- `crates/specify/ess-domain/src/command/related_guard.rs` (~245-257, `INPUT_HINT` ~327) — cited.
- Test: `crates/specify/ess-domain/tests/related_guard.rs` — cited. Run `task test-xtask`.

One unit, today.
