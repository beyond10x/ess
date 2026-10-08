---
format: aep.planning-md/3
id: story:stored-field-equals-returned-response-value
kind: story
status: draft
title: 'Store the value an outcome returns with {response: <field>} in sets: (ess/24)'
relations:
- serves: vision:O2
revision: 1
---
## Outcome

A command outcome that returns a value it also keeps can say so: under `format: ess/24`,
`sets: {<field>: {response: <response field>}}` on a `returns: true` outcome stores exactly the
value that invocation returned, so a later command whose `when_subject` predicate compares its
input with the stored field (a verification code, a one-time PIN, a claim code) is specified and
witnessed end to end. Today the stored side and the returned side can only be declared
`{generated: true}` independently, nothing relates them, and the later check cannot be witnessed at
all. Requested in https://github.com/beyond10x/ess/issues/498.

## Fit review

1. **Need, apart from the syntax.** Domain fact: "the value this command returned to the caller is
   the value it stored, and a later command is answered by comparing its input with it." ESS cannot
   state the equality. Minimal reproduction with `ess 0.56.0`
   (`.engineering/repro/498/stored-response.yaml`: `orders.order.Order` with `pin: String`;
   `Place` creates, `returns: true`, response `pin`, `sets: {pin: {response: pin}}`; `Confirm`
   refuses with `when_subject: {predicate: pin != input.pin}`):
   `[conflicting_declaration] command.orders.order.Place.outcomes.placed.sets.pin: response and
   generated sources are admitted only in event payloads` (`ESS-COMMAND-004`, at
   `stored-response.yaml:31:5`). The only accepted variant
   (`.engineering/repro/498/stored-generated.yaml`, `pin: {generated: true}` in `sets:`)
   validates, but `ess verify conform synthesize` refuses every `Confirm` scenario:
   `refusal[ESS-SYNTH-001]: outcome orders.order.Confirm/confirmed has no scenario … no arranging
   branch sets this stored field from an input or a literal` (same for `mismatch`, the `confirm`
   transition, and `ESS-SYNTH-004` for the `Confirmed` wrong-state cell;
   `.engineering/repro/498/stored-generated.synth.txt`). So an implementation that stores one
   value and returns another passes, and the comparison command is never exercised.
   *Requester's proposal (theirs):* admit `{response: <field>}` in `sets:`, or a payload/response
   source that reads what the outcome's own `sets:` wrote.
2. **Class: gap.** The refusal is deliberate, not a contradiction of documented semantics:
   `docs/design/typed-response-outcome-payloads.md:23` ("Response and generated sources are not
   admitted in entity `sets`"), enforced at `crates/specify/ess-domain/src/command.rs:3988-4006`.
   `docs/design/value-expressions.md:87` admits `{generated: true}` in `sets:` with "Synthesis makes
   no claim about the field after the outcome". No construct relates a stored value to a returned
   one, so the fact is inexpressible, not merely verbose.
3. **Already expressible?** No. Checked: (a) `sets: {pin: input.pin}` makes the caller the authority
   on a server-minted value, which is a different domain (cf. the same objection in
   `docs/design/cross-record-and-stored-field-guards.md:30`); (b) `{generated: true}` on both sides
   validates but states no equality and leaves `Confirm` unwitnessed (repro above); (c)
   `{subject: pin}` reads the value *before* the outcome and is refused on `creates:`
   (`docs/design/value-expressions.md:43-47`); (d) an event payload `pin: {response: pin}` relates
   event to response (`ExpectResponsePayload`, `crates/verify/ess-conformance/src/scenario.rs:2070-2074`)
   but not stored row to response; (e) no outcome-level response mapping exists — a response is
   only asserted by scenarios (`docs/design/direct-library-returns.md`, "Source and scenario
   authority"). The comparison half is already expressible: `when_subject: {predicate: pin !=
   input.pin}` (`website/docs/reference/predicates.md:1042-1043`, ess/15) validates in the repro.
4. **Fit with what exists.**
   - Vocabulary: `{response: <field>}` is the existing spelling of "the value this invocation
     returned" in event payloads (`PayloadSource::ResponseField`,
     `crates/specify/ess-domain/src/command.rs:1068-1073`; `ResolvedPayloadValue::ResponseField`,
     `crates/specify/ess-compiler/src/ir.rs:1086`). Reusing it in `sets:` keeps one spelling for one
     idea; the requester's second option would add a new keyword for the reverse direction.
   - Composition: valid only on an outcome with `returns: true` (the only outcomes with an actual
     response, `docs/design/direct-library-returns.md`); refused on error outcomes, including
     `compensates: true` refusals, and on `accepts: nothing`. Nested `sets:` leaves and `affects:`
     entries of the same outcome are the sibling positions and take the same rule. Guards are
     unchanged: `when_subject` / `when_related` already read the stored field.
   - Interaction with `one_time_response:`: the one-time contract forbids its plaintext in any
     persistent field (`docs/design/one-time-response-values.md`, "Static flow checking": "Existing
     unsupported response-to-state assignments remain refusals; do not make them legal here"), and
     the validator already walks `outcome.sets` for marked response sources
     (`crates/specify/ess-domain/src/command/one_time_response.rs:66-77,106`). That refusal stays:
     `{response: f}` in `sets:` is admitted only for unmarked fields.
   - Targets: interpreter already evaluates `ResponseField` from `work.response`
     (`crates/verify/ess-conformance/src/interpret/execute.rs:2055-2057`,
     `interpret/execute/values.rs:377-395`); generated Rust keeps a command with this source an
     obligation (`crates/generate/ess-synth/src/rust/behaviour.rs:2406-2409`), Go likewise
     (`crates/generate/ess-synth/src/go/behaviour.rs:3216`); the entity runtime refuses it by name
     (`crates/generate/ess-entity-runtime/src/subset.rs:121`, `ValueExpressionUnsupported`);
     `ess verify diff` renders the source (`crates/verify/ess-diff/src/diff.rs:1179`) under
     `OutcomeSetsChanged` (`change.rs:2522`). Synthesis currently treats it as determining nothing
     (`crates/verify/ess-conformance/src/synthesize.rs:7210`) and needs the real work (Scope).
   - The red flag "a format version a nearby unreleased bump could carry" applies: ride the
     unreleased `ess/24` (`.engineering/planning/release-plan/ess-24-one-language.md:47`), no
     separate bump.
5. **Second, unrelated adopter.** A shop issues a pickup claim code: `Reserve` returns `claim_code`
   and stores it on the reservation; `Collect` refuses when `claim_code != input.claim_code`. Also
   a device-pairing flow: `StartPairing` returns a six-digit code shown on screen and stores it;
   `CompletePairing` compares. Both are domain facts, not one adopter's convention.
6. **Cost.**
   - Source: no new keyword; `ess/24` admits an existing source in a new position; `ess/14`–`ess/23`
     keep refusing it with the current diagnostic text updated to name `ess/24`. No migration (old
     documents unchanged, canonical bytes unchanged).
   - New diagnostics: `{response: f}` in `sets:` on an outcome without `returns: true`, on an error
     outcome, or naming an undeclared / type-incompatible response field.
   - IR: none expected (`ResolvedPayloadValue::ResponseField` already exists; inferred).
   - Suite: a stored-equals-returned check (a view row compared with the same invocation's actual
     response, modelled on `ExpectResponsePayload`) and a scenario value that feeds an earlier
     step's actual response field into a later command's input (beside
     `ScenarioValue::Observed`, `crates/verify/ess-conformance/src/scenario.rs:1633-1711`). That is
     new suite vocabulary: one ordinary/coverage suite pair after the highest in source
     (`ess-conformance/39`; inferred allocation), refused by older explicit pins. Go and TypeScript
     runners implement it or refuse by name.
   - Diff: inferred covered by `OutcomeSetsChanged`; confirm no `ess-diff` bump is needed.
   - Generated APIs: none; generated Rust/Go keep the command an obligation.
7. **Alternatives.**
   - *Change nothing* (use `{generated: true}` twice, or move the value into the input): loses the
     fact and leaves the comparison command unwitnessed (repro). Rejected.
   - *Requester's option B: a response/payload source reading what `sets:` wrote* (`{stored: pin}`
     or similar): adds a keyword and an outcome-level response mapping construct ESS does not have;
     the direction (response derived from state) is also what the one-time design must police.
     Rejected as more surface for the same fact.
   - *Requester's option A: `{response: <field>}` in `sets:`*: no new spelling, same meaning as in
     payloads, existing IR variant. Chosen, with the composition limits in 4 added.

## Decisions

**Accept, redesigned.** The need is a gap and the requester's first option is the chosen shape,
but it is not adoptable as written: the request does not state the `returns: true` requirement,
the error/`compensates` refusals, the `affects:`/nested sibling positions, the retained
`one_time_response` refusal, the format (`ess/24`, shared with the unreleased bump rather than a new
one), or the conformance half, without which the source would validate and remain unwitnessed.

Chosen design:
- Under `ess/24`, `PayloadSource::ResponseField` is admitted in `sets:` (top level, nested struct
  leaves, and `affects:` entries of the same outcome) of a successful outcome declaring
  `returns: true`. Meaning: after the outcome the stored field equals the value the same invocation
  returned in that response field, read as the field's type (declared conversions as for
  `input.<field>`).
- Refused with a located diagnostic: below `ess/24`; without `returns: true`; on an error outcome
  (including `compensates: true`); unknown response field; type mismatch without a conversion; a
  field marked by `one_time_response:` (existing refusal kept).
- Synthesis: the origin scenario reads the row back and requires it to equal the actual response
  field of the same invocation; a later guard over that stored field is arranged by sending the
  captured response value (equal witness) and a value differing from it (unequal witness), so the
  `Confirm`-shaped refusal and success are both witnessed.
- Targets: interpreter executes it; generated Rust/Go keep the command an obligation; entity runtime
  refuses by name; generated docs and the schema projection show it.

Requester gets one message: accepted, redesigned, with the limits above.

## Acceptance

- `ess specify validate` on an `ess/24` spec with `sets: {pin: {response: pin}}` on a
  `returns: true` outcome succeeds; the same document at `ess/23` is refused with a diagnostic
  naming `ess/24` (ess-domain test).
- Each refusal case (no `returns: true`, error outcome, `compensates: true`, unknown field, type
  mismatch, `one_time_response`-marked field) is refused at its source location (ess-domain tests).
- `ess specify compile` of an `ess/23` fixture set is byte-identical before and after
  (existing projection/canonical-bytes checks; `cargo xtask schema` regenerated, `projection-check`
  green).
- `ess verify conform synthesize` on the `orders` repro at `ess/24` yields scenarios for
  `Confirm/confirmed`, `Confirm/mismatch`, the `confirm` transition and the `Confirmed` wrong-state
  cell, with zero `ESS-SYNTH-001`/`ESS-SYNTH-004` refusals for them (ess-conformance test).
- `ess verify conform run` against the interpreted reference passes; a faulty target that stores a
  different value from the one it returned fails the origin scenario, and one that ignores the
  stored value in `Confirm` fails the mismatch scenario (ess-conformance test, mutant pair).
- An explicit older suite pin refuses the new suite vocabulary; Go and TypeScript producers run it
  or refuse it by name (ess-conformance tests).
- `ess verify diff` between a `{generated: true}` and a `{response: pin}` revision of the same
  `sets:` reports `outcome-sets-changed` (ess-diff test).
- Entity-runtime lowering refuses the source by name (`ValueExpressionUnsupported`), never drops it
  (ess-entity-runtime test).

## Scope

| Item | Cited / inferred |
|---|---|
| `crates/specify/ess-domain/src/command.rs` `validate_response_contracts` (refusal at 3988-4006): **needed** | cited |
| `crates/specify/ess-domain/src/command/**`: `one_time_response.rs` refusal kept as is (66-77, 106), new `returns`/error/type checks likely beside `value_expression.rs`: **needed** | cited / inferred |
| `crates/specify/ess-domain/src/system.rs` `FormatVersion::V24` gate (shared with ess/24 plan) | inferred |
| `crates/specify/ess-compiler/src/ir.rs`, `ir/**`: **not expected** (`ResolvedPayloadValue::ResponseField` exists, `ir.rs:1086`); `resolve.rs` may need to resolve the source in `sets:` | cited / inferred |
| `schemas/generated/ess.schema.json` via `cargo xtask schema` (only if `RawSpecFile` changes) | inferred |
| `crates/verify/ess-conformance/src/synthesize.rs` (7210, 8318, 8879, 9335 treat `ResponseField` as undetermined): **needed** | cited |
| `crates/verify/ess-conformance/src/synthesize/**` (`set_effects.rs`, `caller.rs`, guard arrangement): **likely needed** | inferred |
| `crates/verify/ess-conformance/src/scenario.rs`, `response.rs`, `admission.rs`, `coverage.rs`, `runner.rs`, Go/TS runtimes: new suite value/observation and format pair | inferred |
| `crates/verify/ess-conformance/src/interpret/execute.rs`: **probably not** (2055-2057 evaluates `ResponseField`; response prepared before effects at 1741/1826); verify sets path | cited / inferred |
| `interpret/execute/{related,existence}.rs`: **not needed** | inferred |
| `crates/verify/ess-diff/src/diff.rs`, `change.rs` (`OutcomeSetsChanged`): test only | cited / inferred |
| `crates/generate/ess-synth/src/{rust,go}/behaviour.rs`: obligation path kept; test only | cited |
| `crates/generate/ess-entity-runtime/src/subset.rs:121`: refusal kept; test only | cited |
| `docs/design/typed-response-outcome-payloads.md:23` and a design page for this construct; `website/docs/reference/` spec-versions entry for `ess/24` | cited / inferred |

## Related finding

ESS-SYNTH-008 reproduces on 0.56.0 when `one_time_response:` sits on a `moves:` outcome beside a
`wrong_state` branch: `.engineering/repro/498/one-time-wrong-state.yaml` (validates;
`catalog.item.Claim` with `claimed` = `moves`, `returns: true`, `one_time_response: [code]`, and
`wrong-state` = `wrong_state: true`). `ess verify conform synthesize --path one-time-wrong-state.yaml
--target ir` (`.engineering/repro/498/one-time-wrong-state.synth.txt`):

```text
refused: refusal[ESS-SYNTH-008]: outcome catalog.item.Claim/claimed has no scenario `catalog.item.Claim/disclosure/claimed/code/command/catalog.item.Claim/wrong-state/as/anonymous`
  its strategy is `arrange_state` and it declares no guard
  help: `TestStrategy` and `OutcomeCondition` have drifted apart in `ess-domain`
```

The same run also refuses the retry cell with `ESS-SYNTH-001` ("the identical retry has no uniquely
witnessed post-state branch"), whose help text ("give the field a type that has a finite value")
does not fit the cause. Not filed.
