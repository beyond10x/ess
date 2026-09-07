# Browser replay fidelity — accepted binding

**Binding state and authority**

This binding for story:review-browser-replay-fidelity addresses F15 in docs/reviews/2026-09-05-architecture-review.md:500–520. Root accepted candidate v2’s conservative declared-trace/explicit-unknown direction on 2026-09-07 after both recorded candidate attacks. The decision is retained in target/review-boundaries-14/preparation/browser-replay-candidate/coordinator-decision.json, SHA256 8d3633373ee4ebb97b6d3a7d19d61dc6024b008d81995542dd8e9aa528007967. The coordinator selects implementation in wave 15 under the standing remediation approval. This published binding governs that unit.

Original review source: ESS 9d84a425e3a0c052bb08975766c5dbd600d04ef0. The public-support refresh through 57e242e8a0eaa721968c3970099b4bc561cb91aa is historical: its six existing browser owners matched the reviewed bytes at that checkpoint. Current inspected source is ESS a45b4081de9352e0b2f0b7a8ec87bb91f99b6cc3 after namespace-observation PR #12. Five existing browser owners and the historical fixture are unchanged; main.rs changed only for namespace import, while its web help/dispatch remains unchanged. Both proposed test/document paths are absent. The complete main delta and exact owner pins are retained in namespace-main-root-refresh.json, SHA256 f01e9d62fe5eabb2345b5406ebfef2fa56cbada28a1597e2dd0ef749c9787157. Preserve those namespace changes and the current dependency graph during the web-help-only edit. No accepted browser behavior, format or B01–B15 consequence changes. Root must fill the final dispatch commit after binding/story publication and recheck source pins if that commit differs from this inspected source; any material binding change requires explicit reconsideration.

**Outcome and minimum complete unit**

Both freshly emitted players must show truthful declarations and explicit unknown results for every F15 counterexample. Preserve typed authored values, make sets participate alongside moves, stop guessing subject identity, and stop presenting guessed view membership or ordering. The existing acceptance permits an unsupported marker where the projection lacks information; the selected review direction uses that alternative. It does not claim to calculate post-assignment values or view results that replay/1 cannot establish.

This is one implementation unit spanning the current conformance emitter/assets and the existing CLI-owned actual-browser tests. No production interpreter, universal evaluator, primitive migration, new story or automatic upgrade to a stronger replay model is included.

**Required behavior**

1. Complete coverage-route admission before creating replay state or its banner. Preserve the closed paired model, exact selected suite reference, every original parent string and full lineage checks. Admission failures remain failures; an unsupported replay marker must never turn invalid input into an admitted world.

2. Preserve each tagged ScenarioValue::Literal as an authored input or expectation, including its admitted Node kind. Null, false, zero, empty text, numeric-looking text, lists and mappings remain distinct. A literal mapping containing an instance property is not an instance reference. Only the actual Instance tag denotes a reference; Observed denotes an unresolved observation request, not an observed value. Do not parse raw text into another scalar kind.

3. Process every sets entry for creates, updates and moves. A null assignment source means the literal value was omitted: show “Unknown: assignment literal is absent from this replay projection.” A named source still lacks source/target types and conversion metadata: preserve the named declaration and show “Unknown: assignment types/conversion are absent from this replay projection.” Missing input receives an additional specific missing-input explanation. Never fall back to an input named like the target. In particular, sets accompanying a move invalidate affected field knowledge instead of leaving an old value displayed as current.

4. Establish only the identity facts actually carried by declarations. A matching creates outcome with one unambiguous matching CaptureInstance may establish a scenario-local declared instance and its declared initial lifecycle state. It does not establish a real generated identifier. Creates without that capture remain unbound. Duplicate/conflicting captures or declarations remain unsupported. Updates and moves omit the precise subject-source field in replay/1: neither the first reference, a single reference nor the only displayed instance proves the subject. Show “Unknown: subject identity source is absent from this replay projection.” Never manufacture an instance to satisfy an update or move.

5. Propagate uncertainty into the internal known state as well as the visible output. An unresolved move invalidates lifecycle certainty for displayed instances of its entity; unresolved sets invalidate their target-field certainty. Preserve unrelated established facts. Refusing outcomes produce no entity effects. Missing knowledge is represented by absence from the internal known-value collection plus a separate explanation; it is never a domain value such as null, false, zero, an empty string, an em dash or the string “unknown.” Render every declared field, including unknown fields. An unbound effect prevents the UI from claiming that nothing exists. Custom skins may need to handle unavailable state; compatibility must not preserve invented known values.

6. Keep every view visible and mark computed results unsupported in this minimum unit, including when there are no candidates. Show the original filter text, declared parameter names and any reached query's supplied parameter values. Parameterized views say “Unknown: this replay does not evaluate parameterized view results.” Filtered views say “Unknown: this replay does not evaluate this view filter.” Every view says that model ordering was not projected. Never present candidate rows, insertion order or “no rows match” as an unsupported view's result. An ordering or row expectation is an unexecuted expectation, not an actual result or replacement for the missing model contract.

7. Preserve the original order and identity of query/assertion steps associated with each displayed command act, including steps before the first command. Retain them for display rather than silently dropping them. Redelivery, timing, eventual observation, halt and implementation-observation controls remain visibly unexecuted/unsupported declarations. Expected events do not become observations or supply Observed input values. Binding cards may remain declarations, but inferred “declared” instances must not enter calculated state or views. Conflicting declarations yield a diagnostic, not a best-effort calculation.

8. Keep progress separate from verification. Replace assertion-like completion marks with neutral replay progress. Rename the current notes heading that claims refusals the model would have made; the player has not evaluated that refusal logic. Correct emitted README and CLI-help claims that a completed walk establishes specification coherence. Continue to say that replay executes no implementation, fills no obligation and emits no execution report or qualifying conformance evidence.

9. Make Step, Back, Reset, Select and Play deterministic over the same declared prefix. Reset clears known state, unknown effects, notes, events and changes. Back reconstructs precisely the earlier prefix. Reset, selection changes, pause/resume and playback restart must cancel or invalidate outstanding callbacks; use a single owned timer and/or generation check so an old callback cannot advance the new prefix. Replaying a prefix must reproduce both API state and DOM markers. These controls change replay progress only.

**Source limits and format decision**

The Rust compiler is semantic authority. ResolvedPayloadValue::Literal.value is String at ir.rs:732–747, and assignment types/conversion are separate fields at 752–764. Subject identity is explicitly resolved at 583–645. View order_by is present at 899–906. The current web projection drops those facts: web.rs:131–175,200–208 and web_replay.rs:44–85. Adding a plausible value in JavaScript cannot recover them.

Keep all existing model.json, suite.json, replay.json and input-carrier semantic bytes and formats unchanged. No new replay/1 fields, meaning, reference rules, canonicalization or hidden metadata channel are allowed. Preserve suite/4 default emission and explicit suite/5 paired emission. Keep exact coverage origins, knowledge, selection, outside inventory, source identities, repeated refusal occurrences and causes. The browser still cannot reconstruct/authenticate the full model from the reduced projection.

The primitive-semantics story remains draft. Preserve current admitted finite-Node numeric behavior and exact unsigned metadata separation; do not add exact domain integer/decimal comparison, reinterpret numeric-looking strings, or admit modeled Binary64 here. The coverage admission predicate parser establishes lineage meaning; it is not a live-state evaluator.

A stronger computed-assignment/view capability is outside this draft. It requires acceptance of exact typed semantics and an explicitly versioned producer/reader/model migration before implementation. No unknown-to-value inference or replay/1 extension is authorized by this binding.

**Historical compatibility and delivery**

Keep crates/verify/ess-conformance/tests/fixtures/coverage/legacy-player.js byte-identical, SHA256 990575f6db31d13fecd7d06df51f3d34190cc8df33fa1a320efe682900d2e23f. The current coverage_browser.rs:145–147 equality between newly emitted player bytes and that fixture must become an explicit historical-player test: install the unchanged historical player into its test bundle, then retain the old-input replay and new-metadata-does-not-grant-admission assertions. Never refresh the historical fixture to make equality pass. Separately exercise both freshly emitted current players.

The generated README and current core UI describe the new conservative behavior; existing public replay-only claims must remain true. No CLI dispatch, defaults, suite selection or coverage policy change is selected. The proposed main.rs edit remains limited to its web-help coherence claim. The earlier byte-identical main.rs observation applies only through 57e242e8a0eaa721968c3970099b4bc561cb91aa; at a45b4081de9352e0b2f0b7a8ec87bb91f99b6cc3 preserve the new namespace-import flag, scan branch and qualified report unchanged. Recheck the complete main.rs source if the final dispatch base differs.

**Independent actual-player validation**

Keep the following B01–B15 matrix complete. Put fixture sources and independent expected vectors in the proposed Rust integration test, not in a copy of the player's evaluator. For semantic families, exercise both explicit suite/4 and suite/5 emission with the actual ESS binary. Serve the emitted bundle using the existing browser::Server and open it with the existing required Firefox/BiDi Browser. Import the actual emitted player, click its controls, await Vue rendering and compare both internal known state and visible output. Do not use the billing WASM lab, emitted-source checks or a browser-absence skip as substitutes.

Retain original command/output, Firefox and BiDi receipts, fixture/source identities and executed case counts. The historical and existing coverage families remain mandatory rather than being replaced by the new semantic vectors.

Authored input maps are sorted before emission; reversing their written insertion order is not a meaningful traversal-order vector. B06 therefore swaps bound aliases across fixed ordered names and checks the emitted distinction. B08 uses a valid unresolved observation value; missing required authored parameters retain their existing refusal before browser state. For control variants the authored frontend cannot emit, first generate the actual current assets with the real CLI, then use admitted persisted suite fixtures through the existing browser fixture route, preserving every pairing, lineage and coverage rule. This fixture route is explicit test input construction, not a new CLI authoring claim.

| Family | Independent vector and required observation |
|---|---|
| B01 — Supported declaration | A creates outcome with one matching capture shows the scenario-local alias and declared initial state; actual identity and undetermined fields remain unknown. |
| B02 — Sets plus move | Create, then select a moving outcome with two sets. Show the declared move and both write markers; no old field value survives as known post-state. Subject uncertainty is visible. |
| B03 — Lost literal | Literal assignments with same-named input decoys must not copy those inputs. Test Boolean-looking, numeric-looking and null-looking text, empty text and an ordinary string. |
| B04 — Typed scenario values | Authored input values `null`, `false`, `0`, `""`, `"false"`, `"0"`, a list and a mapping retain their kinds. A literal mapping with `instance` must not bind an instance. |
| B05 — Lost conversion | Two valid model fixtures differ by an assignment conversion while retaining the same reduced assignment shape. Both show unknown post-field state; the player must not report the source as the assigned value. |
| B06 — Lost subject source | Use fixed ordered input names and swap two already-bound aliases across them, so the first emitted reference actually changes. Assert distinct emitted ordered `(field, tag, alias)` sequences before the browser assertions. Retain a literal identity input beside an instance reference. Every case requires unknown subject; no first-reference selection or fabricated subject is allowed. |
| B07 — Ordering | Create rows in the opposite order to declared ranking; test ascending and descending model declarations. Both player routes show ordering unavailable and no claimed result ordering. |
| B08 — Parameter filter | Two candidate instances, a query matching only one, a different query argument, and a missing/unresolved binding. Every case shows the supplied declaration plus an unknown result; none returns all candidates. |
| B09 — Unsupported predicates | Conjunction, disjunction, range comparison and nested/fact-dependent filters remain visible with unsupported result markers. No expression is partially evaluated into a plausible result. |
| B10 — Unknown versus empty | No candidates, missing fields and an unresolved entity effect must not become a known empty view, known null or “nothing exists.” |
| B11 — Unexecuted assertions | View expectations, event expectations, eventual/timing/halt controls and binding consequences remain visibly unexecuted. They cannot seed field state, observed events or synthetic instances. |
| B12 — Controls | Step through known-create → unknown-effect → unsupported-view prefixes; back and replay must reproduce each prefix. Reset/select during pending playback must prevent stale callbacks from advancing state. |
| B13 — Coverage preservation | Run the existing full browser/Rust case generator unchanged: original-byte lineage, explicit empty selection, 70 generations, omitted-parent large integers, exact metadata, finite Nodes, repeated refusals and closed-field failures. Unknown replay semantics must not alter admission outcomes or the coverage banner. |
| B14 — Pairing before state | Preserve mismatched pair, invalid UTF-8 and every model-field boundary refusal. Require no mounted replay or coverage banner on admission failure. |
| B15 — Historical compatibility | Execute unchanged historical player bytes with old input and new metadata, then separately execute both freshly emitted current players. |


**Exact write reservations and proposed scope commands**

```markdown
## Scope

Derived 2026-09-07 by `aep-drive:story-scoper` 0.8.0. Original reviewed source is ESS 9d84a425e3a0c052bb08975766c5dbd600d04ef0; the earlier public-support refresh at 57e242e8a0eaa721968c3970099b4bc561cb91aa is historical. Current inspected source is a45b4081de9352e0b2f0b7a8ec87bb91f99b6cc3 after namespace-observation PR #12. Five existing browser owners and the historical player remain byte-identical; CLI main.rs has namespace-import changes while its web help/dispatch remains unchanged. Both proposed paths remain absent. All eight path reservations and cited/inferred marks are unchanged.

- **Primary surface:** the existing conformance web emitter and its two current generic players — cited; both current emission routes still contain the F15 behavior.
- **Emitter:** `crates/verify/ess-conformance/src/web.rs` — cited; `emit`, `emit_input`, `model`, `set_source` and `readme` own emitted assets, the shared reduced projection and its explanation. Reserve README wording and package-local projection/asset checks; preserve model and suite bytes.
- **Legacy-route player:** `crates/verify/ess-conformance/assets/player.js` — cited; `literal`, `groupSteps`, `applyAct`, `evaluateViews`, `mark` and playback controls own the current default player's behavior.
- **Coverage-route player:** `crates/verify/ess-conformance/assets/coverage-player.js` — cited; admission precedes replay creation, followed by the same defective assignment, subject-selection and view calculations.
- **Visible presentation:** `crates/verify/ess-conformance/assets/index.html` — cited; state, notes, view results and progress must distinguish declared inputs, unknown projections and unexecuted assertions.
- **Existing browser compatibility test:** `crates/edge/ess-cli/tests/coverage_browser.rs` — cited; its emitted-player-equals-historical-fixture assertion must become an explicit historical-player test while retaining every existing admission, pairing, lineage and old/new-metadata control.
- **New actual-player vectors:** `crates/edge/ess-cli/tests/replay_fidelity_browser.rs` — inferred; one Rust integration test owner containing the finite independent vectors and fixture sources, using the existing Firefox harness unchanged.
- **CLI description:** `crates/edge/ess-cli/src/main.rs` — inferred; update only the web command's claim that a green walk establishes specification coherence. Preserve dispatch, selection and suite-format defaults, including the newly landed namespace-import flags, acquisition branch and coverage report.
- **Binding document:** `docs/design/review-replay-subset.md` — inferred; record the finite subset, unknown propagation, literal/identity/conversion limitations, compatibility decision and executable matrix.
- **Read-only dependencies:** paired-model admission, coverage admission, the existing Firefox harness, compiler/scenario declarations and the retained historical player remain source evidence and regression controls — cited; this candidate requires no implementation changes to those owners.
- **Confidence:** high — cited; current owners and generic-browser infrastructure are present, and the proposed new test/document paths are explicitly identified.
- **Would collide with:** changes to either player, the shared HTML/emitter, CLI web help or generic-browser compatibility tests — cited.
- **Boundary:** no persisted replay extension, primitive migration, production interpreter, universal evaluator, new story or coordinator-owned planning mutation — inferred; stronger computed semantics require a separately accepted format consequence before implementation.
```

Proposed scope commands, **not executed**:

```console
aep plan artifact scope story:review-browser-replay-fidelity --add crates/verify/ess-conformance/src/web.rs
aep plan artifact scope story:review-browser-replay-fidelity --add crates/verify/ess-conformance/assets/player.js
aep plan artifact scope story:review-browser-replay-fidelity --add crates/verify/ess-conformance/assets/coverage-player.js
aep plan artifact scope story:review-browser-replay-fidelity --add crates/verify/ess-conformance/assets/index.html
aep plan artifact scope story:review-browser-replay-fidelity --add crates/edge/ess-cli/tests/coverage_browser.rs
aep plan artifact scope story:review-browser-replay-fidelity --add crates/edge/ess-cli/tests/replay_fidelity_browser.rs --inferred
aep plan artifact scope story:review-browser-replay-fidelity --add crates/edge/ess-cli/src/main.rs --inferred
aep plan artifact scope story:review-browser-replay-fidelity --add docs/design/review-replay-subset.md --inferred
```


**Future gates — not executed**

```console
cargo test -p ess-conformance --locked
cargo test -p ess-cli --locked --test replay_fidelity_browser --test coverage_browser -- --nocapture
task check
task site-build
```

The integration coordinator retains individual gate statuses and actual case counts. No Firefox fallback or silently skipped browser case is permitted.

**Original source pins and retained evidence**

- Exact semantic source: ESS 9d84a425e3a0c052bb08975766c5dbd600d04ef0.
- report.md: 27,218 bytes; SHA256 57b892e06040740ad128b99730abae57f74047945405e5495468cf1d0d99db5b.
- readback.json: 16,097 bytes; SHA256 0f5c99be78711142648b1a3b72beb896f3173ff21f79765285cbde1668181bba.
- The retained readback binds all 27 original Git/local inputs, their complete-byte hashes and inspected extents, totaling 720,756 bytes. The companion binding metadata repeats that manifest and verifies the pins at draft creation. Existing source owners remain pinned there; proposed test/document paths are absent at the frozen subject and carry inferred scope.
- Brief: target/review-boundaries-14/preparation/browser-replay-scope-brief.md, SHA256 39c5dd5bbcfe2b05e4a30deadefb6e70e476f7580fea7dd2b6e1e46042163800.
- Instructions: aep-drive story-scoper 0.8.0 and ess-specify 0.8.0, pinned in the retained readback.

Both candidate attacks and root acceptance are recorded. The earlier refresh through 57e242e8a0eaa721968c3970099b4bc561cb91aa found no change in the six existing write owners. The current a45b4081de9352e0b2f0b7a8ec87bb91f99b6cc3 refresh records namespace-only main.rs changes; browser behavior and the web-help-only edit boundary remain unchanged. Root still owns binding publication, exact typed story-scope synchronization, computed wave selection and resource assignment. Acceptance of this direction does not establish implementation, runtime validation, browser execution, source shipment or deployment authority.


**Correction after binding attack 1**

The complete review is `review-result:browser-replay-binding-pass1`, retained under `candidate-attack-pass1/report.md`, SHA256 `b64600f50f7d1fa8a8d7ecd618e60202e06c40b97fe5581e7fdca63204e9de6e`. Its one introduced validation finding is corrected in B06 above. The runtime behavior, original-byte format decision, scope and complete B01–B15 family set remain unchanged. The second and final candidate attack is recorded as review-result:browser-replay-binding-pass2; its complete report SHA256 is 31673ff9360b515f894d32a5e4cde9fcb8f546a98d87e813bc7849ead0c6fbef. The recorded ledger is carried 0 / new 0 / resolved 1. Both candidate attacks executed zero runtime cases. Root subsequently accepted the direction; no third candidate attack is requested by this publication draft.


**Integrated source refresh record**

The earlier refresh is retained at target/review-boundaries-14/preparation/browser-replay-candidate/integrated-refresh/readback.json (SHA256 c527e88623058ab94f426481b70472c596d311bd5ad5c563e01e8fc3d1ca7099). The final work-order draft readback pins the same 25 material inputs at integrated 57e242e8a0eaa721968c3970099b4bc561cb91aa, both missing-path observations, all six identical existing reservations, and the final marker/metadata delta. The historical fixture remains 12,902 bytes with SHA256 990575f6db31d13fecd7d06df51f3d34190cc8df33fa1a320efe682900d2e23f. The accepted behavior, B01–B15 matrix, explicit fixture routing, original-byte requirements and future gate commands are unchanged from reviewed candidate v2. Root retains full integration and publication obligations separately.


**Dispatch source refresh — 2026-09-07**

The source anchor is a45b4081de9352e0b2f0b7a8ec87bb91f99b6cc3. namespace-main-root-refresh.json is the independently retained root comparison from 57; dispatch-refresh/readback.json records this bounded document refresh and exact source/output pins. The five browser emitter/asset/test owners and historical fixture retain their reviewed bytes; main.rs and Cargo.lock now have namespace-observation identities. Full package and integration checks must use the actual assigned source, including those new infrastructure dependencies. This does not add an infrastructure write reservation or re-open the accepted browser contract. The five protected behavior/validation sections, complete B01–B15 matrix and historical-player preservation requirement remain byte-identical to the accepted publication draft. Older sections explicitly labelled original or earlier remain historical evidence. Root owns N=1 wave selection and fills final commit, unit/branch/lease and mutable-resource slots before dispatch. No build or browser was run by this refresh.


**Implementation clarifications — wave 15**

The two current players retain the same declared replay implementation after their distinct input
loading/admission steps. They keep the original tagged inputs and step positions, including a leading
declaration group. Reached queries expose their declared arguments; no view has a computed `rows`
collection. Instance `fields` and optional lifecycle `state` contain known facts only. Separate
`unknownFields`, `stateUnknown` and `world.unknownEffects` explain unavailable knowledge. Skins must
check for absence rather than interpreting an unknown marker as a domain value.

The existing reduced `outcome.refuses` field is a **wrong-state policy**, not a classifier for every
outcome. `crates/specify/ess-domain/src/command.rs:852–857` defines it as true but unused on ordinary
outcomes; `is_refusal` at line 955 and outcome validation at lines 1135–1179 establish actual refusal
from the declared error. `crates/specify/ess-compiler/src/ir.rs:695–702` retains that distinction.
The web projection carries no error classifier. Its bytes stay unchanged: a validated refusing
outcome has no subject or entity effects, while ordinary creation and movement follow the subject
declaration even when this policy field is true. The actual-browser refusal vector compares all
established instances, unknown effects, notes and observed events before and after the refusal.

A capture's emitted event field can have a different name from the entity's identity field.
`ResolvedInstance::Observed` in `crates/specify/ess-compiler/src/ir.rs:600–615` owns that source;
`crates/verify/ess-conformance/src/authored.rs:1901–1932` retains the explicit capture declaration. Matching the projected entity and
emitted event can establish the declared scenario-local alias. Comparing the two field names would
invent an additional restriction. The player still establishes no actual identifier and refuses
ambiguous capture or outcome declarations. A valid renamed-event-field vector exercises both routes.

Coverage admission's exact unsigned metadata tokens remain intact in the internal declaration.
The display renders decimal tokens only at the scenario vocabulary's metadata owners (`elapsed`,
`after`, view count bounds and positional index), without conversion through JavaScript Number.
Literal Nodes keep their admitted finite-number behavior; literal mappings with similar keys are
not metadata. Original suite, parent and carrier strings remain unchanged.

The executable B01–B12 families and additional refusal, capture, update, binding and timer variants
live in `crates/edge/ess-cli/tests/replay_fidelity_browser.rs`. They emit explicit suite/4 and suite/5
with the real CLI and exercise the emitted assets and DOM in Firefox/BiDi. B13–B15 remain in the
unchanged admission/lineage assertions of `coverage_browser.rs`; its historical test explicitly
installs the immutable fixture before running the original old-input and new-metadata assertions.
The implementation handoff retains the exact route/variant matrix, baseline and assertion-red
receipts, final gate receipts, source and fixture hashes, and the complete scratch census.
