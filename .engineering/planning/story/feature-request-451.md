---
format: aep.planning-md/3
id: story:feature-request-451
kind: story
status: active
title: Rules held as data cannot be given semantics
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#451
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/stored-rules
- confidence: inferred
  path: crates/edge/ess-cli/tests/stored_rules_idiom.rs
- confidence: cited
  path: docs/design/filtered-related-reads.md
- confidence: inferred
  path: docs/design/stored-rules-boundary.md
- confidence: cited
  path: website/docs/guides/specify/guards-and-predicates.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T14:53:32Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-05T14:53:33Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome
Resolve beyond10x/ess#451: Rules held as data cannot be given semantics.

## Origin
beyond10x/ess#451, filed 2026-10-05; reported from an adopter's system that stores conditions as rows (expression, operator, expectation), folds them with ALL/ANY, and evaluates and tests them through commands.

## Fit review
1. Need: a specification should say what it means for a stored rule to hold: which conditions it is built from, how they fold, and how a command that tests it answers. The rules are written at run time, not in the specification. The requester asks for a decision: either ESS models stored rules ("an interpreted predicate stored as data, evaluated by the declared predicate semantics"), or a note putting them out of scope, with the recommended boundary. Minimal reproduction: `<fit-review scratch>/probe-451/` (installed ess 0.52.0). A `Condition` entity with `operator` (enum), `expression`, `expectation` and `fold` (enum `All`/`Any`); `AddCondition` stores a row; `TestRule` answers `met`, or `not-met` as an `external:` refusal. It validates (`probe v1 — 3 file(s), valid`). Synthesis writes 3 scenarios, including "`TestRule` answers `not-met` for the cause it declares as external, injected".
2. Class: local policy at the core, with a convenience around it.
   - The stored grammar is the adopter's own: its operators include REGEXP and reference expressions by namespace (#449).
   - Rows, folds over stored per-condition results, and the test command's answers are all expressible today.
   - ESS evaluating arbitrary stored predicates is not a domain fact the adopter lacks a way to state. It would make ESS an interpreter of another system's rule language.
3. Existing idiom:
   - (a) The rule's parts are typed rows (probe).
   - (b) The verdict of evaluating them is the evaluator's, declared as an outcome the input cannot decide: `external:` with an error, which a generator injects (`website/docs/guides/specify/guards-and-predicates.md:143-155`; `docs/design/guarded-external-outcomes.md:3-11` for `when:` beside `external:`).
   - (c) Where per-condition results are stored (`holds: Boolean` on each row), the ALL/ANY fold is a row-set guard: `when_related: {entity: Condition, where: rule_id == input.rule_id, forall: holds == true}` for ALL, `exists` for ANY (`docs/design/filtered-related-reads.md:22-45`, `CHANGELOG.md:42`, 0.53.0 / ess/22). Not probed: the installed ess is 0.52.0.
   - (d) Per-operator facts, such as "takes a number", become typed enum attributes once #450 lands.
4. Fit of ESS evaluating stored rules:
   - It needs a value type whose values are predicates, plus operand addressing by run-time name. ESS paths are static and resolved when the specification is compiled (`crates/specify/ess-domain/src/expression.rs:584`).
   - Each adopter operator would need an ESS meaning, including REGEXP, which #449 declines.
   - Synthesis would have to generate both stored rules and inputs that make them hold or fail. That is search over the predicate language, against the existing finite bounds (`crates/specify/ess-domain/src/command/finite.rs:10-12`).
   - Rust, Go, TypeScript, the interpreter and Entity Runtime would each need an evaluator for data-borne predicates, and Entity Runtime already refuses value expressions it cannot express (`docs/design/value-expressions.md:324-325`).
   - The construct would be new, spanning guards, values and storage. Nothing else in the language has that shape.
5. Second adopter: a pricing service stores discount rules (field, operator, threshold, fold) and answers `QuotePrice`. The same boundary serves it: rows typed, the "rule applies" verdict `external:`, and the fold a row-set guard where per-rule results are stored. No second adopter needs ESS to execute the stored rules itself.
6. Cost as proposed: a new meta-level type, ess/23, an evaluator in every lane, a new suite step for stored-rule witnesses (ess-conformance/44, /45), and diff and Entity Runtime refusals. Size L+, with an unbounded synthesis question. Cost of the decline: one design note, one guide section and one fixture test. No format bump.
7. Alternatives: (a) change nothing: the boundary stays unwritten and the next adopter asks again. (b) As proposed, interpreted stored predicates: refused, for the reasons in 4. (c) A closed "rule" construct limited to ESS's own operators over a declared field list: still a meta-level evaluator, and it would not cover the adopter's REGEXP or namespaces, so it would not meet the reported need. Refused. (d) Chosen: decline with the boundary written down.

## Decisions
Decline, with the idiom. The story body is the decline record. ESS does not evaluate rules held as data. The recommended boundary:
- the rule's parts are typed rows (operator and fold as enums; operator facts as #450 attributes once built);
- "does this stored rule hold" is the evaluator's decision, declared as an `external:` outcome with an error and injected by conformance;
- an ALL/ANY fold over stored per-condition results is a `when_related` row-set guard (`forall`/`exists`).

Built:
- the design note `docs/design/stored-rules-boundary.md`, with four sections: `## A stored rule's parts are typed rows`, `## Whether a stored rule holds is the evaluator's`, `## A fold over stored results is a row-set guard`, and `## Patterns held as data`, which records #449's pattern boundary (#449 depends on this story and links it);
- the section `## A rule stored as data is evaluated by the system` in `website/docs/guides/specify/guards-and-predicates.md`, after `## An outcome the input cannot decide says that too`, linking the note;
- a fixture holding the probe's model.

Reply on the issue. No format bump.

## Acceptance
- stored_rules_idiom_validates_and_injects_verdict: the fixture validates, and synthesis writes the injected `not-met` scenario and the `met` scenario with 0 refusals.
- stored_rules_fold_as_row_set_guard: the fixture's ALL/ANY variant (`when_related` with `forall`/`exists` over stored `holds`) validates under ess/22 and synthesizes both branches.
- stored_rules_note_states_the_boundary: `docs/design/stored-rules-boundary.md` exists with the four headings above. Their sections contain, in order: "typed rows" and "enum"; "`external:`" and "injected"; "`when_related`", "`forall`" and "`exists`"; "regular expression", "stored", "#449" and "no pattern predicate". The case reads the note and fails naming the missing heading or phrase.
- stored_rules_guide_section_states_the_boundary: `website/docs/guides/specify/guards-and-predicates.md` has the heading `## A rule stored as data is evaluated by the system`. The section contains "`external:`", "`when_related`" and a link whose target ends in `docs/design/stored-rules-boundary.md`. Its fenced model is the fixture's model, compared byte for byte. The case reads the page and fails on a missing heading, phrase, link or model.

All four cases live in `crates/edge/ess-cli/tests/stored_rules_idiom.rs`. `task site-build` is not the check for the note, because the guide links it by a URL the site build does not resolve.

## Scope
- docs/design/stored-rules-boundary.md  inferred — new design note (also the #449 pattern boundary)
- website/docs/guides/specify/guards-and-predicates.md  cited — section after `external:` (143-155)
- docs/design/filtered-related-reads.md  cited — row-set fold the note cites (22-45)
- crates/edge/ess-cli/tests/fixtures/stored-rules  inferred — fixture model
- crates/edge/ess-cli/tests/stored_rules_idiom.rs  inferred — fixture, note and guide-section cases
