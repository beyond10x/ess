---
format: aep.planning-md/3
id: story:browser-response-conformance
kind: story
status: active
title: Support response contracts in browser conformance products
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/coverage.rs
- confidence: cited
  path: crates/edge/ess-cli/src/load.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/browser_response_conformance.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/browser_startup_refusal_boundary.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/browser_startup_slow_serve_boundary.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/conform_web_history.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/conform_web_history_adversary.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/coverage_browser.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/browser-target/Cargo.toml
- confidence: inferred
  path: crates/edge/ess-cli/tests/fixtures/browser-target/src/lib.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/one_time_browser.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/replay_fidelity_browser.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
- confidence: inferred
  path: crates/verify/ess-conformance/assets/browser-index.html
- confidence: inferred
  path: crates/verify/ess-conformance/assets/browser-player.js
- confidence: inferred
  path: crates/verify/ess-conformance/assets/browser-worker.js
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web_execution.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web_execution/abi.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web_execution/bundle.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web_execution/host.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web_execution/presentation.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/browser_product_admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/browser_product_presentation.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/response_payload.rs
- confidence: cited
  path: docs/design/browser-conformance-product.md
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: cited
  path: docs/design/review-replay-subset.md
- confidence: cited
  path: docs/design/typed-response-outcome-payloads.md
revision: 23
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T03:32:57Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-03T03:33:13Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Browser conformance products preserve and support the same admitted response authority as native and generated runtimes. Navigation of declarations remains visibly distinct from executing assertions against an implementation.

## Evidence and fit

Required internal coverage gap under the operator's all-features mandate. Source inspection in the runtime carrier finds crates/verify/ess-conformance/src/web.rs:308 refuses every response::used_by suite. tests/response_payload.rs:218 pins this blanket refusal. The existing player explicitly states at web.rs:288 that replay navigates declarations and emits no qualifying execution evidence. Existing standalone wasm32 execution of the Rust Runner therefore proves runtime portability, not completion of the browser product.

The read-only nested-response scope report has SHA256 2bd62ad5b85e0001b5cd6bb45889d3463bfea378448dd849f8a421dc3264c371. No actual browser probe or product implementation is claimed by that report.

## Acceptance

1. Preserve original suite bytes, parent/coverage authority and every response declaration through actual browser loading, navigation, selection and reset; never invent observed response values.
2. Replace the blanket refusal only with a demonstrated supported route. Execute a response-bearing suite through a real browser target bridge and compare healthy, wrong-response and wrong-event verdicts with native/Go/TypeScript. Independent stale-invocation, optional-presence and exact-integer controls remain required.
3. Keep declaration playback separate from execution results. A standalone WASM test or removal of an admission guard is insufficient.
4. Exercise existing browser fidelity and coverage tests on the integrated candidate; original-byte admission must reject forged or historical unsupported authority before target callbacks.

## Decisions

Accept the required product gap for design and implementation in the full backlog; it is not a permanent exclusion from all-runtime support. No new authored source syntax is proposed. The browser target/runner bridge design and exact production scope must be established from an actual red-capable browser probe before implementation. Coordinate persisted authority with nested-response-observations and the single held ess/21 bundle. This draft reserves no format version and authorizes no production edits.

## Scope

Cited declaration/admission seams: crates/verify/ess-conformance/src/web.rs, src/web_replay.rs, assets/player.js, assets/coverage-player.js and tests/response_payload.rs. Cited browser verification seams: crates/edge/ess-cli/tests/replay_fidelity_browser.rs and tests/coverage_browser.rs. Inferred real target-bridge surface remains unassessed; determine it before dispatch. Native interpret/**, caller synthesis and transport publication are separately owned.

## Product bridge scope proposal, 2026-10-03

# Browser response conformance: bounded product proposal

Read-only scoping, 2026-10-03. No builds, browser executions, production edits, AEP writes, or version reservations performed. This is a proposal for the design owner, not execution evidence or implementation authorization.

Planning source: `story:browser-response-conformance`, revision 1, in the canonical planning checkout at `e36046bdfcb85584778718727d4e349e6e2e6c20`. Runtime citations below use the integrated source checkout at `8b6c95c7e54d571723304fe689dec8965004dcb4`; this matters because its coverage reader includes the recent suite/35 migration. Paths are repository relative and publication safe.

## Recommendation

Add an explicit browser execution route backed by the existing Rust `Runner::run_admitted` and an independently installed `ConformanceTarget`, compiled together into a WASM host. Emit the host source/build instructions and browser integration as a product artifact, following the existing generated Rust/WASM installation and linear-memory conventions. Keep Step, Back, Play, Reset, and Select as declaration navigation. A separate Run action produces execution results with their own input identity and run identity.

The first product probe must go through actual `ess conform web` emission and Firefox/BiDi, install a real independently implemented target, and fail on a corrupted response. A test that dynamically copies fixture code into WASM and calls it through Node is useful portability evidence but cannot close this story.

## What exists

* `crates/verify/ess-conformance/src/web.rs::emit` and `emit_input` emit static declaration players. `response_replay_supported` refuses fixture-value suites and `response::used_by` suites. The latter predicate means `ExpectResponsePayload`; it is not a comprehensive predicate for direct-return or one-time response families. Removing this guard alone proves nothing.
* `assets/player.js:14` reads model/suite through `response.json()`. It preserves declaration groups and explicitly unknown state, but has no target connection, no assertion execution, and no original-byte admission. JavaScript parsing also cannot preserve every integer value for execution. The route can display declarations without proving their observations.
* `assets/coverage-player.js` loads original replay bytes and calls `coverage-admission.js::admitReplay`. At the integrated source base, `admitSuite` allows suite/5, /9, /35, with /35 requiring empty initial state. Its closed `stepFields` table still has legacy steps only: no response assertions/captures, caller declaration setup, fixtures, or broad newer runner vocabulary.
* `web_replay.rs::AdmittedReplay::{new,from_json}` retains the original suite and parent strings through `AdmittedInput`. Its replay/1 model is deliberately reduced, cannot reconstruct/authenticate the complete specification digest, and excludes one-time policy, preserving/deleting effects, aggregation fields, and periodic-binding shape. Broad suite admission must not silently discard these declarations.
* `assets/player.js` and `coverage-player.js` rebuild a declared prefix; `cancelTimer` increments a generation counter. `index.html` expressly labels expectations unexecuted. This behavior is an existing contract, not the place to derive actual responses.
* `crates/generate/ess-synth/src/web/{bridge,page,mod}.rs` is a different product: a generated implementation front end. It has a Rust installation seam and `ess_input_reserve`, `ess_dispatch`, `ess_output_len` exports. Without a realization it refuses. Its existing glue uses `JSON.stringify`/`JSON.parse`; reuse the memory ABI pattern, not its object round trip for exact response values. Its generated `Bound` trait is not `ConformanceTarget` and does not by itself supply conformance isolation, event observations, consistency, or setup capabilities.
* `tests/support_initial_state/mod.rs::wasm_case` generates a temporary Rust host by copying the target fixture, builds wasm32-unknown-unknown, and calls it with Node. It establishes real WASM execution of a library plus fixture, not emitted conformance-product integration or real browser behavior.

## Smallest bridge contract to prove

1. A generated Rust host owns the original suite/input bytes, `AdmittedInput` or `AdmittedSuite`, the admitted declaration representation, and report production. A narrow request envelope carries original JSON as a string; the browser never parses/re-serializes suite payloads before admission. Passing a string through JSON encoding is acceptable; passing its parsed value is not. UTF-8 decoding must fail on malformed bytes.
2. The host admits the entire input/parent chain and checks the selected identity before creating the target or calling target identity/setup/commands. The target is supplied explicitly through a Rust installation/factory seam. An uninstalled host reports unavailable; a reference interpreter is an explicitly named model target and never stands for the consumer implementation. Do not feed expectations to the target or synthesize responses in the bridge.
3. The smallest supported implementation route is a synchronous Rust `ConformanceTarget` linked into the same WASM module. `SemanticCommandResult.response`, direct events, errors and outcome come from one target invocation, remain typed Rust values, and flow unchanged to the existing runner. Public runner/target semantics need no rewrite. Browser `fetch` adapters and arbitrary asynchronous JS targets are additional design work; the synchronous trait cannot be made asynchronous by hiding promises or replaying target callbacks.
4. Reuse `AdmittedInput::select` for explicit run selection; retain the parent chain and original selected bytes. A browser UI selection is not permission to claim execution of the complete inventory. Use `ExecutedRun` plus the existing report/count production APIs (`counts.rs::from_run`), not a JSON object that happens to contain counts. Show incomplete/unsupported status honestly.
5. The host exposes an admitted display representation separately from execution. The original authority remains immutable in Rust; the display representation retains every declaration and uses explicit typed number text where JS cannot represent a value. One-time values remain private runner/target data, absent from DOM, console, downloaded diagnostic detail, and declaration world. Do not make the reduced model an authority for response generation or view evaluation.
6. Navigation never invokes target methods or mutates a completed run. Run results are keyed by suite digest, selected scenario identity, and a run generation. Reset/Select invalidates pending presentation results without claiming rollback of implementation activity. A new execution gets a fresh scenario namespace through normal target begin/end; a successful begin accepts the isolation obligation, it does not prove isolation. If worker execution is chosen for responsive navigation, terminate/discard stale presentation work explicitly and test delayed replies. A worker or an async host API is a packaging decision still to prove, not an implemented capability.
7. Preserve retained replay/1 behavior and historical refusals. New response-bearing replay/presentation authority must use a complete typed reader, preferably the same Rust admission in WASM, rather than independently recreating response validation in JavaScript. Keep exact original suite/input as the authority carrier. Any new persisted presentation envelope or change to replay/1 requires the format owner's decision; this report reserves no version and does not authorize extending a closed historical schema in place.

This approach keeps actual response semantics in Rust and needs no changes to `interpret/**`, caller synthesis, native target issuance, or Go/TypeScript runners. Source emission and browser glue should follow existing repository template conventions; no new non-Rust checker or semantic implementation is proposed.

## Feature accounting

| Family | Product today | Required proof / remaining implementation |
| --- | --- | --- |
| Declaration navigation, unknown state, views/controls | Existing real browser tests; no execution | Preserve unchanged meanings and cancellation behavior while adding a separate execution panel. |
| Response-derived event payloads, including legacy source mappings | Explicit blanket emission refusal | First browser target bridge probe; compare healthy, wrong response, wrong event, missing response, stale invocation. |
| Direct responses, including nested complete values, optional presence and ordered lists | Rust runner supports these; product browser execution absent; legacy navigation is not validation | Run through same bridge and prove omission versus null, order/duplicates, closed fields, exact signed integers and payload depth boundary. |
| One-time response capture/reuse | Ordinary player shows declaration policy; closed coverage replay refuses it | Browser admission must preserve policy/authority and runner redaction; cross-scenario leakage and stale/private data controls required. Do not unlock replay/1 by dropping policy. |
| Response-owned creation identity | Native target now reviewed; browser execution absent | Actual response/event/created-row identity agreement and repeat distinctness through installed target; no display alias substituted as a real ID. |
| Nested response source selections in event payloads | Separate observer design work; `response.rs::Observation::of` currently collects top-level `ResolvedPayloadValue::ResponseField` mappings | Coordinate `nested-response-observations`; bridge cannot repair missing source/observer authority. Complete typed nested values are different from selecting nested source paths. |
| Caller context, grants, fixtures, setup, isolation, views, event/binding/time/scan/reading/periodic controls | Native trait and runner have individual capabilities; closed player vocabulary is narrower | Rust bridge should carry existing runner semantics, but each installed target must implement the relevant capability. Unsupported is a reported non-success, never evidence that the family is supported. Broader browser matrix remains required for the all-features goal. |

## Actual red-capable browser matrix and sequencing

1. **Product boundary red first.** Add a Rust CLI browser integration test that invokes the actual emission command on `tests/fixtures/response-payload.yaml`, builds its emitted host with an independent Rust target, serves emitted files, and drives Firefox via `tests/support/browser.rs::{Server,Browser}`. At present emission itself refuses; retain that actual result, then require execution and a deliberately wrong response to fail with the native diagnostic code. Do not replace a failed product test with a library-only host.
2. **Bridge + ordinary response.** Compare exact scenario verdicts and diagnostic-code sets symmetrically with native/Go/TypeScript using the same admitted original suite. Healthy, missing response, wrong scalar/type, extra field, wrong event, missing event, wrong outcome, stale command/result, optional absent/null and integer values around 2^53 and i64 limits. Response results may not be read back through JS numbers before comparison.
3. **Admission before callbacks.** Callback counters remain zero for forged observations, unsupported historical authority, duplicate/unknown keys, forged parent digest, changed selected scenario body, invalid initial-state contract, integer damage, and malformed UTF-8. Use full multi-level lineage and explicit selection. Repeat the mutation after initial load to prove re-admission before each run.
4. **Navigation/control proof.** Load, Step, Back, Play, Reset and Select must preserve every original response declaration and remain unexecuted. An actual Run changes only execution state. Navigate while a run/load response is pending, switch selection, reset/re-run, and prove an old result cannot attach to the new selection. Existing unknown state/view behavior must remain unchanged.
5. **Remaining response families.** Direct-return nested data, response-owned identity, and one-time secrets get separate healthy/fault pairs. Nested source-path observations wait for their accepted authority design, then use this same bridge. No declaration-only or explicit unsupported case counts toward execution completion.
6. **Existing integrated regressions.** Run `replay_fidelity_browser`, `coverage_browser`, `one_time_browser`, `conform_web_history` and its adversary, browser startup boundary tests, plus response payload/direct-return/one-time tests appropriate to the touched source. Actual commands should be focused `cargo test -p ess-cli --test <name>` and `cargo test -p ess-conformance --test <name>` using the agreed resource limits. No test was run during this scoping pass.

Reuse the real browser harness: `replay_fidelity_browser.rs::Fixture::{emit, ...}` emits CLI routes and navigates `player.js`; b01–b12 test unknown facts, typed values and controls; `coverage_browser.rs::actual_browser_checks_full_lineage_and_integer_metadata` exercises original lineage; `one_time_browser.rs` already separates displayed policy from closed coverage refusal. Add a focused `browser_response_conformance.rs` rather than making the declaration fidelity suite fabricate observations. Test target implementations remain Rust.

## Scope

Derived 2026-10-03 by story-scoper; every entry labels its evidence. These are proposed implementation surfaces, not authorizations.

- **Primary surface:** `crates/verify/ess-conformance/src/web.rs` — cited: `emit`, `emit_input`, response refusal and emitted artifact packaging.
- **Files:** `crates/verify/ess-conformance/src/web_replay.rs` — cited: closed model/input pair and one-time authority refusal; preserve historical reader while deciding new representation.
- **Files:** `crates/verify/ess-conformance/assets/player.js`, `crates/verify/ess-conformance/assets/coverage-player.js`, `crates/verify/ess-conformance/assets/coverage-admission.js`, `crates/verify/ess-conformance/assets/index.html` — cited: actual product loading, navigation, closed reader and UI.
- **Files:** `crates/verify/ess-conformance/tests/response_payload.rs`, `crates/edge/ess-cli/tests/replay_fidelity_browser.rs`, `crates/edge/ess-cli/tests/coverage_browser.rs`, `crates/edge/ess-cli/tests/one_time_browser.rs` — cited: refusal and browser fidelity contracts.
- **New bridge emitter:** `crates/verify/ess-conformance/src/web_execution.rs`, `crates/verify/ess-conformance/src/lib.rs` — inferred: generated Rust host, installation protocol and artifact exposure following existing emitter conventions.
- **CLI wiring:** `crates/edge/ess-cli/src/main.rs`, `crates/edge/ess-cli/src/coverage.rs` — inferred: actual product route/options and coverage emission need integration with the new execution artifact.
- **New browser test:** `crates/edge/ess-cli/tests/browser_response_conformance.rs` — inferred: actual emitted product + installed target + Firefox fault matrix.
- **Documents:** `docs/design/typed-response-outcome-payloads.md`, `docs/design/review-replay-subset.md` — cited: response and declaration replay contracts need accurate browser support boundaries.
- **Confidence:** medium — inferred: existing gap and seams are concrete, but a generated host/public installation contract and packaging have not been built.
- **Would collide with:** browser declaration assets/readers, CLI conform-web wiring, response persistence authority, and conformance module exports — inferred: serialize these with any nested-response or format reader work; interpreter and caller synthesis need not be touched.
- **Safety fact:** `Runner::run_admitted` uses actual target responses while existing Step/Back controls only rebuild declarations — cited: runner.rs:373, target.rs:575, player.js:195; stage 2, unproven in the proposed product. Actual browser fault pairs must prove the boundary.

## Scope commands for the planning owner

These commands were not executed. Paths marked inferred above remain inferred here.

```sh
aep plan artifact scope story:browser-response-conformance --add crates/verify/ess-conformance/src/web.rs
aep plan artifact scope story:browser-response-conformance --add crates/verify/ess-conformance/src/web_replay.rs
aep plan artifact scope story:browser-response-conformance --add crates/verify/ess-conformance/assets/player.js
aep plan artifact scope story:browser-response-conformance --add crates/verify/ess-conformance/assets/coverage-player.js
aep plan artifact scope story:browser-response-conformance --add crates/verify/ess-conformance/assets/coverage-admission.js
aep plan artifact scope story:browser-response-conformance --add crates/verify/ess-conformance/assets/index.html
aep plan artifact scope story:browser-response-conformance --add crates/verify/ess-conformance/tests/response_payload.rs
aep plan artifact scope story:browser-response-conformance --add crates/edge/ess-cli/tests/replay_fidelity_browser.rs
aep plan artifact scope story:browser-response-conformance --add crates/edge/ess-cli/tests/coverage_browser.rs
aep plan artifact scope story:browser-response-conformance --add crates/edge/ess-cli/tests/one_time_browser.rs
aep plan artifact scope story:browser-response-conformance --add crates/verify/ess-conformance/src/web_execution.rs --inferred
aep plan artifact scope story:browser-response-conformance --add crates/verify/ess-conformance/src/lib.rs --inferred
aep plan artifact scope story:browser-response-conformance --add crates/edge/ess-cli/src/main.rs --inferred
aep plan artifact scope story:browser-response-conformance --add crates/edge/ess-cli/src/coverage.rs --inferred
aep plan artifact scope story:browser-response-conformance --add crates/edge/ess-cli/tests/browser_response_conformance.rs --inferred
aep plan artifact scope story:browser-response-conformance --add docs/design/typed-response-outcome-payloads.md
aep plan artifact scope story:browser-response-conformance --add docs/design/review-replay-subset.md
```

## Not established by this pass

* Host build/packaging cost, worker integration and browser responsiveness: no product bridge exists and no build was authorized.
* Exact public installation API or CLI flag spelling: design decision, not a discovered existing interface.
* New persisted replay representation/version: coordinate with the single held source-format bundle and the nested-observer owner; do not invent a reservation.
* Support for arbitrary network implementations from a static page: requires an explicit transport and async design, not supplied by the current synchronous target trait.
* Full feature support in the browser product: unproven until each admitted feature family has an actual healthy/fault witness through the emitted route.

## Actual CLI and Firefox baseline, 2026-10-03

Root executed both real CLI web products and loaded their emitted pages in actual Firefox through BiDi. The private pinned CLI has SHA256 d397e76cd811915e6efb9bfd33e0a7c7483ce65d9518ff229de0cbcab56fd23b, built from source parent6727e07363877925aa6a0f2bb1cb47b08d2348e3 with only the #293 explorer assets changed in production. Its CLI/web production files are unchanged; this fixture uses a flat response field and does not claim the later nested-response implementation was in that binary. No compilation or remote gate was started by this probe.

The admitted source is the existing response-payload fixture, SHA256 cb03d385ca8d0bb5492d89026fb1b438c43833935e6dc7b7e16065f6fbbc304b. The authored ess-scenario/4 claim SHA256 a138a665c49f83e596f5fd0337ed2aaf2e8d54bddfbc6617ee55bad80058283a names demo.api.Cancel/cancelled, declares a direct response containing remaining=9007199254740993, and expects demo.api.Returned. Both ordinary and --suite-format5 actual web CLI invocations exit0. Ordinary emits1scenario/7artifacts. Thus the design candidate's prediction that this authored response would fail response_replay_supported is incorrect: that guard tests ExpectResponsePayload, while the authored response compiles to ExpectDirectResponse. The prior library response-mapping refusal remains a separate verified source fact.

Actual emitted ordinary suite SHA256 cc78007f33b62dd5d36f38be676c016ddcb024cd97071ccb1197fa5533300338 retains the exact integer token. Its unchanged player parses JSON through Number, and the real Firefox DOM instead contains9007199254740992 and no9007199254740993. The ordinary DOM also contains only declaration-navigation actions and explicitly labels expectations unexecuted; no implementation target or execution report was produced. Actual Node assertion BROWSER_RESPONSE_EXACT_INTEGER_DISPLAY also fails1, independently matching this DOM observation.

Actual emitted coverage replay SHA256 adcd5ada4cf1980d91517ed41017b059bf2a2e58ac67bf62ee9dc1537c225e02 is emitted successfully but its browser admission aborts at the closed unsupported-step check. Firefox displays the admission.js:413 stack and leaves the Vue template unmounted. Executing that exact emitted admission module on that exact replay in Node separately yields Invalid coverage replay: unsupported suite step, exit1. Do not confuse successful artifact emission with successful browser loading.

Private evidence is retained under ess-browser-response-probe-20261003. The final actual-browser probe log SHA256 c7e364bdae9410c567e141099aaae80b4d45096bdd440e4abba7e9f47e62ba4a records BROWSER_RESPONSE_PRODUCT_GAP and terminal1; ordinary DOM9ec85109eb03d5e9ac80eda2997ad36417d24d53fdd9abdd2d55a6dde62628a2; coverage DOMebb796b9248db584fa7c459cf1b1b9b77cd32cbdb86ed7388956ba19f722ff52; full BiDi receipt62ec30cfad5b962e62345af7ff554f84c5da3f8d5d45e89e826a97dc231d7e66. Exact emitted assets, source inputs, logs, HTTP requests and private probe are retained. The first browser probe also observed both defects but its harness incorrectly required the Error message inside Firefox's stack-only DOM; its receipts remain unchanged, and the second probe corrects only that evidence predicate. No product source changed between probes.

This establishes a real product regression baseline, not a healthy/fault implementation conformance matrix. Target installation, generated host compilation, all-feature execution, one-time disclosure, full original-byte admission and native/browser parity remain required implementation acceptance. Revised browser design is being scoped with source-built CLI routes, explicit tested consumer Cargo packaging and concrete bounded ABI. No production edit or format reservation is authorized by this evidence record.

## Reviewed browser product direction

Root reviewed the original browser product candidate with final v2 delta (SHA256 140902410e0a8bb29809bc03e8d61418aeb83253b2f0ee794db909b4a385d4f2), with v2 governing all conflicts. review-result:consumer-browser-product-design-pass1 records approve with no findings. The design now has actual CLI and Firefox reds, a concrete default navigation/build boundary, exact original-byte Rust admission, concrete Target/Clock installation, closed ABI and proposed finite limits.

Choose useful static navigation immediately after CLI emission using a Rust-generated complete lossless declaration document. JavaScript renders display-only tagged numbers/text and checks original-file integrity; it cannot interpret source behavior or mint execution authority. Building the exact emitted Rust module with an explicit consumer Cargo manifest supplies the separate execution route. Before factory callbacks, that module re-admits complete original execution/lineage and source archive through existing Rust authorities and regenerates presentation. The real Rust Runner produces actual execution reports. No prebuilt reader, new packaging helper, target oracle, async transport or default reference target is part of this unit.

The unified binding design and browser product/presentation/ABI version1 catalog entries are being prepared before source dispatch. Exact implementation scope must be recorded and the isolated worktree established first. All original acceptance remains: actual emitted module build, independent healthy/fault target in Firefox, every admitted feature, exact large integers, full coverage selection/lineage, clocks/namespaces, stale-run handling and protected-value disclosure checks. Finite resource limits are unmeasured until actual boundary/peak evidence passes. This story is not implemented by its design approval; no source changes or format publication have occurred yet.

## Bound implementation scope

Binding design docs/design/browser-conformance-product.md, SHA256 02162cbaa70e722e782d187bdcd54f07d9fa7bf9ebb9b295a46a7265e699b88a, is committed with browser product/presentation/ABI version1 catalog entries in carrier commit d849ea01c. This is design registration, not a released reader or execution claim. Root design review consumer-browser-product-design-pass1 is approved. Actual CLI/Firefox regression evidence remains retained as recorded above.

The machine-readable scope now follows design section9. New source modules, embedded browser assets, admission/presentation tests and independent target fixture are marked inferred until implementation confirms them. Existing CLI acquisition/emission and browser test seams are cited. Legacy assets and web_replay.rs are removed from edit scope: preserve their bytes/behavior through an explicit legacy route. Existing response-payload and history/browser tests are neighbors and change only when dispatch expectations require it. Any additional file or helper first requires root scope correction.

Root owns planning, binding design, format catalog and changelog. Worker owns only the scoped implementation source/assets/tests and related response/replay design clarification after authorization. No transport, compiler constructor, Runner semantics, async target protocol, build helper, import flags or new suite major is authorized. Every new committed executable test/fixture is Rust; established embedded browser glue follows the existing asset convention. Work remains on the one held integration delivery path; no separate PR, remote gate or publication is authorized.

Implementation may proceed in isolated ess-browser-conformance-product-20261003 once provisioned from the exact binding-design carrier. Source/test work can proceed under current storage pressure, but compiler/build/browser starts require resource coordination; no third large cache is created. Existing servers and synthesis caches remain exclusively owned by #293 and #292. The browser worker must await an explicit cache handoff before any compile. Completion still requires the full design feature matrix, actual emitted module packaging and independent Firefox healthy/fault execution; static navigation alone cannot close this story.

## Bounded Run nonce inventory and build hold

Binding design follow-up commit 6b15cceb7cef5e1897056d0e9b0da9fc7f76e204 adds a finite inventory of 65,536 distinct Run nonces per module with no eviction. Validate frame and loaded handle, then consume the new nonce before installation factory or target callbacks, retaining it even if installation/execution fails. Duplicate reuse remains invalid; a new nonce at capacity returns resource_limit before callbacks. Release/selection never clear the inventory. Continuation requires a fresh worker/module and full Load. Current binding document SHA256 a0b8d60aee3fb42f334877a2a3abdd0346dacbe7fc3d3db690f72a74656f1890. This is a bounded design clarification, not a measured resource profile or execution result.

Owner has authored the scoped source/product/assets, independent fixture and admission/execution tests but has started no compiler or browser execution for this unit. Compilation remains held during the release owner's final check; the newly idle servers cache has not been handed to this worker. Rust formatting parsed source; it is not a build or test result. Original ordinary/coverage browser product reds remain the acceptance baseline.

## Uncompiled source checkpoint and first validation step

Owner checkpoint during the release compilation hold: authored source now includes independent healthy/fault direct responses, nested observations, protected-value controls, binding order controls, and a controlled timer installation with tick/busy/pending/drop/read/stop state sharing the execution clock. Ordinary and coverage routes are represented in the proposed matrix. Original coverage restoration retains the module nonce inventory; Load/Select transitions serialize; navigation is bounded per declaration step; duplicate control-JSON keys are rejected in the proposed implementation.

This is owner-reported source progress, not executed acceptance. The owner explicitly paused further fixture expansion after accumulating roughly 2,700 new lines across product/ABI/assets and independent fixtures. Scoped rustfmt syntax and whitespace checks pass; compiler, emitted-host, browser, resource-bound and complete capability-matrix execution evidence remains zero. No build cache or browser process is owned by this unit.

Next action after explicit release/cache clearance is a single-job check of ess-conformance and the exact browser test target on the authorized existing cache, then correction and actual execution before adding remaining matrix rows. The full all-feature acceptance remains required. This checkpoint grants no compilation clearance and does not freeze or transfer the unfinished source. story:feature-request-389 now has a depends_on edge to this story so its completion cannot lose this product execution requirement.

## Preliminary correction evidence during compilation hold

Preliminary review consumer-browser-product-preliminary-pass1 found two concrete defects in the uncompiled implementation. Root measured a display wrapper/depth mismatch with actual Node execution of the exact asset; separately, source inspection found arbitrary source paths could overwrite fixed output artifacts after manifest validation. Both were sent to the existing owner without releasing the compiler/browser hold.

The owner corrected the JavaScript raw-envelope budget to account for serialization wrappers while retaining logical display depth1024. Root then independently ran the same retained 26-control probe against both assets: the original SHA256 715ee9f33ee65ddfacb3e93b1fdd6abdb7e9c42bde9b297c42fa137b5d8dcbd1 fails eight controls and exits1; corrected asset bceead4c48731cdb49c4899b4d857ec1e61fe667070ae539725285bfe4dda4a6 passes26/26 and exits0. Controls cover Object and List values in model and scenario envelopes at logical depths0,340,341,400,1024,1025, plus literal and escaped duplicate member names. Depth1024 remains accepted;1025 refuses; duplicate keys remain invalid. Green log SHA256 db8ed686974fbe68370c258d54c74bd0d51ca203f80d092fa106bfd25eb69914. Frozen before/after assets, exact probe and both logs/exit files are retained privately in ess-browser-preliminary-review-20261003.

Root inspected the collision correction in web::emit_product: insertion of any fixed resource over an original blob now returns InvalidBundle rather than returning an internally inconsistent artifact set. Its Rust regression and persistent depth regressions still require the owner/compiler stage. This source inspection is not an executed Rust result, complete source review, or Firefox product proof. The owner retains unfinished source; no new frozen handoff or publication follows. Record the review's final disposition after the remaining correction validation rather than treating this bounded JavaScript green as approval of the whole product.

## Bounded compilation clearance after local release checks

Root released one bounded compilation lane after reading the release owner's final exact-candidate receipts. Local candidate e68684efb6a4ac22052c77d3ed8292fd44f9ace5 has final check.exit=0 and site-build.exit=0; earlier attempt2.exit=201 is retained and is not the final result. Root rehashed the complete final logs: check.log SHA256 949920adfa37c81f1a1182c5632c758604719330a46821670a097233c08726a1, site-build.log a41fb4900f775a6ed774acf532b9aab339a8b3c894541c232634e5d39c509224, matching final-local-verification.json in the owner's release-final-404 evidence. The site-build log includes the real site-lab prerequisites. The previously observed task/check/site processes are terminal. This is local release validation evidence only; no remote gate, tag, published Release or artifact completion is inferred.

Free disk measured16141500416bytes before dispatch. Managed inspection of the root-owned ess-backlog-servers-20261002 cache reports zero live leases, after #293 explicitly released both ownership leases. Browser owner receives exclusive use of its existing target cache, subject to acquiring a fresh own lease and checking for actual users. #292 remains held. No release/transport cache or source is modified.

Authorized first validation is sequential cargo check of ess-conformance --lib, then the exact ess-cli browser_response_conformance integration target, with one job, debug0, incremental0 and external TMPDIR. Each start requires at least12884901888 available bytes. Correct actual compiler errors within the accepted browser scope and retain all failed and successful logs/exits. Full-package tests, generated WASM, Firefox and parallel build starts are not yet cleared; root will sequence them after this first compile result and a fresh disk observation. This is root resource clearance for already authorized implementation, not an implementation/review/completion claim.

Owner's bounded correction checkpoint remains private and unvalidated as a whole: preliminary-corrections.patch SHA256 67bc83e027c4b7dadc983359a5620aebabd2b1b28f1faaf8536da3f0ae66c6e8, rehashed by root. Both persistent depth/collision regressions and actual product acceptance still require execution. The frozen 60-source handoff remains unchanged, with sole eventual delivery through the held shared integration branch.

## Native compilation and measured stack correction

Both narrow compilation checks passed without diagnostics: ess-conformance library and the exact ess-cli browser_response_conformance integration target, exit0. Root rehashed logs69a472edd4ce2a1384d340d450baf2c823ee33c2655ccbf355135402f9c6d79e andc2e611138ea0cad81cb83a06704d8c9d283a43b5482efe3ad21113f53a1a2f5a. Compilation alone did not prove the implementation.

Actual focused tests first passed admission7/0, while three presentation tests failed before product execution because the authored fixture declared an eventless/errorless unobservable outcome (EmptyChange). The owner corrected that fixture to declare and observe its mapped event; no production or acceptance weakening was involved. The fixture-authoring failure is retained as test-product-1.log SHA25654043744464b544db516bbb7d8bbd12445a0332e42323b9fc50fb7a2119e8329 and is not claimed as a product regression red.

The exact producer-depth unit then exposed a real product defect: Budget::value's recursive traversal overflows the ordinary test-thread stack at logical depth1024, aborting with SIGABRT and cargo exit101. Deciding red log SHA256f1a7dfa2be701fa0b72046f80028b4a5350a250cc5f23147aabeb227731aeef5 is retained. Bounded phase diagnostics isolated the first failure to parsing, before encoding or drop. The correction uses an explicit heap task stack, preserving the same depth/node limits and ordered display representation. RUST_MIN_STACK stayed unset; no global recursion override, enlarged stack or reduced depth allowance was introduced.

Final corrected source: native-validation-checkpoint.patch SHA256f6f869e6974cb921bdc6693166f245d49f5b4df8eba9c4f06c36ee26f8ad8ae8, rehashed by root. Both complete focused integration binaries pass, admission7/0 and presentation4/0, including the fixed source-label collision and persistent Node depth controls. Log test-product-3.log SHA25620d1092f757bf1dddcfb521e3427d8cdcd595658023a4bc3482540375723c13f, exit0. Exact producer-depth unit passes1/0 with97 unrelated library tests filtered; final log2df5bf65dba4d70523c9fc0e02766ecd37d49470306da2e4dabde97d064ade5a, exit0. At1024, List and Object values parse/encode/drop on the normal native stack, producing10527765 and25229333 bytes respectively;1025 returns ResourceLimit. Native success does not establish WASM stack or peak-memory behavior.

Following explicit operator release of the build reservation, root authorizes the next single static Firefox navigation test for both ordinary and coverage products, with all original assertions retained. It performs no WASM target execution. Same exclusively leased servers cache, one job and12GiB start guard remain. Full native/browser feature matrix, actual emitted host packaging/execution, resource measurements, strict lint, neighboring checks and independent final review remain outstanding; the full held source handoff is unchanged.

## Actual Firefox navigation and emitted host response validation

Actual Firefox static navigation passed for both ordinary and coverage products: test ordinary_and_coverage_navigate_exact_response_without_wasm, 1 passed, 0 failed, 6 filtered, exit 0. Both routes display 9007199254740993 exactly, reject the rounded display, and show build_required / Not executed without runner.wasm. Initial receipt-directory failure occurred before Firefox startup and was a test-authoring error; the caller now creates its own evidence directory. Final log SHA256 e994d00949883939d20dfc4f48396303006c8ef15e01ca5add6595c08c54fca5; checkpoint 1a45e7a1ad57077000d567c5c3bc217d3e2bfc6c15dac85ccfebbd4a40cbdce8. Root inspected both DOM and BiDi receipts; both DOM hashes are 1c3c6ae0de0afe941840698036ee56220b1840f8295ca6b1cef233b952b1f01c.

The exact emitted consumer host then compiled offline and executed through actual Firefox: emitted_consumer_module_runs_independent_response_target_in_actual_firefox passed, 1 test, 0 failed, 6 filtered, 122.92 seconds, exit 0. Fourteen executions cover ordinary and coverage routes, each with a healthy independent target and six faults: wrong response, wrong event, missing response, wrong response type, extra response field and wrong outcome. Both healthy scenarios pass and all twelve faulty scenarios fail, with zero skipped, unsupported or error scenarios. The healthy ordinary run retains unknown coverage and inconclusive conformance; the healthy coverage run has passed conformance. These are distinct from execution status.

Root rehashed the checkpoint ca4a129381bc0d9bfc965996f01aeafeb61005599dc89bef47b14cdaf93facdc, report 38012b2945269f271c88bc0825748e832d842b25a50c453986f68b82453205ea and terminal log 8b63c3dda8a18ded140c5b6012d53cd107846176d3dbfe341a8d0b9086e8fdaf. Root checked every entry of executed-response-evidence.sha256 (inventory hash eb06fdd2723bdec9769e6f0c08fb550bbd503bae4aad0e84d0dddbe540f590ed) and independently compared full native/browser CountReport and CountRun bytes for all fourteen cases: identical. Retained private evidence in ess-browser-product-progress-20261003 includes original files, exact emitted host, consumer manifests and locks, fixture and runtime source hashes, actual per-mode WASM, build exits and disk guards, DOM and Firefox/BiDi receipts. No production correction was needed in this packaging run.

Compilation uses one exclusively leased existing servers target and one reusable browser-wasm subdirectory, one job, debug 0, incremental 0, external temporary storage and a 12 GiB start floor before outer and every inner build. The host linker limits linear memory to 512 MiB; this is not measured peak-memory or boundary acceptance. Toolchain is rustc 1.98.1 / LLVM 22.1.8, cargo 1.98.1; this targeted local result does not substitute for the pinned release gate. Free bytes observed by root after evidence verification: 20849315840.

Next bounded validation is sequential protected_actual_values_never_enter_browser_reports_dom_or_messages and generated_nested_observations_compare_independent_response_and_event_in_firefox, preserving both routes and every healthy/fault control. The owner may correct scoped source/test defects with retained deciding failures, but no full gate, publication, separate PR or frozen handoff change is authorized. Remaining order, periodic, admission, full capability coverage, WASM resource measurements, strict lint and independent review remain required. Full all-feature acceptance is unfinished; fourteen response executions do not close the browser story or feature-request-389.

## Protected and nested actual-browser validation

Both next exact CLI tests passed with exit 0: protected_actual_values_never_enter_browser_reports_dom_or_messages in 25.50 seconds and generated_nested_observations_compare_independent_response_and_event_in_firefox in 36.50 seconds. Each test reports 1 passed, 0 failed, 6 filtered. This represents 20 actual Firefox executions through the exact emitted consumer host: 8 protected-value cases (four expected passes and four expected fault failures), plus 12 nested-response cases (two healthy passes and ten expected fault failures), spanning ordinary and coverage routes. All twenty reports show zero skipped, unsupported and error scenarios. Root independently compared complete native/browser CountReport and CountRun files for every case: byte-identical.

Protected cases retain actual DOM, worker-message and BiDi sentinel assertions. Nested cases compare the generated Packet.value observation with independent actual response/event values; wrong response, event, missing response, wrong type and extra field all fail. No production correction or acceptance weakening was required. The owner added raw report and DOM receipt retention before execution. These cases do not yet cover every protected failure/trap/abort surface or all nested container forms.

Root rehashed both terminal logs: protected 91788dfa59b489faed2bc36d3cdd0454452271d05aa7bcb196fc169c521941e8; nested 70f724f9ae72a02c6931696ddf56043edf46b310b229d5e9e6f794a2d3eb663a. Root verified all entries in protected-nested-evidence.sha256, inventory hash 2360f5483a663391912c0c8b7fc2b109803e7ec3bf3db432675c6d16067bff39; source checkpoint 31d8dda8b4a5c118b9f9d7afd4937ddae1cc882a60263a78da305055aeeb7c91; report 7785067e481ddf915e86ffe788aef362c717537e8cc6446fa0975ecf65240586. Private ess-browser-product-progress-20261003 retains exact original files, source/fixture/consumer/lock/module/WASM identities, reports, DOM/messages, build preflights and Firefox receipts. Root observed 20162854912 free bytes after tests; owner later measured 20095123456.

Next validation is sequential execution of the three remaining authored CLI tests: full_generated_order_service_exercises_views_grants_lifecycle_and_real_binding_faults; periodic_host_executes_real_ticks_reads_queue_and_stop_through_emitted_module; actual_emitted_rust_reader_refuses_original_byte_mutations_before_factories. Same single leased cache, jobs 1, debug/incremental 0, external temporary storage and 12 GiB guard before outer/every inner build. Retain failures and distinguish test-authoring errors from product defects. No full gate, commit/publication of unfinished product source, separate PR or handoff update follows.

Root source inspection additionally identified a possible stale-result presentation problem for the remaining scheduling matrix: setDisplay/loadOriginal/select/restore retain prior resultDisplay and download links when selected digest changes. This is a review hypothesis, not an executed finding; an actual completed-run to selection/restore control must decide it against the design's original-run identity requirement. Full feature, resource, isolation, scheduling, neighboring-test, lint and independent-review obligations remain unfinished.

## Order periodic and original-byte actual-browser validation

All three remaining authored CLI tests reached terminal exit 0. The browser owner confirmed no owned compiler or test process remains and holds further starts while #292 uses the single compilation lane. No production code changed in this group; source delta is Rust test receipt retention and correction of two invalid fixture assumptions.

Order: six actual Firefox executions run all 34 admitted generated scenarios. Healthy is 34 passed; dropped-binding fault is 30 passed and 4 failed; swapped-mapping fault is 33 passed and 1 failed, for both ordinary and coverage routes. Every run has zero skipped, unsupported and error scenarios. The unchanged oracle model also has exactly six synthesis refusals: weight_grams >= 0 is unpublished after AmendOrder/amended (Placed), CancelOrder/cancelled (Cancelled), HoldOrder/held (Held), PlaceOrder/accepted (Placed), ShipOrder/shipped (Shipped); handoff-on-shipped/binding/on-failure is PolicySilent. The first run incorrectly asserted zero refusals before browser execution. Correction pins these exact typed causes and preserves the full generated suite and coverage inventory. Healthy coverage remains inconclusive with 34 generated and 6 refused; ordinary coverage remains unknown. These refused obligations are not claimed as executed support.

Periodic: six actual Firefox executions, each with five scenarios. Healthy is 5 passed; each fault (wrong fresh-read mapping and early tick) is 1 passed, 4 failed. All four controlled binding IDs delivery/flow/mapping/on-failure under poll-status/binding execute; direct example.poll.Refresh/outcome/refreshed also executes. Zero skipped, unsupported or error scenarios. First run returned five healthy passes but hit a test-authoring ID filter searching for the literal periodic; correction checks the exact four binding IDs without removing scenarios or status assertions.

Reader: two actual Firefox sessions contain ten fresh emitted-WASM instances, covering five mutations per route: changed numeric token, unknown member, duplicate member, malformed UTF-8 and source/model mismatch. Each performs valid Load and healthy Run before bad replacement Load and stale-handle Run. Factory counts remain 0 after Load, become 1 after healthy Run, and remain 1 after both refusals. All ten mutations refuse. Rust decodes the captured actual ABI report/run bytes and compares them with native healthy bytes. This is direct emitted-module execution in Firefox, distinct from the other tests' page/worker execution.

Root independently verified complete native/browser report and run byte equality for all twelve order/periodic runs, inspected coverage/refusal and reader receipts, and checked every entry in order-periodic-reader-evidence.sha256. Inventory hash 7a34ee5b3ac936c71a5297dd71456a3db01daaec2b0bccb9f797252bf207f6d5; source checkpoint 364a92684731d71778c24da777bae69a2bf0dbdda1737b656a6d0a03821f56bb; report 830701262fd380aa147d27d272672c22280357ac14aa968d674005559cfe5792. Final logs: order f9ad86418c77d92aeb58ea9f09026de7b52450eb5702af2a58c0c7cbdaf13259 (19.06 seconds), periodic bb2fae1f148925de2f0a316899fb3a453197ac4748ecf0ab54c31215bb14b0fc (18.91 seconds), reader 4c4249a31084146867185efb787dae9f68ecb51ae810d5dda25cd5a41f8a136c (7.43 seconds). Retained first authoring failures: order 03abe07703f659f9bc3e97800b75ae251d59ff6bca04b738c33cf057a3f8c001 and periodic fcfeb2ec7fe39d127eb28ad1eab989d22250c183e534a00288eaef755417a58b. Exact inputs/source/host/lock/WASM/build preflights/DOM/BiDi remain private in ess-browser-product-progress-20261003.

The seven authored CLI tests have each passed their focused run; this does not complete full-feature acceptance. Actual root Firefox review consumer-browser-product-stale-output-pass2 now confirms obsolete completed output survives selection/restore. The owner is authorized to correct that association and author its persistent Rust regression without starting compilers/tests during #292's reservation. A concrete remaining capability/control inventory is also required. Additional feature families, lineage, resource/peak-memory, isolation, in-flight stale completion, abort/disclosure surfaces, legacy neighbors, strict lint and independent final review remain. All source remains unfinished and held; no source handoff or publication is authorized by these results.

## Measured stale-output correction and remaining capability inventory

The owner corrected the measured association defect in source: clearResult removes retained display state and mounted result text, disables result pagination, removes/revokes download URLs, and runs on admitted selection replacement, runtime discard/reconnect and a new Run. Corrected player SHA256 236720ab7c97b71efb3a40bcc3194e8fa05b73000d219d629a140381cd33c837. Full unfinished source checkpoint ad5ac9bba9e5d982a5a752d0f2b10c3e92088137968c750699ed5706412f2b21 is retained, not frozen for delivery.

Root independently ran the same retained regression-probe.mjs (SHA256 feea687aa22a00ff663fdb484d3a407d4775f1137a6732cdb4229f79b8ba34af) in actual Firefox before and after this asset correction, using the unchanged previously verified healthy WASM 17a25e7e203c30fea6d1e414a88a36e4ef285e1b3fbe2725d9cf2841be7a747b. Before asset bceead4c48731cdb49c4899b4d857ec1e61fe667070ae539725285bfe4dda4a6 fails with BROWSER_STALE_COMPLETED_OUTPUT and exit 1; corrected asset passes BROWSER_COMPLETED_OUTPUT_REPLACEMENT_GREEN and exit 0. Red log 8545342f57af9d59ce91b33520bd69263dc5f2adf184406dd5f780a4a977d457; green log 63b926c637bc90e37e6d52db47bf77b1856874f786812bc25a22951460ffbb48. Green DOM 6498b70b377d23bcda57199f25503a2d9d0543ec01f106afa23d22f00ef00419 independently shows 3460 result characters and two links before selection, then zero characters and zero links after both Select and Restore. A diagnostic boolean in the probe uses every(empty) and is vacuously true for the cleared link list; the decisive assertion explicitly requires length zero, and raw DOM counts establish removal. Original logs are retained without rewriting this limitation.

Green BiDi hash 2fed8379dc5f348a8102e9c7b4991edea9531fcb6661a97d862635c408d64171. Before/after assets, exact common probe, DOM, BiDi, HTTP and process receipts remain in private ess-browser-stale-result-review-20261003. Both root-owned Firefox processes terminated and their recorded PIDs were verified absent. No compiler or worker-owned site/cache was used.

Persistent Rust test completed_output_is_cleared_when_coverage_selection_or_runtime_changes now covers intended Run/Select/Run/Restore/Run/Reconnect, identity changes, three report successes and six URL revocations, but remains uncompiled and unexecuted while #292 owns the lane. This bounded actual-asset green does not close final product review or the persistent regression obligation. Owner's browser-remaining-capabilities.md SHA256 c82af597232c28e05de2dafcc9b47c17190f528b6e513fc462f9e7cb9964ce8f records every remaining family with concrete existing independent target/vector reuse and gaps. In-flight stale completion, identity echoes, lineage, all remaining Runner semantics, disclosure/trap/abort, real WASM resource boundaries, neighbors and strict lint remain required.

## Next source-only response value-form family

While #292 owns the compilation lane, the existing browser owner is authorized to author one next source-only family within the already accepted browser_response_conformance.rs and independent fixtures/browser-target/src/lib.rs scope. Cover Optional absence versus null and presence policy, List order and duplicate preservation, exact adjacent integers above 2^53 and nested combinations. Actual compiler admission and independent healthy/fault native plus ordinary/coverage Firefox execution remain mandatory; authoring assertions alone supplies no evidence.

Reuse existing response vectors and the independently implemented fixture, with wrong presence/order/dedup/value mutations. Do not read expected suite outputs into target behavior, substitute Interpreter, broaden into transport, introduce a new API or edit unscoped modules. Retain complete report parity and the pending stale-output persistent regression. No compiler, browser or test start, cache reacquisition, source commit, PR or publication is authorized by this source-only step. The compiler cache may be handed back later at servers target/native-build; the existing browser-wasm cache and all retained evidence remain untouched.

## Response forms authored and focused lane resumed

The next response-value family is authored in the two existing scoped Rust test files, uncompiled and unexecuted at this checkpoint. Seven source commands cover optional absence, explicit null, permitted absence/null equivalence, present exact integer, both explicit presence policies, and nested ordered lists with duplicates and adjacent integers 9007199254740993 and 9007199254740994. Six independent installation faults exercise policy violation, reorder, deduplication, adjacent-value corruption and missing present response. Intended matrix: seven scenarios across seven modes on both browser routes, with complete native/browser report and run byte parity. These are pending assertions, not measured counts.

Retained response-value-forms-authored.md digest 86e8a4f3587497116a98e1cf72200808283aef8b99764845b46085262e1b7c0f; source checkpoint digest 1e1b97f3b240f862edc741d89d374300895e0451a1953967890a30b89d7291ab. The prior stale-output fix remains unchanged at player digest 236720ab7c97b71efb3a40bcc3194e8fa05b73000d219d629a140381cd33c837.

After the history CLI terminal result released the compiler lane, the browser worker was cleared to reacquire its cache lease and run scoped cargo check, completed_output_is_cleared_when_coverage_selection_or_runtime_changes, then optional_presence_and_nested_ordered_values_execute_in_both_browser_routes sequentially. Native cache path is target/native-build; WASM keeps its separate existing target/browser-wasm. The 12,884,901,888-byte pre-start floor, one job, debug disabled, incremental disabled and external temporary directory remain required. No cleanup, broader matrix, commit or publication was authorized by this handback.

## Persistent stale-output and response value forms measured green

Both newly scheduled exact browser tests passed without source repairs. completed_output_is_cleared_when_coverage_selection_or_runtime_changes: 1 passed, 0 failed, exit 0, 37.56 seconds; log SHA256 fd25e96a886d26b775bb9cb74dae307f07a956282155651b3f9979720db1376c. The actual emitted healthy host completed three passing runs in Firefox. Root independently read the retained lifecycle receipt: selection changed the selected digest, restore recovered the original digest, all six obsolete download URLs were revoked, and Select/Restore/Reconnect each left empty results, no links and disabled pagination. This closes the persistent regression execution gap for the previously retained stale-output red; it does not close final product review or in-flight stale-completion coverage.

optional_presence_and_nested_ordered_values_execute_in_both_browser_routes: 1 passed, 0 failed, exit 0, 42.28 seconds; log SHA256 6411a3624a67317ddc347f29d33d2ca99895fe8984ae8cdf6d6230a0ba864984. Exact authored source was admitted. Fourteen actual Firefox executions covered seven scenarios on each ordinary/coverage route with seven independent installation modes. Each healthy mode passed all seven; each of six fault modes passed six and failed its exact named scenario. All reports contain zero skipped, unsupported and error counts. Faults detect required-null omission, forbidden null, nested-list reordering, duplicate loss, adjacent integer corruption above 2^53, and a missing present response.

Root independently compared all fourteen complete native/browser report pairs and all fourteen complete run pairs byte-for-byte and read report verdicts. Healthy ordinary execution remains conformance-inconclusive because it has no coverage claim; healthy coverage execution reports conformance-passed. All twelve fault executions report conformance-failed. These measured semantic fault failures are the intended regression controls, not twelve failing Cargo tests. This is one focused run per test, not repeated full gate or all-feature acceptance.

Private receipt roots retain completed-output-lifecycle-5-1375160 and response-value-forms-{4,5}-1380886, including actual DOM/BiDi, original product, source, module builds and native/browser reports. Broader capability families, resource-boundary evidence, native neighbors, strict lint and independent final review remain open. No source commit, independent PR, publication retry or release blocker resolution follows from these results. All four frozen runtime handoff digests were rechecked unchanged.
