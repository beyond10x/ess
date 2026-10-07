---
format: aep.planning-md/3
id: story:binding-conditions-compare-boolean-event-fields
kind: story
status: draft
title: A binding condition may compare a Boolean event field with true or false
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

Gap 1 of https://github.com/beyond10x/ess/issues/492: `where: [event.is_bridged == false]` in a
binding condition validates and is witnessed like the enum comparisons (`condition-false` flips the
flag). Today it is refused as `ESS-BINDING-002` ("a binding condition compares a String or enum leaf
with a text literal").

Spec first; fit review owed (is an existing construct or idiom an answer? if so, the issue is told).

## Fit review

Reproductions: `target/wave-scratch/fit/492/g1/` (ess 0.56.0, `format: ess/23`).

1. **Need.** An event carries a `Boolean` discriminator, and a binding must fire only for one of its
   two values. Two bindings on one event split by the flag, each with an enum conjunct beside it.
   Minimal reproduction (`g1/requested.yaml`):

   ```yaml
   events:
     - name: fit.calls.CallFinished
       fields:
         - {name: call_id, type: String}
         - {name: direction, type: fit.calls.Direction}   # enum [inbound, outbound]
         - {name: bridged, type: Boolean}
   bindings:
     - id: direct
       when:
         event: fit.calls.CallFinished
         where: [event.direction == inbound, event.bridged == false]
       invoke: {command: fit.calls.RecordDirect}
   ```

   `ess specify validate --path requested.yaml` exits 1: `error[ESS-BINDING-002]: `event.bridged` is
   compared with `false`, which is not text; a binding condition compares a String or enum leaf with
   a text literal`, once per binding.
   Requester's syntax (theirs): `where: [event.type == INBOUND, event.domain == EXTERNAL,
   event.is_bridged == false]`, witnessed like the enum comparisons, with `condition-false` flipping
   the flag.

2. **Class: gap.** This is not a defect. The binding design limits comparisons to String and enum
   leaves on purpose (`docs/design/conditional-binding-failure-policies.md:121-122`, "comparisons end
   at String or enum leaves"), and the code does the same (`crates/specify/ess-domain/src/binding/condition.rs:198`
   resolves only `String` and enum leaves, `:351-376` `literal_fits`, `:343-344` the evaluator
   stores only text). It is a gap because no spelling keeps the binding's claim (question 3). The
   sibling positions all compare Booleans already: a command guard (`when: bridged == true`
   validates in `g1/idiom-command-guard.yaml`), a view filter and an invariant
   (`website/docs/reference/predicates.md:188`: "`true` and `false` are booleans"; `:1324-1325`).

3. **Already expressible? No.** Every attempt was refused or changes the claim:
   - bare path `event.bridged` / `not event.bridged` (`g1/barepath.yaml`): `ESS-BINDING-009`, "outside
     the bounded binding condition";
   - quoted `event.bridged == "false"` (`g1/quoted.yaml`): `ESS-BINDING-002`, "`event.bridged` is not
     a String or enum leaf";
   - `defined(event.bridged)` validates (`g1/defined.yaml`), but it tests presence, not the value;
   - move the test into the invoked command, `bridged-skipped: {when: bridged == true, accepts:
     nothing}` (`g1/idiom-command-guard.yaml`): validates, but synthesis loses the binding's
     `flow` and `delivery` scenarios (`refusal[ESS-SYNTH-010] ... whose branch is decided by an input
     the event fills`, 9 scenarios / 6 refusals against 11 / 2 for the same model without the
     conjunct, `g1/without-conjunct.yaml`). It also says that bridged calls invoke the writer, which
     the system does not do;
   - re-declaring the flag as an enum changes the event's wire type from JSON boolean to text, so a
     retrofit of a published event cannot do it.

4. **Fit.**
   - Vocabulary: `==`/`!=` against `true`/`false` is how guards, filters and invariants already spell
     a Boolean comparison (`predicates.md:188`). It adds no new key.
   - Composition: the comparison proves presence like any definite comparison
     (`conditional-binding-failure-policies.md:135-156`), and it works under `all`/`any`/`not`.
   - Siblings: the binding selection `where` is the same bounded fragment and refuses the same thing.
     `g1/selection-sibling.yaml` with `where: item.held == false` gets `ESS-BINDING-009`, "selection
     admits only Always/Never/Defined, String or enum Eq/Ne literals ..."
     (`crates/specify/ess-domain/src/selection.rs:285-290`). The request leaves it unable to say
     the same thing, which is a red flag, so the design covers both.
   - Targets: the interpreter evaluator (`condition.rs:343-344`), synthesis `condition-false`
     (`crates/verify/ess-conformance/src/synthesize/binding_condition.rs:979` only varies enum
     variants), generated Rust/Go dispatch (`crates/generate/ess-synth/src/condition.rs:44-52`,
     `End` has String/Enum/Other), web through the Rust system, docs/graph/AsyncAPI rendering, and
     `ess verify diff` (`binding/predicate-changed` is already classified,
     `conditional-binding-failure-policies.md:95`). I don't know whether Entity Runtime reads
     binding conditions at all; the lowering page has no row for a binding cause
     (`website/docs/reference/entity-runtime-lowering.md:10`, bindings are host obligations).
   - Canonical bytes: today a Boolean leaf is serialized `{"kind":"other"}` in the IR
     (`ess specify compile --path g1/defined.yaml` → `"event.bridged":{...,"leaf":{"kind":"other"}}`).
     If a `boolean` leaf were minted unconditionally, the IR bytes of every existing `defined()` over
     a Boolean would change.

5. **Second adopter.** A shop's `OrderPlaced` event carries `is_gift: Boolean`. A gift-wrapping
   binding fires only on `event.is_gift == true`, and a receipt-printing binding only on `== false`.
   Event flags like `is_test`, `is_retry` and `opt_in` are common in published event schemas, and a
   consumer of someone else's event cannot re-type them.

6. **Cost.**
   - Format: one admitted comparison in the next source format (ess/24; none exists in tree,
     `crates/specify/ess-domain/src/system.rs:332` is `V23`), refused below it as today.
   - IR: a new `Leaf` variant `boolean`, minted only under that format, so existing IR keeps its bytes.
   - Diagnostics: none new. `ESS-BINDING-002` still refuses a text literal against a Boolean leaf.
   - Migration: none, because only documents that were refused before are affected.
   - Generated API: the Rust/Go condition functions gain a Boolean comparison, and existing
     generated code is unchanged.
   - Suite steps: none new (`expect_no_invocation` exists).

7. **Considered.**
   - (a) Change nothing: refused (question 3). No spelling keeps the claim, and the command-guard
     route costs the binding's flow and delivery witnesses.
   - (b) The requester's design, conditions only: changed, because it leaves the binding selection
     `where` unable to say the same and it would change existing IR bytes.
   - (c) Also admit a bare Boolean path (`event.flag`, `not event.flag`): refused. It would be a second
     spelling inside the bounded fragment, and the fragment has no truthiness, so a bare path over a
     String would mean something else. The explicit comparison is the spelling every predicate
     position shares.
   - (d) Chosen: (b) extended to the shared fragment, with the leaf fenced by format.

## Decisions

**Accept, redesigned.**

The need is real. No idiom keeps the binding's claim. The comparison reuses the predicate spelling
every other position already accepts. The redesign changes three things in the request:

- **Scope.** The Boolean leaf goes into the bounded fragment that binding conditions and binding
  selections share, not into conditions alone. Otherwise the selection `where` stays unable to say
  the same thing.
- **Grammar.** Only `==`/`!=` against `true`/`false`. A bare path stays `ESS-BINDING-009`, and a
  text literal against a Boolean stays `ESS-BINDING-002`.
- **Bytes.** The IR leaf `boolean` is minted only from the next source format, so `defined()` over a
  Boolean keeps `{"kind":"other"}` below it.

Design. Spec first, then:

- `crates/specify/ess-domain/src/binding/condition.rs`: `Leaf::Boolean`, `literal_fits` admits
  `FactValue::Bool`, and `evaluate` stores `Node::Bool` as `FactValue::Bool`.
- `crates/specify/ess-domain/src/selection.rs:285`: the same leaf and literal.
- Format fence at ess/24.
- `crates/verify/ess-conformance/src/synthesize/binding_condition.rs`: `condition-false` flips the
  Boolean, and `condition-true` uses the branch's literal.
- `crates/generate/ess-synth/src/condition.rs`: `End::Boolean` for Rust and Go.
- Docs: `website/docs/reference/predicates.md` (binding selection row) and
  `docs/design/conditional-binding-failure-policies.md:121-122`.

Acceptance:

- `g1/requested.yaml`, under the new format, validates and synthesizes `condition-false` for each
  binding with the flag flipped.
- `g1/selection-sibling.yaml` validates.
- A target that ignores the flag fails `condition-false`.
