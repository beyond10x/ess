---
format: aep.planning-md/3
id: story:counter-reachability-arithmetic-completeness
kind: story
status: active
title: Counter reachability refuses incomplete arithmetic bounds
refs:
- provider: github
  reference: beyond10x/ess#413
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/counter_limit.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T21:39:27Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"approval":1}}}
- {from: "proposed", to: "active", at: "2026-10-03T21:39:27Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"approval":1}}}
---
## Outcome

A counter reachability enumeration returns the existing incomplete result whenever any required bound or widest-step arithmetic cannot be represented, while finite ordinary counter/CAS execution and shared related-counter semantics remain correct.

## Fit review

1. Need: the brand-free Counter reproduction in the retained ESS413 revision2 fit/design has Create set revision0, guarded increment1, stale-input and signed-MAX refusals. Measured0.51/0.52 synthesis reports two refusals and incorrectly describes1 as the nearest reachable value below MAX; the short authored Create/Authorize0/Authorize1 path reaches2. These are retained owner observations, not new coordinator executions. Fit/designSHA5916502a20b823b4b4bec949cb0101e77462f8f8c3b3129143946593c3c27f3e; independent reviewSHA59595e768daf1a481a4da1963375167c6187e0e61c3412025fb80b1ab77dbf43.
2. Class: defect in arithmetic completeness, not a new seed capability. Current subject_fact.rs:2524-2554 promises a complete reachable span or None, but chain(checked_add(...)) silently omits unavailable literal bounds and filter_map silently omits unavailable negative magnitudes. Primitive facts.rs checked_add intentionally refuses signed-i64 overflow. Missing required arithmetic cannot establish completeness.
3. Existing idiom: ordinary finite command arrangement works; authored entity setup can separately test extreme states but does not feed generated synthesis. These measured controls and docs/design/authored-entity-state-arrangement.md:105 explain the separate seed gap. No arbitrary reset, larger search budget or weaker zero-refusal gate is required to correct the enumerator.
4. Fit: keep the current Option incomplete result, exact Number arithmetic, existing search budget and shared reachable helper. related_guard.rs consumes the same helper and needs a regression, not a speculative production edit. Preserve all command/input/related precedence and exact faulty-target assertions. All suites still use the current format/initial-state authority.
5. Second adopter: a sequence-number allocator with compare-and-swap increments and signed-limit exhaustion needs the same accurate bounded reachability; tests/counter_limit.rs already models this class.
6. Cost: no source, IR, suite, report or generated public API change. Correct malformed selection output only; preserve unaffected deterministic suites. Production scope is one helper file and its existing counter-limit test file. No primitive Number, CAS grounding, seeds or runtime-adapter changes.
7. Alternatives: change nothing preserves a false completeness claim; raising the bound cannot repair missing arithmetic and cannot enumerate to MAX; production reset changes the modeled domain for its checker. Chosen checked propagation returns an honest existing incomplete/refused result. Explicit synthesis seeds are separately proposed413B/42-43 and are not adopted here.

## Decisions

Accept413A as the previously reviewed bounded correctness correction, supplemental to the unchanged pinned50 ledger. The prior runbook accepted the fit in principle; this artifact settles exact implementation ownership. External412/413 owner may prepare the two-path candidate from the coordinator's current integration commit. Neither304 unit currently has a production writer on subject_fact.rs; integrate the independently reviewed A correction before dispatching304 production so that304 rebases onto it. No concurrent writer to those two paths. Compiler custody remains separately scheduled.

No413B implementation, new seeds/provenance,42/43 adoption, primitive arithmetic change, speculative CAS change, production setter or relaxed refusal gate. Whole413 remains open after A if its seed obligation remains unimplemented. Return any required scope expansion or independently red finite-CAS defect before editing more.

## Acceptance

Failing base tests and corrected tests prove MAX, MIN, padded-bound and widest-negative-magnitude arithmetic never yield a falsely complete set; a finite limit2 CAS fixture executes ordinary create/advance0/advance1, observes2, rejects stale expectation and exhausts at2 without EstablishEntity; and the shared related-counter regression plus existing counter/subject-input/precedence suites preserve healthy and decisive faulty-target behavior.

Keep honest extreme-state refusals after A; A does not promise zero refusals or extreme-state reachability. Preserve unchanged suite bytes where output is unaffected, current fresh provenance and old-format refusals. Required exact controls, strict scoped lint, owning formatting, affected tests and independent whole-unit review precede serial integration. Initial production author grant is source-only; no compiler lane or implementation acceptance is inferred from design approval.

## Production implementation boundary

The expected arithmetic baseline is measured red at required MAX/MIN window bounds (3pass/2fail), while the full counter_limit baseline now passes17/17 on unchanged production after a test-only hash-formatting compile correction. Source freeze2b7d64232621c327024b418597767a6e198154e162cdf5efc8c8b8c7bca45d47a preserves all arithmetic tests and original production bytes. Retain both failed logs; the initial compiler error was not a finite-CAS semantic verdict.

The external owner may now implement only the approved arithmetic-completeness correction in subject_fact.rs and scoped regression assertions in counter_limit.rs. Every required negation/padded-bound computation must propagate None on arithmetic failure rather than omit a bound and report a partial reachable set as complete. Preserve numeric semantics, search budgets, honest extreme-state refusal, shared-related behavior, finite-CAS arrangement/outcomes and canonical unaffected bytes. No seed capability, additional format, production state setter or unrelated CAS/input/guard change. Full affected proof and independent whole-unit review remain required before serial integration. Source edits may proceed while root verifies the disjoint authored/aggregate test unit; compiler custody is separately granted.
