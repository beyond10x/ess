---
format: aep.planning-md/3
id: specification:consume-and-issue-is-a-command-plus-a-binding
kind: specification
status: draft
title: Consuming one record and issuing another with a minted identity is a command plus a binding
relations:
- serves: vision:O2
revision: 1
---
## Outcome

An adopter whose command consumes one single-use record and issues a new record of another entity,
with an identity the server mints, has a validated, synthesizable ESS form for it today: the
consuming command moves its subject and emits the minted identity in an event, and a binding on
that event invokes a creating command, so the issued record exists under its own identity and later
commands address it. The reply on https://github.com/beyond10x/ess/issues/497 shows that idiom
running on `ess 0.56.0`. The proposed `affects: [{creates: {instance: …}}]` is not adopted.
Creating the record in the same atomic step as the consumption stays an open question for a later
format (see Decisions).

## Fit review

1. **Need, apart from the syntax.** One command consumes one record and issues one new record of
   another entity, and the server mints the new record's identity. In the minimal reproduction,
   `PlaceOrder` accepts a `Quote` (`Open -> Accepted`, single-use) and issues an `Order` whose
   `order_id` is minted. The issue's claim is that "the issued record cannot be addressed by later
   commands". The requester's proposed syntax, labelled as theirs:
   `affects: [{entity: <E>, creates: {instance: <event field>}, sets: {…}}]`.
   Reproduced on `ess 0.56.0` (`ess --version`) with
   `.engineering/repro/497/base.yaml` (moves + creates on one outcome):
   `[conflicting_declaration] command.demo.orders.PlaceOrder.outcomes.placed.creates: outcome `placed` declares ``creates``, ``moves``; one outcome does one thing to one entity`.
   With `.engineering/repro/497/affects-create.yaml` (an `affects:` entry with neither
   selector): `[missing_declaration] command.demo.orders.PlaceOrder.outcomes.placed.affects[0]: outcome `placed` declares `affects[0]` with neither `where:` nor `each:`, so the entry names no rows`.
   The proposal as written (`.engineering/repro/497/proposal.yaml`):
   `unknown field `creates`, expected one of `entity`, `where`, `each`, `instance`, `sets`, `moves`, `deletes``.
   That is the parse refusal from `#[serde(deny_unknown_fields)]` on `RawAffect`
   (`crates/specify/ess-domain/src/command/set_effects.rs:88-89`).

2. **Class: convenience.** Both refusals are documented semantics, so they are not defects. The
   rule is at `docs/design/outcome-shapes.md:78` ("one outcome does one thing to one entity as
   today") and `crates/specify/ess-domain/src/command.rs:6448-6461` (`ESS-COMMAND-004`, whose hint
   says to split the outcome). The `affects:` selector requirement is at
   `docs/design/set-effects-over-filtered-instances.md:239-243`. The domain fact the issue says
   is missing (a new record with a minted identity that later commands can address) is
   expressible today in a longer form (question 3). The one thing the idiom does not state is that
   the creation is atomic with the consumption. Whether that is a domain fact or a convenience is
   the open question recorded under Decisions.

3. **Already expressible: yes, as a command plus a binding.** `.engineering/repro/497/idiom.yaml`
   has the following parts:
   - `PlaceOrder` has `moves: demo.orders.Quote.accept` and emits
     `OrderPlaced {quote_id: input.quote_id, order_id: {generated: true}, buyer: {subject: buyer}}`.
   - `RecordOrder` has `creates: demo.orders.Order` with `instance: order_id` from its input.
   - The binding `record-order-on-placed` maps `order_id: event.order_id` and `buyer: event.buyer`,
     with `delivery: at_least_once` and `on_failure: retry`.

   `RecordOrder` is in no actor's `may:`, so only the binding invokes it.
   `ess specify validate --path idiom.yaml` prints `demo v1 — 1 file(s), valid`.
   `ess verify conform synthesize --path idiom.yaml` prints `8 scenario(s) (0 authored), 2 refusal(s)`.
   Those scenarios include `PlaceOrder/outcome/placed`, `RecordOrder/outcome/recorded` and
   `record-order-on-placed/binding/{flow,mapping,delivery}`. The two refusals are
   `Quote/state/Accepted/refuses/PlaceOrder` (ESS-SYNTH-001) and `binding/on-failure`
   (ESS-SYNTH-010, help: "declare the branch that fails `external:`"). Neither one is about the
   creation. The minted identity reaches the caller in the event payload and names a stored
   `Order` that later commands can address. ESS already points cross-record consequences at
   bindings: `docs/design/set-effects-over-filtered-instances.md:197` says "a domain boundary is
   where bindings belong".
   One finding came up along the way: adding an `existing_instance:` refusal to `RecordOrder` with a
   keyed `on_failure` (`.engineering/repro/497/idiom-existence.yaml`) validates, but
   synthesis then refuses `binding/flow`, `binding/delivery` and `binding/refusal/already-recorded`
   with ESS-SYNTH-010. That is a separate synthesis gap. It is noted here and not filed.

4. **Fit of the proposed design: it fails three checks.**
   - *Targets.* Every `affects:` entry is refused by Entity Runtime with `SetEffectUnsupported` and
     by Rust, Go, Web and Clap synthesis with `MissingRepresentation`
     (`docs/design/set-effects-over-filtered-instances.md:287-288, 294-296`). The proposed creation
     would run in the interpreter only. The binding idiom runs on every target that dispatches
     bindings (`docs/design/binding-delivery-guarantees.md`, "The synthesized transport does not
     fork").
   - *Vocabulary.* `creates:` is an outcome verb whose value is an entity name
     (`examples/gatepass/domains/visit.yaml`, `creates: gatepass.visit.Visit` beside
     `instance: visit_id`). The proposal reuses that key inside `affects:` with a mapping value
     `{instance: …}`, so `creates:` would mean two shapes depending on where it is written. An
     `affects:` entry already spells creation as `each:` + `instance:`
     (`set-effects-over-filtered-instances.md:216-219`).
   - *Composition.* `affects:` `sets:` refuses `{subject: …}` "under a set effect"
     (`set-effects-over-filtered-instances.md:47-49`). The proposal's `sets: {holder: {subject: holder}}`
     needs that refusal lifted for one entry kind only, which leaves sibling entries unable to say
     the same thing.

5. **Second, unrelated adopter.** A shop converts a cart into an order: the cart moves to
   `CheckedOut`, and an order with a minted identity is created. A help desk escalates a chat into
   a ticket. Both are served by the same idiom (command plus binding). So the need is general, but
   the request is not local policy, and the general need is already met by the idiom.

6. **Cost of the proposal.** A new key in `RawAffect` means a `RawSpecFile` change, a
   `cargo xtask schema` regeneration and a source format bump. `ess/23` is the newest supported
   format (`website/docs/reference/spec-versions.md:65`), so the bump would go into `ess/24`, which
   is unscheduled (`.engineering/planning/release-plan/ess-24-one-language.md`). It would also
   need:
   - a new interpreter path in `interpret/execute.rs`;
   - a new synthesis segment in `synthesize/**`, because the existing `each:` segment arranges by
     input identity and a minted identity has to be read back from an event;
   - a new `ess-diff` line on `outcome-set-effect-changed`, and a new generated-docs sentence;
   - a new refusal on four generated targets.

   The idiom costs nothing: no keyword, no diagnostic, no format bump, no migration.

7. **Designs considered.**
   - (a) *Change nothing and answer with the idiom.* No surface added, and it works on every
     target today (question 3).
   - (b) *The proposal as written.* Refused for the vocabulary, composition and target reasons in
     question 4.
   - (c) *Lift ESS-COMMAND-004 for exactly `moves:` + `creates:`, with `instance:` and a second
     `creates_instance:`.* This gives one outcome two subjects. Every single-subject reader
     (synthesis arrangement, `wrong_state:`, `unknown_instance:` precedence, Entity Runtime's
     one-instance operation) would then need a rule for which subject is meant. That is
     ADR-level, not a story.
   - (d) *An `affects:` entry creating one row with `instance: {generated: true}`.* This fits the
     `each:` vocabulary better than (b). It still inherits every target's `affects:` refusal, needs
     `ess/24`, and still needs a decision on whether a set effect may read `{subject: …}`.

   (a) wins: the request's reproduction shows that the direct spellings are refused, but it does
   not show that the need cannot be stated. (d) is the shape to prefer if atomicity is later shown
   to be a domain fact.

## Decisions

**Decline, with the idiom.** The need (consume one record, issue a new record with a minted
identity that later commands address) is expressible and synthesizable today as a consuming command
that emits the minted identity, plus a binding that invokes a creating command no actor is granted
(`.engineering/repro/497/idiom.yaml`, valid on 0.56.0, 8 scenarios including the binding's
flow, mapping and delivery). The proposed `affects: [{creates: {instance: …}}]` is declined for
three reasons. It gives `creates:` a second shape. It needs `{subject: …}` admitted under one kind
of set effect only. It would run only in the interpreter, because every generated target refuses
`affects:`. It would also cost a source format bump.

The idiom is recorded as a `specification` artifact that states it, with the validate and
synthesize output above. The issue is answered with the idiom and closed as declined.

**Open question, not decided here:** is "the issued record exists in the same atomic step as the
consumption" a domain fact ESS must state? In the idiom the creation is eventually consistent, and
`on_failure: retry` covers it. If an adopter shows a behaviour that the binding form gets wrong (a
read between the two steps that must not see the consumed record without the issued one), that
reopens as a `decision-blocker` on design (d) for `ess/24`.

## Acceptance

- `ess specify validate --path .engineering/repro/497/idiom.yaml` prints `valid` on the
  newest release. The same document, carried as the `specification` artifact's example, validates
  in the artifact's own check.
- `ess verify conform synthesize` on the idiom lists `<binding>/binding/flow`, `/mapping` and
  `/delivery`, and `<creating command>/outcome/<created>`, with no ESS-SYNTH-010 for flow or
  delivery.
- `ess specify validate` on the moves + creates form still refuses with `conflicting_declaration`
  at `outcomes.<name>.creates` (`crates/specify/ess-compiler/tests/typed_diagnostics.rs`,
  `ESS-COMMAND-004` cases, unchanged).
- An `affects:` entry carrying `creates:` is still refused at parse ("unknown field `creates`").
  There is no `RawAffect` change, so `cargo xtask schema` produces no diff in
  `schemas/generated/ess.schema.json`.
- The reply on https://github.com/beyond10x/ess/issues/497 carries the idiom and the decision.

## Scope

- `.engineering/planning/` `specification` artifact stating the idiom: cited (skill
  `.agents/skills/assessing-external-requests/SKILL.md`, decision table, "decline, with the idiom").
- No source change. `crates/specify/ess-domain/src/command/set_effects.rs` (`RawAffect`, `affects`)
  is unchanged: cited, `set_effects.rs:88-89, 407`.
- No format change: `crates/specify/ess-domain/src/system.rs` `SUPPORTED_FORMATS` is unchanged
  (cited, `system.rs:53`).
- Optional follow-up, not in this story: the ESS-SYNTH-010 binding refusals when the invoked
  command has an `existing_instance:` refusal (`.engineering/repro/497/idiom-existence.yaml`).
  Its cause in `synthesize/**` is inferred and was not read.
- Held files: needs none of `crates/verify/ess-conformance/src/synthesize.rs`, `src/synthesize/**`,
  `src/interpret/execute.rs`, `interpret/execute/{related,existence}.rs`,
  `crates/specify/ess-domain/src/command.rs`, `command/**`,
  `crates/specify/ess-compiler/src/ir.rs` or `ir/**`.
