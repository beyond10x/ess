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
revision: 16
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
