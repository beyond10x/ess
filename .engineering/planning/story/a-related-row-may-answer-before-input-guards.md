---
format: aep.planning-md/3
id: story:a-related-row-may-answer-before-input-guards
kind: story
status: draft
title: A command may declare that an existing related row answers before its input guards
summary: 'Adopter report R2: a second initialize answers already-initialized before its payload is read; the precedence rule forbids that order.'
tags:
- ess-0.71.0
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#554
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:generic-guard-precedence
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/edge/ess-xtask
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/precedence_classification.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/input_guard_overlap.rs
- confidence: cited
  path: docs/design/generic-language-features.md
- confidence: cited
  path: docs/design/input-guard-overlap-precedence.md
revision: 8
---
## Need

A create command refuses a second call for an identity a row already carries before it reads the
rest of the request: an existing session with an invalid payload answers `already-initialized`,
not `malformed`. The one precedence rule answers input refusals at step 1 and related rows at step
4 (`docs/design/generic-language-features.md:482-526`), so the order as the service checks it is
refused, and the spelling that states it (the input refusal narrowed to a missing row) validates
but is never checked at the point where the two branches disagree.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/b-row-before-input`, `repro/b-idiom`
of the triage of beyond10x/ess#554):

```yaml
commands:
  - name: demo.session.Initialize
    input:
      - {name: session_id, type: demo.session.SessionId}
      - {name: payload_valid, type: Boolean}
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

```text
$ ess specify validate --path b-row-before-input
error[ESS-COMMAND-004]: `malformed` answers at step 1 of the precedence order and `already-initialized`, declared above it, at step 4; where both hold, the declaration order and the precedence order disagree
help: declare `malformed` before `already-initialized`: step 1 answers first in either order
exit 1
```

The idiom adds the complement row test to `malformed`
(`when_related: {entity: demo.session.Session, where: session_id == input.session_id, exists: false}`).
It validates and synthesizes 3 scenarios, but every scenario sends `payload_valid: true` except
`malformed`'s, which sends no stored session; the overlap (a stored session, `payload_valid: false`)
is never sent. `ess verify conform mutate --target interpreted`: 4 mutants, 3 killed, 1 stillborn;
no mutant drops the `exists: false` conjunct. A target that checks the payload first passes.

## Acceptance

- `ess verify conform synthesize --path <idiom model>` writes a scenario that arranges a stored
  session, sends that `session_id` with `payload_valid: false`, and expects `already-initialized`
  with no event; `ess verify conform run --target interpreted` passes it.
- The same suite run against an interpreted target with `malformed`'s `when_related:` conjunct
  dropped fails that scenario (`ess verify conform mutate --target interpreted` reports the
  conjunct-drop mutant `killed`).
- A sibling input refusal without a complementary row test (`exists: false` over the same entity
  and `where:`) adds no scenario: a model without one keeps its suite bytes.
- `ess specify validate --path b-row-before-input` exits 1 with the same ESS-COMMAND-004 message,
  and its `help:` also names the narrowing: "or keep the order and narrow `malformed` with
  `when_related: {…, exists: false}`".

Pinned by `crates/verify/ess-conformance/tests/input_guard_overlap.rs`
(`a_related_row_refusal_is_sent_at_its_siblings_input_conjunct`,
`a_target_that_checks_the_payload_first_fails_the_overlap`,
`a_mutant_dropping_the_exists_false_conjunct_is_killed`,
`no_overlap_scenario_without_a_complementary_row_test`) and
`crates/specify/ess-domain/tests/precedence_classification.rs`
(`a_related_row_ordering_refusal_names_the_conjunction_idiom`).

## Scope

- cited: crates/specify/ess-domain/src/command.rs (ordering refusal and its hint, around line 4018)
- inferred: crates/verify/ess-conformance/src/synthesize/row_set.rs
- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- inferred: crates/verify/ess-conformance/src/synthesize.rs (`overlap_inputs`)
- cited: crates/verify/ess-conformance/tests/input_guard_overlap.rs
- cited: crates/specify/ess-domain/tests/precedence_classification.rs
- cited: docs/design/input-guard-overlap-precedence.md (§ "What synthesis does", rule 2)
- cited: docs/design/generic-language-features.md (§ G4, answer 1 names the idiom)
- inferred: crates/edge/ess-xtask (typed diagnostic snapshots, hint text)
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** Where a row with the requested identity exists and the payload is also invalid, the
   specification must say which error answers and the suite must hold a target to it. Reproduction
   above (0.92.0, `ess/36`). Requester's proposal (theirs, issue comment of 2026-10-23, ess 0.67.0):
   declare the `when_related: {exists: true}` refusal above the `when:` refusal and have that order
   honoured.
2. **Class.** Gap at one overlap point; the rest is convenience. Every input outside the overlap
   has one answer in either order; only an existing row with an invalid payload is in dispute
   (run.txt § B).
3. **Already expressible?** Yes in the source: narrowing `malformed` with the complement row test
   validates (run.txt § B, `b-idiom`, exit 0). Not yet checked: no scenario sends the overlap, and
   no mutant drops the conjunct (run.txt § B, mutate). By policy 1
   (`docs/design/generic-language-features.md` § Policy, 1) the idiom is an answer only once the
   suite checks the figure, which is this story's change.
4. **Fit.** Grid R row `when:` beside `when_related:` row-set reads (`docs/design/language-grids.md`
   grid R) is already admitted; no cell opens. The rule keeps input refusals at step 1
   (`generic-language-features.md:491`, answer 1 at `:705`): "moving them would change every
   validating command's answer". The witness extends rule 2 of
   `docs/design/input-guard-overlap-precedence.md:76` (each input-guarded refusal sent again at
   every overlap point) from input-guard siblings to a related-row refusal whose sibling carries
   the complementary row test. Targets: the interpreter already answers the idiom correctly
   (run.txt § B, 3/3 pass); generated Rust and Go keep row-set commands an obligation and Entity
   Runtime refuses `when_related:` by name (`generic-language-features.md:566-584`); no target
   changes.
5. **Second adopter.** A signup that refuses a taken user name before it validates the password
   policy; an "already exists" refusal beside any input validation has this shape.
6. **Cost.** No source format, no IR change, no new key, no new diagnostic code. The hint text
   changes (typed diagnostic snapshots). Suites of models with the complementary pair grow by one
   scenario per overlap; no suite format moves, the vocabulary is unchanged.
7. **Alternatives.** (a) A per-branch `before_input:` key: a second ordering axis beside
   `before_row`, conflicts with the one rule; refused. (b) Reorder the rule (rows before input):
   closed by answer 1. (c) Decline with the idiom alone: the suite would not check the overlap,
   so policy 1 rules it out. (d) Chosen: the idiom is the spelling, synthesis witnesses its
   overlap, and the ordering refusal's hint names it. The requester's order is kept as
   behaviour; their spelling is changed to the narrowing.

## Decisions

**Accept, redesigned.** No new syntax: the input refusal narrowed with the complementary row test
is the spelling. Synthesis sends a related-row refusal again at each sibling's input conjunct where
that sibling carries the complementary row test (`exists: true` against `exists: false` over the
same entity and `where:`), expecting the related-row refusal. ESS-COMMAND-004's hint offers the
narrowing when the later-step branch is a related-row refusal. No source format, no IR change, no
suite format. What changed from the request: the order is stated by narrowing, not by a
declaration-order exception. Reply: "The order stays: input refusals answer first, by design. Say
that a malformed payload is only refused for a new session: add `when_related: {…, exists: false}`
to `malformed`. The suite sends an existing session with an invalid payload and expects
`already-initialized`."

## Source

github:beyond10x/ess#554
