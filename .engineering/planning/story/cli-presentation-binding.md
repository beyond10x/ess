---
format: aep.planning-md/1
id: story:cli-presentation-binding
kind: story
status: active
title: Declare and project typed CLI presentation without changing service ownership
refs:
- provider: local
  reference: connectors_v2:specification:local-cli-wave-20260909
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates/edge/ess-cli/Cargo.toml
- confidence: inferred
  path: crates/edge/ess-cli/src/cli_binding.rs
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/output_ownership/state.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/cache_origin.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/cli_binding.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/command_surface.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage
- confidence: inferred
  path: crates/edge/ess-xtask/src/support.rs
- confidence: cited
  path: crates/generate/ess-cli-project
- confidence: cited
  path: crates/specify/ess-cli-contract
- confidence: cited
  path: docs/design/cli-consumer-pipeline-coverage.md
- confidence: cited
  path: docs/design/cli-presentation-binding.md
- confidence: cited
  path: docs/design/cli-schema-metadata-accounting.md
- confidence: cited
  path: docs/design/review-consumer-coverage.md
- confidence: inferred
  path: website/docs/reference/cli.md
- confidence: inferred
  path: website/docs/status/where-this-stands.md
revision: 19
---
## Operator authorization
The Connectors operator approved closing necessary ESS gaps upstream to specify its local CLI. Work is local in the isolated cli-binding-ess-20260909 managed tree; preserve the live review-boundaries-21 wave and current main. Serving objective O2. No release/external publication or production provider/credential execution.
## Context
At 0.20.0, crates/specify/ess-domain/src/component.rs declares owner-local cli placement and refuses foreign accepts. crates/generate/ess-synth/src/clap/tree.rs emits grammar-only handlers and omits view parameters. The new requirement is an explicit presentation binding, not changed service ownership or an invented semantic Read.
## Acceptance
An additive versioned ess-cli/1 binding resolves existing ESS types and explicit callable owners into a deterministic Rust/Clap parser and process adapter whose positive/negative fixtures prove type/source/process rules, foreign-owner preservation, unknown-format/field refusal, deterministic bytes and unchanged legacy CLI/composition output.
## Design before implementation
Write docs/design/cli-presentation-binding.md before adding types. Define a closed format: binary, declared callables (local handler, service-forwarded, schema-selected dynamic invocation), exact typed input/result references, owner metadata, command path and finite explicit aliases, argument-to-field source mapping, global config/state/output inputs, protected-input source rules and process result/error/exit behavior. A local callable is interface metadata, never an entity or a claim of business implementation. Resolve every type through the selected ESS model; service forward targets retain explicit owner and operation. Generic invocation validates its outer declaration; native payload schema identity/validation remains an explicit runtime obligation.
Root/one-group commands and explicit aliases suffice. Do not add arbitrary workflow orchestration, a generic Read entity, configuration templating, HTTP endpoints for local actions, typed composition-client rewrite, or provider-specific leaves. Preserve ess/1 and existing byte-oriented composition. New declarations are presentation metadata and typed values, no new runtime entity/relationship.
The compiler must report supported projection versus unresolved runtime handler obligations. Secure acquisition channels and source exclusivity must be enforceable by the generated process adapter, not only comments. Output/error policy needs executable recording-handler tests.
## Source assignment
New crates/specify/ess-cli-contract for closed binding types/resolution; new crates/generate/ess-cli-project for deterministic projection and generated process fixtures. docs/design/cli-presentation-binding.md is unit-owned. Provide coordinator patches for shared root Cargo files, existing CLI edge routing, public docs and indexes. Do not edit any existing ownership validator or existing synth CLI output.
## Integration
Coordinate the exact frozen binding example/schema with the Connectors lane before it authors presentation input. New ESS CLI edge commands are ess specify cli (validate/compile the binding against --path and --binding) and ess generate cli (same inputs plus --out for generated artifacts). Refuse before writing on any invalid/unsupported input. Coordinator applies edge wiring and runs final gates. This upstream story has an independent local source commit and retained build provenance; it is not a new official release.

## Scope

Derived 2026-09-09 by aep-drive:story-scoper. New crates/specify/ess-cli-contract, crates/generate/ess-cli-project and docs/design/cli-presentation-binding.md are explicit unit-owned assignments — cited. Cargo.toml, Cargo.lock and crates/edge/ess-cli/src/main.rs are coordinator integration surfaces — cited. Edge dependencies in crates/edge/ess-cli/Cargo.toml, tests in crates/edge/ess-cli/tests/command_surface.rs and public website/docs/reference/cli.md require coordinator work — inferred and adopted. Existing ownership validators and legacy synth output remain read-only — cited. Confidence high for explicit unit paths — cited; exact symbols/fixtures await the format freeze. Collides with changes to these new crates or shared edge/manifests/reference — inferred; coordinator ownership prevents parallel writes but does not erase integration overlap.

## Latest-main refresh
The operator requested current source rather than a release pin. Initial source 19de6406f97dca339136d7c9075ecc9b8fdb7af7 was superseded during this run by remote main 113f5925ebb6a0a57a73e661687d9fa5deddb0b8, merging reviewed wave 21. Preserve the preparation branch before applying this unit on that source; recreate this story through AEP rather than merging journals by hand. Pre-refresh unit checks are historical evidence, not final validation. Final source gates, independent review and Connectors regeneration run on the refreshed candidate.


## Current-main gate integration

The refreshed source adds finite model-consumer coverage obligations for the binding compiler, Rust artifact emission and source process adapter. Preserve the accepted unknown baseline and pre-existing profiles/cases. The initial direct-library checkpoint retained 470 model IDs/1,410 candidate pairs and 4,029 unaccounted pairs; it remains historical, not qualification evidence. The independently approved docs/design/cli-consumer-pipeline-coverage.md selects real authored-input-through-terminal pipelines, with exact upstream refusals and admitted terminal controls, for the three new profiles.

Finite candidate witnesses now account for 5,433 of the 5,439 new pairs. The six remaining relationships describe the schema document's root dialect and definitions container, not authored fields consumed by CLI behavior. docs/design/cli-schema-metadata-accounting.md proposes a narrow fourth bookkeeping disposition, exact six-row authority and executed guard, and first explicitly versioned accounting envelope; independent review is required before implementing this separate policy amendment. No baseline widening or behavioral qualification by snapshot is permitted.

The final adversary's sole defect was exact JSON integer -0 rejected at scalar/list/map argument positions. The process adapter now preserves that integer token as zero without rewriting quoted text, fractional or exponent spellings. Both adversary files remain unchanged; the final two-crate suite passes all 159 parent cases, including generated standalone-package verification, and strict all-targets Clippy passes. Additional focused controls prove malformed/fractional input still refuses before dispatch. Logs: .local/tmp/cli-wave/cli-final-owner-suite-01.log, cli-final-owner-clippy-01.log and integer-token-controls-02.log. The new malformed-token control uses --value=<text> so Clap does not mistake --0 for another option; its earlier cli_parse observation remains in integer-token-controls-01.log.

Candidate accounting and focused tests are intermediate evidence. Both routes remain observed by the support renderer and both crates participate in formatting. Full task check and task site-build, exact ESS commit/pin/build receipt, downstream regeneration and Connectors gates remain mandatory. Production credentials, supervisor/provider handlers, MCP and federation remain deferred.

## Accepted schema metadata amendment

The coordinator accepts docs/design/cli-schema-metadata-accounting.md under the existing operator authorization to close necessary ESS dependencies for the local CLI contract fixture. Independent review-result:cli-schema-metadata-decision-r1-20260909 approves the exact six-row bookkeeping disposition, fresh opaque guard proof, separate counts and explicit accounting-format migration. The earlier pipeline decision did not authorize a policy change; this decision does so explicitly and narrowly. Preserve inventory,87 pre-existing model profiles and the frozen baseline; no general exemptions or changes to ESS model/CLI formats.

Actual pre-amendment extraction05 passed; consumer-check06 refused exactly six schema metadata cells, with zero stale, duplicate or other accounting defects. It registers152 cases and171 requirements, with new CLI candidates4,251 Supported and1,182 Refused. This is unqualified candidate evidence. The report is .local/tmp/cli-wave/gate-pipeline-pre-amendment-checkpoint.md; it includes exact source/shape/profile/baseline preservation checks. Implement only the accepted metadata module/manifest, owning policy amendment, accounting envelopes and meaningful positive/negative compatibility tests; then fresh qualification and full task check/site-build remain required. The gate implementor owns these already-scoped files; the coordinator remains the sole planning writer.

## Metadata implementation and full-gate handoff

The coordinator completed the interrupted metadata implementation after the gate agent hit its usage limit. The code now implements the accepted closed six-row manifest, fresh compiled-provider/source/build authority checks, exact opaque proof/plan matching, separate metadata counts and ess-consumer-accounting/1 candidate/plan/qualified envelopes. The owning consumer coverage policy contains the explicit additive amendment. No model or profile fingerprint algorithm, wire extractor, baseline bytes, behavior cases or native execution protocol changed.

The first compilation exposed an unfinished PathBuf borrow, corrected in mod.rs. All86 coverage tests then passed, including13 new metadata/accounting cases and the real current-provider guard. Strict all-targets Clippy passed after a behavior-preserving helper extraction and style fixes. Logs: metadata-root-tests-01.log (compile refusal), metadata-root-tests-02.log (86passed), metadata-root-clippy-01.log (style findings), metadata-root-clippy-02.log (pass), all under .local/tmp/cli-wave.

Fresh qualification01 correctly refused the55 new gate helper entries pending exact classification. They are now classified individually as gate-owned helpers; the sole retired production entry is enforce::plan, retained as a cfg(test) compatibility wrapper. All87 prior model profiles and baseline remain unchanged. Qualification02 passed extraction/accounting and executed one fresh guard proving all6 metadata relationships; it was deliberately stopped before completing behavioral qualification to proceed once through the full required repository gate, with final registry byte ordering restored. Its partial evidence is diagnostic only at target/cli-metadata-root-check-02; no final qualification success is claimed. The full task check is next.

The separately required task site-build passed:21 browser claims,28 deterministic steps over64 rows, WASM and production Docusaurus build. Log .local/tmp/cli-wave/ess-site-build-01.log. Wanted preparation commits76fd70cac3911413b0cd1abe705963deaf67d8a1 and afde628e243b4dceb93956741108fdc10b672c16 are published only to local-recovery preparation branches. No source release or external publication occurred.

## Gate environment and candidate validation

Full gate 01 passed formatting and strict workspace Clippy, then exposed a stale
57-leaf command-tree expectation and an observation fixture whose temporary output
must live outside Git. The command-tree test now accounts for 59 leaves, with the
two new area-only CLI routes retaining no ambiguous flat alias. All 13 CLI binary
unit tests pass. The observation production guard and test remain unchanged;
TMPDIR is the task-owned /home/timo/.local/tmp/connectors-cli-wave-20260909.

Full gate 02 passed those checks and all 39 native ownership/recovery tests, then
stopped at three WASM linking cases because the coordinator's global native linker
flag reached the WASM linker. No product correction was needed. With RUSTFLAGS
unset and CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS selecting the same native
linker, all 45 ess-synth feasibility cases pass. The source-profile validator
already admits this exact target-scoped flag. The consumer task retains its
separate reviewed native environment.

Retained logs under .local/tmp/cli-wave: ess-full-gate-01.log,
ess-command-regression-02.log, ess-full-gate-02.log and
ess-wasm-environment-regression-01.log. These are partial gate attempts, never a
full-gate success claim. Preserve a local candidate commit for exact-source
downstream validation while rerunning task check; keep it unmerged until the full
required checks pass. The story remains active. The separately required site
build already passed. Current Atlas authority is clean at verified remote main
0602933d597c47f900d86a9946940b2fad74ad96; its new foundation extraction records do
not expand Connectors scope or authorize external publication.

## Candidate integration and fixture correction

Candidate fbd9b7a3ce751c8077a862f05e00c20751b08b80 is committed with the bot
identity and preserved on the local recovery branch candidate/cli-binding-20260909.
It is not merged or externally published. Connectors built that exact clean source
in an independent clone and produced its executable receipt. Its selected CLI
generates 10 artifacts; 64 structural fixtures, 2 cached expectations, 5 acquisition
and 3 page-consistency checks pass, and all 7 downstream CLI conformance tests pass.

Full gate 03 exposed an existing cache test fixture race: two independent recovery
drivers both truncated the same synthetic registry while another admitted it. Each
cache caller now provisions a separate synthetic authority root while retaining
the same cache, desired input, fake executors, concurrency barrier and exact
publication assertions. No production authority or cache code changed. The exact
two-writer regression passes for both bundle and Helm paths. Log:
.local/tmp/cli-wave/cache-writer-fixture-regression-01.log.

Downstream Cargo integration established that an independent generated workspace
cannot be nested under an existing member package without conflicting with Cargo
membership. Removing its workspace header instead silently enrolls the fixture in
the outer workspace; that attempted change was rejected and restored. The ESS
generator and its format remain unchanged. Connectors now generates the independent
fixture at apps/connectors-cli-contract, beside the runtime package, and excludes
it from ordinary workspace membership. A new Rust integration test compiles and
executes an enclosing consumer of the generated library, and proves exactly the
two declared runtime/consumer packages remain members. The nested layout's refusal
is retained in workspace-consumer-red-02.log; the supported layout passes in
workspace-consumer-sibling.log. The first exploratory control also passed with a
sibling layout; workspace-consumer-green-01.log records the rejected header removal.

The scope includes the cache fixture correction. Preserve this follow-up as a local
candidate, then rerun the full ESS gate with native-only linker flags and select
the exact clean follow-up source for final Connectors regeneration. No full gate
success or implemented lifecycle move is claimed yet.
