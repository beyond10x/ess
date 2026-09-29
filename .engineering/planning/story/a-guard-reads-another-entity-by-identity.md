---
format: aep.planning-md/3
id: story:a-guard-reads-another-entity-by-identity
kind: story
status: implemented
title: A creating command cannot be guarded by another entity (no configuration / no matching entry refusals stay UNMAPPED)
refs:
- provider: github
  reference: beyond10x/ess#211
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:50Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:50Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T01:53:29Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
# Story: A creating command cannot be guarded by another entity (no configuration / no matching entry refusals stay UNMAPPED)

## Why

beyond10x/ess#211. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #211 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

**#211: a creating command cannot be guarded by a record of another entity**

1. (cited) The defect reproduces on this tree (HEAD `e932781965`, release 0.40.0). In `crates/specify/ess-domain/src/command/subject_fact.rs`, `common_subject` (l.40-46) only returns a subject that some branch names through its command input. A `creates:` subject never qualifies ("a command whose only subject-bearing branch creates has none", l.39).
2. (cited) The same file, `validate_shape` l.86-110, then refuses a subjectless `when_subject` refusal with `conflicting_declaration` at `outcomes.<name>.when_subject`, hint "a command whose only subject is the one it creates has no stored row to read".
3. (cited) Every `when_subject` form reads only the addressed subject. `OutcomeCondition::SubjectField` and `SubjectPredicate` are at `command.rs:393-418`, and `reads_subject_fact` is at `command.rs:563`. No condition reads a record of a different entity.
4. (cited) The design doc `docs/design/cross-record-and-stored-field-guards.md` l.3 and l.401 still say "Rule 2 stays out of scope".
5. (cited) It is not already fixed. The nearest existing pieces are:
   - `existing_instance:` (`command.rs:4384`, #164). It refuses on the same entity only, keyed by the created identity.
   - `{related: {via, field}}` (`command/related_value.rs:1-60`, #166). It reads another entity's row through `RelatedVia::Input` ("via: input.customer_id"), but only as a payload or `sets:` value, never as a guard.
6. (cited) Since ess/15 a `when_subject` predicate may already use `input.` operands (`subject_fact.rs:262-284`). So "the configuration has no redirect for `input.client`" can already be written as a predicate once there is a row to read.

**Where the fix lands**

7. (inferred) Reader and model in `crates/specify/ess-domain/src/command.rs`:
   - a new `OutcomeCondition` variant near l.393
   - a raw reader beside `RawSubjectField` (l.4233-4300)
   - `reads_*` helpers near l.563
   - a format gate (next major after `V17`, `system.rs:106`)
8. (inferred) Validation in a new `crates/specify/ess-domain/src/command/related_guard.rs`. It reuses `Referenced` and the entity resolution from `related_value.rs`, and the environment from `subject_fact.rs::check` (l.255), with the related entity's fields plus `input.`.
9. (inferred) Compiler IR in `crates/specify/ess-compiler/src/resolve.rs`, reusing `ResolvedRelatedVia` (l.2537-2548), and in `ir.rs`.
10. (inferred) Conformance: `crates/verify/ess-conformance/src/synthesize/related.rs` already arranges the referenced row plus decoys. Extend it to arrange the row as absent, or as present with a goal over its fields using the goal search in `synthesize/subject_fact.rs`. The evaluator in `interpret/execute.rs` and the selection in `decision.rs` also change.
11. (inferred) Other consumers of `SubjectPredicate` that must learn the new variant: `ess-entity-runtime/src/lib.rs`, `ess-gen` (`openapi.rs`, `docs.rs`, `http.rs`), `ess-synth/src/plan.rs`, `ess-diff/src/diff.rs`. Also a new ADR section replacing the doc's l.401 "out of scope" wording for the single-row case.

**Collisions**

12. (inferred) High, on `synthesize/subject_fact.rs` and `command/subject_fact.rs`: #198, #199, #204.
13. (inferred) High, on the refusal-arrangement path in `synthesize.rs`: #209.
14. (inferred) Medium, on the `command.rs` outcome keys and reader: #201.
15. (inferred) Low (mutate tooling): #203, #210, #202.
16. (inferred) None: #196, #195, #193, #205.

**Design decision (smallest proposal)**

17. (inferred) Add a new outcome key `when_related`, usable on any branch including `creates:` and subjectless refusals, in one of two shapes:
    - `{via: input.<field>, exists: false}`
    - `{via: input.<field>, predicate: <over related fields and input.>}`
18. (inferred) It is keyed only by the other entity's **identity** type, resolved the way `Referenced` resolves `via`: one hop, input-bound. For the issue's example this means the configuration record's identity is the tenant.
19. (inferred) A missing row makes a predicate `Unknown`, so it selects only `exists: false`, never the predicate branch and never the default. This matches the `Unknown` rule for optional fields.
20. (inferred) Two things stay out of scope: lookup by a field that is not the identity (that is a query, i.e. rule 2), and combining `when_related` with `when_subject*` on one branch (refuse it with `conflicting_declaration`, as `subject_fact.rs` does today).
21. (inferred) Open fork for the implementor: should `exists: false` be a boolean on the new key, or a separate `related_absent:` flag modelled on `existing_instance:`? The boolean on one key is the smaller surface.

Confidence: high on reproduction (cited code path); medium on file list and collisions (inferred from module ownership, not traced).

Paths:
- crates/specify/ess-domain/src/command.rs
- crates/specify/ess-domain/src/command/subject_fact.rs
- crates/specify/ess-domain/src/command/related_value.rs
- crates/specify/ess-domain/src/command/related_guard.rs (inferred, new)
- crates/specify/ess-domain/src/system.rs (inferred)
- crates/specify/ess-compiler/src/resolve.rs (inferred)
- crates/specify/ess-compiler/src/ir.rs (inferred)
- crates/verify/ess-conformance/src/synthesize/related.rs (inferred)
- crates/verify/ess-conformance/src/synthesize/subject_fact.rs (inferred)
- crates/verify/ess-conformance/src/interpret/execute.rs (inferred)
- crates/verify/ess-conformance/src/decision.rs (inferred)
- crates/generate/ess-entity-runtime/src/lib.rs (inferred)
- crates/generate/ess-gen/src/openapi.rs (inferred)
- crates/generate/ess-gen/src/docs.rs (inferred)
- crates/generate/ess-gen/src/http.rs (inferred)
- crates/generate/ess-synth/src/plan.rs (inferred)
- crates/verify/ess-diff/src/diff.rs (inferred)
- docs/design/cross-record-and-stored-field-guards.md

Verdict: needs-design
