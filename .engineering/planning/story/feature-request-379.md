---
format: aep.planning-md/3
id: story:feature-request-379
kind: story
status: active
title: Existence branches project through Web and Clap target seams
refs:
- provider: github
  reference: beyond10x/ess#379
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/clap/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/existence.rs
- confidence: cited
  path: crates/generate/ess-synth/src/web/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/upsert_by_existence.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T09:36:18Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T09:36:18Z", actor: "human:timo", revision: 7, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

Valid existing_instance and creating unknown_instance branches remain projectable to Go, Web and Clap, with generated selection where the target provides it and explicit named handler obligations otherwise.

## Fit review

1. Need: a create-or-refuse model must project to each requested code target. Issue379 explicitly accepts named obligations instead of automatic selection. Existing upsert-by-existence.yaml is the minimal fixture (tests/upsert_by_existence.rs:28-43).
2. Class: remaining target-support gap, not a language gap; semantics already exist in docs/design/outcome-shapes.md:247-278.
3. Existing expression: the source branches already express this. Go admission/lookup is implemented in candidate f0b220099 (lib.rs:418-422, go/behaviour.rs:1388/:1534-1544; tests/upsert_by_existence.rs:79-103). Web and Clap still have blanket refusals (web/mod.rs:302, clap/mod.rs:65).
4. Fit: Web delegates generated behavior to linked Rust (web/mod.rs:338-341); verify actual dispatch preserves existence selection. Clap already has a replaceable Handler and command-naming Unimplemented error (clap/tree.rs:328/:360-376); keep that honest seam. Do not claim Clap generates storage selection. Reassess existence.rs:9-14's obsolete no-storage rationale.
5. Second adopter: inventory UI and CLI importer create missing externally identified stock entries and update/refuse existing entries under the same existing contract.
6. Cost: no source format or keyword. Admission, reports and generated documentation change; preserve PLAN.md/plan.json's target-independent contract (lib.rs:27/:303), rather than invent target-specific plan dispositions.
7. Alternatives: retain blanket refusal; invent storage in every target; selected use existing linked behavior and explicit handlers, as the issue permits.

## Decisions

Accept, redesigned around existing target seams. Go is already implemented but the whole issue remains open. Web must exercise linked generated Rust behavior. Clap must project a named handler obligation and fail explicitly when unimplemented.

## Acceptance

- web_existence_branches_reach_linked_rust_behavior: actual absent/present addressed-record dispatch handles existing_instance and creating unknown_instance.
- clap_existence_branches_emit_named_handler_obligations: projection succeeds; default handler refuses explicitly without claiming success.
- Direct workspace and public entry projections agree; unchanged target-independent plan bytes and existing Go/Rust coverage remain.
- Replace blanket-refusal tests with stronger positive projection/runtime evidence, not deletion of tests.

## Scope

Cited: crates/generate/ess-synth/src/existence.rs; src/lib.rs; src/web/mod.rs; src/clap/mod.rs; tests/upsert_by_existence.rs; tests/web.rs; tests/clap.rs. Bridge/tree edits require concrete runtime/report evidence.
