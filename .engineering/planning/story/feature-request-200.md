---
format: aep.planning-md/3
id: story:feature-request-200
kind: story
status: active
title: a search parameter on a view (a view parameter as the operand of contains / starts_with)
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#200
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T15:27:22Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-04T15:27:23Z", actor: "human:timo", revision: 5}
---
## Outcome

a search parameter on a view (a view parameter as the operand of contains / starts_with) (beyond10x/ess#200).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# story:feature-request-200 — fit review (beyond10x/ess#200)

Read tree: `gaps-270`. CLI: `ess 0.44.0`; newer releases may differ.

1. **Need.** A list view takes caller text and returns the rows whose text field contains it, or starts or ends with it. This is a search. Today the parameter cannot be declared at all. Repro `repro-200/` (ess 0.44.0):
   - `v1-contains-param.yaml` (`filter: {note: {contains: param.query}}`) and `v3-starts-param-string.yaml` are refused with ESS-VIEW-003 `unobservable_fact`.
   - `v0-eq-param.yaml` (`note == param.query`) is valid, but it states an exact match.
   - `v2-contains-literal.yaml` (literal operand) is valid.
   - Requester's syntax: a bare `param.<name>` as the operand of `contains` / `starts_with` / `ends_with` (and the `_ignore_case` forms) for a `String` parameter.
2. **Class: gap.** Literal-only operands are documented: `website/docs/reference/predicates.md:593`, `docs/design/string-predicate-operators.md:56-59`. So this is not a defect. Two traps sit next to it (defect candidates, not this story):
   - `v4-contains-param-text-undeclared.yaml`: with no `params:`, `{contains: param.query}` validates and filters on the text "param.query".
   - The ESS-VIEW-003 hint (`crates/specify/ess-domain/src/view.rs:1582`) tells the author to write `param.query`, which is exactly what they wrote.
3. **Already expressible? No.**
   - `==` is a different fact.
   - The caller-supplied free-form filter was declined (#174; `story:view-paging-and-caller-filters`, coordinator decision 2026-09-27).
   - There is no idiom.
4. **Fit: the proposed bare `param.<name>` fails; the redesigned operand passes.**
   - **Bare `param.<name>`:**
     - It reinterprets text that is a valid literal today (`v4`), so an existing document would silently change meaning.
     - Siblings: guards have the same question. `{x: {starts_with: input.prefix}}` in `when:` / `when_subject:` reads the literal today (`string-predicate-operators.md:56-57`; `repro-200/g/guard-input-operand.yaml` validates on 0.44.0). A view-only change leaves guards unable to say the same thing (red flag 2). This overlaps #233 (fact operands, sibling-field comparison).
     - `_ignore_case` exists only as `equals_ignore_case` / `in_ignore_case` (`crates/specify/ess-primitives/src/predicate.rs:284-304`), so there are no ignore-case forms of these three operators to extend.
   - **Redesign:** a mapping operand `{param: <name>}` in view filters, mirroring the `{input: …}` source in `sets:`. A mapping is a reader error today ("a comparison operand must be a scalar", `string-predicate-operators.md:73-75`), so no existing text changes meaning.
     - Guards get `{input: <name>}` in the same story, or a refusal by name.
     - Synthesis chooses the parameter value as it already does for `== param.x` (CHANGELOG.md:892-893). It arranges one matching row and one differing in one character.
     - Targets: Go/TS view-query evaluation, Entity Runtime lowering (handle it or refuse it by name), the 0.46 generated-behaviour obligation for view `params:` (CHANGELOG.md:264-266), and `ess-diff`. OpenAPI needs nothing, because the parameter is already a query parameter.
5. **Second adopter: yes.** A contacts view searched by name prefix; a log view filtered by a term the message contains. Nearly every list endpoint with a search box.
6. **Cost.**
   - Source format bump: ride the unreleased `ess/20` if it lands in time.
   - One operand form in two places (view filter, guards).
   - One diagnostic for a non-String parameter.
   - Synthesis witness and Go/TS runtime evaluation.
   - A diff classification.
   - No migration: the new form is a reader error today.
7. **Alternatives.**
   - (a) Change nothing: no parameter, and a comment.
   - (b) The requester's bare `param.<name>`: refused, because it silently re-reads admitted literals.
   - (c) `{param: <name>}` / `{input: <name>}` mapping operand: chosen, because it adds no keyword and keeps literals verbatim.
   - (d) Free-form filter: already declined in #174.

## Decisions

- **accept, redesigned (proposed):**
  - **The construct:** a view filter's `starts_with` / `ends_with` / `contains` takes a parameter as a mapping operand, `{param: <name>}`, for a `String` or a newtype of `String`.
  - **Guards in the same story:** they get `{input: <name>}`, or a refusal by name that states why not.
  - **Changed from the request:**
    - The bare `param.<name>` is not adopted. It is a valid literal today (`repro-200/v4`), and reading it as a parameter would silently change existing documents.
    - There are no `_ignore_case` forms of these operators to extend.
  - **Fold in a small defect fix:** refuse a string-operator literal spelled `param.<declared>` / `input.<declared>` (as field-name literals already are), and correct the ESS-VIEW-003 hint (`view.rs:1582`).
  - **Overlaps, not duplicates:**
    - #174, closed: the free-form filter was declined, and this is the narrow case.
    - #95, closed: where the string operators came from.
    - #233, open: fact operands in value expressions. Coordinate the operand spelling.
  - **Already fixed?** No.

- Coordinator (2026-10-01): adopted as proposed above. The ESS-VIEW-003 hint that suggests the literal `param.query` is corrected in the same unit.
