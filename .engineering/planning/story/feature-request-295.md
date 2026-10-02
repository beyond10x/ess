---
format: aep.planning-md/3
id: story:feature-request-295
kind: story
status: draft
title: 'mutate: emit-drop is stillborn on every outcome that emits one event, so single-event emission is never audited'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#295
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 2
---
## Outcome

Resolve beyond10x/ess#295: mutate: emit-drop is stillborn on every outcome that emits one event, so single-event emission is never audited.

## Origin

beyond10x/ess#295, from a downstream hardening run on ess 0.48.0; reproduced minimally (triage item 6b, `~/.cache/ess-gaps/triage-cb/`).

## Fit review

295/212 final fit, read-only at next-tree7cc1bf440, own executions0, lease released. Seven answers for295: (1) Need: mutation auditing must test a single required event, without requiring a fake second event; issue295's suggestions are event replacement or deleting a suite ExpectEvent. Minimal case is a subjectless Ping→Ponged, plus creating/updating/moving variants. (2) Class: capability gap, not contradiction of current semantics: public mutation-audit.md:29–34 explicitly documents sole-event Stillborn; mutation_audit.rs:188–225 pins it. (3) Existing behavior: outcome_sites mutate.rs:704 enumerates every emit; apply:899 deletes its emits+payload entry; compile:994 uses normal validator; command.rs:2313–2320 refuses ordinary sole-event branches. Existing deletes/returns/replays can remain admissible, so 'every single-event outcome' is overbroad. Existing accepts:nothing cannot be simply added to a mutating branch: outcome_shapes.rs:74–94 refuses subject/sets/replay and only admits when/default. (4) Fit: retain model-valid alteration and ordinary suite runners. Blindly dropping ExpectEvent weakens the suite and cannot make the unchanged correct target fail, violating mutate.rs:1–30/public guide:20–27; that proposal should be rejected. Compatible event-swap fits specification mutation but requires deterministic replacement/payload rules and an explicit no-compatible-alternative policy, so it does not by itself cover every sole-event model. (5) Second adopter: a cache invalidation command publishes only CacheInvalidated; missed event obligation is equally material. (6) Cost depends on chosen operator: no authored syntax needed for a bounded swap, but operator identity, cardinality, CLI enum, public Rust enum and emitted manifest semantics require explicit compatibility decision; do not relabel swap as existing drop silently. (7) Alternatives: keep honest Stillborn (current); compatible event-swap (bounded useful extension); actual target-emission suppression against original suite (tests missing assertions but is a separate implementation-mutation mechanism); suite-expectation deletion (reject: weakening). Recommendation: defer295 implementation until root selects/binds operator + no-alternative policy; gap is real and still present, design not yet accepted (canonical295 rev1 has Pending fit).

Overlap/scope:212 is still unimplemented, but unlike295 has coordinator-adopted redesign in canonical rev3: non-creating sets-drop + witness separation; precedence-swap for adjacent overlapping accepting when branches/input refusals; leaf equality flip + outward integral±1 guard arms. mutate.rs still has nine classes:68–119, only ordering strictness at525–558, only SetsRetarget at676–691, no precedence swap. Shared code/test/docs batch is coherent, but295 is neither duplicate nor supplied by that acceptance. Existing212 report/manifest '/3 because strings' assumption needs compatibility recheck: EmittedMutant.class at mutate.rs:2318 is a deserialized closed MutantClass enum, so old collectors reject new class names even though JSON spells strings. Smallest295 regression seam: mutation_audit.rs public mutants→apply→compile→evaluate on a brand-free single-event fixture and target; controls for two-event drop, fieldful compatible/incompatible replacement, no replacement event, entity effect, deterministic IDs/bytes, then mutation_external.rs emit/collect equality. Existing retained execution: group-packages-f863ee.log:2757–2774 mutation13pass; :2776–2789 external8pass, demonstrating current behavior only. Main mutate source and mutation tests match0.51.0 exactly. Unknowns: no newly executed minimal probe in this intake; no measurement of proposed replacement killability or full format compatibility.

The seven-answer paragraph above is retained verbatim from the handoff. This report is the sole
authorized ignored scratch write; no source, planning-store or remote state was changed. GitHub
issue bodies/comments for beyond10x/ess#295 and #212 were read through the previously authorized
read-only fallback. The coordinator owns any design or lifecycle decision.
