---
format: aep.planning-md/3
id: story:a-related-row-may-answer-before-input-guards
kind: story
status: draft
title: A command may declare that an existing related row answers before its input guards
summary: 'Adopter report R2: a second initialize answers already-initialized before its payload is read; the precedence rule forbids that order.'
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Report

An adopter retrofitting an existing service (2026-10-23, ess 0.67.0, ess/29): the service refuses a second initialize before it reads the payload, so a malformed second initialize answers `already-initialized`. Declaring the `when_related: {exists: true}` refusal above a `when:` input guard is refused:

`error[ESS-COMMAND-004]: malformed answers at step 1 of the precedence order and already-initialized, declared above it, at step 4`

Related: #554 (asks for a declared precedence with a fixed order). This case needs the opposite order for one command.

## Acceptance (draft, fit review pending)

- A command can state that an existing related row answers before its input guards, and validation, synthesis and both targets honour that order.

## Fit review

Request (an adopter, verbatim): an outcome on an existing related row (`when_related: {entity:
Session, where: session_id == input.session_id, exists: true}`, error AlreadyInitialized), declared
before an input guard outcome (`when: {payload_valid: false}`, error Malformed), is refused:
"error[ESS-COMMAND-004]: `malformed` answers at step 1 of the precedence order and
`already-initialized`, declared above it, at step 4". Need: the implementation refuses a second
initialize before reading the payload.

## Fit review

1. **Need.** Where a session already exists and the payload is also invalid, answer
   AlreadyInitialized, not Malformed, and have the suite hold an implementation to that.

2. **Class.** Semantics at one overlap point; convenience in spelling. Only the overlap (existing
   row and invalid payload) is in dispute; every other input has one answer either way.

3. **Already expressible?** Yes, by narrowing the input refusal. Reproduction (ess 0.68.0, ess/30,
   fit/repro/r2-adopter-shape.yaml):

   ```yaml
   outcomes:
     - name: already-initialized
       when_related: {entity: demo.session.Session, where: session_id == input.session_id, exists: true}
       error: demo.session.AlreadyInitialized
     - name: malformed
       when: {payload_valid: false}
       error: demo.session.Malformed
     - name: initialized
       creates: demo.session.Session
       instance: session_id
   ```

   Refused exactly as reported, hint "declare `malformed` before `already-initialized`". The idiom
   adds the complement row test to `malformed` (fit/repro/r2-idiom.yaml):

   ```yaml
     - name: malformed
       when: {payload_valid: false}
       when_related: {entity: demo.session.Session, where: session_id == input.session_id, exists: false}
       error: demo.session.Malformed
   ```

   It validates; 3 scenarios, 0 synthesis refusals, all pass on the interpreted target; mutate: 4
   mutants, 3 killed, 1 stillborn. **Gap:** the already-initialized scenario sends
   `payload_valid: true` (suite line 56), so the overlap point (an existing session with an invalid
   payload) is never sent. A target that checks the payload first passes, and no mutant drops the
   `exists: false` conjunct. By the skill's policy 1 the idiom is an answer only once the suite
   checks it.

4. **Fit with grids and targets.** The G4 order (docs/design/generic-language-features.md:440-480)
   puts input-guarded refusals at step 1 and related rows at step 4. "Open questions, answered",
   answer 1 (around line 660): "Input refusals first. Kept… moving them would change every
   validating command's answer and reopen the disclosure before_row closed." The ess/29 stricter
   row refuses a branch declared after one a later step answers wherever both can hold. #554 (open)
   and story the-precedence-rule-places-every-branch move toward one fixed order, not a per-command
   one. docs/design/input-guard-overlap-precedence.md "What synthesis does", rule 2, already sends
   each input-guarded refusal again at every overlap point, but only for input-guard siblings; a
   related-row refusal whose sibling carries an input conjunct is not covered.

5. **Second adopter.** Any "already exists" refusal beside input validation has this shape; the
   ordering question is generic.

6. **Cost.**
   - Declared per-command precedence (e.g. `before_input: true`, a sibling of `before_row`): a
     second ordering axis, a lowering change pinned by Entity Runtime, and a direct conflict with
     #554's fixed order. L, and it reopens answer 1.
   - Reordering the rule (rows before input): changes every validating command's answer; closed by
     answer 1.
   - Idiom plus an overlap witness: no source surface; synthesis sends the overlap point. M.

7. **Alternatives.** (a) `before_input:` per branch: declined. (b) Reorder steps 1 and 4:
   declined. (c) Decline with the idiom only: declined, the suite would not check the figure.
   (d) Idiom plus an overlap witness and a hint: chosen.

## Decisions

**Accept, redesigned: no new syntax; the conjunction idiom is the spelling, and synthesis witnesses
it.**

- **Synthesis.** Generalise rule 2 of input-guard-overlap-precedence.md: a related-row refusal is
  sent again with an input that satisfies the input conjunct of each sibling whose row test is its
  complement (`exists: true` against `exists: false` over the same entity and `where:`). For the
  idiom that is an existing session with `payload_valid: false`, expecting AlreadyInitialized. A
  target that checks the payload first now fails.
- **Hint.** When the later-step branch of the ESS-COMMAND-004 ordering refusal (command.rs around
  3958) is a related-row refusal, the hint also offers: "or keep the order and narrow `malformed`
  with `when_related: {…, exists: false}`".
- **Format.** No source format, no IR change. No suite format: the vocabulary is unchanged, suites
  grow by the overlap scenarios.
- **ValidationCode.** None new. Hint text moves: run `task test-xtask` and
  `cargo test -p ess-compiler --test typed_diagnostics`.
- **Files.** crates/verify/ess-conformance synthesize/row_set.rs, related_guard.rs and the
  input-guard overlap code (`overlap_inputs`/`boundary_inputs`, `subject_fact::overlaps`);
  crates/specify/ess-domain/src/command.rs (hint); docs/design/input-guard-overlap-precedence.md
  (rule 2 extended); docs/design/generic-language-features.md G4 (the idiom named under answer 1).
- **Reply to the adopter.** "The order stays: input refusals answer first, by design. Say that a
  malformed payload is only refused for a new session: add `when_related: {…, exists: false}` to
  `malformed`. The suite will send an existing session with an invalid payload and expect
  AlreadyInitialized."
- **Size.** M.

## Acceptance

In crates/verify/ess-conformance/tests/input_guard_overlap.rs:

- `a_related_row_refusal_is_sent_at_its_siblings_input_conjunct` (first red test): the idiom model's
  suite holds a scenario with an existing session and `payload_valid: false` expecting
  AlreadyInitialized.
- `a_target_that_checks_the_payload_first_fails_the_overlap`: an interpreted target with the two
  checks swapped fails that scenario.
- `a_mutant_dropping_the_exists_false_conjunct_is_killed`.
- `no_overlap_scenario_without_a_complementary_row_test`: a sibling without the complement row test
  adds no scenario.

In crates/specify/ess-domain/tests/precedence_classification.rs:

- `a_related_row_ordering_refusal_names_the_conjunction_idiom`.
