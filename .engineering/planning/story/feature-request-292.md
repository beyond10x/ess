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
revision: 10
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
