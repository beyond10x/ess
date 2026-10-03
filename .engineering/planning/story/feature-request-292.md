---
format: aep.planning-md/3
id: story:feature-request-292
kind: story
status: active
title: 'check-history: a generated Timestamp makes every history uncheckable (check.model-undetermined, exit 2)'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#292
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/check_history.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/caller.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute/history
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute/history.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/related.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/subject.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/values.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/linearize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/linearize/generated.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/generated_history_values.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/linearizability.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/linearizability_adversary.rs
- confidence: inferred
  path: docs/design/generated-history-values.md
revision: 29
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T02:16:17Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
- {from: "proposed", to: "active", at: "2026-10-03T02:16:17Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
---
## Outcome

Resolve beyond10x/ess#292: check-history: a generated Timestamp makes every history uncheckable (check.model-undetermined, exit 2).

## Origin

beyond10x/ess#292, from a downstream hardening run on ess 0.48.0; reproduced minimally (triage item 4, `~/.cache/ess-gaps/triage-cb/`).

## Fit review

# #292 fit proposal: generated stored values in outcome-only histories

Decision proposal: **accept, redesigned**, contingent on the bounded probe below and root recording the authority decision. This is an intake proposal, not implementation approval or a closure claim.

Inspected carrier: `ess-backlog-next-20261002`, exact HEAD `046db8a6805154aa5cafc63b0a4b741bc26e954d`. Issue evidence: supplied `issue-292-current.json`, issue #292, “check-history: a generated Timestamp makes every history uncheckable (check.model-undetermined, exit 2).” No network request, build, source mutation, or executable probe was performed for this review. All predicted runtime results below are **source-based inferences**, not measured red/green evidence.

The requested original minting change appears present locally: `interpret/execute.rs:1450`–1485 now uses `witness::fields` for non-UUID generated values, and `witness.rs:3630`–3647 supplies Timestamp, Duration, String, Integer and Decimal. The dangerous remaining assumption is the issue's statement that later guards still refuse as Undecidable. Current source does not preserve that distinction.

## Seven-question fit review

### 1. Need, separated from the proposal

Need: an outcome-only history should remain checkable when an operation writes an implementation-owned field whose actual value is irrelevant to the recorded answers. A successful creation storing an unobserved login instant should not fail solely because Timestamp has no implementation witness. When a later answer depends on that unobserved value, the checker must explain its missing authority rather than silently treating its own sample as the observed value.

Requester proposal: deterministically mint placeholders for Timestamp and sibling primitives, with later guards continuing to produce the existing Undecidable refusal. Source evidence supports the first part locally and contradicts the second part. Their historical reproduction is one returned StartSession/started operation, with generated `login_at`, versus the same source without that field (`issue-292-current.json`). A new brand-free minimal source/probe is specified below; no adopter repository or private data is needed.

### 2. Classification

The original report is an interpreter capability gap over already-admitted source, not a request for new syntax. `{generated: true}` in assignments already means that the implementation chooses the value (`docs/design/value-expressions.md`, E3, lines59–68; `website/docs/reference/spec-versions.md:152`). The current non-UUID mint fallback appears to close that isolated gap, but this intake has not measured it.

The authority issue is a checking limitation requiring a decision. `linearize.rs:55`–74 explicitly describes bounded candidate inputs and warns that neither verdict recovers actual unrecorded input. It also explicitly says other generated values use the model counter. Therefore do not overstate every definitive outcome from a generated witness as a newly proven contradiction of the existing approximate-checker contract. What is definitely false is the requester's premise that a later guard remains Unknown. To accept the requested safety condition, history execution must distinguish unknown generated stored data from concrete native implementation data, or conservatively refuse histories that need that distinction.

### 3. Can it already be expressed?

The source can already express generated Timestamp assignments and stored predicates. Existing source fixtures `tests/fixtures/state-in-subject-predicate.yaml` and tests `interpreted_stored_guards.rs:125` onward exercise stored Timestamp comparisons; `docs/design/value-expressions.md` E3/E6 supplies the language authority. No new noun, keyword, input spelling or source format is needed.

A plain Timestamp is currently generated through `mint` -> `witness::fields` -> `primitive_value`; source predicts the original one-operation history now checks. This must be shown by the proposed real probe before claiming fixed. No `ess specify validate` or test was run in this read-only assignment.

The history cannot express the actual stored timestamp: `history.rs:195`–225 declares operation command, subject string, time bounds, completion, outcome, rows and retry; it carries neither input nor generated stored values. `linearize.rs:482` gives Recorded only the creating identity slots found by `observed` (`:574`). An input example is not authority for an implementation-generated stored field, and substituting an authored literal would change the consumer's contract.

### 4. Fit and composition

Reuse the current authored surface and the existing `check.model-undetermined` refusal (`linearize.rs:282`; CLI exit2 documented in `crates/edge/ess-cli/tests/check_history.rs:1`–4). Keep budget exhaustion Unknown/exit3 separate from a capability or authority refusal. Do not extend native Interpreted's uncertainty: the native implementation genuinely owns its chosen values and later reads of those values are sound.

The complete read path is concrete today:

1. `Generated::Recorded` means missing observable slots are minted; its map is event+field keyed, not stored entity+field (`execute.rs:172`–202).
2. `linearize::split` fills only history-created identities in that map (`linearize.rs:465`–486, `observed` at574).
3. `write` calls `value`; Generated calls `mint`; the resulting Node is inserted into the real `Instance.fields` map (`execute.rs:1311`–1380). There is no provenance tag or unknown-value marker in Store/Instance (`execute.rs:91`–150).
4. `subject::Held::new` binds those fields as ordinary known typed facts (`execute/subject.rs:17`–35). Stored predicates evaluate them and produce True/False; only a genuinely Unknown result becomes Undecidable (`:62`–113).
5. `linearize::step` executes with Externals::Open, filters the resulting outcome names, and keeps the next concrete Store (`linearize.rs:602`–657). It does not track alternate generated values or their source authority.

Composition must cover generated leaves in nested assignments; InputOrGenerated fallback; copies through SubjectField and RelatedField; increments; whole aggregate copies; overrides by later known assignments/clears; entity invariants and at-rest validity; selection by stored-field equality/predicates; subject/related reads; and set-effect selectors. A field's generated presence is unknown too: deleting it as a shortcut would make `defined`/`missing` falsely known. A literal or input overwrite can reestablish knowledge and should not inherit permanent taint. Tests must include a false input conjunct whose result is independent of the unavailable stored fact and existing refusal precedence.

Current linearization partitions by subject and documents an assumption that no step reads another instance (`linearize.rs:19`–23). Related-field history checking can already violate that assumption; this is an adjacent preexisting issue, not authority to redesign partitioning in #292. Any #292 behavior for related reads must either route through a sound existing partition or explicitly refuse. Do not advertise cross-record history completeness based on native runtime support.

Targets: Rust owns the only history checker; Go and TypeScript explorers write histories and invoke it (`linearize.rs:3`–5; `docs/design/concurrent-history-conformance.md`, decision1). No independent Go/TS history algorithm needs to be added. Browser/WASM conformance is not a history-verdict authority. Native runtime, ordinary conformance, synthesis, diff, generated types and Entity Runtime should retain current generation semantics. If source/IR/history bytes do not change, those targets require no feature rewrite; retain existing named refusals for unrelated unsupported types (e.g. Binary64 witness generation).

### 5. Second unrelated adopter

A fulfillment service records a generated packing timestamp while returning only a package identity and outcome. Its create-only history should be checkable; a later route decision comparing that timestamp with a cutoff cannot use the checker's guessed date as the actual date. Another sibling is a generated numeric priority later compared to a threshold. Both use existing assignment and stored-predicate constructs (`docs/design/value-expressions.md`, E3/E6). This is not a local naming policy.

### 6. Cost

The smallest safe continuation adds no source keyword, IR format, history format, generated API or diff classification. It adds focused Rust regressions and clarifies checker refusal authority. A conservative history-only dependency refusal can live at the linearizer boundary with an explicit unsupported reason; a precision-preserving runtime implementation would need unobserved-value provenance across state/value paths and cache equality, a materially larger design.

A new history envelope that records actual inputs/generated stored values is a separate format and recorder/adapter migration, not implied by this issue. A complete existential solver for all admissible values and predicates is also separate. A refusal can reduce apparent checkability, but is preferable to claiming the newly accepted uncertainty contract while emitting a sample-dependent verdict. Existing approximate input-search caveats remain.

### 7. Alternatives

A. Change nothing / close from source inspection: reject. It is reasonable to verify and credit the already-present primitive fallback, but no executed reproduction exists in this intake and the requester's later-guard guarantee does not hold.

B. Mint more placeholders and leave them as stored truth: this appears to describe current code. It unblocks irrelevant fields but cannot establish the requested refusal behavior. Even a fixed seed is not evidence about an actual implementation value.

C. Conservative history-only refusal when a history can depend on an unobserved generated stored value: recommended first bounded implementation option, after a written precision/false-refusal decision. Permit the original inert one-operation case and independent lifecycle operations. Refuse relevant stored predicate/value/invariant dependencies by name. A source-level dependency analysis must propagate aliases/copies or conservatively refuse any unresolved propagation; an incomplete “same field name only” analysis is not acceptable. It can be sound yet conservative, and should say so.

D. Track unknown generated values through a history-specific abstract state: better precision but larger scope. Unknown must be distinct from absent/null; propagate through nested/copy/increment/related reads and cache keys, preserve concrete observed identities, and allow a known overwrite to discharge uncertainty. This requires a dedicated design rather than adding a flag that only one guard reader honors.

E. Record actual values in a new history format or solve all possible generated values: potentially stronger conclusions, but a broader contract. Neither is needed for the original inert Timestamp history.

Proposed decision wording: “Accept the need, redesigned: verify already-present primitive witness support for histories whose recorded answers do not depend on unobserved generated data, and refuse unresolved generated-value dependencies rather than interpreting placeholders as observation authority. Record a separate design before introducing abstract-state execution or a new history envelope.” Root should choose C or D explicitly before authorizing production work; this report does not silently choose an implementation architecture.

## Smallest red-capable probe plan

All future executable probes are Rust, in a newly leased managed tree/cache allocated by root; no probe was executed here.

1. Add `crates/verify/ess-conformance/tests/generated_history_values.rs` with a self-contained source18+ fixture, `Specification::assemble`/compile, and history JSON admitted through `history::read` with `SuiteProvenance::of(&ir).spec_digest`. Reuse helpers from `tests/linearizability.rs:77`–115 and the history/recorder seam in `linearizability_adversary.rs:104` onward. This tests the actual checker, not a mock.
2. Baseline control: one returned Start/started operation, actual generated UUID history identity, literal paused=false, generated login_at:Timestamp. Expected local result by inspection: Linearizable. Removing login_at must retain that result. Repeat Duration/String/Integer/Decimal/Boolean/Bytes, plus an Optional case, as explicit admitted sibling controls. Do not require currently refused Binary64 under a claim of all primitives.
3. Authority red: two nonoverlapping operations in one subject partition: Start/started, then Inspect/early or Inspect/late. Stored predicate: `login_at < "2021-01-01T00:00:00Z"`; early and fallback late both preserve the existing subject and emit nothing. Only difference between the two histories is the recorded second outcome. Counter Timestamp chooses January2020 (`witness.rs:3076`–3083, `:3642`–3645), so source predicts early -> Linearizable and late -> Violation, with neither refusing. Under the proposed conservative contract both must instead produce CheckRefusal::Model for missing generated-value authority. This is the red-capable assertion.
4. Add a stronger legitimate-control source replacing generated login_at with explicit typed literal values on opposite sides of cutoff. Both corresponding histories must check. This demonstrates that the late history has a valid interpretation under the generated source, while avoiding a mock expected response or a native target using the same placeholder as the checker.
5. Guard-dependence metamorphism: prepend a generation of an unused Boolean stored field before the generated guarded Boolean; counter parity flips. Keep relevant command/outcome history identical except recomputed model digest. Current code predicts a changed verdict despite the unused field not constraining the guarded value. A refusal-based fix must be invariant under this irrelevant source change.
6. Preserve controls: given creation identity stays exact; a duplicate creation remains illegal; literal/input-determined stored reads remain checkable; successful known overwrite removes generated dependency; a missing required stored fact remains refused; false input conjunct does not turn unknown into a default selection; generated event-only fields continue to check. Existing `linearizability_adversary.rs:104` and `linearizability_adversary_pass2.rs:122` onward already protect event-only/identity behavior.
7. Before choosing implementation scope, run one each of nested generated leaf -> copied subject leaf -> stored predicate, invariant over generated field, InputOrGenerated absent fallback, and related lookup. A conservative implementation must explicitly refuse unresolved paths, not silently pass them. Related partition semantics remain separately tracked.
8. Once library behavior is correct, add one CLI regression in `crates/edge/ess-cli/tests/check_history.rs` proving exit0 for the inert field and exit2/check.model-undetermined for dependent history. Exit1 is a violation and exit3 is exhausted search; do not conflate them.

Suggested minimal source shape (proposed, NOT compiled in this review):

```yaml
format: ess/18
system: demo
version: v1
domain: demo.sessions
entities:
  - name: demo.sessions.Session
    identity: {name: session_id, type: Uuid}
    fields:
      - {name: paused, type: Boolean}
      - {name: login_at, type: Timestamp}
    lifecycle: {initial: Active, states: [Active], terminal: []}
events:
  - name: demo.sessions.Started
    fields: [{name: session_id, type: Uuid}]
commands:
  - name: demo.sessions.Start
    input: []
    outcomes:
      - name: started
        creates: demo.sessions.Session
        instance: session_id
        sets: {paused: 'false', login_at: {generated: true}}
        emits: [demo.sessions.Started]
        payload: {demo.sessions.Started: {session_id: {generated: true}}}
  - name: demo.sessions.Inspect
    input: [{name: session_id, type: Uuid}]
    outcomes:
      - name: early
        when_subject: {predicate: 'login_at < "2021-01-01T00:00:00Z"'}
        preserves: demo.sessions.Session
        instance: session_id
      - name: late
        preserves: demo.sessions.Session
        instance: session_id
```

Do not call this source admitted until the probe compiles it. Use one real UUID subject, Start invoked_at1/returned_at2, Inspect invoked_at3/returned_at4, completion Returned, outcomes started and early/late. No concurrency is needed to expose the authority problem; a single forced order removes scheduling ambiguity.

## Completion boundary

This intake establishes a traceable source concern and a bounded way to measure it. It does not establish executed failure, an accepted design, a patch, a released fix or full all-feature conformance. Root owns AEP recording and any later assignment. Native response work was already recovered and integrated separately as `046db8a680`; it is not part of this #292 scope.

## Executed evidence, 2026-10-03

A private standalone Rust probe invokes the production source compiler, history reader and linearize::check against runtime source046db8a680. Documentation-only HEAD6f007d2db2 has identical crates/Cargo source; both recorded runtime diffs are empty. Brand-free source50883803ea38a7e63f95a4060413bd59cb271c61d8bbfcc1baecc00b69a2a059 stores generated login_at:Timestamp, then a second command selects early before2021-01-01 or fallback late. Histories carry outcomes and subject identity only, no timestamp authority.

Measured terminal exit0: one-created Linearizable; two-early Linearizable; two-late Violation. This confirms the original primitive mint limitation is absent locally and the claimed later Undecidable behavior is absent too. The current probe is observation evidence, not a failing regression asserting the new contract. Earlier Rust harness compile/source-admission errors are retained separately and are not product red evidence.

Probe source4ba1382349dafbb6f0c5e19811093f2264795e1dc97b5f9c1ea1a4950a76e3b1; actual loge1d1e050a43c1d8f01406151e59de6ea015b788e54272c5d372a6326a371c2d4; report2a573af494be9384fd0385b93f5d4512a47f1e26f62f887bbc5a4e5c3eec6d16. Original source, histories, locks, logs, hash manifest and runtime-delta evidence are retained in private ess-292-history-probe-20261003. Raw report contains local reproduction paths and is not copied into public source. Fit proposalee1ec07069aebe10fe7e73408dcdac0e42b9271b339bf0e86ef8fc15c4fa1985 is source-only; this section supplies actual execution.

## Decisions

Accept, redesigned: preserve checkability of histories with inert implementation-generated fields and conservatively refuse outcome dependencies on unrecorded generated data with the existing model-undetermined authority refusal. Native concrete Interpreted execution genuinely owns its generated values and must retain its behavior. Do not change source syntax/history format or treat a chosen witness as an observation. The checker's existing bounded-input approximation remains documented; this decision strengthens the specific generated-value boundary requested by the issue, without claiming the probe independently proves a stronger theorem than that documented checker contract.

Implementation mechanism remains under bounded design review: prefer the smallest sound history-specific dependency refusal, using private provenance/abstract state only where necessary. Cover value copies, known overwrite, nested/optional presence, increments, invariant/guard/related/set-selector reads and nonselected fallbacks. Do not close the issue merely because one creation checks. Story remains draft until exact scope/binding design and genuine red regression are ready; no production implementation dispatched yet.

## Design review direction, 2026-10-03

Read-only candidate292-generated-history-design-candidate.md SHAfe9d01464748d7f5e57e1d164636a11afcb75c34bee98d68c398a132d3e46f43 proposes top-level provenance plus an all-branch pre-execution dependency audit. Root rejects that candidate as the final implementation: false conjuncts, earlier refusal branches, known siblings and cross-row/set combinations would gain blanket refusals despite not necessarily depending on unrecorded data. That would narrow the user's full-feature target to an easier subset.

Preserve this candidate as a considered alternative. Revised design must route private history provenance through actual execution/read seams, preserve Kleene predicate and selection precedence, track nested known/unknown value and presence, and permit known overwrite/clear/sibling reads. Related/set effects require actual row provenance and sound history partition handling, not a blanket feature ban. Refuse only genuinely unresolved value-dependent obligations; do not use a guessed value as fact. Existing concrete native execution and public generation behavior remain required. Exact larger scope and staged acceptance are being assessed before independent review or production authorization.

## Expanded executor design under independent review

Root has read candidate 292-generated-history-executor-design.md, SHA256 ac79aaa8d0e6dc32ce41d6d7886ea9df8fc365d4425b67b7477c0ec26a090c8f. It supersedes the rejected narrow proposal as the candidate, not yet as a binding implementation design. The report contains source review only, zero new executions.

The proposal retains one actual executor control flow behind private concrete/history contexts. Path-level abstract values preserve known presence, absence, shape and siblings, with no sentinel Nodes. An additive defaulted FactSource presence observation permits shared Kleene evaluation. Actual reads/writes and validation use private history authority; copies and known overwrites preserve precision. Cross-row access merges interacting search partitions rather than refusing the commands. Unresolved alternatives prevent a false violation, while a fully proven explanation remains valid; view reachability retains incompleteness separately. Recorded outcomes cannot select the implementation branch or manufacture generated data.

Independent review is assigned before source edits. Proposed staged units A through D comprise one completion bundle: genuine early/late red, private value/executor integration, related/set/search/view integration, actual CLI and regression verification. Expected scope includes two primitive files and the native interpreter/history modules, materially larger than the rejected five-file shortcut. Any implementation must audit direct Store/Node bypasses and preserve ordinary native execution. No story completion or source release is claimed.

## Binding executor design and implementation authorization

Root adopts docs/design/generated-history-values.md SHA256a84c7962f5d6f78fc7b1b66b6d02fd8ffeba701aa0ce8c94ff9581d2b10233fc under standing full-backlog/all-features authorization. This adoption supersedes candidate-status prose in the immutable reviewed document. Independent pass2 report85b1d7f86a9652057e29c6eca0cb75753fd566f7c18a6dfcfd6679f9a325a654 approves with findings[] and zero reviewer executions. Both review blockers are fixed in the design; production remains unchanged.

Explicitly approve the additive defaulted FactSource observed_presence method in the two primitive files, private abstract history values/store/context integrated into the actual executor control flow, complete transfer/reader audit, proof-separated generated feasibility, shared interacting history partition and acknowledged-client barrier, and unresolved search/reach alternatives. Public native generation/Store/Step behavior remains concrete. Never use a witness or expected outcome as recorded generated authority. Do not implement the rejected blanket preflight-refusal design.

Scope permits narrow crate-private boundary exports in interpret.rs, reusable existing typed validation/fact access in input.rs and existing witness helper visibility in witness.rs only if required by the adopted mechanism. These are not permission to change compiler/source admission or witness semantics. Report the exact helper delta before freezing. Other production surfaces and new private modules are recorded in typed scope; all executable changes are Rust.

Staged A through D are one completion bundle: establish real early/late regression red; implement abstract facts and actual executor transfer/control flow; integrate related/set/search/view authority and admitted source controls; execute CLI0/1/2/3 and complete focused neighbors/review. The pass1 affects.moves fixture suggestion was inadmissible and is corrected by the preserved pass2 report; the revised when_related/affected-row source must compile before it counts as evidence. No partial source approval or issue closure at the first Model refusal.

Implementation is assigned to an isolated managed worker after its current bounded aggregate design review. Initial source/test work may proceed, but compilation waits for an explicitly free cache; nested response worker currently owns synthesis and293 owns servers. Maintain jobs1/debug0/incremental0, external TMPDIR and8GiB floor. Root alone owns AEP, independent source approval, integration and the held delivery branch. No extra PR, push or remote full gate.

## Admitted generated-history regression baseline

Implementation owner established genuine current-source regression evidence before production edits. Baseline report SHA256 bced862f4a5b3f81035a7630cee64536070d398e08b8a32cb279073d44e33dcf; final 14-test source SHA256 5a7e15ba24860a1f771d87f9e3c7c1c5754ef28f1451165a5233153847d91869. Source base remains 86b4a994481f4391f7f38cc2677cdc493ca20404 under the accepted a84c7962 design. Evidence is retained in private ess-292-history-probe-20261003/implementation.

The final admitted 13-case baseline exits 101: 6 pass, 7 product assertions fail, no source-admission failures. Passed controls include inert generated Timestamp, source-known Integer true/false outcomes, impossible Boolean/Enum/joint-Struct domains and Optional-empty generation. Failures include fabricated early/late Timestamp decisions, inert inhabited Boolean generation constrained to true, A-addressed related-B healthy/stale/other-client histories, and subjectless-set stale omission. A-addressed failures currently expose partitioning before view judgment; the stale test requires the specific StaleRead finding so any unrelated Violation is not accepted as success. The subjectless control wrongly returns Linearizable.

A separate actual native control then passes 1 test with 13 filtered: create exact A/B rows, execute the admitted A-addressed related-B Active guard, update A and affect B, and verify both original rows changed. Thus the cross-row fixture is not a prose-only or unsupported affects.moves example. The final 14-test baseline is supported by these two runs (7 controls pass, 7 product assertions fail), not a claimed single full 14-test run. Original intermediate fixture-admission failures remain retained and explicitly distinguished from product defects. The first initial log had 1 pass/3 failures, of which one was an invalid literal Timestamp sets fixture; that fixture was corrected without changing source admission.

Exact admitted baseline log c19d58ed726fc7a7cd410f782a1ac616e64126c97da933a070d62b9afa928915; native control log ad515eb014bb32f4f15eb8191b971894dafaaf4412513c5130b34bfc7730a539; original initial log 488eb89e47e65449cb01a13f4fe696083dd3e839a16e0a4ffa3e2dde9ef9a29d. Root inspected the report and initial terminal output; these are owner executions, not independent reruns.

Root directs continuation of the already accepted stages A-D implementation. Preserve the shared executor semantics and actual unknown/feasibility authority, no blanket-refusal shortcut. The owner keeps exclusive synthesis cache with one-job settings and the 12 GiB pre-start floor. Aggregate #361/#362 is a future scenario adapter dependency only; it grants no additional aggregate or binding production scope to this unit.

## Shared-state first-stage evidence

The private shared State/Row executor refactor reached a first green14-case regression matrix. Root inspected actual shared-state-first-stage.log: 14 passed,0failed,0ignored,0filtered, terminal0. Log SHA256 5bbf2afd06d8f99e81d570c8c102d6956bc7bf6f849ce04e05dff697802059a9. Prior abstract-first-stage run retained10passes/4failures; adding shared-state grouping and the cross-row client barrier cleared those four initial regressions. This includes the genuine early/late Timestamp cases, constrained Boolean feasibility controls, actual native cross-row control, and acknowledged-write versus other-client view ordering.

This is an in-progress source checkpoint, not a frozen candidate or full design completion. Precise abstract typed-value validation, invariant dependence, provenance paths, safe increments, uncertainty/search alternatives and the remaining stage A-D acceptance still require work and broader validation. No source has been transferred for this story, and no aggregate implementation is authorized by its private Context. The future aggregate adapter must also account for unknown generated creation identity without fabricating concrete Node keys; the history's actual recorded identity is not automatically that authority.

Resource starts now use an integer comparison against12884901888available bytes immediately before cargo, with one job/debug0/incremental0 and external TMPDIR. The worker reported one earlier visual GB/GiB threshold mistake and attempted to interrupt its exact process, but the22-second run had already completed; it made no termination claim. No other owner's work was paused or cleaned. Future source development continues independently while starts await the coordinated floor.

## Prepared matrix reconciliation and bounded build resumption

The owner reconciled its held source and cache without starting a compiler or cleaning anything. Retained pre-build-reconciliation.md SHA256 b96d565ba3afe99cbe5a8228cb4f65c1e76059a570d96657666ed5076f7c5063 is source inspection, not fresh execution. Base remains 86b4a994481f4391f7f38cc2677cdc493ca20404 with twelve tracked modified files and four new files, no staged source candidate, and diff-check passing. Source and synthesis-cache leases were renewed after expiring during the hold; the owner observed no cache users and changed no other owner's lease or process.

Last measured history matrix is search-stage.log: 24 passed, 0 failed, exit 0, SHA256 eba51122adf69df302bb7b8a9c17a8c2fc45eb370841ffbc81779efc42842de8. The preceding reader-transfer run had 23 passes and one source-admission fixture failure: an unset invariant field was required; it was corrected to Optional without suppressing admission. Current source differs from that last executed checkpoint. The prepared suite contains 43 test cases, correcting the earlier count of 42; the final related-creation-mappings case was added before this reconciliation and is unexecuted. Prepared CLI status controls and native neighbors also remain unexecuted.

The browser owner has now reported its final three authored test groups terminal, and root read all three final zero exit files. Root clears one bounded synthesis-cache lane for cargo test --locked --offline -p ess-conformance --test generated_history_values -- --nocapture. Browser compilation is held during this transfer; browser source/evidence and cache remain owned and retained. #292 uses only its exclusively leased existing synthesis target, one job, debug 0, incremental 0, RUSTC_WRAPPER unset and its external temporary directory. Before every start require at least 12884901888 free bytes and no actual competing cache user. Last browser-owner available observation is 18305445888 bytes. The first-run additional allowance is an estimated 1 GiB, not a measured maximum; do not automatically start colder CLI or neighbor compilations afterward.

Retain separate terminal logs and classify compilation errors, invalid test sources and actual product defects accurately. Fix only within the accepted staged A-D scope, preserving the actual abstract executor and no guessed generated authority. Return the first complete corrected matrix and measured disk growth, then hold for the next phase. This is not source freeze, review approval, integration, publication or aggregate-adapter authorization. All source delivery remains in the one held bundle; the release ancestry and publication-identity blockers remain outside this unit and cannot be bypassed.

## Corrected 43-case matrix and next bounded validation

The prepared full history matrix compiled and executed: first 40 passed and 3 failed, exit 101, solely at fixture admission; corrected run 43 passed, 0 failed, 0 ignored, 0 filtered, exit 0. Root read both actual logs/exits and rehashed first log 90f0847e330a5a94127beb57e4f19f2d24c4e5d7d728db7849e8e2fb8596ba1e and corrected log 3a99a44fa1939947dffe604e0bba1889455112672300caf706ab47560cf9c7f7. Report first-corrected-matrix-report.md SHA256 4e8b7189480fd807ad0384a53b7a06d0b1478c620cc83257e78c3045befb0bd7 and all evidence remain private under ess-292-history-probe-20261003/implementation.

Zero increment is correctly forbidden by source admission; its revised test preserves an explicit admission-negative control, uses admitted subject-copy for unchanged unknown data, and retains legal +1 unknown/overflow obligations. The other two malformed fixtures incorrectly gave indeterminate operations a return instant; they now omit outcome and returned_at as the actual history contract requires. These were test-authoring corrections, not measured production regressions. No production file changed during this bounded run. Corrected test-source hash ffc252b2f3fb4907c99d48a9d9d26874d32583b07b29e40fdb9ea37ea71a9e55; tracked production/CLI diff remains c4e27da7e20618f3dbdc51b491ecf6fa6b62356f8e057986f1b660253e5207cd.

Owner reports both compilers terminal, no synthesis-cache process, leases renewed and net retained target growth 983040 bytes; this is not a transient peak measurement. Post-run available bytes 18105962496; root's later filesystem observation 17763872768 includes unrelated activity. The 43-case result is a bounded checkpoint, not staged A-D completion. Actual CLI statuses, primitive delegation, native/history neighbors, remaining acceptance audit and strict lint/review remain required.

Next resource clearance is the two exact primitive presence-delegation unit tests, then the exact CLI generated_history_decisions_preserve_cli_statuses_zero_one_two_and_three test from the owner's reconciliation. All remain sequential in the existing exclusively leased synthesis cache, jobs 1, debug/incremental 0, external temporary storage and minimum 12884901888 free bytes before each start. The colder CLI start additionally requires 4 GiB estimated headroom above that floor, at least 17179869184 bytes; remeasure after primitive tests and hold/report if unavailable. Retain each terminal log/exit and actual CLI invocation statuses. No automatic native-neighbor/full-gate start, source freeze, source commit, publication or aggregate work is authorized by this phase.

## Compatible warm CLI cache handoff under disk pressure

The cold synthesis-cache CLI start remains held: available disk fell below its 17179869184-byte headroom requirement while unrelated workspace builds continued. Root did not alter those builds or delete their caches. The first exact primitive test passed; the second is being completed separately under the existing 12 GiB start floor.

Root instead inspected the existing browser-owned servers target for a compatible warm CLI build. It contains the actual previously compiled ess binary (190165744 bytes), full ess-cli dependencies and browser integration fingerprint. Browser owner confirmed its native toolchain rustc 1.98.1 / LLVM 22.1.8, cargo 1.98.1, default features, locked dependencies, jobs 1, dev/test debug 0, incremental 0 and the ambient native lld flag. Root independently compared fingerprints: both cache configurations use rustc identity 1625334936438085654, test profile 18419998579205588157, config 9396254390672932401, compile_kind 0, and rustflags [-C, link-arg=-fuse-ld=lld]. This is compatible native cache reuse; WASM remains in its separate browser-wasm subdirectory.

The browser owner explicitly released only codex-recover-caller-browser-cache-20261003 after checking no cache processes or open handles. Its source lease, all source and all evidence remain retained. Fresh root worktree inspection reports zero live leases for ess-backlog-servers-20261002 and no tracked/untracked source changes; its existing registry-head discrepancy and ignored files were observed, not repaired or cleaned. That observation is not cleanup authorization.

Root clears a temporary exclusive handoff of this warm native target to #292 for only the exact CLI status-matrix test, after acquiring its own cache lease and rechecking actual users. Its existing synthesis cache remains separate for native history tests. The cold-cache 4 GiB estimate does not apply to this already-built CLI dependency set; use an estimated additional 1 GiB warm rebuild allowance, requiring at least 13958643712 free bytes before the start, including the unchanged 12 GiB floor. Latest root observation was 14710378496 bytes. The allowance is an estimate, not a proven peak: record actual growth and stop/report a resource shortfall. No copying stale executables as results, no new cache, no source override or admission weakening.

Run from the actual #292 source with CARGO_TARGET_DIR pointing to the handed-off servers target, matching one-job/debug0/incremental0/default-feature/lld settings, locked offline dependencies and external temporary storage. Record the actual CLI exits and test result. Return this single phase terminal, then hold further builds for explicit cache handback. No source commit, publication, gate retry or delivery-ancestry remedy is part of this cache coordination.

## Native compiler cache separation after Cargo safety refusal

The first warm-cache attempt compiled no test and ran no CLI invocation: Cargo reused ess-primitives from another worktree and the actual #292 conformance compile failed because FactSource lacked observed_presence. The retained dep-info names ess-backlog-next-20261002 source paths, so touching the current source would not reliably repair freshness. The owner preserved the failure and made no production correction.

The proposed exact three-package Cargo clean dry-run then refused: "missing or invalid CACHEDIR.TAG" and "cleaning has been aborted to prevent accidental deletion of unrelated files". Root independently confirmed the tag is absent. The parent target directory mixes native compiler outputs with retained backlog-input/review-boundaries evidence and the separately owned browser-wasm cache. It must not be globally tagged disposable or cleaned. No removal candidates were emitted, and no deletion occurred. Retained dry-run report SHA256 63abdfff5152dbc1aa99b81d53bd6ba3b845344cd76cab112b60c00ce79f3cb0.

Root chooses explicit storage separation before retrying Cargo's dry-run. With all native cache users terminal and the temporary #292 cache lease retained, move only the existing compiler-owned target/debug directory by same-filesystem rename into a new target/native-build directory. The source and destination must be real directories, not symlinks, and destination debug must not already exist. No source, backlog-input, review-boundaries, browser-wasm, logs, frozen binaries or handoff files move. This creates no duplicate large cache and deletes no contents. The resulting native-build directory contains only reproducible native compiler artifacts, so it may carry the standard CACHEDIR.TAG; the mixed parent remains untagged. Record exact paths, ownership/user checks, directory sizes and rename outcome.

After separation, use CARGO_TARGET_DIR=target/native-build for this temporary CLI lane and future native cache handback. Re-run only Cargo's dry-run for ess-primitives, ess-conformance and ess-cli, retaining its full exact candidate inventory. No actual package clean or compiler restart is authorized until that fresh inventory is reviewed. This supplies Cargo with a correctly classified compiler-only cache rather than suppressing its refusal on the mixed evidence directory. The release's separate provenance refusal and publication-identity blocker remain unchanged; this operation has no delivery authority.

## Primitive results and return to original CLI cache

Both exact primitive controls passed with terminal exit 0, one test each: unobserved_presence_keeps_kleene_logic_and_current_time_wrapping and nested_binders_forward_unobserved_presence_and_observation. Root rehashed logs a5306cb34701c5b465ca2e586cfca66d4be5749491a8fff6b76b227864f04845 and 45cf96e9242f58840206c5fc2e810e1306f37a16a26b22ec08587e5aebe112e7 and read both zero exits. The owner observed zero retained synthesis-cache growth. These controls do not replace the pending actual CLI or neighboring suites.

The warm-cache attempt is retained as a build-environment failure, not a product red: no CLI invocation ran. It exposed a cross-worktree Cargo freshness mismatch, with dependency metadata referring to an older source tree. Private report cli-warm-first-attempt-report.md SHA256 049f2c468d2d3b612025ed312c10a9459124deb0c36c3c8c9e90ad72e2b8a5da preserves the exact attempt. Merely matching toolchain/profile flags did not prove source freshness.

Following the previously recorded safety refusal, the owner separated only the native compiler directory by same-filesystem rename into servers target/native-build/debug. It reports unchanged inode 32039155, device 66306 and allocated bytes 3328761856. The mixed evidence parent remains untagged; only the new compiler-only cache is tagged. The subsequent exact three-package dry-run succeeded, but Cargo's verbose path list covered only a subset of its reported removal count. No actual package clean or deletion was performed.

Root then independently observed 30805438464 available bytes, enough for the original cold synthesis-cache CLI guard. The reason for this shared-filesystem recovery is not attributed to this lane; neither root nor the worker deleted another owner's files. Root stopped further cleanup investigation and restored the original authorized CLI plan in the existing synthesis target, requiring at least 17179869184 bytes before start. That cache already contains the actual #292 primitives verified by the 43-case matrix and both exact units. The temporary servers-cache lease is to be released by its owner after confirming no users, with all source/evidence and the new native-build location retained for later explicit browser handback. The CLI result remains pending until actual execution; no completion, publication or ancestry remedy follows from this cache recovery.

## Actual CLI status matrix green

The exact generated_history_decisions_preserve_cli_statuses_zero_one_two_and_three integration test passed (1 passed, 0 failed, exit 0), executing eight actual child CLI invocations. For unknown stored values, inert creation returned 0, early and late outcomes returned 2 with check.model-undetermined, and budget 1 returned 3. For known stored values, inert creation and early returned 0, late returned 1, and budget 1 returned 3. This is eight CLI invocations inside one test, not eight independent Cargo tests. Root read the terminal log and independently verified its digest e823ee2a8aa1ca0251bb9b1b413b00b90b428e83d800c4cf505ac789f9d1ed86; retained cli-status-green-report.md digest 40c902bc79a96752b2680629f7006721a6ae2366411c03433add3cf243904d51.

The original synthesis cache compiled the current source. Retained allocation grew by 973,516,800 bytes; terminal free space was 27,990,667,264 bytes. No actual package clean was applied. The temporary servers lease ended after checking native cache users; its compiler-only path is now target/native-build, preserving the moved debug directory inode and bytes. Cache-separation report digest e82ad3a8cfcde0d78677f9e3a0be7bfebc91c3e47d0425ac43a8b36f24acb082. The prior warm-cache compile failure remains environment evidence, not product regression evidence.

The compiler lane passed to the browser worker for its two pending focused tests. This story continues source-only Stage B/C readiness audit while neighbors, strict lint, final independent review and integration remain outstanding. No source commit or publication occurred; release ancestry and publication-identity blockers remain open.

## Stage B C readiness findings and next accepted controls

The implementor's source-only readiness audit is retained at stage-bc-source-readiness-audit.md SHA256 8d202d5d4e9551b5f263eed527b3ff017096d4dc270fa1e836206ed77537ae05, with feasibility/design addendum SHA256 4b3162a622f34b1b839a92eb3c0f0497930e8c5b94ff7a6ddf1bd3514eee60f8. Root read both. No new execution occurred in that audit. Existing 43 history cases, two primitive controls and eight actual CLI invocations remain the measured baseline, not completion of all Stage B/C requirements.

Concrete source finding: repeated set-affect entries generating the same field on the same row within one operation/branch currently share Origin because location lacks generation occurrence. Repeated effects are source-admitted by an existing native declaration-order test. This is a provenance-identity mismatch with the accepted design, not yet an executed wrong-verdict claim. Next bounded implementation step is an actual source-admitted/private-state regression demonstrating unequal generation origins, followed by the smallest occurrence-path repair, with copy preserving origin and different rows/operations retaining their distinctions. Keep execution shared with the actual native executor; do not infer scalar equality from origin equality or add expected-outcome authority.

The same accepted scope permits missing history controls for original-reference reads after reassignment, missing related row versus absent member, lifecycle-ineligible set rows, multirow provenance, rollback after unresolved touched-row validity, namespace precedence, and generated retry/indeterminate alternatives. Author and run these in the existing scoped Rust test/module files; classify fixture-admission failures separately from product reds. The sole compiler lane returns to the history owner after terminal browser runs, using the original synthesis cache, jobs 1, debug/incremental disabled, external temporary directory, locked offline dependencies and a fresh 12,884,901,888-byte floor before each expensive start. Run focused tests first; no broader full gate or publication.

Required feasibility traversal lacks its own work/depth bound before witness generation. Admitted recursive Union/List/Map/Optional shapes do not alone demonstrate infinite recursion because this precheck stops at them; pure required cycles are source-refused. Retain this distinction. Source-admitted recursive/deep-chain controls and a concrete bounded-proof policy remain required before fixing that separate path. Budget exhaustion must never prove Empty.

Correct the binding design's zero-increment positive-control sentence: actual source admission forbids zero, as the executed regression asserts. Use admitted same-field copy for unchanged-value transfer, retain legal exact nonzero arithmetic, and resolve the bounded-safe unknown increment item against genuinely admitted source and nominal assignability. No compiler widening or invented supported case is authorized. Full freeze still requires remaining native neighbors, scoped strict lint/format authority and independent review.

## Generation occurrence regression red and green

Root independently read both terminal logs and inspected the source-admitted regression and minimal production diff. The actual shared executor regression generated_origins_distinguish_effect_occurrences_rows_and_operations_but_preserve_copies first failed: 0 passed, 1 failed, exit 101, with identical operation/branch/row/field Origin for the first and second generation. Red log SHA256 b12a698a46f06cc1a567a41e957a06cded2638896804697b057761f67b994d5a. The fixture is parsed, assembled and compiled from a modified existing set-effects source; native Generated::Given establishes actual initial rows before the private history executor runs.

The repair adds the stable set-effect plan occurrence index to the private generation location. Exact regression then passed: 1 passed, 0 failed, exit 0; green log SHA256 73956b66cdb5c49868605b2a2df9940f1a2217a45e357e05af911b5e132fbe4f. Assertions cover repeated same-row/field generation, distinct affected rows, distinct operations, source-copy origin preservation and unchanged input state. This is provenance identity evidence, not a claim that the old code inferred equal scalar contents or produced a measured wrong history verdict. Wider Stage B/C tests remain in progress; no final freeze or publication.

Separate independent source review identified a more specific feasibility concern: finite enumeration filters every concrete validator error out, potentially turning depth exhaustion or Unknown into a cached Empty domain. A candidate required Struct chain ending Boolean may fit enumeration's depth count but exceed setup validation's larger count. This is an unexecuted source deduction; actual source admission and red evidence are still required. A typed valid/invalid/unresolved validation result is being evaluated to avoid deriving proof from error strings. It is separate from the completed occurrence repair and must preserve productive recursive types and truthful bounded uncertainty.

## Bounded feasibility proof policy adopted for next regression phase

Root read and adopts the bounded implementation direction in the independent source-only feasibility-traversal-policy-review.md SHA256 6d1995df93d9e34177149b7965349a116359fdea5d7229b55c8c39a4f84c8c95, subject to actual source-admitted regression evidence. The policy is not itself executed proof. Existing machine-readable scope already includes input.rs, history/values.rs, generated_history_values.rs and witness.rs; a narrow optional bounded proof entry in witness.rs is within that accepted scope and does not authorize a general synthesis rewrite.

Use typed Valid/Invalid/Unresolved validation, preserving the existing native Result/String wrapper and its diagnostics. Proved False or invalid shape may exclude a candidate; Unknown, depth/work exhaustion or active recursion may not. Complete finite enumeration may prove Empty only when every candidate is conclusively invalid; any inconclusive candidate preserves uncertainty, and any fully validated candidate proves inhabitation without making its content history authority. Keep known contradictory-domain and optional-empty controls.

One private proof context owns active state, completed-proof cache, depth and work accounting. Cache only completed Inhabited/Empty proofs, never context-limited Unresolved. Use existing depth/candidate bounds and an explicit proposed 16384-unit proof search budget, returning honest Model uncertainty on exhaustion. Thread the budget through candidate construction and proof validation; do not claim a total work bound while calling an unmetered witness helper. Absence uncertainty cannot prove requiredness, and an unresolved inner type cannot force Optional absence. Productive recursion must try independent terminating alternatives; an active branch cannot suppress a later valid leaf.

After the separate provenance/transfer phase reaches its checkpoint, establish actual admission and red controls before remediation: the 17-required-Struct/Boolean candidate and 16-Struct boundary control, Optional candidate retaining unknown presence, productive Union with recursive variant first, Optional/List/Map recursive base, deep-chain bounded visits, and completed-cache order independence. If a proposed source is rejected, retain that result and correct the fixture without weakening source admission. Internal proof units must remain labeled internal; they cannot substitute for source-admitted behavior. Existing 43-case and CLI evidence must be rerun only where these production changes invalidate it. Native neighbors, strict lint, final review and one shared bundle integration remain required.

## Transfer matrix green and feasibility false-verdict red

The provenance/transfer checkpoint is frozen at stage-bc-transfer-checkpoint.md SHA256 ead1a9851b87e08c08849ae9d6951358afc85edfee5e2da55e83b0c4632e3340. Root read it and the prior private transfer logs. Three actual shared-executor private-state controls pass, including old-reference copy, missing row versus absent member and multirow rollback; the initial EmptyChange fixtures failed source admission and remain retained separately. The outcome-only history matrix now passes 46 tests (0 failed), adding lifecycle-ineligible unknown membership, shadowed stored caller namespace and retry/indeterminate history controls. Final log SHA256 e8e95b858dc4f947615421cf46bb1a78cd3073b3e36c17b35f3ba9d93c59bcc1, exit 0. Earlier lifecycle attempts failed source reachability/causation admission, not runtime behavior. The only production change in that checkpoint is the stable set-effect occurrence path; the zero-increment design correction is also retained.

The next separately authorized phase has now established an actual feasibility red on admitted source. feasibility-depth-red.log SHA256 4deae0a8710a7047c81edeccbd84f5f66e0e549d3849a7c17a5ed92d6aecbb2a records 3 passed, 2 failed, exit 101. The 16-Struct control has a valid generated inhabitant. The 17-Struct case incorrectly returns Violation where bounded validation cannot establish emptiness; its Optional form incorrectly returns Linearizable for absence after fabricating a definitely absent value. Root independently read both deciding outputs. These are measured checker defects, not source-admission failures. Typed bounded-proof remediation is now in progress; no green result is yet claimed for this phase.

The checkpoint retains exact source copies and hashes before feasibility changes. Native neighbor tests, updated CLI evidence where invalidated, strict lint, remaining acceptance audit and independent final review are still required. No final source freeze, integration, PR or publication follows from this checkpoint.

## Typed feasibility depth controls measured green

The same focused Struct filter that exposed both false verdicts now passes all five tests (0 failed, exit 0). Root independently read the terminal output and verified feasibility-typed-depth-first.log SHA256 0853bdeb07aecca89ecd63fb28799c642b9e6217c0ea363816aa2fe473bcb006. The 16-Struct inhabited control remains green; 17-Struct validation uncertainty no longer becomes a proven-empty violation, and Optional inner uncertainty no longer fabricates absence and a false linearizable verdict.

The first repair introduces typed Invalid versus Unresolved validation while retaining the native Result/String API. This is a bounded intermediate checkpoint, not completed proof-policy implementation: shared work/context accounting, productive recursion, completed-cache behavior and order-sensitive conjunction/member controls are still being implemented. Root source review also identified partial early-return paths and diagnostic context to preserve while finishing that work. Existing red logs and source checkpoints remain retained. Full matrix and native-neighbor verification must run after the relevant production changes settle; no final source freeze or review approval is claimed.

## Source-grounded safe increment obligation

Source-only language audit increment-language-disposition-review.md SHA256 eef828749d298be8269847a57b00dab6faffb7229d1da6ad73651e13cb40e87a identifies an explicit mechanical conversion from bounded Narrow Integer newtype (0..10) to unconstrained Wide Integer newtype. The shared generator planner classifies equal underlying representations as mechanical and the Rust emitter preserves the underlying value. SubjectField source admission consults declared conversions. Therefore a preserved smaller domain followed by Wide +1 is a credible legal safe-unknown positive control; a blanket claim that all nonzero unknown increments are impossible is incorrect.

No compilation or execution is claimed by this audit. Complete feasibility repairs first, then admit the supplied source and prove the copy and arithmetic over every source-domain member with bounded typed validation. Keep derived domain and provenance; never pick a witness, discard invalid alternatives, or turn a valid transition into observed value authority. Required controls include partial/overflow domains, undeclared and nonmechanical crossing rejection, repeated increments and later observation-sensitive branches. Existing story scope already includes the private history value/proof surfaces; any additional scope must be recorded before edits.

The audit separately suspects nested increment lookup reads an unrelated top-level same-name field. This is not legitimate evidence for safe transfer. An independent admission/execution probe is required before classifying that potential existing native defect. It must not silently broaden the current implementation or redefine source semantics.

## Bounded feasibility matrix and admitted widening source

The ongoing private proof-budget/context implementation passed the focused generated-history matrix: 52 passed, zero failed/ignored, exit0. Root independently read the terminal output and rehashed feasibility-bounded-matrix-first.log SHA256 65003eedd9064dee6ea2214bdb1d8ca5f719286f6acdca8db7e871ae26003a59. This includes the actual deep-Struct/Optional false-verdict regressions, productive recursive union in both label orders, recursive Optional/List/Map terminating bases, and a valid finite witness despite another unresolved candidate. Work/cache-order and False-dominant child-validation controls are still in progress; an unused-helper warning remains. This is not a complete phase, lint or package-check receipt.

Root independently ran the frozen CLI's source validation for the proposed mechanical widening source. The audit's first Copy/Advance definitions lacked observable events and were refused as empty_change; those original bytes and logs are retained. Adding a Changed event carrying only input counter_id to each update makes the source valid with exit0. Corrected widening-model.yaml SHA256 83b8f0de517be2245c7abc7097fc431285e82e0edad7579eb527e6badfb1471e; widening-validate.log 0a3b0f16e8c9baaf52b8b8b07e954f15a39aaf604237b23280a3ba445df7f60c in private ess-nested-increment-probe-20261003. This proves source admission only, not history transfer correctness. The owner has the fixture for the next phase after feasibility is complete.

The separate nested lookup issue is now reproduced and tracked as story:nested-increment-previous-location, draft, dependent on this story solely for shared-file sequencing. It must not become a positive control for bounded unknown arithmetic or expand current source work implicitly.

## Frozen feasibility checkpoint awaiting correction

The owner froze the feasibility implementation for independent review. Checkpoint report SHA256 7ca2b64b62594ea5d03465f21b34fe727b09350ab153111eaf097f3fbfe78cc4; 18-file source manifest ecb0f1bd5211a9c21c928aab2b0a6569024e5aca63e4d499ea29da5de3a5c745; source archive 5368f68bb3e06930e6f83885e313bcc3546c40116260b7939c9ebcb2bdcfd9c6; tracked patch 2ef073590506bf48ca6fc3f81796efe4290ab652c5eb7e1c0c49ba368d057d4a. Root independently verified all18 live source entries and read the complete report.

Exact frozen source passes corrected52-case history matrix, log SHA256 16746861d5b2164709bac8715e5996098e256d62aba1df4da45fcca6fe40102c, and four private proof tests, log 801d190307b281106b732f42366a8f59a28abb0af056a7a6583d2115ef1c5080. Both exits0, no warning shown. The report retains an introduced51/1 regression caused by overcharging maximum path lengths on the16-Struct control, corrected using actual traversed lengths without raising the16,384 limit. Earlier compile-only failures and the original actual false-empty/false-absence red are retained distinctly.

Independent source review subsequently found an exact-zero-budget List/Map validation defect: success on one child can exhaust the budget, break the loop and return Valid while later children are unchecked. The checkpoint is not accepted despite its52+4 green tests. The reviewer provided source-derived budget6 List and budget8 Map probes, not executed failures. The owner is to author regression controls, execute the red when the compiler lane returns, then repair and revalidate. Abstract transfer validation's early-Unknown precision gap, safe widening/derived increments, scoped neighbors, strict lint and final CLI revalidation remain outstanding.

The browser owner now holds the single ESS compiler lane for retained replay/creation identity validation; feasibility source stays frozen except for separately authorized regression-test authoring after review. Publication and integration holds remain unchanged.

## Complete concrete Narrow domain transfer control

Root executed a concrete semantic control with frozen CLI d397e76cd811915e6efb9bfd33e0a7c7483ce65d9518ff229de0cbcab56fd23b. Eleven independent authored scenarios explicitly arrange every known Narrow value0..10, copy through the declared mechanical Narrow-to-Wide conversion, increment Wide by1, and assert the unchanged sample plus resulting count1..11. Actual authoring reports11 scenarios, zero refusals, suite/34; native interpreted execution passes all11 with no failed/error/skipped/unsupported scenarios, exit0. Coverage remains unknown and conformance_status is inconclusive; this is a concrete execution control, not full coverage or an unknown-value history proof.

Private ess-nested-increment-probe-20261003: widening-observed-model.yaml SHA256 794f8748cc62319f7f4b96b65ee6f3f6909da123652f73c43bb1f40ae1912eb1; widening-suite.json 29ef57802223ab2ede47c2a132f6a3eb155fe2965e3b03be4e20dcee88464dbe; widening-run.json dbf19a2acc7430baff8747c084e9d472e5f9d4c6deb7b391ffd18b9153a5d143; widening-report.json 2fd3958f7868ec0e7c66679fabff5906c635bb45fd524d506ed9604f264f2268. All source/scenario/output/exit files are retained. No generated-target parity, abstract transfer correctness or observation authority is inferred from these known samples.

## Ordinary control report import limitation

The actual attempt to import widening-report.json with its exact widening-suite.json through AEP0.68.0 failed with exit1: `error: MissingField at $suite.coverage: required field absent`. The ordinary suite/34 carries no coverage field; the execution report truthfully declares unknown coverage and inconclusive conformance. No report or suite was rewritten to satisfy the importer, and no replacement asserted evidence record was used to evade this refusal. The11 known-value execution results and exact original hashes remain in the story's concrete-control section and private receipts. This import limitation does not establish a failing ESS execution or full conformance evidence.

## Terminal suffix correction handed to integrator

The independent exact-zero-budget suffix finding was reproduced against unchanged production:8 private tests,4 passed/4 failed, exit101. All four new List/Map cases failed because an unchecked suffix returned Ok; sufficient-budget Invalid and fully completed exact-budget Valid controls passed first. The narrow correction tracks the child index and returns Unresolved only when exhaustion leaves an unvisited suffix. Current source passes8 private tests and52 generated-history tests, exits0, no warnings. Root independently verified all18 current source hashes and read the terminal results. Independent correction review is still pending; no review_outcome fixed or full-unit acceptance is recorded.

Private implementation report feasibility-suffix-correction-report.md SHA256 c72da8918035e3c0a32387dd14d0cf64fc2cbee94b83d9d7f764ab9d0c041dfc; source manifest e9be947fbd92f95b37d0dab6914bfcb5cd25d5f54681db6e5f9dee8b6f9cbb16; complete source archive8c20d15d58b2dd5d460486030256f11b118772bb3e493a329cd148836e8abcfa; production delta51b6fa7a28bf428baf01f4380435c34d730316aa6d6da391d5d64d540d7aeb27. Red log24a726a4b1102906e69aca903e5d537e2c8de26c991e88dec653ec820cbe6435; green2543ae0cae545e3da153c090a6358dd7162e2f776e3770f30c2896a224a9e3e9; matrix031e1cdc739b60e1a1dfc3f2847f88202debb4aec60e5e6b3bfd6377e08daaae.

User directed handoff to session01a0fc77-da4d-7490-a045-41c09aecdb4e. All worker processes are terminal and both owned source/synthesis leases were explicitly released, exits0; separate handoff-lease-release-addendum.md SHA25635fb1497016ab22f978003e3dd4624da7c3a6e908f19855d4c34245edabb0d38 supersedes lease-held statements in frozen reports. Receiver must acquire own leases. No further semantic edits, builds, commits or publication occur in this worker. Abstract early-Unknown precision, widening/derived-domain increment proof and final scoped validation remain outstanding.
