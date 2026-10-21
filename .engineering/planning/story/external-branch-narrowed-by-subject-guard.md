---
format: aep.planning-md/3
id: story:external-branch-narrowed-by-subject-guard
kind: story
status: active
title: An external branch is narrowed to the records a subject guard selects
tags:
- adopter-report
- design-first
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T02:37:44Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-21T02:37:44Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

An `external:` branch can be narrowed to records a stored-state guard selects, so an external
cause that applies only to some configured records is declared once, for those records.

## Evidence

An adopter on ess 0.56.0: `external:` beside `when_subject:` is refused ESS-COMMAND-004 ("a
subject fact has one selection authority"), so an external cause that applies only to records in
a stored configuration must be declared for every record. Related to
https://github.com/beyond10x/ess/issues/486's ordering rule; this asks for a narrowing guard on an
external branch, not a reordering.

## Acceptance

- A design page under `docs/design/` decides how the external branch and the subject guard share
  selection, or records why they cannot and names the idiom to use instead.
- If adopted: validation admits the narrowed external branch, synthesis writes a scenario on a
  selected row and refuses or skips one on an unselected row, and the targets agree.

## Intake 2026-10-16: https://github.com/beyond10x/ess/issues/584

A second report (ess 0.60.0, `ess/25`) needs the same narrowing: an `external:` refusal that
answers only for rows whose stored `method == S256` and `challenge != ""`. Without `when_subject:`
the branch claims rows that never take it, and the held-state refusal declared after it is
reordered by validation (`conflicting_declaration`, "the held state selects first in either
order"), which changes the answer for a request that is wrong in both ways. Under the G4 rule
(`docs/design/generic-language-features.md`, G4) a narrowed external branch is a step 3 branch (it
reads the held row), so declaration order among held-state branches decides; that is the ordering
this story's design page must state. For #584 the preferred answer is
`story:digest-and-encoding-functions-in-expressions`, which removes the need for an external branch
there; this story stays for causes that are truly outside the specification.

## State 2026-10-21

Shipped on the 0.67.0 batch branch, from `format: ess/29`, as decided in
`docs/design/generic-language-features.md` § G4 "An external branch narrowed by a subject guard":
`external:` beside `when_subject: {predicate: …}` is admitted; the branch answers at the held-state
step in declaration order, only on a record the predicate selects. Below `ess/29` the pair keeps its
`conflicting_declaration`, the hint naming `ess/29`; `before_row: true` and the `{field, equals}`
shape beside `external:` stay refused.

Evidence, one test binary each: `crates/specify/ess-domain/tests/external_narrowed_by_subject.rs`
(validation, placement, refusals), `crates/verify/ess-conformance/tests/external_narrowed_by_subject.rs`
(interpreter; synthesis witnesses the branch on a selected row where a later held-state branch
also holds, forces it beside siblings on unselected rows; a target ignoring the narrowing or moving
it after the later branch fails), `crates/generate/ess-synth/tests/external_narrowed_served.rs`
(generated Rust and Go ask the provider only on a selected row),
`crates/generate/ess-entity-runtime/tests/external_narrowed.rs` (lowered to the row guard and the
host's evidence). Fixture: `crates/verify/ess-conformance/tests/fixtures/external-narrowed-by-subject.yaml`.

Not done: a narrowed external refusal with `compensates: true` keeps the refusal every non-external
compensation gets; the doc comment of the generated `Context::external` port still says the
provider is asked before the branch's input guard.
