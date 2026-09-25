---
format: aep.planning-md/2
id: task:consumer-accounting-wire-behavior-period-parity
kind: task
status: active
title: Align Period schema and prove direct-file wire behavior
owner: ESS maintainers
relations:
- derived_from: story:consumer-accounting-baseline-never-extended
- serves: vision:O2
- depends_on: task:consumer-accounting-aggregate-reference-closure
revision: 22
---
## Outcome

Repair the Period schema to its existing canonical string contract and prove direct-file wire
behavior through the reviewed native binary execution path. This sequential implementation slice
belongs to story:consumer-accounting-baseline-never-extended and follows the implemented native
model-behavior task. It does not close the parent accounting matrix.

## Binding design and typed home

docs/design/consumer-wire-behavior-period-parity.md is the proposed contract. Existing Period,
RawSpecFile, model_behavior::Claim and reconciliation::Identity are the concrete typed homes;
no new product entity, format envelope or AEP dependency is introduced. Period's Serde String
boundary and canonical1..u32::MAX PT<seconds>S meaning already exist in periodic.rs:8-35 and
docs/design/periodic-binding-triggers.md:19-25. The fresh schema currently describes an integer.
Correct that derived projection, retain authored semantics and frozen historical baseline bytes,
and explicitly coordinate fresh current inventory and aggregate pins. Independent technical
design review must resolve the exact schema and causal evidence contract before source dispatch.

## Acceptance

- Preserve Period parsing, serialization and runtime representation; fix JsonSchema to exact
  canonical positive u32 seconds strings. Retain independent red-to-green schema/parser parity
  across decimal range-arm boundaries, adjacent values, invalid forms, types and newline controls.
- Extend the seven actual loader cases with exact fresh schema-pointer/structural assertions and
  path-specific valid controls, returned IR observations and stage-attributed refusal mutations.
  Prove each claimed container, branch and leaf; neither schema assertions nor unrelated failures
  alone qualify wire behavior. Production loading does not gain JSON Schema validation.
- Regenerate complete Rust/wire/profile/schema inventories; report the finite identity/shape delta.
  The initial58wire rows in six groups are a pre-fix frontier, not a cap. Refresh source bindings of
  all27 existing Rust claims affected by case-module edits without broadening their meaning.
- Produce exact proposed wire claims and four wire ShapeDelta closures bottom-up, preserving the
  original five aggregate identities, frozen witnesses, proof modes, profile and residual meaning.
  Root alone adopts authority, classifications, preservation and exact current-hash amendments.
  No new unknown eligibility, wildcard proof, prefix inference or partial reconciliation bypass.
- Retain real diagnostic native execution after root adoption and source freeze, exact current
  model/shape admission and keyed case receipts. Diagnostic cells remain unqualified. Complete
  affected package suites, strict Clippy, formatting and generated drift checks must pass; independent
  code examinations follow exact local submission. Parent default consumer-check, task check and
  site-build remain mandatory before integration; no skipped full gate counts as acceptance.

## Scope and serialized ownership

Cited: crates/specify/ess-domain/src/binding/periodic.rs, ess-domain/Cargo.toml,
crates/edge/ess-cli/Cargo.toml and src/load_accounting_tests.rs, schemas/generated/ess.schema.json,
crates/edge/ess-xtask/src/consumer_coverage/aggregate.rs and its authority/classification/preservation
inputs, docs/design/consumer-accounting-applicability.md and the binding design named above.
Inferred: focused Period parity tests and fresh generated inventory/evidence outputs. One source
implementor owns domain/CLI tests and generation proposals. Root serializes all shared Cargo/data,
design, AEP and exact aggregate-pin changes. No parallel writer touches these surfaces.

## Evidence and limits

Read local-evidence:ess-evolution/waves/0004-ess-accounting/wire-obligation-scope-result.md,
SHA256 cdee89bb41fcb3fd3aa8601340a6e35e15a9419dc969f62e2da1a3fcf193adbf,
for the58exact rows, source citations, six observation/mutation groups and required closure order.
The root design proposal fixes the concrete Period range schema before review. Proposed new hashes
cannot be known before actual provider regeneration. Native capture, ER/AEP migrations, all other
replacement/profile/service obligations and all full-goal gates remain outside this task and owed.
No source implementation, adopted wire claims, full qualification, integration or publication yet.

## Additional demonstrated string projection correction

Current ExternalRef parses and serializes provider:key strings in refs.rs:33-112, but the pinned
schemars_derive0.8.22 ignores its container schemars(with="String") attribute and emits a struct
object. The retained fresh native provider confirms the mismatch. The binding design now includes
an explicit JsonSchema correction to the intended string representation, preserving name,
description, manual Serde, lexical parser and canonical bytes. String-schema admission is not a
claim of lexical parser equivalence; invalid references remain parser/loader refusals.

Add canonical string/schema/Serde tests and object-representation refusals, plus ordered exact
reference observations and isolated lexical controls at every refs-bearing container site. Scope
includes crates/specify/ess-domain/src/refs.rs and focused tests. Regenerate all source inventories
after Period AND ExternalRef repairs; the earlier conditional64-row count describes Period only.
Compute the final finite direct/frontier/removal set and exact aggregate reference/pin consequences.
No frozen witness, eligible unknown, format or aggregate identity is changed. Root's source finding
is local-evidence:ess-evolution/waves/0004-ess-accounting/external-ref-schema-source-finding.md.
This amendment precedes implementation and will be included in the final technical design review.

## Complete container evidence contract

The binding companion docs/design/consumer-wire-container-completeness.md now defines the
complete finite member/default/null/branch/reference/control obligations for the seven existing
case identities. It is adopted from local-evidence:ess-evolution/waves/0004-ess-accounting/
wire-container-completeness-result.md, SHA256
9fef825501ebe93b782cc6a1ca8868ba030cffcf085665653124f8dde9ed03af, with explicit expansion of
abbreviated property pointers into full literal executable assertions and candidate ledger entries.
The immutable first design review is unchanged. This amendment answers its whole-container gap;
the second/final technical review must judge the concrete revised contract before implementation.

Acceptance requires every member of each claimed container, including ten RawComponentSpec
members, complete NamedType branches, complete binding/command/error surfaces and finite shared
reference targets. Observe actual resolved IR at each source site; defaults, nulls and named
single-edit mutations retain actual parse/assembly stage. A mutation must match exactly once.
No schema prefix or unrelated failing fixture qualifies a container. Prepare a machine-readable
candidate member/control/reference ledger keyed by exact current identity and existing case ID;
root refuses adoption when an obligation is absent. No new production evidence envelope is added.

Both Period and ExternalRef repairs precede fresh inventory/frontier generation and reference
closure. The 58-row pre-fix and 64-row Period-only counts remain conditional historical measures,
not admission caps. Keep exactly the existing five aggregate identities, frozen witnesses and
residual meaning. The complete parent, migrations and full gates remain open.

## Observed command-default correction during implementation

The implementor identified a contradiction in the reviewed Q-default fixture prescription.
Root verified command.rs:1169-1182 and existing regression
a_command_with_no_outcomes_is_refused at3104: a raw name-only command parses, but assembly
refuses no outcomes as EmptyDeclaration. The binding companion now preserves the raw/schema
default assertion and requires typed A refusal for omitted and explicit-empty outcomes. A separate
minimally valid one-outcome command observes omitted input/response/naming/refs in compiled IR.
This corrects the proposed fixture's stage, not runtime admission or whole-container coverage.
No parser/compiler relaxation, new aggregate, skipped assertion or adopted claim is authorized.

## Compiler-owned setting type refusal stage

The implementor observed entity-valued settings return Refused with empty problems and nonempty
compiler diagnostics. Root verified load.rs:69-78 and resolve.rs:979: the compiler invokes
ess_domain::component::validate_setting_types, whose entity and undeclared-type branches are
component.rs:1464-1494. The function's crate does not determine the loader stage.

The binding companion now defines K as exact compiler refusal, distinct from C compiled success,
and corrects the setting-type controls to K TypeMismatch/UndeclaredReference. Preserve the exact
problems/diagnostics partition, located code, valid control and single causal mutation. Other
raw setting contradictions retain their actual A behavior. No general accept-any-refusal helper,
compiler relaxation or coverage reduction is permitted. Further observed stage discrepancies
must retain source/runtime evidence and receive an explicit correction before DATA adoption.

## Schema helper adoption and observed site corrections

Root adopted only four new concrete Period/ExternalRef JsonSchema entry classifications as
OwnedHelper, retaining every previous classification and feature-preservation row. No behavior
support follows from those classifications. Root also refreshed the two changed current aggregate
pins from the fresh provider inventory, retaining all five admitted identities and frozen old
witnesses. Reviewed wire/model/closure claims and case bindings remain pending the complete ledger.

The companion now reflects two concrete loader observations without changing runtime admission.
Named struct fields with the same semantic source name but different effective wire keys compile;
the existing domain invariant refuses duplicate effective wire keys. N records both the exact
admitted pair and an isolated DuplicateDeclaration at the later colliding field. RawOutcome refs
null normalizes to an empty sequence through the actual Value-based YAML reader, so Q records that
exact compiled default while keeping malformed/non-string item refusals. Other sites retain their
independently established behavior. See docs/design/consumer-wire-container-completeness.md and
docs/design/unique-wire-field-identity.md. These are source-informed contract corrections, not
new support claims, new parser rules or relaxed tests against an unchanged asserted refusal.

The first all-seven green execution was intermediate: the completeness audit found further controls
needed in every lane. Source and case hashes remain unfrozen until all finite companion obligations
are implemented and checked. Fresh candidate DATA adoption precedes the normal native diagnostic;
full package/parent gates and independent code examinations remain required.

## Exact current schema authority corrections

After the two schema repairs, root refreshed exactly three RootDefinitionsContainer metadata
rows to the fresh definitions shape for cli-binding-resolution, rust-emission and
process-execution. Their decisions, guards and profile meanings remain unchanged. Root then
retired the current model-ingress-external-reference-object-members-refusal requirement, whose
eight object descendant nodes no longer exist in the repaired string schema, and its three
feature-preservation references. The original full requirement remains in retained evidence.

The model-ingress-external-reference-object-refusal requirement on the current root/type nodes
retains the same case and Refused meaning; only its explanation reflects the repaired schema.
Rust string-reference requirements, other feature mappings and the frozen baseline remain.
No new wire/model/closure claim or case hash has been adopted by these exact corrections.

The strengthened seven-case suite now passes. The implementor has produced a finite66-identity
ledger with313 named controls and durable reproduced schema-mutation failures followed by a
restored green run. Root review, complete candidate DATA adoption, source freeze, normal native
diagnostic, affected package checks, independent code examinations and parent gates remain.
Original non-durable red/green tool observations are separately identified; reproduced logs
are not represented as the original executions. Detailed exact authority hashes and retained
mutation evidence are in local-evidence:ess-evolution/waves/0004-ess-accounting/wire-behavior-evidence/.

## Coordinator adoption review requires completion

The strengthened seven-case run passed, but root's source review found remaining gaps against
the accepted completeness companion. Several enum/pattern nodes are only referenced, not
asserted with independent literal structural payloads. Object member-name/required-set checks
do not cover their full member/default/null/branch schemas. Examples include ReadingEncoding,
ReadingRole, OffsetAuthority, periodic enums, Delivery, name patterns and Period's exact pattern.

The accepted site controls also remain incomplete: full Naming triples, omission/empty/null
and member-kind/unknown-member controls at each parent, plus complete component ownership and
surface observations/refusals. The66-identity/313-control ledger validates finite membership;
it is not executable proof of every member. Root has withheld all proposed wire/model/closure
DATA adoption and source freeze. Existing successful runs remain accurately retained.

The implementor must complete every finite companion row, add independent literal expected
nodes and explicit member-to-observation/control mappings, and regenerate source/AST/claim
proposals after the correction. No external ledger oracle, schema self-comparison, weakened
runtime admission, reduced frontier or partial reconciliation is permitted. See
local-evidence:ess-evolution/waves/0004-ess-accounting/wire-behavior-evidence/coordinator-adoption-review.md.
This is the required root DATA adoption review, not another independent technical design round.

## Source-defined component surface discard behavior

While completing the remaining component controls, the implementor identified another proposed
oracle inconsistent with current conversion. Root verified component.rs:285-307 and675-690:
RawComponentSurface parses both QualifiedName arrays, but ComponentSpec takes only accepts.commands
and publishes.events. The cross-surface accepts.events and publishes.commands arrays are discarded
before validation; their target existence cannot cause an assembly lookup refusal.

The binding companion now requires explicit isolated compiled observations with unchanged exact
accepted/published sets for these discarded arrays, including syntactically valid undeclared names.
Malformed QualifiedName items still fail parsing; undeclared references at consumed owns.domains,
accepts.commands and publishes.events sites retain their actual lookup refusals. No resolution or
runtime effect is attributed to a discarded name, and no production behavior or schema is changed.
These controls remain to execute and must appear explicitly in the complete member ledger. This
corrects an asserted oracle using source evidence; it does not shrink the whole-container scope.

## Nested Naming null in the actual Value-based reader

The expanded controls exposed another incorrect proposed parse-refusal oracle. The retained
active11-focused-in-progress run observed compiled naming:null at N/E/P/Q/S; G first stopped on
a separate expected IR spelling mismatch. Root verified RawSpecFile::parse at spec.rs:151-153,
Naming defaults at name.rs:195-213, and pinned serde_yaml Value deserialization at value/de.rs:417-437:
Null supplies an empty map to struct deserialization. The binding companion now requires explicit
site-specific compiled default observations for nested Naming null, including exact parent summary
fallback where present. Each selected N/G/Q/E/P/S site must execute independently.

The follow-up focused run passed six cases but G stopped at the existing empty-component guard;
neither discovery run is full acceptance. Preserve those logs and correct the G control to retain
one valid active surface, as already required by the companion. Non-null wrong object kinds,
wrong member kinds and unknown Naming keys retain parse refusals. The schema remains the literal
object schema; no universal schema/parser admission equality or arbitrary null behavior is claimed.
No production parser/conversion change is introduced. Complete corrected controls, current
source/AST/ledger regeneration, DATA review, native diagnostic and full gates remain required.

## Payload-source refusal stage correction

The direct-file classification run retained at local-evidence:ess-evolution/waves/0004-ess-accounting/wire-behavior-evidence/active12-q-payload-classification-2.log observes malformed explicit payload objects as parse problems with empty diagnostics. Source confirms PayloadTable::deserialize calls PayloadSource::try_from and maps failures to serde::de::Error::custom (command.rs:804-807). Empty/dotted response, false generated, no admitted member, and conflicting non-null members therefore require P controls, not assembly A. The companion now records this exact stage; valid source forms still reach assembly target/type checks. No parser, runtime behavior or persisted format changes are implied. This correction does not qualify the wire claims or aggregate frontiers.

## Site-specific selection exclusion observation

The completion audit observes First2.excluding:null compiling to the exact empty exclusion list
at Selection.first.excluding. The C control S-first-excluding-null-current-acceptance is retained
in the active13-completeness-classification-2 execution material. Source selection.rs:63-76 stores
defaulted Vec<String> through the already identified RawSpecFile Value-to-Serde path. The companion
now requires this exact site observation; it makes no general Vec/null assertion. Non-null wrong
kinds and wrong element kinds remain parse refusals, and reference-order checks remain assembly.

The enum row also makes its observed behavior explicit: duplicate variant strings compile and
preserve exact duplicate IR; types.rs:472 checks nonempty variants. This resolves the prior
instruction to observe actual duplicate behavior, not a new parser or validator change. Empty
variants still refuse. Original active13 green evidence precedes these added audit controls and
is historical until the expanded suite runs. No DATA, source freeze or qualification is adopted.

## Source-verified union tag observation correction

The finite-row audit found that the completeness companion had inferred a tag-collision refusal
from a source comment. Actual ess-domain types.rs:488-505 only refuses an empty tag; the compiler's
resolve.rs:1256-1276 resolves variant types and copies the nonempty tag without testing variant
fields. Correct the union row to require C with a tag matching a referenced struct field and exact
returned tag/variant/field observations. Keep empty-tag/empty-variants and unresolved-type refusals.
This changes the mistaken evidence expectation, not the authored parser/compiler semantics.

The source inspection is not an execution receipt. The implementor must execute this exact control
with the remaining finite-row checks before a fresh evidence claim; earlier active13 seven-case
receipts remain historical. No DATA adoption, source freeze, native qualification or full gate is
claimed. This task remains dependent on the active aggregate-reference implementation.

## Executed null-vector stage corrections

The actual locked/offline seven-case run active15-focused-seven-cases-3 exited 0 with seven passed,
zero failed/ignored and 25 filtered. Original failed controls and corrected runs remain retained.
It independently observes refs:null compiling to exact empty refs at component G, periodic P,
command Q and event-binding S, alongside the earlier RawOutcome observation. The companion's
claimed component refusal was wrong. Keep malformed/non-string items and non-null wrong kinds
as separate P controls; do not generalize to unexecuted Serde sites or alter the literal array schema.

ReadingContract.origins:null becomes an empty vector and reaches assembly refusal ESS-TYPE-002 at
types.chronology.reading.EpochSeconds.reading. Missing origins and invalid items remain P controls.
The union tag matching a referenced struct field also passed with exact returned tag/variant/field
observations. Runtime semantics were preserved; only mistaken evidence expectations were corrected.

Retained source hash 8901c988a9399558d3759de760e19ee362215c346a46cf8112319d5e58ae8959;
green log ac4f1643b61fa12594f92904b1aa09a84ffaa79b9160854d127380ee0b80a416 under
local-evidence:ess-evolution/waves/0004-ess-accounting/wire-behavior-evidence/.
This is focused behavior evidence, not complete DATA adoption, native qualification or a full gate.

## Optional reference and outcome omission correction

The finite audit exposed two overbroad expectations in the wire companion. Required/item
QualifiedName null remains P, but Option-wrapped references accept null as absence and then follow
their own assembly rules. The active17 exact controls observed S event:null without periodic as A
missing_declaration / ESS-BINDING-005 at binding.choose.when, and SS creates:null with retained
instance as A missing_declaration / ESS-COMMAND-005 at
command.calls.core.Open.outcomes.opened.instance. The companion now states both explicitly and
retains their per-site controls. Q summary omission/null serializes no IR summary member; an exact
object comparison is required, not Value indexing that hides the absent-vs-null distinction.

Source load_accounting_tests.rs SHA
9a5b04cc846cea67743bbc0ced04ed9e53ee891894a1861be9eae5eb4b24ea88.
Raw active17-focused-seven-cases-4-red.log:6pass/1fail, the remaining failure a Q replacement-marker
cardinality, not the already executed S/SS null assertions. Corrected
active17-focused-seven-cases-5.log:7pass/0fail/0ignored/25filtered, exit0 reported by worker;
root readback6d669b confirms both runner summaries. This is focused evidence only, not full gate or
finite-ledger adoption. Whole affected checks and fresh source/evidence closure remain required.
No current claim DATA or frozen witness changed. No new technical-design round or fabricated approval.

## Finite wire candidate audit, first pass

The exact v3 candidate was independently audited against all72schema identities and all98member
rows in seven case tables. Structural nodes match, but the original verdict is needs-revision:
the index loses4K-stage uses, several required member controls are absent,26executed controls lack
member/index mappings, and SS-null does not assert the actual resulting condition/test strategy.
Original report is retained at local-evidence:ess-evolution/waves/0004-ess-accounting/
finite-wire-adoption-audit-pass-1.md, SHA
a72ba65184ddd662c6dadb17c38dee4739630ff61b478ffb281bd1850032d269.

Preserve both frozen v3 files and their source vector. Correction proceeds into a new v4 candidate,
including every C/P/A/K use and exact per-site executable observations. Report any source-observed
stage contradiction for an explicit companion amendment; do not force a P expectation on a reader
that actually accepts null. No candidate DATA is adopted. One final independent data-audit pass
follows corrections; it is not a third technical-design review or a substitute for code examination.
Current focused7pass and fullCLI566pass/0fail/5ignored are retained historical receipts once source
changes. Domain/Clippy/schema/extraction/native/full gates and complete accounting remain required.

## Selection-array null observations

The actual active18 focused run terminated101 with6passed/1failed/25filtered. The
selection_inputs:null control returned Compiled, contradicting the companion's P expectation.
The next selections:null control was not executed because that case stopped on the first panic.
The same pinned serde_yaml Value deserializer maps null to an empty sequence for both Vec fields;
the companion now requires site-specific C controls for P's otherwise-valid empty-selection fixture.
Both controls must assert exact unchanged IR and absence of a serialized selection plan. This is
an expected-stage correction, not a production change or evidence that the second control ran.
Non-null wrong-kind/item controls and dependent selection assembly checks remain mandatory.

Original source e9487ff1dc6b14dbb8672e17737c0d2972533bc53f03863be17344021b20769a and
active18-focused-seven-cases-initial.command/log/exit are preserved. Root read actual failure
and exit before amendment. Worker must execute both corrected controls and reconcile v4; the
frozen v3 ledger/index remain immutable. No DATA qualification or additional design approval.

## First finite audit corrections and current execution

All four findings from review-result:finite-wire-adoption-audit-pass-1 were acted on:
1. Index now includes all4K uses; C/P/A/K counts409/556/175/4.
2. All9 missing executable boundaries were added. Actual initial6/1 red observed a null-array
stage contradiction; the companion was corrected to site-specific C. Both null arrays then
asserted exact unchanged binding IR and absent selection plan in corrected7/0 execution.
3. All26 omitted member mappings were restored with exact source labels/stages.
4. SS-null now asserts resolved otherwise condition and default_branch test strategy.

Loader7b1544ccfc78263dad29f63915d728f098bf12ffa5c1aeb377121356044524cf.
Corrected focused log5d063cf02aa3c1639a49bba1e4451d748b1952a9bc475785097901c7c982f0cc,
actual7passed/0failed/0ignored/25filtered exit0. Root read every summary and exit.
V4 ledger85b01125a07614888737069166be4f4e09f1265eff400705fc13a3d0d578a650;
indexe209d673d40737cbf44b9ab214377596c1e213e52afe77a40fd3399cf9552feb.
Root checked all seven source manifest hashes. Frozen v3 remains unchanged.
The same independent auditor is resumed for second/final finite DATA audit. These fixed outcomes
record actions, not independent acceptance, source qualification or complete accounting.

After exact independently reviewed helper classification adoption, fresh consumer-extract
terminated0. Stage1 reports198112unaccepted cells,82profiles,2416models,5463case candidates,
192643pending owners and6metadata candidates; executed/qualified counts remain0. No behavior
claim, aggregate closure or reconciliation authority has been admitted by extraction alone.

## Exact current model and wire behavior authority

The final finite DATA audit approves v4, original report SHA
73218fed2dd0f5270602ec92d1a221a8eb45542e1272e7fe790d0db08d30ebf7.
All4 prior findings resolved, no carried/new findings. Original report recorded unchanged as
review-result:finite-wire-adoption-audit-pass-2; there is no third DATA audit or new design review.

Root adopted reviewed-model-behavior.json SHA
ed0c3c9d8157743b6be7b9993888987071c115caa419aea1c63b112dba387c35:
99exactclaims/7cases, adding66wire and6stable-target claims to the27 existing Rust claims.
Root verified all27 existing claim values are unchanged, not merely their identities. Only
their shared case metadata is refreshed to the exact compiled current source/AST/target hashes
and accepted complete assertions. The independent final audit confirms all72wire claim case
sets against the complete finite ledger; root confirms exact model/profile/entrypoint/source
bindings and case partition against successful fresh extraction-active19-classified.

Current loader source7b1544ccfc78263dad29f63915d728f098bf12ffa5c1aeb377121356044524cf;
the actual corrected focused runner passed7/0/0/25filtered. This source evidence supports the
candidate authority but does not substitute for fresh native execution/qualification at its
new source digest. Original27claim authority is retained separately. Frozen baseline unchanged.

The five-row aggregate candidate remains external and explicitly incomplete, with no invented
reconciliation digest. Complete reconciliation across all consumers, actual native receipts,
two independent source examinations, consumer-check, task check and site-build remain pending.
No whole-accounting acceptance, integration, release or ESS-evolution completion is claimed.

## Actual99claim native diagnostic

Root's recorded active21-native-behavior.command ran locked/offline Rust1.98.1 with the bounded
two-job profile and no wrappers. Session82558 terminated0; all source remained frozen until its
own source-freshness check completed. Actual99claims,7selected/7executed cases, each1passed,
0failed/0ignored. Result is DIAGNOSTIC_ONLY with0qualified cells, not complete accounting.

Authority file SHAed0c3c9d8157743b6be7b9993888987071c115caa419aea1c63b112dba387c35;
canonical authority digest6474bf7b1138be2cef5b289e46d5a7312ad6e729fbffb4d757beef764e1b6431.
Executed source80f8e0d352517ddf98c6f7fa944a3267fd56d0eb744ee7bf6568b105eda60fd9;
provider profile3be6064415fd45c6afc2cc9212d5bf06e7460b7e5670e8d7201e907e5d77c0cb.
Result bytesbde01922b362b7b4e84400b31fe6edf0f9013dfb938f7fdf715c3d7cb6cf30af;
execution bytes28a7c7bd6004d026a64db78ff3378b5b1bbddc1f634e552caa14ff12b1b6849b.
Small receipts copied to active21-native-result.json and active21-native-execution.json;
actual process logs and retained native images remain at target/wire-behavior/native-active21/.

Root independently read terminal status, all receipt pass/fail/ignored fields and raw summaries.
Subsequent design/AEP/source edits invalidate that global source digest for future qualification;
they do not erase the actual diagnostic evidence. Fresh final native execution remains required.
The next blocking mechanism for these6stable targets is unchanged-unknown/current-behavior
collision; its new subordinate task and proposed design preserve all current/full-matrix obligations.
