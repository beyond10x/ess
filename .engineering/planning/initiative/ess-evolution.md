---
format: aep.planning-md/2
id: initiative:ess-evolution
kind: initiative
status: active
title: Evolve ESS without losing capabilities
relations:
- informed_by: epic:review-boundary-remediation
- informed_by: story:cli-presentation-binding
revision: 17
---
## Outcome

Complete the revised ESS evolution initiative: Eventlog-backed recorded execution, real AEP planning-store migration, ESS/Service SDK convergence, Connectors v2 adoption and infrastructure acceptance. Preserve ESS names, package identities, supported formats and implemented behavior.

## Authority

The operator approved plan ess-evolution-20260915 revision 1 on 2026-09-15 and authorized Astra orchestration with Sol implementors and independent reviewers. approval-record:ess-evolution-20260915 records the decision; design:ess-evolution-orchestration preserves its public-safe scope and source digest. This supersedes the broader 2026-09-10 completion requirements. No approval bypass or implementation completion is claimed.

## Scope boundary

The excluded application and fake-backend requirements are retired from current work. Historical artifacts and journals remain auditable. Generic protobuf, UI and Flutter work belongs to task:deferred-protocol-ui-bindings and does not block this initiative. Existing synthetic billing/gatepass fixtures remain required service acceptance.

## Sequence

1. Reconcile current scope, owner-local work and coordinated migration designs; preserve existing edits and one journal lineage per repository.
2. Qualify existing Eventlog file/SQLite/PostgreSQL storage and atomic groups, then complete asynchronous ER recorded execution and its Eventlog adapter.
3. Deliver exactly one owning AEP migration story, including Eventlog authority, Markdown projections, all legacy inputs and recovery.
4. Cut over the six real planning stores in order: Eventlog, Entity Runtime, Service SDK, ESS, Connectors v2, AEP last.
5. Extract ess-service-contract, add ess-entity-runtime and converge Service SDK decisions/replay on ER while preserving binding behavior.
6. Complete Connectors local metadata adoption, exact dependency/tool pins and independently observed infrastructure acceptance.

## Compatibility

Use aep.project/2 for changed authority/default semantics and opt-in service-runtime-ir/4 plus service-realization-plan/4 for ER execution. Preserve old-reader meanings and explicit legacy imports. Raise Eventlog-backed runtime minima to Rust 1.91 while retaining supported minima for independent pure libraries. Preserve kernel purity, exact provenance and typed format boundaries.

## Completion

All six real planning stores use verified Eventlog authority; ER-backed service fixtures and Connectors pass their acceptance; every required gate succeeds against the recorded source/dependency/configuration vector. Required PostgreSQL lanes must actually execute. Missing or skipped evidence is incomplete. Local acceptance is the boundary; publication, releases and deployment remain separate.

## Current evidence

The source baseline is ESS 41da2281e99402602c25d8faf219fc75b1954d04. The earlier preservation mapping and integration receipts remain in their owning records and journal. Existing Eventlog features are implemented, but qualification at the current source vector and later runtime/store migration remain pending.

The 2026-09-15 scope work selects the primary's complete planning snapshot as authority, including the original excluded drafts and their history. Git snapshot 1b44c6171aa2c20547d7fa4ebc9f1201221c2505 transported those exact bytes into the managed planning tree; the independently appended handoff records were recreated through AEP. Journals were not concatenated, and the primary was not changed. Detailed preservation evidence: local-evidence:ess-evolution/verification/scope-20260915/.

Owner-local decomposition, the coordinated ADR and runtime acceptance remain separately tracked work; this scope reconciliation does not satisfy them.

## Operator outcome-control amendment

The operator instructed the coordinator on 2026-09-15 to complete the unchanged approved outcome
with a concise requirement-to-evidence table, explicit critical path and bounded worker contracts.
Approved scope remains ess-evolution-20260915 revision1, SHA-256
7579145c3de5a1c6f8088fd7fb804d29dac8903ec505048f3ce595c45023b787.

The root completion index now distinguishes implementation, full verification and local integration
for governed scope, provider acceptance, ER persistence, migration commands, each of six actual
store cutovers, ESS service lowering, SDK services, Connectors and infrastructure acceptance, plus
preservation and complete final gates. Local evidence: ess-evolution/OUTCOME-CONTROL.md and the
root handoff requirement-to-evidence table. No local path is publication evidence.

Every new prerequisite must identify the approved requirement it blocks, cite why existing
evidence is insufficient, define an exact bounded deliverable/checks and a stopping condition.
Incidental improvements do not become completion requirements. When a worker's agreed deliverable
and checks are complete, close the assignment; subsequent work needs a separately justified one.
Review limits apply to the underlying issue, not the name of a new review unit.

The missing SQL second examination remains an external dependency; its recorded automatic review
rejection must not be retried through equivalent work. Missing actual migration writer control
blocks apply qualification and all six cutovers. The ESS unchanged-cell conflict is an engineering
repair under its existing task, followed by complete accounting rather than partial-matrix acceptance.
Independent implementation continues without renewed authorization. No acceptance gate is waived.

Progress reports use completed provider/runtime/command/cutover/service/Connectors outcomes.
A repeated open milestone must state its specific remaining cause and whether the next action
has changed. Test totals support outcome evidence and do not replace completion.

## Execution refinement 2026-09-15

Execution refinement under unchanged approved revision 1.
Root completion table, full nested roadmap and exact contracts are persisted at
local-evidence:ess-evolution/PLAN-REFINEMENT.md and the root handoff.

S1 residual and S2 loader implementation/native diagnostic contracts are complete; S2 remains
implemented revision 6. Diagnostic execution of 27 cases/2061 claims is not full qualification.
Normal consumer-check exited 1 after qualifying eight scenario acquisition rows; the required
reviewed-reconciliation.json and reviewed-aggregate-closures.json are absent. Complete their
evidence before repeating that gate; preserve the frozen baseline and existing exact receipts.

The S3 owner retains the full per-consumer attribution gap: reconcile each separate profile and
validator output versus helper observations before adopting source/claims. S4–S18 and the added
service profile retain their existing full behavior/aggregate obligations. No new proof microtasks.

In parallel, ER task:service-semantic-contract receives one bounded whole-contract correction
against its first independent review (seven blockers and two warnings); final substantive review
pass remains, with no budget reset. Complete required billing/gatepass semantics and compatibility,
then implement under the same M5 outcome. Neither extraction nor design closes service acceptance.

B-SQL still blocks provider/adapter/migration acceptance; do not retry equivalent denied review.
B-WRITER still blocks apply and all six cutovers. Pure semantics/accounting and IO-free migration
implementation are independent where inputs are available. Exact external inputs remain in root
handoff. No top-level milestone, store cutover, release or publication is claimed complete.

## Current completed outcomes and remaining critical path

Current reconciled outcome state, 2026-09-17. Approved plan ess-evolution-20260915/1 remains unchanged. SQL integrity8746a693 and consistent capture d016adb0 are verified/integrated. ER operation-field and selected-identity semantics7fd93ef4 are verified/integrated. Lowerer b5e980fe is reviewed and composed; complete consumer-enabled ESS/site acceptance remains. No full provider, migration, service or Connectors milestone is newly accepted;0/6 real cutovers.

The critical path is accepted provider vector (M1), qualified ER adapter/facades (M2), usable public migration commands (M3), then Eventlog, ER, SDK, ESS, Connectors and AEP store cutovers (M4). Administration43ceaa09 has author gates but its independent source examination was interrupted by automatic platform restriction; this is distinct from solved SQL/capture review. Preserve the interrupted evidence and original budget. No equivalent retry, alternative-model workaround or waiver is scheduled. Native provider fault acceptance also remains incomplete.

M2 source-bound legacy import correction and M3 selector/fence/physical-capture corrections are composed into their existing SQLite compatibility candidates, uncompiled. Eventlog offline lock/host metadata selects rusqlite0.40.2 and libsqlite3-sys0.38.2; ER/AEP locks need actual qualified upstream commits. Existing compatible formats, pure minima and runtime1.91 checks remain mandatory. Public WriterControl is unfinished; selecting its real adapter needs only the first Eventlog store's actual writers and existing stop/drain/restart-exclusion controller. Copies are not writers; observed process absence is not exclusion.

SDK's five introduced source defects are corrected, source manifest independently rehashed, and worker closed PARTIAL. Root owns final-source affected/generated/provider/full acceptance, original final review2 and integration. Connectors worker also closed PARTIAL; source now selects authored ER commands and four invalid fixtures are corrected, but generated definitions, coherent final pins/locks and complete actual CLI/provider/Secret Service acceptance remain. Original contracts and review budgets are unchanged. M8 still needs actual accepted service/process configurations and independently supplied observations; no projection is deployed or treated as observation.

M9 retains the frozen initial baseline. Eighteen groups are routing, not eighteen mandatory new engineering projects. Current reconciliation preserves frozen unknowns when exact model shape and consumer profile match, and refuses reconciliation of those unchanged tuples (consumer_coverage/reconciliation.rs:541–552). Changed/new required tuples and existing mandatory cases need their admitted finite evidence/decisions. Existing exact claims/tests can be reused where bindings, attribution, assertions and causal sensitivity cover the obligation; gate-required fresh native execution remains mandatory. The nested mechanism source is settled but both final native attempts exited130; no native run is active. Complete its existing acceptance/adoption and qualify already prepared evidence before scheduling unsupported new group work. Pending exact71 S7 tuples and five common-policy exceptions remain undecided; do not extrapolate the approved168 S3 pairs.

Root accepts storage responsibility under the operator's explicit correction. Fresh archive checksum and full tar comparison both succeeded; exactly32 closed historical native-evidence directories (6517735424 allocated bytes) were retired with their lossless archive and recovery command retained. No source/worktree was removed. At each subsequent accepted contract/integration checkpoint, retain manifests/logs, stop owned fixtures/processes, preserve the next lane's needed cache and clean exact verified disposable outputs. Publish wanted source before worktree finish, review gc --dry-run and apply exact eligible IDs only. Current inspected closed trees lack remote recovery proof or retain edits, so no forced removal or cleanup-driven publication.

Execution priority while provider review remains external: finish capacity recovery, execute the existing settled SDK correction contract on its retained cache, then the existing final native mechanism acceptance/adoption once its stable-start capacity condition is met. These are finite intermediate deliverables, not substitute product completion. No short whole product outcome is currently established closable without unresolved external provider review; the provider-independent M5 path still requires full M9/ESS gates. At most3 workers, initially one acceptance build; root alone owns integration and planning. No additional process review, evidence engine, adopter or speculative controller framework. Each actual new failure must name the original blocked requirement and bounded correction before additional work. Root roadmap ESS-EVOLUTION-ROADMAP.md records exact next-three outcome contracts and stopping evidence.

## Lowerer design first review correction and bounded target gaps

# Lowerer proposal correction after review 1

Approved M5/M6, existing initiative:ess-evolution. Review-result:entity-runtime-lowering-design-pass-1
is recorded unchanged. This is a correction of the original proposal, not an additional review.

F1: recorded the concrete PayInvoice nonpositive-input/unknown-identity counterexample as a durable
acceptance blocker and specified the bounded pure pre-load/continuation deliverable. The host
does not select outcomes. Current ER cannot provide it; no target implementation is claimed.
F2: optional host-owned entity/event/response presence now has one rule: exact per-invocation
host choice; current target returns OptionalBoundOutputUnsupported at its precise path. Required
billing/gatepass acceptance stays open. Named bounded target delivery includes compatibility,
all three output positions and value/absence/replay checks, not an optional enhancement.
F3: removed lossy duplicate public nouns by renaming them Inventory. Explicitly non-normative
diagnostic model; Rust interface remains normative. It cannot generate or serialize the API.

Email notification retains the existing SDK/provider external-effect binding. A separate stateless
kernel is not a prerequisite for invoice-service acceptance; actual effect behavior still needs tests.

Installed ESS CLI model validate and compile each exit0. Corrected design SHA
c937aa9f039016a77e9448d8860980d554512da8d2b1aece42a48ce247a1a528;
model contract SHA363a0b2ed3b5a89af5d2e9754cf67dbeddcff4198ef07d178b3a32aa2cfd392d;
system file unchanged a88342f8bcfb4a331949b0656ae4865f5c3524b9fd01c39c35f5f91406bd9564.

F1/F2 are fixed as proposal honesty/explicit blockers, not fixed target behavior. Required target
design/implementation/verification must precede complete lowerer admission, per approved plan §6.
One final lowerer design review remains; hold it until the target shape is concrete. Accepted ER
pure semantics and extraction review budgets remain closed. No third review or reset.

## Current lowerer final findings

# Final lowerer findings: correction remains inside original scope

Final whole design reviewd3621b48 needs revision. BOTH underlying passes are CLOSED;
there is no third lowerer design review. Immutable review-result:entity-runtime-lowering-design-pass-2
records the original report. Accepted extraction and ER source remain accepted within their
actual contracts; no runtime acceptance is inferred for the missing operation-field mapping.

The root statement that all required target gaps were resolved was too broad. ERda5d3687
implements pre-load refusal and conditional creation/event/response presence, but its explicit
operation-set refusal leaves the original optional-output finding partly unresolved. This is
the same requirement class, not an independent new product requirement or a new review unit.

Exact required counterexamples: billing.invoice.IssueInvoice.issued.issued_at and
gatepass.visit.AdmitVisitor.admitted.badge. ESS omission delegates ownership; it does not
specify unchanged. Full M5/M6 fixture acceptance stays open. The nonexistent RegistryRuntime
name must also become entity_core::Runtime.

Changed approach: one bounded author correction addresses the complete operation-field class
using actual ESS, ER and SDK source. It must first establish whether explicit typed fulfillment
through existing operation.set expresses the required behavior. If not, it must define the
minimal pure target amendment and exact version/compatibility consequence. It cannot choose
preserve implicitly, fabricate values for selection, move branch selection into the host, or
patch state after recording a different decision. Whole original semantic/fixture acceptance
and closed review budgets remain intact. Exact contract lowerer-final-correction-brief.md.

Owner: existing approved initiative M5/M6 and proposed story:entity-runtime-service-lowering.
Stopping condition: corrected complete implementable interface, actual model checks, exact
remaining source delivery and hashes. This is technical work root can resolve; no renewed
user authorization is needed and no external approval is requested.

Prepared implementation tree ess-evolution-er-lowerer-implementation-20260916 remains clean at
be604d87; no Rust implementation was dispatched. Root prep lease ended. Exact accepted ER object
da5d3687 was copied from the local repository into the existing Cargo Git cache; treec84addcc
verified. No remote publication or dependency override was performed.

## Final author correction delivered; target amendment still required

Both design reviews remain closed. The finite final correction answers the omitted-operation-field class with explicit typed Set/Preserve/optional Remove, immutable identity, post-load selected-outcome fulfillment, and durable action/removal replay. Existing ER da5d3687 cannot express removal through ordinary set; source-cited proof and rejected alternatives are in the correction receipt. Complete billing/gatepass acceptance is explicitly blocked pending the bounded pure target amendment, not claimed by omitted writes or diagnostic refusal.

Corrected design SHA25622d0eb4bb0c0847855b75e2458c3ce57d573c3d12e068ddcf360a31001a536ce; diagnostic model3f9bffda8526f31f890231a6467d967feae247a8b5024bee6031dbce5697fe6e; system a88342f8bcfb4a331949b0656ae4865f5c3524b9fd01c39c35f5f91406bd9564 unchanged. Installed ESS model validate/compile0;16,242bytes. Author assignment closed, no third design examination. Receipt: local-evidence:ess-evolution/waves/0009-service-convergence/er-lowering-design/final-correction-result.md.

Approved M5/M6 requires exact real fixture behavior. Remaining implementation is service/3 operation fulfillment plus record/request4 evidence with unchanged old readers/bytes, then complete lowerer and SDK/4 integration. Prepared lowerer tree remains clean and source is not dispatched until its required target contract is concrete. Existing accepted ER and SQL/provider reviews are not reopened.
