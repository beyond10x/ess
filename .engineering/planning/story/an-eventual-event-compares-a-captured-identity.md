---
format: aep.planning-md/3
id: story:an-eventual-event-compares-a-captured-identity
kind: story
status: draft
title: An eventual event compares a downstream identity copied from a captured instance
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/verify/ess-conformance/assets/coverage-admission.js
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/delivery_context.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_273_pass1.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/binding_arrangement.rs
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 6
---
## Finding

Out of scope in wave D357 U4 (`story:binding-site-identity-comparisons`, decided before the unit): identities further down a binding chain, such as the `job_id` of an event published by the invoked command (`Started.job_id`), are expected through `EventuallyEvent`, which carries literals only, so a downstream event's identity copied from a captured instance is not compared.

## Acceptance

- An eventual-event expectation can name a captured instance for an identity field, and binding scenarios use it for identities the downstream event copies from an instance the scenario holds.
- A target that publishes a wrong downstream identity fails at least one binding scenario; the correct interpreted target passes.
- Models whose bindings publish no downstream identity keep their suite bytes.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave D368 on 6437975cb. Every line is **cited** (read from the story, the tree or a probe run) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance`: the binding-aspect synthesis sites, the scenario step vocabulary, and the three runners (Rust, Go, TypeScript) plus the browser coverage reader — cited
- **Today:** `ScenarioStep::EventuallyEvent` (`src/scenario.rs:2759`) takes `payload: BTreeMap<String, Node>` (:2763), literals only; `ExpectEventValues` (:2252) takes `ScenarioValue` including `Instance` (:1868), but only for the preceding invocation's direct events — cited
- **Probe:** `ess` 0.82.0 synthesize on `tests/fixtures/binding-arrangement.yaml`: the `created-starts` and `kicked-starts` `/binding/flow` and `/binding/delivery` scenarios end in `eventually_event jobs.job.Started` with no payload, so `Started.job_id` is never compared — cited (run)
- **Synthesis sites:** `src/synthesize.rs` `flow` :19212, `delivery` :19318, `on_failure` :19383 (retry :19439, escalate :19456); `delivery_context.rs` `consequences` :766-776, :927; `binding_effects.rs` `observe_published` :845-858; reuse `determined_identities` :9154, `expect_event_step` :9195, `expect_trigger_event` :9252 — cited
- **Format:** a new suite format major is needed. The step is serialized as tag `eventually_event` (`scenario.rs:2245`) and its key set is closed in every reader: Rust `admission.rs:651`, `runner.rs:748`, `:1105`; Go `src/go/runtime.go:946`, `:2456`, `:2865`, `:5793`, `:5983`; TypeScript `src/ts/runtime.ts:1798`, `:3890`, `:4546`, `:8634`, `:8964`; `assets/coverage-admission.js:405` — cited. A captured instance there means a new step or `ScenarioValue` entries, which readers refuse; `SuiteFormat` docs (`scenario.rs:596-617`) say the major moves then — inferred
- **The new major touches:** `SUPPORTED_SUITE_FORMATS` (`scenario.rs:558-562`), `newestSuiteMajor` (`go/runtime.go:104`), `NEWEST_ADMITTED_SUITE_MAJOR` (`go/mod.rs:429`), `SUITE_MAJORS` (`ts/runtime.ts:7075`), `FORMAT_RELEASES` (`crates/edge/ess-xtask/src/docs.rs:147`), the newest-major test pins (`tests/suite.rs`, `one_time_reader_cells.rs`, `adversary_generated_docs_pass2.rs`, `src/count_json.rs`, `src/coverage.rs`, `src/coverage_build.rs`, `ts/predicate.test.ts`, `ess-cli/tests/coverage_cli.rs`) — cited (grep), each needing a change inferred
- **Not affected:** `schemas/` and the spec-language `ess/N` — cited
- **Bytes that could move:** the binding rows of `synthesis-suite-bytes-table.tsv`, `synthesis-component-suites-table.tsv`, `external-beside-held-guard-base.tsv`; committed `suites/generated/billing` probably not — inferred
- **Docs:** `CHANGELOG.md`, `website/docs/reference/formats.md`, `spec-versions.md` — cited
- **Size:** large (about 700-1200 changed lines across three runners, the format bump and the pins) — inferred
- **Confidence:** medium (sites, readers and the miss read and run; new step vs new key undecided)
- **Would collide with:** any unit moving the suite major; `synthesize.rs` :9154-9300 and :19212-19470; `delivery_context.rs`, `binding_effects.rs`, `refusal_policy.rs` :497-560; the runners; `adversary_273_pass1.rs`; the three bytes tables
