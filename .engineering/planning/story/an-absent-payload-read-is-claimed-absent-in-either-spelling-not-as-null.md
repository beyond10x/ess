---
format: aep.planning-md/3
id: story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null
kind: story
status: active
title: An absent payload read is claimed absent in either spelling, not as null
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#765
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/absent_payload_read.rs
- confidence: inferred
  path: docs/design/value-expressions.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:38:31Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:38:32Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A synthesized scenario claims an exact `null` for a top-level `Optional` event field whose payload
source reads a subject field the arrangement left absent. The interpreted target leaves an absent
event field out, so the suite fails the target it was synthesized beside. The claim is also wrong
against the specification: it is written even where the event field declares
`presence: omitted_when_absent`, under which `null` is the refused spelling.

Minimal reproduction (`format: ess/25`, one domain): the creator stores the field absent, a later
move publishes it from the row.

```yaml
entities:
  - name: example.token.PoolToken
    identity: {name: token_id, type: example.token.TokenId}
    fields:
      - {name: owner, type: String}
      - {name: agent, type: Optional<example.token.AgentName>}
    lifecycle:
      initial: Contributed
      states: [Contributed, Leased, Withdrawn]
      terminal: [Withdrawn]
      transitions:
        - {name: lease, from: [Contributed], to: Leased}
        - {name: withdraw, from: [Contributed, Leased], to: Withdrawn}
events:
  - name: example.token.TokenWithdrawn
    fields:
      - {name: token_id, type: example.token.TokenId}
      - {name: owner, type: String}
      - {name: agent, type: Optional<example.token.AgentName>}
commands:
  - name: example.token.ContributeToken
    outcomes:
      - name: contributed
        creates: example.token.PoolToken
        instance: token_id
        sets: {owner: input.owner, agent: {cleared: true}}
  - name: example.token.WithdrawToken
    outcomes:
      - name: withdrawn
        moves: example.token.PoolToken.withdraw
        instance: token_id
        sets: {agent: {cleared: true}}
        emits: [example.token.TokenWithdrawn]
        payload:
          example.token.TokenWithdrawn:
            token_id: input.token_id
            owner: {subject: owner}
            agent: {subject: agent}
```

`ess` 0.92.0:

```
$ ess specify validate --path spec
example v1 — 3 file(s), valid
$ ess verify conform run --path spec --target interpreted --report-format 2
  failed example.token.PoolToken/transition/withdraw/by/example.token.WithdrawToken/withdrawn
  failed example.token.WithdrawToken/outcome/withdrawn
  8 scenarios: 6 passed, 2 failed, 0 error, 0 unsupported
      expected:
      - example.token.TokenWithdrawn.agent = null
      observed:
      - agent was not carried
```

The same `= null` is expected with `presence: omitted_when_absent` on `TokenWithdrawn.agent`. When
the creator does not set `agent` at all the arrangement determines nothing for it, the suite claims
nothing, and all 8 scenarios pass; the failure needs a row the arrangement knows to hold the field
absent (a `{cleared: true}`, or an `Optional` input left out).

Synthesis already has the right claim for two sibling sources. A `{related: …}` read through a
reference left absent and a `{changed: <field>}` read of a field held absent are not written as a
value; the field's payload leaf is narrowed to admit no present value, so the field must be absent
in whichever spelling its declared presence admits. `{subject: <field>}` and every other source
the arrangement evaluates to `null` were left on the exact-value path.

## Acceptance

- `ess verify conform run --path <the reproduction> --target interpreted --report-format 2` exits 0
  with `8 scenarios: 8 passed`, and the same with `presence: omitted_when_absent` on
  `TokenWithdrawn.agent`.
- In the suite `ess verify conform synthesize` writes for the reproduction, the
  `expect_event_values` step of `example.token.WithdrawToken/outcome/withdrawn` has no `agent`
  entry in `payload`, and its `shape.agent` leaf admits no present value (`holds: enum` with no
  variants, `optional: true`, the declared `presence` kept).
- A test target that publishes `TokenWithdrawn.agent` as a present string in that scenario fails it
  with `ESS-CF-PAYLOAD`; one that leaves it out and one that publishes `null` both pass when the
  field declares no presence.
- With the creator changed to `sets: {owner: input.owner, agent: lessee-1}` the scenario still
  claims `agent = "lessee-1"` exactly: a present value keeps its value claim.
- A specification with no payload source that evaluates to an absent value synthesizes the suite
  bytes it did before (pinned by an existing golden suite).

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs (`determined_fields` at 10026-10060, line 10045 skips `null` only when `absent_by_reference` holds; `absent_by_reference_leaves` at 10066-10107 and `absent_by_reference` at 10111-10115; the call site in `exercise_run_in` at 4475; also `from_source` at 17150-17163, which calls `determined_payload` with `held` and applies no absent-leaf narrowing)
- cited: crates/verify/ess-conformance/src/synthesize/set_effects.rs (`absent_changed_leaves` at 1796-1840 and its call site at 1707, both replaced by the shared rule)
- inferred: crates/verify/ess-conformance/tests/absent_payload_read.rs (new; model and fake target inline, as `omitted_or_null.rs` does)
- inferred: docs/design/value-expressions.md (lines 51-53 need the absent-value exception for `{subject: <field>}`)

confidence: high — the story names the defect site and both per-source helpers, and every anchor was checked against the integration checkout (story-scoper, 2026-11-10)

Shared surfaces this story moves (epic § Shared surfaces, not scope entries): `CHANGELOG.md`. The byte-pin tables are not expected to move: no committed suite under `suites/generated/` has a payload entry equal to `null` (scoper, read-only count).

## Fit review

1. **The need.** A payload source that reads a value the scenario's arrangement knows to be absent
   is asserted as an exact `null`, a spelling the specification does not require and under
   `omitted_when_absent` forbids. Reproduction above; command output in the triage directory's
   `repro/run.txt`. The requester's proposal, labelled as theirs: either the suite expects the
   member absent, or the interpreter carries it as `null`; and, if `{subject: agent}` beside
   `sets: agent: {cleared: true}` reads the value after the clear, a refusal at `validate`.
2. **Class: defect.** The conformance design says an absent value's spelling is the field's
   declared presence and nothing else: `LeafShape::admits` admits a left-out value unless
   `null_when_absent` and a `null` unless `omitted_when_absent`
   (`crates/verify/ess-conformance/src/scenario.rs:2082-2087`;
   `docs/design/wire-presence-json-prefix.md`, "Conformance"). The exact `null` contradicts it. The
   synthesizer's own comment states the rule for the sibling source: "an absent top-level field
   may be published left out or as `null`, which an exact payload comparison cannot say"
   (`crates/verify/ess-conformance/src/synthesize.rs:10042-10047`).
3. **Already expressible?** Nothing to express: no authored key is involved. The specification
   validates (`ess specify validate --path spec` prints `valid`), and no rewrite of the payload
   keeps the field checked; dropping `agent` from the payload is refused by the one-filling rule or
   leaves it implementation-chosen, which moves the checked figure out of the specification
   (`docs/design/generic-language-features.md`, § Policy, policy 1).
4. **Fit.** Grid R, row "event `payload:`", column subject field: already open
   (`docs/design/language-grids.md:35`); this story opens no cell. The fix is general across the
   row: every payload source the arrangement evaluates to `null` for a top-level `Optional` event
   field takes the leaf claim, not one source kind. Today the rule is applied per source in two
   places, `absent_by_reference_leaves` for `{related: …}`
   (`crates/verify/ess-conformance/src/synthesize.rs:10066-10115`) and `absent_changed_leaves` for
   `{changed: …}` (`crates/verify/ess-conformance/src/synthesize/set_effects.rs:1796-1830`); the
   value path skips `null` only under `absent_by_reference`
   (`crates/verify/ess-conformance/src/synthesize.rs:10039-10048`). The two helpers become one rule
   keyed on the evaluated value. Targets: the claim uses the leaf vocabulary every runner already
   reads for the `{related: …}` case, so Rust, Go and TypeScript runners need no change. Error
   payloads are untouched: an expected error's field already compares `null` and absent as one
   value (`CHANGELOG.md:498`).
5. **A second adopter.** A shipment whose `carrier: Optional<CarrierName>` is unset until dispatch,
   cancelled before dispatch with `ShipmentCancelled.carrier: {subject: carrier}`; the creator
   takes `carrier` as an `Optional` input the scenario leaves out.
6. **Cost.** No source format, no suite format (the empty-variant leaf is written since the
   `{related: …}` rule), no keyword, no diagnostic. Suite bytes move only for a specification that
   has such a read, whose suite failed the interpreted target before. An implementation that
   published `null` there keeps passing unless the field declares `omitted_when_absent`, where it
   was wrong already.
7. **Alternatives.** (a) Change nothing: the suite keeps contradicting the declared presence and
   the reference target. (b) Make the interpreter publish `null` for every absent event field: it
   would pass this suite and then fail every `omitted_when_absent` leaf, and the exact `null` would
   still fail a real implementation that leaves the field out. (c) The requester's refusal at
   `validate`: not needed, the semantics are already decided; `{subject: <field>}` reads "the
   addressed entity's field, as it was immediately before this outcome"
   (`docs/design/value-expressions.md:45`), so the read beside a `{cleared: true}` is well defined.
   Chosen: one synthesis rule for every absent payload read.

## Decisions

accept, redesigned. The requester offered two fixes for one source kind; the change is the general
rule "an absent payload read is claimed through its leaf, in the spelling the field's presence
declares", applied to every source, replacing the two per-source helpers. The interpreter's side of
the declared presence is `story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence`.
No refusal is added.

## Source

github:beyond10x/ess#765
