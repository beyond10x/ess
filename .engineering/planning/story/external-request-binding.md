---
format: aep.planning-md/3
id: story:external-request-binding
kind: story
status: active
title: A Rust external decision port binds authorization to the actual command input
refs:
- provider: github
  reference: beyond10x/ess#412
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T19:17:27Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-04T19:17:27Z", actor: "human:timo", revision: 3}
---
## Outcome

Allow a generated Rust and Go external-outcome decision to inspect the exact typed command input currently being executed, including direct Generated calls. The owner can refuse absent, stale or mismatched authority instead of trusting a decision prepared for a different request. Issue https://github.com/beyond10x/ess/issues/412.

## Fit review

1. Need: two calls with the same addressed identity and CAS revision but different deadline/evidence values must be distinguishable at the external decision boundary. Current rust/behaviour.rs297-306 and987-994 transmit names only. Brand-free example: a lease renewal is authorized for one absolute expiry, but the direct handler receives another. This is a source/API capability proof, not a claimed observed production exploit. Requester proposes typed input or a faithful envelope; no syntax is prescribed.
2. Class: generated API gap. Existing external semantics explicitly leave the decision to the implementor, but the generated callback omits facts needed to make it. No new authored construct is needed. See generated Context comment and docs/design/cross-record-and-stored-field-guards.md ordering.
3. Existing expression: Context external accepts only names in both0.51.0 and0.52.0. A surrounding ingress can validate a request but cannot distinguish direct generated invocationB using requestA context. Required storage reads do not expose unpersisted input deadline. Custom replacement behavior is already possible (generated HEADER), but replacing an otherwise generated command duplicates its local policy and is not an equivalent fix to this missing external port. No existing input-aware port found.
4. Fit: retain existing external outcomes and ordering, expose borrowed typed requests through a closed generated ExternalCommand enum in the behaviour module. Context.external takes this enum and outcome; enum.name() supplies canonical command identity. Collect only commands that actually use generated external branches in ordered maps; qualify variant identities without collisions. Preserve both output layouts and code naming overrides. Go also generates this callback; extend its required Context.External to a sealed typed ExternalCommand interface with Name() and a private marker, using typed value wrappers. Go Emit.reference resolves package/code-alias paths and writer.locals.input must be respected. Web consumes generated Rust behavior; TypeScript is not a synth behavior target (Rust/Go/Web/Clap). Separate TypeScript schema/conformance emitters have no equivalent callback. Interpreter, conformance, diff and schema semantics remain unchanged. Full qualified target behavior/collision and callback-order tests required.
5. Second unrelated use: an external payment authorization is bound to payee and amount; identical invoice identity does not authorize another amount/payee. The same typed input mechanism applies without a domain-specific key.
6. Cost: generated Rust and Go Context API changes and all in-repository compiled harness implementations must migrate explicitly. No authored/IR/suite format change, dependencies, defaults or compatibility shim. Existing already-generated artifacts remain unchanged until regeneration. Regeneration intentionally produces a compile-time migration requirement; document it. Consumer owner must still verify trusted context, grants and freshness. Passing input is not authentication.
7. Alternatives: change nothing/require wrappers leaves direct generated calls unprotected; add an untyped JSON/Any envelope weakens the existing typed boundary and adds serialization or downcast requirements; per-command methods are type-safe but multiply callback naming/API surface. Select a closed borrowed enum and one required callback with no permissive default. Final names/collision handling must use existing naming conventions and be reviewed before source integration.

## Decisions

Accept, redesigned: input-bearing generated Rust and Go context ports, no new authored syntax, no generic metadata bag and no shell-only workaround. Root authorizes design/implementation under standing waves. No publication or release; ESS coordinator owns integrated bundle and full integration gate. Coordinator confirmed no active writer of rust/behaviour.rs; later issue319 overlaps and must be sequenced. Local base2f554561bef25125a93a1fb1d6517d50cb24ed20, branch fix/external-request-binding-412, managed wt-cd1fd476eae8. Source-scoped implementation/generator regressions and independent review precede handoff. No ESS build without coordinator lane grant; floor12GiB and per-tree target.

## Acceptance

Run a failing regression on current generator/actual compiled behavior before product edits. Matching request-bound decision accepts; changing the deadline or evidence while reusing the decision refuses with no storage/event effects. Missing proof refuses. Exercise direct Generated and served/dispatch routes, local guard precedence and multiple external outcomes. Verify exact i64 and optional/nested typed values without conversion, variant naming collisions, ordered deterministic output and both crate/workspace layouts. Existing external-outcome tests remain and migrate only to the explicit new required signature. Old handler implementations must fail to compile until explicitly updated, never silently default to false/allow. Required formatting/clippy and focused generator tests, independent adversary, then full ESS coordinator gate before integration. No weakenings or pin hacks.

## Scope

Cited: crates/generate/ess-synth/src/rust/behaviour.rs; tests/declared_behaviour.rs, synthesis.rs, served_publication.rs, served_correction_pass2_bridge.rs, adversary_served_pass2.rs, adversary_served_pass2_at_most_once.rs, served_unfinished_committed.rs, adversary_served_pass1.rs under crates/generate/ess-synth; tests/fixtures/declared-behaviour-harness/main.rs under the same crate. Inferred: crates/generate/ess-synth/tests/external_request_binding.rs; docs/design/external-request-binding.md. A fixture path expansion is allowed only after naming the exact file in AEP. Cited additional Go paths: crates/generate/ess-synth/src/go/behaviour.rs and crates/generate/ess-synth/tests/fixtures/declared-behaviour-go-harness/main.go. New Go compiled cases share external_request_binding.rs and existing declared_behaviour.rs. No conformance-synthesizer/related_guard/plan.rs edits in this unit; those overlap the ESS coordinator bundle. Root sole AEP writer for this unit.


## Accepted independent design refinements and wave

The reviewer identified three implementation constraints: allocate deterministic collision-free variant/wrapper names, rather than assuming qualified Pascal flattening is unique; use Layout/Emit resolved payload paths including naming.code aliases. Emit request type and required callback only when at least one actually generated command uses external branches; no-use and owed-only fixtures must compile without unused lifetime/types. The baseline semanticRED must compile against the old names-only API and demonstrate substituted request acceptance; preserve literal outcome/storage/event assertions through explicit callback-signature migration to GREEN. Compilerfailure alone is not semanticRED. Add Rust and Go siblings within the bounded two-renderer scope; ESS coordinator explicitly confirmed both are free before319.

One-unit wave; no decomposition panel because there is one story. Skill0.19.2 implementing wave with standing approval. Root owns planning, author native_adoption_scope executes implementor role, aep_private_record_design executes independent adversary procedure. Managed wt-cd1fd476eae8; target <private-cache-root>/ess-wt-cd1fd476eae8; scratch .engineering/drafts/external-request-binding-412; allrecordsretained. Author may write RED regression then await explicit coordinator buildslot. Per-process jobs1/debug0/incremental0, dev/teststrip=symbols; integer minimumfree12884901888. No shared target or interference with activeESSbundle. One sourceunitcommit after focusedchecks/independentreview, required AEP evidence and coordinated bundleintegrationauthorized; no push/tag/release. Full affected/finalESSgate belongs to bundlecoordinator before integration is accepted. No storycompletion before required gate evidence.


Public planning records use a symbolic cache root; exact machine paths remain in the ignored wave record. This privacy correction changes no approved behavior or scope.

## Boundary owner correction and supported profiles

Two focused boundary runs exposed pre-existing owner defects after fixture observability was repaired. Focused2 actual terminal101,0passed2failed6filtered,25.638275588s; logSHA2565e5c4444bc5c9559e66e8c160a934f490ae132be42cbea4dd9781d7e938c8350. Source comparison attributes the Go compile failure to unchanged go/port.rs and Rust served refusal to unchanged wire.rs/feasibility.rs; no separate historical baseline compile is claimed. Upstream414 tracks the Go defect; upstream415 tracks the distinct codec support gap. No finding is hidden by a model rename or a feasibility bypass.

Explicit bounded amendment: add cited crates/generate/ess-synth/src/go/port.rs::handler to the existing14paths (15total). Allocate c/input/outcome/unmet/value in fixed order against Layout::package_names with existing invariant::fresh; substitute every usage consistently. Preserve exported types/methods/parameter types/wire identities and ordinary unreserved generated bytes. Keep the original renewal.input no-use/owed-only compile regression; add legal domain-name matrix, repeated-suffix control, byte determinism and real handler/event-drain proof in the already-owned external_request_binding.rs. No other production emitter, package allocator or transport code is authorized. Existing docs path may record owner defect and supported-profile limits.

The same AB/AB2/A_B collisions and First/Third/Second aliases are tested on the existing supported embedded/direct profile, compiling full output in Rust crate/workspace and Go and asserting actual callback payload/identity/call behavior. Preserve original served collision fixture as typed WireCollision refusal control in both Rust layouts, including all3codec identities and both canonical sources; this is existing target admission, not proof of codec support. Preserve every original direct+HTTP request-binding and legacy signature refusal case. A separate served alias-only positive control may distinguish supported alias behavior. ESS415 remains open for a separately designed compatible codec allocation fix; no wire.rs/feasibility.rs edits or false closure in this unit.

Independent design review-result:external-request-binding-boundary-design-1 approves with findings[]; reportSHA256b616616458ad434efebad1e914e5e34ed03fd248c9aaf54f01e7acd5416a2431. Root accepts this explicit scope amendment under user's standing owner-fix/wave authority. ESS bundle coordinator confirmed no active owners of port.rs/wire.rs/feasibility.rs and requested frozen handoff before later319 overlap. Root remains sole AEP writer; native_adoption_scope is sole source/test implementor. No compiler launch until a fresh coordinated slot; full source review and package/integration gates remain required.

## Suffix probe evidence distinction

Source inspection corrected one test-design assumption: go/name.rs:171 package_ident strips underscores except keyword repair, and go/layout.rs domain_idents/repair appends domain/component. Authored domains input and input_ therefore do not produce simultaneously reserved input/input_ packages. Do not claim that impossible legal-domain matrix as runtime evidence. Keep actual legal-domain compilation/execution matrix for all five handler local bases, original input regression, determinism and ordinary-byte control. The repeated-suffix boundary is an explicitly synthetic reserved-set unit probe ({input,input_}) of the actual small private handler_locals helper in go/port.rs, which the production handler must use. This adds no path or allocator framework; no package allocator edit. Root approved this bounded interpretation after implementor source citations; final independent source review must verify the distinction and no fabricated generated-model coverage.
