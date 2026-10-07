---
format: aep.planning-md/3
id: story:an-outcome-that-only-stores-is-observed-through-a-view
kind: story
status: draft
title: An accepted outcome that stores a row and emits nothing is admitted when a declared view observes it
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#492
relations:
- serves: vision:O2
revision: 2
---
## Outcome

Gap 2 of https://github.com/beyond10x/ess/issues/492: an outcome with `creates:` or `updates:` and
no event and no error is admitted when a declared view of that entity can observe the change, and
synthesis checks it by reading that view. Today `ESS-COMMAND-007` refuses it as unobservable;
`accepts: nothing` (#144) declares a no-op, which this is not.

Spec first; fit review owed.

## Fit review

Reproductions: `target/wave-scratch/fit/492/g2/` (ess 0.56.0, `format: ess/23`).

1. **Need.** A write stores a row and announces nothing. No event, no error and no response follow.
   The row is then visible through a declared view of that entity. Minimal reproduction
   (`g2/requested.yaml`):

   ```yaml
   outcomes:
     - name: typed-stored
       when: defined(kind)
       creates: fit.samples.Sample
       instance: sample_id
       sets: {kind: input.kind, amount: input.amount}
       emits: [fit.samples.TypedSampleStored]
       payload: {fit.samples.TypedSampleStored: {sample_id: input.sample_id, kind: input.kind}}
     - name: untyped-stored          # stores the row, announces nothing
       creates: fit.samples.Sample
       instance: sample_id
       sets: {amount: input.amount}
   views:
     - {name: fit.samples.Samples, source: fit.samples.Sample, consistency: read_your_writes, fields: [sample_id, kind, amount]}
   ```

   The view is abbreviated here; the file declares typed fields. `ess specify validate` exits 1:
   `error[ESS-COMMAND-007]: outcome `untyped-stored` neither emits an event nor names an error, so
   nothing about it is observable and no test can check it`.
   Requester's syntax (theirs): `- {name: untyped-recorded, when: not defined(event_type), creates:
   demo.store.Row, instance: id}`, to be admitted when a declared view of the entity can observe the
   change, with synthesis checking it by reading that view.

2. **Class: gap.** The rule's stated reason does not hold where a view exists. ESS already exempts
   the same case for removal: "A removal is observable without an event: the row is gone from every
   immediate view" (`crates/specify/ess-domain/src/command.rs:2829-2833`), and the exemption list
   is `:2845-2864`. Synthesis also already reads the view after a creation and asserts the row
   (`g2/with-event.yaml`, scenario `untyped-stored`: `query_view fit.samples.Samples`, then
   `expect_view contains {amount: 1, sample_id: observed from TypedSampleStored}`). The only role
   the event plays there is supplying the identity. A domain fact, "stored, not announced", cannot be
   stated today. It is not classed as a defect because the refusal is documented behaviour
   (`docs/design/outcome-shapes.md:21`).

3. **Already expressible? No.**
   - An invented event specifies behaviour the system does not have, which ESS refuses on principle:
     "Requiring an invented event or a last-result view would specify behavior the library does not
     provide" (`docs/design/direct-library-returns.md:3-5`).
   - `accepts: nothing` declares that no immediate view changes (`docs/design/outcome-shapes.md:104-107`),
     so it is false here.
   - `returns: true` needs a response the write may not have. Even with one, the creation is then
     refused for its identity: `g2/returns.yaml` gets `error[ESS-COMMAND-001]: outcome
     `untyped-stored` ... acts on the instance named by `sample_id`, which is no field of an emitted
     event of it`. A creation reads `instance:` from an emitted event only
     (`command.rs:1001`, `Effect::Creates => InstanceSurface::EmittedEvent`).
   - A create-or-update pair with no events (`g2/upsert.yaml`) gets `ESS-COMMAND-007` on both
     branches, plus `ESS-COMMAND-004` ("its payload does not take that identity from the input").
   - Siblings: a store-only `updates:` (`g2/upsert.yaml`, branch `restored`) and a store-only
     `moves:` (`g2/moves.yaml`) are refused by `ESS-COMMAND-007` alone.

4. **Fit.**
   - Vocabulary: no new key. Whether the outcome is admitted is read off the model (a view exists),
     the way `deletes:` is.
   - Composition with `instance:`: as requested, the outcome passes `ESS-COMMAND-007` and is then
     refused by `ESS-COMMAND-001`, because a creation's identity is read from an emitted event. The
     request does not address this, and the design has to.
   - Siblings: `updates:` (the requester names it) and `moves:` (not named; observable where a view
     projects `state`, which synthesis already asserts, `crates/verify/ess-conformance/src/synthesize.rs:9880-9897`)
     raise the same question.
   - Guards: unaffected. The branch is selected as before.
   - Targets:
     - Generated servers already answer an accepted outcome with no event (`deletes:`, `accepts: nothing`).
     - Suite: no new step. `query_view` and `expect_view` exist
       (`crates/verify/ess-conformance/src/runner.rs:668-669`). The identity becomes the input
       literal instead of `observed`.
     - Entity Runtime: I don't know whether entity-core admits a creation that publishes nothing.
       Its lowering table refuses "a creation [that] does not observe its identity in an emitted
       event" as `AmbiguousInstanceBinding` (`website/docs/reference/entity-runtime-lowering.md:60,102`),
       so a refusal by name is the floor.
     - `ess verify diff`: I don't know how an `emits:` list emptied by a revision is classified today.

5. **Second adopter.** A settings service stores `PUT /settings/{key}` and answers 204, with no
   event and no body. A `Settings` view returns the stored value. Append-only ingest endpoints that
   only persist (audit records, telemetry samples) have the same shape.

6. **Cost.**
   - Format: next source format (ess/24), refused below it as today, as ess/15 fenced `deletes:` and
     `accepts: nothing` (`docs/design/outcome-shapes.md:128-129`).
   - Diagnostics: none new. `ESS-COMMAND-007` keeps its code, and its hint gains "or declare a view of
     `<entity>` that projects `<identity>`".
   - Model: the creation's instance surface becomes the input field when the outcome emits nothing.
     No IR field is expected to be added; whether the IR records the surface needs checking.
   - Migration: none, because only documents refused today are affected.
   - Generated API: unchanged for existing models.

7. **Considered.**
   - (a) Change nothing and emit an invented event: refused, because it specifies behaviour that does
     not exist.
   - (b) A marker key naming the observing view (`observed_by: <view>`): refused. It is new surface,
     and the model already says which views read the entity, as `deletes:` relies on.
   - (c) The requester's rule as written: changed. As written it is refused next by
     `ESS-COMMAND-001`, and it leaves `moves:` out.
   - (d) Chosen: the requester's rule, with the identity read from the input and the sibling verbs
     included.

## Decisions

**Accept, redesigned.**

An accepting outcome with `creates:`, `updates:` (with `sets:`) or `moves:` and no event, no error
and no `returns:` is admitted from the next source format when a declared row view of that entity
projects its identity. For `moves:`, the view must also project `state`. The redesign changes the
request in three ways:

1. A store-only `creates:` reads `instance:` from the command input, because there is no event to
   publish it. `{generated: true}` stays refused, since nothing would name the row.
2. `moves:` is included as a sibling.
3. An `updates:` with no `sets:` stays `ESS-COMMAND-007`, because it changes nothing a view can show.

Synthesis reads the view after the command, as it already does. The created or changed row is
asserted with the identity as the input literal, and a target that does not store fails the read.
If no view admits the arranged row, synthesis refuses by name. No new suite step.

Files:

- Domain: `crates/specify/ess-domain/src/command.rs`, around `:1000-1004` for the instance surface
  and `:2845-2864` for the `EmptyChange` exemption.
- Format fence.
- Synthesis: `crates/verify/ess-conformance/src/synthesize.rs`, the view expectation for a subject
  with no event.
- Entity Runtime: lower it or refuse it by name.
- Docs: `website/docs/guides/specify/commands-and-outcomes.md` and `docs/design/outcome-shapes.md`.

Acceptance: `g2/requested.yaml` under the new format validates and synthesizes `untyped-stored`
with a `query_view`/`expect_view contains` read. A target that drops the row fails it.
