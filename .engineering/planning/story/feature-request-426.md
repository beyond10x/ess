---
format: aep.planning-md/3
id: story:feature-request-426
kind: story
status: draft
title: Import adapter from Canon protocol/1 to an ESS case-record domain
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#426
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 12
---
## Outcome
Resolve beyond10x/ess#426: Import adapter from Canon protocol/1 to an ESS case-record domain.

## Origin
beyond10x/ess#426, filed 2026-10-05; a downstream factory's spike that hand-derived one 452-line case-record domain from an engineering protocol on ess 0.52.0 and listed three generator pitfalls.

## Fit review
1. Need: a downstream specification wants the case record of a Canon protocol (claims, evidence rows, revisions, outcome guards, grants) as a typed ESS domain, generated and never hand-edited, with Canon staying the meaning. The requester's proposal: an ESS import adapter `protocol/1` -> ESS domain, beside `ess infra import openapi|kubernetes`. Minimal reproduction, written fresh: `<fit-review scratch>/probe-426/{a..g}/system.yaml`.
2. Class: the adapter is a gap only on the producer side. ESS can already express and validate the whole derived domain: the spike reports `valid` / `22 declaration(s), compiled` on 0.52.0 (issue body), and probe `b` (quoted variants, structured `all:`, refusal default) gives `probe v1 — 1 file(s), valid`. Per finding: (1) `True`/`False` as YAML booleans is not a defect: YAML reads them as booleans before ESS sees them, and ESS cannot recover the spelling without guessing; quoting is the idiom. The diagnostic is weaker than its siblings: a variant declaration has no `visit_bool` repair (crates/specify/ess-domain/src/types.rs:778-790), so the error has no key path (probe `a`). A field literal gets `hint: variants: True, False, Unknown` rather than the `quote it` the design table promises (docs/design/typed-literals-and-unknown-instances.md:24; probe `g`; command.rs:4400-4414). That is a diagnostic gap, 426b. (2) No compact `and` is not a defect or a gap. The design decided it: "No infix `or`, `and` or `not` is added to the compact form" (docs/design/refused-misparsed-predicate-disjunctions.md:17-18), and the refusal names the repair (probe `c`: "use structured any/all/not"). (3) A guarded refusal beside a guarded success is not refused as such: two claims and three claims validate (probes `d`, `i`). The proof declines past `MAX_ASSIGNMENTS = 64` (crates/specify/ess-domain/src/command/finite.rs:10; four three-valued claims = 81, probe `f`). That cap is documented (docs/design/cross-record-and-stored-field-guards.md:215, :602). The diagnostic says "open, unsupported, or exceeds 64" without saying which (subject_fact.rs:520-530), another diagnostic gap, 426b.
3. Existing idiom: a generator writes authored `ess/N` source and holds it with `ess specify validate --strict-requires`, `compile` and `verify conform synthesize`. Canon already gates its own `ess/` that way (canon AGENTS.md "ESS"; canon crates/canon/tests/ess_model_matches.rs:166 runs `ess` from PATH). The three idioms: quote every scalar (Canon's generator already double-quotes every string, canon crates/canon/src/generate/mod.rs:41-45); structured `all/any/not`; the refusal as the default branch (cap hint, subject_fact.rs:526-529).
4. Fit: an ESS-hosted adapter does not fit. The existing imports read a source ESS does not own into ESS IR or envelopes: `ess-openapi-import/1` (crates/generate/ess-openapi/src/accounting.rs:10) and `infra-ir`. None writes authored domain source. They sit under `ess infra` (main.rs:370-398), the infra bounded context, and a case record is not infra. The derivation rules are Canon semantics: three-valued claim tests, invalidation of "every claim built on" a named claim, and expiry (canon website/docs/reference/evaluation.md:44-48; concepts/evidence-and-revisions.md:11-12, :47-69). ADR 0067 says "Canon conformance, not ESS, is authoritative for Canon language semantics", and ADR 0076 keeps "the evaluation semantics … in Canon's conformance suite, not in ESS" (atlas origin/main architecture/adr/). Hosting them in ESS would make ESS's suite assert Canon's meaning. ESS would also need a second `protocol/1` parser or a Cargo dependency on the unreleased `b10x-canon` 0.0.0 (canon crates/canon/Cargo.toml), coupling ESS releases to every protocol/1 change. Naming: avoid "protocol" on the ESS side, because `ess specify protocol` / `ess-protospec/1` already mean communicating peers (CHANGELOG.md 0.53.0 "Experimental `ess-protospec/1`"). ESS's word is "case-record domain". The Canon-side name is the profile `canon-case-record/1` with a proposed verb `canon project ess`. `canon generate` already writes `canon-conformance/1` scenarios (canon crates/canon-cli/src/lib.rs:123), so that verb would be overloaded.
5. Second adopter: any producer that owns a declarative format and wants an ESS record of it, for example an approval-workflow engine emitting one domain per workflow definition. It needs the same three idioms and the same diagnostics. It does not need a Canon reader inside ESS.
6. Cost in ESS: no new key, no format bump (`ess/22` and `ess-conformance/43` unchanged), no new diff class, no generated-API change. It costs one design note and three diagnostic refinements (426b). Cost in Canon: one generator with refusals and coverage gaps, tested against Canon's own evaluator.
7. Alternatives: (a) change nothing, with each adopter hand-deriving; rejected because the issue's 452 hand-written lines drift with every protocol revision. (b) The requester's ESS import adapter; rejected (Q4). (c) A Canon-side generator plus an ESS design note and diagnostics; chosen because it adds no ESS surface and keeps the meaning where its conformance lives.

## Decisions
Accept, redesigned: no ESS import adapter. The generator belongs to Canon, as profile `canon-case-record/1`, to be filed in Canon's own store; ESS does not file it. It follows the fixed rules below. ESS ships 426a (a design note placing `protocol/1` import out of ESS scope, with the idioms held by a test) and 426b (diagnostics). No format bump (ess/22, ess-conformance/43 unchanged).

Fixed derivation rules recommended to the Canon story (Canon's to decide; listed so ESS's note can point at them):
- Domain from the protocol id, spec version from its revision. Every Canon identifier maps 1:1 to an ESS name; an unmappable or colliding identifier is refused by name, never mangled.
- Truth is `enum [Holds, Fails, Unknown]`, never `True`/`False`. Every emitted scalar is quoted.
- Artifacts become `<artifact>_revision` fields on `Case` plus one `Revise<Artifact>` command. Evidence kinds become `RecordEvidence` creating `Evidence` rows (kind, subject, revision, result).
- Claims become Truth fields, set only by a `Governor` actor's `RecordEvaluation` from Canon's evaluation, which also sets `evaluation: Current`. Every `Revise*` sets `evaluation: Stale`. The spike's per-claim reset to `Unknown` is rejected because it is not Canon's value: an `is: unknown` test is decided by that reset (evaluation.md:47-48), and invalidation is transitive and per claim.
- An outcome `requires` or action `precondition` over claim tests becomes an exact TRUE-translation conjoined with `evaluation == Current`: T(claim c)=`c == Holds`, T(c is false)=`c == Fails`, T(c is unknown)=`c == Unknown`, F of each the converse, T(not p)=F(p), T(all)=all T, F(all)=any F, and the mirror for `any`. Only structured `all/any/not` is emitted, and the refusal is the default branch.
- `requires: decision:` becomes a terminal command granted to a deciding actor. A capability becomes an actor grant.
- Refused or reported as coverage gaps, never approximated: an evidence match outside a claim, `max_age`, obligations, `approval-required` and per-evaluation denials, and `effect` (summary only).
- Acceptance is Canon's: the output validates, compiles and synthesizes with 0 refusals (no `wrong_state` beside `when_subject`, ESS-SYNTH-008, website/docs/reference/diagnostics.md:190), and every `canon generate` witness agrees with the derived guards.

## Acceptance
- 426a and 426b are implemented.

## Scope
- none; delivered by 426a and 426b
