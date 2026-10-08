---
format: aep.planning-md/3
id: story:response-string-newtype-constraints-checked-not-refused
kind: story
status: active
title: Check String-newtype constraints on returned response values instead of refusing the outcome
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-08T02:12:49Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-08T02:12:50Z", actor: "human:timo", revision: 3}
---
## Outcome

A command whose response field has a String newtype with `alphabet:`, `prefix:` or `value`
invariants keeps its success scenarios. The returned value is checked against those rules on every
target that runs the suite. At the moment synthesis refuses the whole returning outcome with
`ESS-SYNTH-001`, so an author has to pick between stating the field's grammar and having any
success scenario for the command
(https://github.com/beyond10x/ess/issues/499). The rules are already carried and executed for
one-time response values. This story brings that authority to ordinary direct response
observations (`expect_direct_response`) and to response-mapped payloads (`expect_response_payload`).
Record invariants on a response type stay refused, and so do reading-attached response types,
each with a refusal that names the cause.

## Fit review

1. **Need, apart from the syntax.** A returned text field often has a grammar: a code drawn from
   a fixed character set, a token with a fixed prefix, or a minimum length. ESS can state that
   grammar on a String newtype. When such a type appears in a command's `response:`, every
   returning outcome loses its scenario. Minimal reproduction (neutral nouns, written for this
   review): `.engineering/repro/499/catalog.yaml` declares `catalog.items.Code` (newtype of
   String, `alphabet: 'ABC…Z0123456789-'`) and returns it from `catalog.items.Register`
   (`returns: true`). `ess verify conform synthesize` (ess 0.56.0) prints
   `refused: refusal[ESS-SYNTH-001]: outcome catalog.items.Register/registered has no scenario …
   no witness: \`catalog.items.Register.response: response constrained type needs an executable
   invariant/reading observer\` … typed direct response observation cannot execute this contract`,
   `0 scenario(s) (0 authored), 1 refusal(s)`. With `invariants: ['value.count >= 4']` in place of
   the alphabet (`catalog-invariant.yaml`) the result is the same ESS-SYNTH-001. With the
   constraint removed (`catalog-plain.yaml`) the result is `1 scenario(s) … 0 refusal(s)`.
   *Requester's proposal (theirs):* the response observer checks `alphabet:`, plus the invariants
   the predicate language already evaluates (`value.count`, `starts_with`, `ends_with`,
   `contains`), and refuses by name only what it cannot evaluate.
2. **Class: gap.** This is documented behaviour, not a defect.
   `docs/design/typed-response-outcome-payloads.md:39` says "Recursive, Binary64,
   invariant-constrained and clock-attached response types are explicitly refused at synthesis.
   This does not claim general invariant validation." The refusal is raised at
   `crates/verify/ess-conformance/src/typed_fields.rs:47-53`, which refuses any
   `ResolvedBody::is_constrained()` type (`crates/specify/ess-compiler/src/ir.rs:373-384`). The
   domain fact can be written, but it can't be checked where it matters, and writing it removes
   the outcome's coverage.
3. **Already expressible?** Only partly, and the idiom changes the meaning.
   `one_time_response: [code]` on the outcome (`catalog-onetime.yaml`) synthesizes
   `5 scenario(s) … 0 refusal(s)`, and the suite carries `"alphabet": "ABC…-"`
   (`ess-conformance/34`). That marks the value as a one-time disclosure, though, which is a
   different domain fact, so it is not an idiom for ordinary returns. The other workaround moves
   the grammar to request types only, which is what the issue reports the adopter did. That
   gives up the claim about the response. No idiom found in `docs/design/` or under
   `website/docs/reference/`.
4. **Fit with what exists.**
   - *Vocabulary:* no new authored key. `alphabet:`, `prefix:` and `invariants:` on a newtype are
     the existing spellings. The suite already has a closed carrier for exactly these rules,
     `one_time_response::StringConstraints {alphabet, prefix, invariants}`
     (`crates/verify/ess-conformance/src/one_time_response.rs:37-47`), built from compiled
     declarations by `Response::of` (`one_time_response.rs:112-157`) and validated against the
     type registry (`one_time_response.rs:205-222`). The redesign reuses that carrier and adds no
     second one. The requester named only `alphabet` and some predicates. `prefix:` is the
     sibling rule and is carried too.
   - *Composition:* the check runs on the actual returned value at each reachable String-newtype
     position (newtype, record field, union variant, Optional, List and Map element). That is the
     traversal the native one-time observer already does (`runner/disclosure.rs:137-190`).
     Guards, bindings and views are untouched. Predicates are evaluated over a lone `value` text
     fact, as `runner/disclosure.rs:158-172` does. A predicate that is not decidable over `value`
     (reads another path) is refused at synthesis by name and never reaches the runtime
     `Unknown` branch.
   - *Siblings:* `expect_response_payload` (`response.rs:108`, `typed_fields::declarations`)
     refuses constrained types in the same way and gets the same authority. Record (`Struct`)
     invariants and `reading` attachments on response types stay refused, with refusals naming
     "record invariant on a response type" and "reading-attached response type" instead of the
     current combined text. Fixture inputs (`fixtures.rs:74`) are out of scope.
   - *Targets:* native runner (`runner.rs:1747` `expect_direct_response`), generated Go
     (`go/prerequisites.go:181` `admitDirectResponse`, `:343` `expectDirectResponse`; the
     constraint check exists in `go/one_time.go`), generated TypeScript (`ts/direct_response.ts`;
     the check exists in `ts/one_time_response.ts`). The model interpreter already generates
     constrained response values through `crate::input::validate_typed_value`
     (`interpret/response.rs:115-133`; inferred to honour the alphabet). The browser replay
     refuses suite envelopes newer than it admits (design page line 51), so a new pair is refused
     there by name and never ignored. `ess verify diff`, generated Rust/Go/TS APIs, the
     authoring grammar and generated docs are unaffected: no authored or generated-API change.
     Entity Runtime: I don't know whether it executes `expect_direct_response` itself. The story
     checks this.
5. **Second adopter.** An order service returns `order_number`, a newtype of String with
   `prefix: 'ORD-'` and `alphabet: '0123456789ORD-'`. A shipping service returns a carrier
   tracking code with `invariants: ['value.count >= 10']`. Both want the success scenario to
   reject a malformed returned value, and both lose every success scenario today.
6. **Cost.** One new suite format pair, `ess-conformance/46` (ordinary) and `/47` (coverage),
   because `direct_response::Observation` is `deny_unknown_fields`
   (`direct_response.rs:30-38`) and gains a `constraints` member. There is no unreleased suite
   bump to join: the newest is `/44` and `/45` in 0.56.0 (`CHANGELOG.md:165`), and the
   `[Unreleased]` section names no suite format. The member is emitted only when a reachable
   response type is constrained, so every suite that synthesizes today keeps its bytes and
   format. There is no `ess/N`, no `ess-diff/N`, no new authored key, no migration and no
   generated-API change. Diagnostics: the combined refusal text is split into two named
   refusals (record invariant, reading), and one refusal is added for a non-`value` predicate.
   The existing ESS-SYNTH-001 help line, "drop it from the command's input", is wrong for a
   response field and should be corrected on the same path.
7. **Alternatives.**
   (a) *Change nothing:* authors keep giving up either the grammar or the coverage. Rejected,
   because the authority and the runtime checks already exist for one-time values.
   (b) *Take the proposal literally:* an observer-side predicate allow-list (`count`,
   `starts_with`, `ends_with`, `contains`). Rejected in that form. It would be a second
   definition of "evaluable" beside `Predicate::evaluate` and the one-time profile, and it
   leaves out `prefix:`.
   (c) *Chosen:* carry `StringConstraints` on direct and mapped response observations and
   execute them with the one-time check that already exists on each target. The set of
   evaluable predicates is decided once, at synthesis, by whether the predicate decides over a
   lone `value` fact.
   (d) *Discharge the per-type value-invariant obligation (ESS-SYNTH-013, "no view publishes a
   field position …") through response observations as well:* deferred. It changes the type
   coverage inventory in `synthesize.rs:1560-1600`, which is a separate question. Today
   `catalog-invariant.yaml` also prints ESS-SYNTH-013, and this story leaves that refusal as it
   is.

## Decisions

**accept, redesigned.** The need is real (a gap), and the requester's outcome (check what is
evaluable, refuse the rest by name) is kept. Changes from the request:

- The rules travel as the existing closed `StringConstraints` carrier, keyed by nominal type, on
  `direct_response::Observation` and `response::Observation`. No new observer predicate list.
- `prefix:` is checked as well as `alphabet:` and `value` invariants.
- Which invariants are admitted is decided at synthesis: a predicate that does not decide over a
  lone `value` text fact is refused by name.
- `ExpectResponsePayload` gets the same authority (sibling rule).
- Record invariants and reading-attached response types stay refused, now with separate named
  refusals.
- Spec-first: amend `docs/design/typed-response-outcome-payloads.md:39` (and the "Persisted
  meaning" section) to state the String-newtype constraint profile and the `ess-conformance/46`
  and `/47` allocation before code. No ESS authoring-language change is needed.
- ESS-SYNTH-013 type coverage through response positions is out of scope (alternative d).

## Acceptance

- `ess verify conform synthesize` on a String newtype with `alphabet:` returned by a
  `returns: true` outcome yields the outcome scenario and no ESS-SYNTH-001. The suite is
  `ess-conformance/46` and carries the constraint. Named test in
  `crates/verify/ess-conformance/tests/` (for example `direct_response_constraints.rs`).
- The same holds for `prefix:` and for `invariants: ['value.count >= 4']`. ESS-SYNTH-013 for the
  invariant type is unchanged.
- A specification whose response types carry no constraint synthesizes byte-identical suites at
  the same format as before (existing suite snapshot tests stay green, unchanged).
- A response type with a record invariant, or one reaching a `reading`, is still refused with a
  refusal naming that cause. A test asserts each message.
- The native runner fails the outcome scenario when a target returns a value outside the
  alphabet, without the prefix, or violating the `value` invariant, and passes a conforming value.
  The test uses the existing healthy/faulty target pattern.
- Generated Go and TypeScript runners give the same pass/fail on the same vectors (Go/TS parity
  tests beside `runtime_parity_go_28_35.rs` and `ts/direct_response` tests).
- `expect_response_payload` on a constrained response-mapped field is admitted and checked the
  same way.
- Older runners (Go/TS/native admission at `< 46`) and the browser replay refuse an
  `ess-conformance/46` suite by name. Admission test.
- The model interpreter passes its own synthesized suite for the alphabet, prefix and invariant
  examples (`ess verify conform run` against the interpreted target).
- `task check` is green, and `docs/design/typed-response-outcome-payloads.md` no longer lists
  String-newtype constraints among the refused response types.

## Scope

- `crates/verify/ess-conformance/src/typed_fields.rs`: split the constrained refusal and admit
  String-newtype constraints for response positions (cited, :47-53).
- `crates/verify/ess-conformance/src/direct_response.rs`: `Observation` gains `constraints`,
  `of`/`validate`/`compare` (cited, :14-140).
- `crates/verify/ess-conformance/src/response.rs`: same for `ExpectResponsePayload` (cited, :108,
  :142).
- `crates/verify/ess-conformance/src/one_time_response.rs`: factor `StringConstraints`
  derivation and validation into a shared helper (cited, :37-47, :112-157, :205-222).
- `crates/verify/ess-conformance/src/runner.rs` (`expect_direct_response`, :1747) and
  `runner/disclosure.rs` (`constraints`, :137-190), with the check shared (cited).
- `crates/verify/ess-conformance/src/scenario.rs`: suite format floor for `/46` and `/47`
  (inferred from :175), and `admission.rs` (:537, cited).
- `crates/verify/ess-conformance/src/go/{prerequisites.go,mod.rs,one_time.go}` and
  `src/ts/{direct_response.ts,response.ts,one_time_response.ts,runtime.ts}`: admit `/46` and
  `/47`, execute the constraints (cited for paths, inferred for exact edits).
- `crates/verify/ess-conformance/src/web_replay.rs`: confirm a by-name refusal of `/46` (inferred).
- Refusal help text for a response-field NoWitness: location not yet established (inferred to be
  refusal rendering in `synthesize.rs`; leave it if it lives there, see below).
- `docs/design/typed-response-outcome-payloads.md` (cited, :39, :45-51); `CHANGELOG.md`.
- **Held files:**
  - `crates/verify/ess-conformance/src/synthesize.rs`: not needed. The call site at :3131 stays
    as it is, and `Observation::of` changes behind it (cited). If the misleading help line is
    rendered there, that fix is dropped from this story.
  - `src/synthesize/**`: not needed (inferred).
  - `src/interpret/execute.rs`, `interpret/execute/related.rs`, `interpret/execute/existence.rs`:
    not needed. Response values come from `interpret/response.rs` (cited :115-133).
  - `crates/specify/ess-domain/src/command.rs`, `command/**`: not needed, no authored change
    (inferred).
  - `crates/specify/ess-compiler/src/ir.rs`, `ir/**`: not needed. `is_constrained` and the
    `ResolvedBody` fields are read only (cited :373-384).
