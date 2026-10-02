---
format: aep.planning-md/3
id: verification-report:consumer-interpreter-capability-audit
kind: verification-report
status: draft
title: Remaining interpreter capabilities beyond Billing and Oracle
relations:
- verifies: task:consumer-backlog-20261002
revision: 3
---
Read-only capability audit of native Interpreted at ff128cbdb

Own executions: 0. No builds, probes, source edits or AEP writes. Findings below are source-derived reachable paths, not measured new failures. Production interpreter was read with git show ff128cbdb because the carrier advanced during the audit. Fixture/target/IR citations were compared against ff128cbdb; the later changes were unrelated refusal tests and interpreted_obligations. Paths below are repository-relative. No claim that the green Billing/Oracle comparison proves general feature completeness.

Acceptance authority

Canonical coordinator story interpreted-command-execution is implemented, revision7. The other four stories, interpreted-bindings-and-unmet-obligations, interpreted-eventual-views, interpreted-scenario-supplied-facts and interpreted-trust-gate, are active. This corrects only the status transcription in the preserved original audit. Their named acceptance is the Billing/Oracle comparison plus discriminating faults and explicit supplied facts. The added complete-runtime authorization explicitly includes bindings/redelivery/invocation observation, ordinary responses, view ordering/paging/aggregation and supplied facts; it does not convert opaque external facts into source-derived facts. The separate the-interpreter-executes-stored-field-guards remains draft revision2 in the canonical files read here, with mixed_guard_wrong_state.rs/adversary_mixed_guard_pass1.rs acceptance. Root has separately authorized the discovered stored-guard correction. These facts should not be conflated into a claim that all remaining features have independent newly accepted designs.

Concrete admitted-feature gaps

1. Subject guards (already identified by root): interpret/execute.rs692–695 refuses SubjectField and SubjectPredicate even when the stored row and inputs are complete. state-in-subject-predicate.yaml and mixed_guard_wrong_state.rs are the existing routes. Include compound state+stored-field predicates and precedence before wrong_state. Do not merely special-case one state atom.

2. Related-row selection: execute.rs551–575 refuses an existing-instance sibling and stored-field lookup; 697–698 rejects every remaining Related condition. Only a missing input-named text identity with a declared Absent branch can escape through related_absent at578–595. A present related row is still refused, even when its fields are fully set. Existing tests/fixtures/related-guard-sign-in.yaml54–57 has absent and redirect_client!=input.client branches; tests/stored_map_entries.rs30–110 provides populated collection predicates through both related and subject rows. Group this with stored-guard evaluation, preserving related absence/input refusal/subject existence precedence and copied-field exclusion.

3. Absent-input commands: interpret.rs289–296 always returns Unsupported, and execute.rs700–701 refuses a command having InputAbsent even on a normal supplied-input invocation. Compiler fixture tests/fixtures/absent-input.yaml25 has the declaration; conformance tests/absent_input.rs270,309,326,338,348 pin omitted versus empty input and the unsupported fallback. Implement the explicit absent hook and avoid refusing unrelated selected branches merely because another outcome uses absence.

4. Supplied creation / existing-instance upsert: execute.rs703–704 refuses ExistingInstance;875–887 refuses Creates+Supplied. Compiler fixture upsert-by-existence.yaml and conformance singleton_identity.rs509,531 exercise a second installation and singleton identity. These are determined store/existence facts, not unavailable adapter capabilities. Supplied creation needs duplicate/existing precedence and unchanged-state observations, not generated replacement identities.

5. Retained replay: execute.rs707–708 refuses both retained origins and replay outcomes for the actual target. The Recorded checker allowance at665–671 explicitly does not provide target execution. tests/fixtures/retained-replay.yaml, retained-commit.yaml and retained_replay.rs181,298,424 provide actual-original-result, identity and fresh-state controls. This needs retained origin state/result ownership and actual request identity. Do not implement by rerunning an origin or minting another response; source-blind identity details may require an explicit seam decision. It is a real absent implementation, with a larger contract than a local match-arm fix.

6. Determined value sources: execute.rs1085–1100 supports only InputField, Literal and Generated. write at1055–1078 additionally supports Cleared and SubjectField only for sets. Consequently Increment, InputOrGenerated, Struct, RelatedField, CallerAttribute, ResponseField and ChangedCount fall through; SubjectField in emitted payload also falls through. declared_error at773–789 supports input/subject/literal but rejects the other admitted error sources. Existing routes: value_expressions.rs195–196 (integer/decimal increments), literal_fallback_in_struct.rs21–68 and165/220 (structured generated leaf plus explicit fallback), related_values.rs221/241/272 (stored related field), caller_values.rs20–90 (supplied caller attributes), fixtures/response-payload.yaml (actual response-fed event), fixtures/error-payload-sources.yaml. Group the generic typed value evaluator with explicit pre-outcome subject/related state and one actual response value. Response-fed payload cannot use an independently minted second value.

7. Caller facts already supplied are discarded: interpret.rs185–222 checks actor grants but passes only request.input into execute::execute; request.caller is unused. CallerAttribute therefore fails at the value-source fallback, and caller-aware when predicates have no facts. caller_values.rs CreateNote/EditNote is a concrete existing route. This differs from an unsupplied authenticated context: the caller payload can already be present in SemanticCommandRequest. Validate it against the declared actor contract and carry it through selection and effects.

8. Filtered/secondary set effects: ResolvedOutcome.instances and affects are declared at compiler ir.rs942/947 but never read by execute::take (849–935). With sets and no subject, filtered instances is refused at920–923; count:changed also falls through value(). More seriously, the existing set-effects.yaml Invite at115–133 updates its subject normally but silently drops affects on other team rows: no refusal protects this case. Existing set_effects.rs describes independent skip/filter/from/count/subject-exclusion mutants. Group instances, affects and changed-count implementation; preserve the original subject exclusion and evaluate secondary filters against the proper pre-effect snapshot. This silent omission is higher priority than adding another Unsupported branch.

9. Repeated external controls: Interpreted does not override target.rs238–250 configure_external_outcome_repeatedly; default Unsupported is reached before bounded-retry execution. A single forced slot consumed at interpret.rs206–213 is insufficient. bounded_retry.rs319,351,391,418 already pin exact N attempts/final refusal/unsupported behavior. Implement a remaining-invocation count with exact-command ownership/reset, retaining normal unscripted external alternatives and unmet-obligation/no-escalation distinction.

10. Explicit upstream entity setup: target.rs165 defaults Unsupported; Interpreted has no override. authored.rs99–100 emits EstablishEntity and its target implementation at320 shows the seam. The request supplies identity, fields and state; the model permits validation. This is a missing interpreter capability when these facts are complete. Implement typed validation/invariants and durable read visibility without invented creation events or commands, with no mutation on invalid setup.

11. Ordered scan: target.rs413 defaults Unsupported; Interpreted has no override. halt.rs96 exercises the actual callback and documents distinct true-stop/materialized-prefix/exhausted/unsupported verdicts. A truthful implementation must track its own row production and consumer termination; slicing views::query's fully materialized Vec and asserting halted would manufacture evidence. This needs an explicit iterator/production seam, not a count-only adapter.

Additional restricted paths requiring focused admission controls

- Text-only identity_key at execute.rs973–980 and related lookup571 reject nontext identity nodes. Entity identity types are resolved by the domain rather than constrained here to text. This is a typed-store limitation; no existing nontext-identity interpreter fixture was verified in this read-only pass. Before expanding identity support, add an admitted Integer/Boolean identity control and preserve typed equality rather than debug-string keys. views.rs64 also reconstructs identities as Node::Text.
- literal at1105–1126 refuses constrained newtypes because representation at1212 only unwraps unconstrained newtypes. A valid String-backed literal admitted through a constrained wrapper can therefore be refused; the compiler IR Literal contract at999–1010 says literals are already admitted through wrappers. Verify a valid constrained literal with a dedicated compiler+execution control. Do not infer that unsupported Bytes/Json/structured literal branches are gaps: domain binding literal admission at binding.rs1883 restricts literals to text/enum representations.
- NoValue at1171 and protected.rs61/81 is a bounded witness-search failure, not proof that an admitted type is empty. Keep the refusal honest, but test concrete finite/structured admitted shapes if such a model is reported; this audit found no additional measured witness counterexample.

Required missing-fact / invalid-input / resource refusals, not evidence of another feature implementation gap

- interpret.rs131: no model supplied;232–239: multiple actual steps still open. These must not be guessed. Unknown command, invalid request and broken invariant are mapped to unavailable/Error by136–140 and execute.rs266–271, unlike capability gaps.
- execute.rs679–680 current-time guard: mark/elapsed supply relative observation, not an absolute epoch. Preserve refusal until an actual absolute clock fact is bound. Early global refusal of a later unreachable now-guard deserves a separate precedence regression, but no new execution was performed here.
- execute.rs364/368 unresolved state subject,875's non-creation Observed subject,921 sets-without-subject: distinguish validated-IR impossibility from admitted instances above. Do not claim each defensive match is a missing feature.
- facts.rs40–45,53–88,146–164,191–208: inactive/mismatched scenario, unknown or internally published external event, wrong context authority, missing/wrong/unknown supplied fields, unmarked instant, invalid watched event or overflow. These are deliberately unsatisfied/invalid requests. facts.rs121 is a finite mark budget.
- facts.rs137–144 and bindings.rs191–192: periodic host facts absent. Interpreted inherits target.rs118/128/138 periodic callbacks and150 reading evidence callback. A host authority plus actual loop lifecycle/source/epoch/formatter evidence cannot be invented from a relative counter. These are explicit integration capabilities still unavailable; acceptance requiring a supplied host will need an adapter seam, not silently green synthetic causation. periodic.rs and clock_reading.rs178 provide existing behavior controls.
- target.rs106 fixture_values default: independent pre-execution provisioning is absent. pre_execution_fixtures.rs45 supplies actual principal ID/email and tests invalid/missing fields. This is not permission to mint a real external principal. Allow explicit fixture provider injection if accepted; keep unsupported without one.
- bindings.rs48/101/150 finite queue/invocation/retry budgets;117 missing correlation;119 undeclared outcome;197–207 invalid/absent typed input or unevaluable projection. Accessor/selection refusal may represent an opaque declared conversion, Binary64 or bounded observer limit, rather than omitted deterministic identity mapping. Escalation223 requires an actual declared event;227/231 require typed field witnesses. None should be turned into fabricated success or escalation on unmet obligations.
- views.rs26/40/43/56/71/79/96–116/214–269: undeclared/invalid params, unknown filter facts, invalid paging, missing required source/group/aggregate values, type mismatch or undefined aggregation. A declared but never-set stored field is not a license to mint one. Paging partial/negative/overflow cases should be invalid-input diagnostics, not described publicly as unsupported language features. No additional valid-complete-row refusal was demonstrated beyond typed identity above.
- protected.rs36/48–52/61/81/124–133/151/167: missing declaration, counter overflow, no bounded valid/fresh witness, capture budgets, invalid marked type/alphabet. Preserve bounded and value-free refusal. Ordinary typed response support is present; this audit does not relabel it missing.

Suggested bounded delivery order

First stored+related guards and explicit caller facts; then typed value expressions and set effects (including the silent affects omission); then absent input/supplied creation/repeated controls/entity setup. Retained-result ownership and truthful ordered scans need their own coherent implementation slices. Keep externally provisioned fixture/periodic/reading authority separate and explicit. Run concrete existing fixture scenarios against Interpreted with exact statuses and decisive mutants, in addition to preserving all33 Billing/34 Oracle scenarios and the16 fault controls. No new green evidence is claimed by this report.

## Coordinator actual execution probes

After the source audit, the coordinator ran the built carrier CLI against eight existing fixtures with `ess verify conform run --path <fixture> --target interpreted --report-format 2 --format json`. Source carrier9ce0e5930 includes the native implementationff128cbdb and subsequent test/tutorial/Timestamp corrections. Each stdout, stderr and exact command exit is retained as `ess-backlog-next-20261002/target/backlog-input/capability-<name>.json/.stderr/.exit`. These are eight actual CLI executions, not reviewer test executions.

| Fixture | Passed | Failed | Error | Unsupported | Exit |
|---|---:|---:|---:|---:|---:|
| compiler tests/fixtures/set-effects.yaml | 11 | 1 | 0 | 2 | 1 |
| compiler tests/fixtures/upsert-by-existence.yaml | 0 | 1 | 1 | 2 | 1 |
| compiler tests/fixtures/absent-input.yaml | 0 | 0 | 0 | 3 | 1 |
| conformance tests/fixtures/response-payload.yaml | 0 | 0 | 0 | 1 | 1 |
| conformance tests/fixtures/retained-replay.yaml | 0 | 0 | 0 | 2 | 1 |
| conformance tests/fixtures/error-payload-sources.yaml | 6 | 0 | 0 | 0 | 0 |
| conformance tests/fixtures/retained-commit.yaml | 19 | 0 | 0 | 10 | 1 |
| conformance tests/fixtures/related-guard-sign-in.yaml | 2 | 0 | 0 | 2 | 1 |

The set-effects silent omission is now measured: demo.desk.Invite/outcome/invited returns an admitted outcome but SessionDetails still shows on_hold=false for each secondary row where the synthesized check requires true. EndTeam and NoteTeam are Unsupported at sets-without-subject. The upsert fixture additionally exposes missing events and an unestablished captured identity; these actual failure/error results must be rechecked after supplied/existence execution, not normalized into successful Unsupported counts. The error-payload fixture passes all six scenarios, narrowing the audit's generic value-source concern: that source-derived concern is not a claim that this particular existing fixture fails.

These probes confirm implementation work remains. They do not cover every typed value source, scan control or externally supplied authority; the source inventory's other claims remain explicitly unexecuted until their dedicated regressions run.

## Entity identity follow-up

The entity-setup implementor now measured the formerly unverified identity limitation in a compiler-admitted Integer-identity model: the real Interpreted setup path returns a named text-store Unsupported result. This is remaining admitted-feature implementation work, not an invalid model or permission to stringify identities. Its source/test evidence will be pinned in the entity-setup unit report. Text-identity setup progress alone does not discharge the all-features release requirement. Preserve typed equality and view reconstruction when correcting the Store representation.
