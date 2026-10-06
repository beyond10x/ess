---
format: aep.planning-md/3
id: story:feature-request-316
kind: story
status: implemented
title: Generated creation honors the declared input identity
refs:
- provider: github
  reference: beyond10x/ess#316
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/adversary_upsert_by_existence_pass1.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/upsert_by_existence.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T09:36:17Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T09:36:17Z", actor: "human:timo", revision: 7, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "active", to: "implemented", at: "2026-10-06T09:46:27Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

A generated Rust or Go creation stores and publishes the identity selected by its declared payload, including ordinary creations without existence-selected branches.

## Fit review

1. Need: plain BookSlot creation invoked with slot_id must use that supplied value, not a generated replacement. The ignored regression in ess-synth/tests/adversary_upsert_by_existence_pass1.rs:284-327 reproduces the issue. No new syntax is proposed.
2. Class: defect. determined::identity_source identifies the declared source (determined.rs:956-973), but Rust behaviour.rs:1132-1137 and Go behaviour.rs:1712-1717 use it only when by_existence holds.
3. Existing expression: payload Event.slot_id: input.slot_id already declares the identity. Directly supplied creation identities remain an explicit obligation where unsupported (determined.rs:265-267); do not silently generate a different identity.
4. Fit: resolve supported identity sources for every creation, preserving required/Optional input and fallback semantics. Fix Rust and Go; Web delegates to generated Rust (web/mod.rs:338-341), Clap retains named handler obligations. Preserve existence lookup validation (determined.rs:991-1041).
5. Second adopter: caller-selected reservation number must identify both stored booking and published event, using the same existing payload vocabulary.
6. Cost: no authored format, keyword or migration. Generated context requirements may shrink when an input-determined identity no longer needs generate_* (rust/behaviour.rs:1486-1502); verify generated API and compile fixtures consistently.
7. Alternatives: retain corrupt identity; mark every plain creation owed; selected honor supported declared sources and retain explicit obligations for unsupported sources.

## Decisions

Accept as proposed for Rust and Go together. This is a correction within the generated-server batch. No expansion of source grammar or arbitrary identity expressions.

## Acceptance

- Unignore and prove red then green: a_plain_creation_stores_and_publishes_the_identity_its_payload_takes_from_the_input.
- Rust test executes an equivalent Go generated target and asserts both stored and event identity.
- Required input identity never invokes the context generator. Optional supplied and omitted cases preserve the existing fallback contract.
- Existing existence-selected branches and unsupported-source obligations retain behavior; regeneration/projection checks hold affected fixtures.

## Scope

Cited: crates/generate/ess-synth/src/rust/behaviour.rs; src/go/behaviour.rs; tests/adversary_upsert_by_existence_pass1.rs; tests/upsert_by_existence.rs. Go behavioral harness and determined.rs changes only where demonstrated by this correction. Coordinator owns planning and publication.

## Verification 2026-10-02

Committed 936b119fcbfde45d7700bfc8915dcf267531784e in the server group after independent review with no findings. Required-input regression failed with minted-1 instead of slot-chosen; Optional present/absent tests reproduced the error in both Rust and Go. Focused treatment: 25 passed, one pre-existing ignored case; strict all-target ess-synth Clippy and formatting passed. Local evidence: managed tree ess-backlog-servers-20261002, target/backlog-input/316-report.md and named red/green logs. All three frozen source hashes verified before commit. Full package and generated fixture projection checks are deferred to the complete group; story remains active until those checks and integration.
