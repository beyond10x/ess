---
format: aep.planning-md/3
id: story:synthesis-offers-an-input-that-breaks-an-entity-invariant
kind: story
status: active
title: Synthesis names the input that breaks an entity invariant when the model declares no outcome for it
summary: Synthesis writes invariant scenarios only with inputs inside the invariant, so an implementation that never checks invariants passes (beyond10x/ess#738)
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#738
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/invariant_counterexample.rs
- confidence: cited
  path: docs/design/ess-closed-loop-execution-conformance-design-v0.1.md
- confidence: cited
  path: website/docs/reference/diagnostics.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:46:32Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:46:34Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

An entity invariant is checked after every branch that creates or changes the entity
(`website/docs/reference/predicates.md:29`). Where a branch fills an invariant's field from the
input and no guard keeps the input inside the invariant, the model declares no outcome for an
input outside it: the interpreter reports it as undetermined (`BrokenInvariant`, "the model's own
outcome leaves … violating its invariant"), and generated Rust answers it as an unmet obligation.
Synthesis is silent about that hole. It offers only inputs inside the invariant, reports
"0 refusal(s)", and an implementation that commits whatever `sets:` supplies passes every
scenario.

Minimal reproduction, ess 0.92.0: entity `bounded.counter.Counter` (`value: Integer`, invariant
`value <= 8`), command `bounded.counter.Set` with branches `set` (`unknown_instance: true`,
`creates:`) and `reset` (`updates:`), both `sets: {value: input.value}`, no guard on `value`.

```console
$ ess verify conform synthesize --path unguarded --out suite.json
4 scenario(s) (0 authored), 0 refusal(s), written to suite.json
```

Every `value` offered is 1, 2 or 8 or below. The same suite with the `reset` step edited to send
`value: 9`:

```console
$ ess verify conform run --target interpreted --path unguarded --suite nine.json --report-format 2 --report-out r.json
  error bounded.counter.Counter/invariant/after/bounded.counter.Set/reset
      ESS-CF-TARGET: invoking `bounded.counter.Set` failed: the model's own outcome leaves `bounded.counter.Counter` `Text("counter_id")` violating its invariant `value <= 8`
```

The model has no declared refusal for `value: 9`, so a scenario "expect it refused, nothing
changed" has no outcome, error or event to expect. Where the author declares one (a branch
`too-large`, `when: value > 8`, `error: bounded.counter.ValueTooLarge`), 0.92.0 already writes
`bounded.counter.Set/outcome/too-large`: it sends `value: 9`, expects `too-large`, the error, no
event and the subject unchanged, and the interpreter passes it. What is missing is the refusal
that tells the author the hole exists.

## Acceptance

- `ess verify conform synthesize --path unguarded` (the specification above) reports two refusals
  with the new synthesis code (the next free `ESS-SYNTH-0NN`), one per branch:
  `bounded.counter.Counter/invariant/after/bounded.counter.Set/set` and `…/reset`. Each names the
  branch, the invariant `value <= 8`, the input it was sent (`value: 9`) and the help line "guard
  the branch on the input, or declare a refusal branch whose `when:` answers it". The
  `invariant/after` scenarios are still written beside the refusals.
- The guarded specification (`too-large` branch added) synthesizes with 0 refusals, and
  `bounded.counter.Set/outcome/too-large` sends `value: 9` and expects `too-large` with
  `expect_complete_subject_unchanged`. A regression test pins this.
- A branch whose own `when:` keeps the input inside the invariant (`when: value <= 8`) gets no
  such refusal.
- A coverage suite whose inventory holds the new code is written at the coverage major that admits
  it. A reader below that major refuses the suite by version, as for `ESS-SYNTH-023`/`-024`
  (`coverage.rs` `system_issued_refusals`). An ordinary suite keeps its bytes.
- `website/docs/reference/diagnostics.md` lists the code with its meaning and fix.

## Scope

- cited: docs/design/ess-closed-loop-execution-conformance-design-v0.1.md (§20 "Invariant Scenarios", line 901: the bounded counterexample search before the property-based "Later")
- cited: crates/verify/ess-conformance/src/synthesize.rs (`invariants` and `holds_after`, about lines 18966-19050; `RefusalCause` catalogue and code table, about lines 990-1050 and 1758)
- cited: crates/verify/ess-conformance/src/witness.rs (`candidates`, line 280: the bounded neighbour search, here asked for one candidate outside the invariant)
- cited: crates/verify/ess-conformance/src/interpret/execute.rs (`at_rest`, about line 3225: the same three-valued reading decides "breaks")
- cited: crates/verify/ess-conformance/src/coverage.rs (coverage-major floor beside `system_issued_refusals`, about line 444)
- cited: crates/verify/ess-conformance/src/go/runtime.go (per-code coverage floor, about line 4917)
- cited: website/docs/reference/diagnostics.md (synthesis code table, lines 194-217)
- cited: website/docs/reference/formats.md (coverage suite row for the new major)
- inferred: crates/verify/ess-conformance/src/scenario.rs (the newest coverage major and its pins)
- inferred: crates/verify/ess-conformance/tests/invariant_counterexample.rs (new)
confidence: medium

## Fit review

1. **Need.** A command can fill a field from its input past what the entity's invariant allows,
   and the specification does not say what answers that input. ESS knows this (the interpreter
   reports it as undetermined, `interpret/execute.rs:307` and `:3250`). Synthesis does not say it,
   so a suite that cannot catch a target that skips the invariant check reports no refusal
   (`repro/run.txt` §1). The requester's proposal was a scenario
   `<Entity>/invariant/refuses/<Command>/<outcome>` that sends the nearest outside value and expects
   "refused, nothing changed", or failing that a refusal.
2. **Class.** Defect. §20 of the conformance design asks synthesis to evaluate invariants after
   state-changing commands. Synthesis's contract is that a check it cannot write is a named refusal,
   never silence (`synthesize.rs` module doc, lines 13-17, citing §36). Here the check is silently
   absent.
3. **Already expressible?** Yes for the specification. A refusal branch guarded on the input makes
   0.92.0 send `value: 9`, expect the refusal and check the subject is unchanged (`repro/run.txt`
   §3). The suite then checks the figure the reader sees, so policy 1 accepts the idiom. What
   cannot be expressed is the author finding out that the idiom is needed. That needs a diagnostic,
   not new authored surface.
4. **Design fit.** Grid R row "entity / type invariant" (`docs/design/language-grids.md:28`) does
   not change, and the decision adds no authored key. A scenario expecting a refusal the model never
   declares would assert behaviour the specification leaves to the implementation. It would also
   contradict the interpreter (`Undetermined::BrokenInvariant`) and generated Rust
   (`rust/behaviour.rs:173-175`, "the model declares no outcome for the request"). So the
   requester's scenario is refused, and their fallback (a refusal) is taken. Sibling: a newtype or
   struct invariant reached only through input is
   `story:an-input-only-type-invariant-is-checked-by-a-refused-input`. It uses the same "boundary
   value outside" search, and the two should share the helper in `witness.rs`. The targets are
   unaffected: synthesis is the only producer. Readers of coverage suites gain one code under a
   floor, with the Rust admission and the Go runtime refusing it below the major.
5. **Second adopter.** `example.stock.Item` with `quantity >= 0`, and `AdjustStock` setting
   `quantity: input.quantity`. Without a `when: quantity < 0` refusal, an implementation that
   stores −1 passes every synthesized scenario.
6. **Cost.** One synthesis refusal code and its diagnostics row. A coverage-suite major floor for
   inventories that carry it, following the precedent of `ESS-SYNTH-016`/`-017` (/17) and
   `-023`/`-024` (/53). No source format, no IR change, ordinary suites unchanged. Existing
   specifications with an unguarded branch gain refusals, which is the point. The search is bounded
   (one neighbour per comparison, as `witness::candidates` already does), so it is no invariant
   solver (`docs/design/closed-enum-outcome-coverage.md:52-53`).
7. **Alternatives.** (a) Change nothing: the suite stays green against a target that skips
   invariants, and the report says "0 refusal(s)". Rejected (§36). (b) The requester's
   `invariant/refuses` scenario, expecting an undeclared refusal: rejected under 4. It would need a
   new "any refusal" expectation step and a new suite major for behaviour the model does not
   declare. (c) A validate-time diagnostic (`ESS-COMMAND-…`): rejected because it needs the
   invariant solver that validation explicitly does not have. Synthesis already holds the bounded
   witness search. (d) Chosen: a synthesis refusal naming the input that breaks the invariant, with
   the idiom as its help.

## Decisions

Accept, redesigned. Synthesis does not write a scenario that expects a refusal the model does not
declare. It searches each state-changing branch for an input that branch admits and that leaves the
entity breaking an invariant. Where it finds one, it emits a named refusal (a new `ESS-SYNTH` code)
naming the branch, the invariant and the input, with the guarded-refusal idiom as its help. Once the
author declares the refusal, the existing outcome scenario sends the outside value and checks
nothing changed. Changed from the request: no `invariant/refuses` scenario family and no new step.
The requester's fallback (a named refusal instead of silence) is the design. Format: no source
format. A coverage-suite major floor applies to an inventory carrying the new code, and ordinary
suites are unchanged. The design text extends §20 of
`docs/design/ess-closed-loop-execution-conformance-design-v0.1.md` before the code.

## Source

github:beyond10x/ess#738
