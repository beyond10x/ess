# Consumer coverage for model extensions

Status: accepted implementation policy for story:review-consumer-coverage, 2026-09-07.
The coordinator has accepted the finite Stage 1 eligibility checkpoint and selected Stage 2
enforcement under the recorded standing implementation approval. Completion remains pending.
The complete reviewed v3 policy below is unchanged; its historical remaining-selection language
is resolved only by the explicit current assignment at the end of this document.

Source authority: ESS a0cf3ca8681ce06f6fbdbc988d457b23f2136c04, after authored discovery.
The v3 candidate SHA256 is 63d6075781dc7f348fff838962c4392df3c8e9335eaf7b5d06f8a690f630812c.
The two original document reviews and narrow Draft 7 vocabulary verification remain recorded in
AEP. Current independent scoping read 44 inputs and verified 38 current Git blobs. The four
reservations remain ess-xtask, Taskfile.yml, inferred Cargo.lock and this internal binding.
Model and consumer owners remain read dependencies unless a concrete additional test is scoped.

## What the gate establishes

Every discovered model declaration/member/variant has an explicit disposition for every bound
consumer profile. A new member, changed shape, consumer or profile cannot inherit an old unknown
or a package-wide support claim. Supported/refused cells name behavioral cases that the gate
actually executes. The gate checks discovery, accounting, case identity and execution; reviewed
attribution and mutation establish that a case tests the claimed behavior.

Acceptance applies to the bound owned consumer families and entry points below. This is not a
whole Rust call graph or proof that no private helper reads a model. Internal evaluator changes
without declaration changes still require behavioral tests and ordinary review. Unknown remains
a visible pre-existing gap with an owner; it never means supported, refused or conformant.

## Model authority and identities

The source declaration roots are RawSpecFile, Specification, EssIr, EssIrParts, EssSemanticRef and
SemanticDependencyGraph. EssIrParts is separately included as compiler-private assembly parts,
which are not reached merely by traversing EssIr. This is AST visibility, not an external callable
API: its behavior is exercised through public compilation or existing compiler-local tests. Do not
widen its visibility or expose EssIr::from_parts. Root identity is an exact crate/module/type
path, not a prefix such as Resolved*. Derive the reachable type graph from the real declarations
under ess-domain, ess-compiler and ess-primitives, including private/optional members and nested
types. Runtime schemars::schema_for!(RawSpecFile) supplies a separate authored wire inventory;
do not read the committed generated schema as authority. Both inventories must participate in
accounting. String-backed TypeRef and Ranking retain their Rust semantic structure.

An obligation identity is a qualified owner plus declaration/member kind and name or tuple index;
enum variants and their fields are distinct. Line numbers and allocation handles are not IDs.
Record a separate canonical shape fingerprint for types, wrapper/container structure, variant
shape, visibility and representation attributes. Keep Rust identity and wire paths/names/tags
separate. Changes to serde/schemars rename, tag, flatten, skip, default or custom representation
cannot silently preserve an old obligation. Formatting/comments do not change identity.

Wire obligations have their own tagged namespace, separate from Rust obligations. A wire ID is
`wire:RawSpecFile#` plus the RFC6901 JSON pointer into the freshly generated root schema document.
The root, every schema node, every structural keyword/value and every properties/definitions entry are
accounted; each occurrence under flattened/alternative paths stays distinct. A custom schema name
is the actual definitions key, not a guessed Rust type name. Local $ref values are obligations themselves
and must resolve against that same root's definitions; traverse each definition once and retain
reference edges without infinite expansion. External or unresolved references fail. No total
wire-to-Rust mapping is presumed or required: both tagged namespaces enter exact cell accounting.

The wire shape fingerprint is canonical JSON with lexicographically ordered object keys, exact
scalar values and array order preserved, over the node's complete structural subtree. This is a
conservative identity profile, not JSON Schema semantic equivalence. Required membership, enum,
const, default, type, alternatives, bounds and representation/resource identifiers affect shape.
Exclude only description, title, examples and $comment at schema annotation positions; the same
names inside a const/default/enum literal remain data. Parent fingerprints also change when their
structural descendants change, making invalidation conservative and explicit. Adding a property
inside a manual JsonSchema implementation creates a new wire ID even if all Rust members stay
the same. A changed existing schema value changes its shape fingerprint.

Use an explicit closed schema-keyword/position grammar covering the locked schemars 0.8.22
Draft 7 output, including definitions and #/definitions/ references. A different schema profile
is a reviewed migration, not automatic $defs support. Distinguish schema objects, maps of property/definition names and arbitrary literal data;
do not misinterpret property names or literal object keys as keywords. Unknown schema keywords,
unrecognized shapes or reference forms fail extraction. Boolean schemas have concrete true/false
obligations. Freeze and test this canonical wire profile before accepting a baseline; changes to
the profile require reviewed migration, never automatic re-grandfathering.

Resolve actual local modules, inline modules, use/re-exports, crate-qualified paths and aliases.
Follow referenced local declarations recursively, recording visited nodes without cutting cycles
into omissions. A closed, explicit standard-container/external-leaf table handles Option, Box,
Vec, BTreeMap, BTreeSet, primitive scalars and other actually encountered external leaves. Unknown
types, owners, imports, include forms, attributes affecting shape and unresolved generics fail
extraction with owner/path diagnostics. A leaf exemption must state and test its meaning; it
cannot be an arbitrary "unresolved means opaque" rule.

The implementation must support the exact current handles! and semantic_refs! invocation
grammars, inventory the generated transparent identity wrappers and underlying types, and pin
the corresponding macro-definition AST fingerprints. A changed definition requires reviewed
reclassification. semantic_ref_from! is explicitly classified as conversion-implementation-only,
with a definition guard. Unknown item macros fail. Normal cfg(test) modules are excluded and
tested as such; production cfg/cfg_attr is evaluated against a declared finite package-feature
and target profile. An unhandled condition fails, never selects a convenient branch. Establish
the concrete profiles from the final source before implementation dispatch.

## Consumer authority

Use bounded owned entry-point families, each with an exact owner, callable/type entry identity
and execution/target profile. Inventory the current Cargo workspace and production module/public
entry-point boundary of the model-consuming packages. Every package/module/relevant public API
addition or target alternative needs explicit reviewed classification. Known helper and foreign
boundaries can be classified with a concrete reason; a new entry cannot inherit support from its
parent package. Source inventories and classifications remain visible alongside the matrix.

The minimum families are:

- Authored admission/validation and assembly; compiler typed resolution and its private parts
  assembly exercised through the actual public compiler boundary or compiler-local cases.
- Stable semantic references, dependency graph, semantic diff and impact.
- Each published ess_gen::generators() entry, checked against that actual runtime list; docs IR,
  Markdown, HTML, graph rendering, shared schema mapping and HTTP embedding are separate bounded
  intermediate consumers when not represented by that list.
- Neutral implementation planning and each actual ess_synth::Target alternative; emitted source,
  target refusal and executed generated behavior have separate claim profiles.
- Conformance generation, authored compilation, final merge, suite/input admission, Rust runner,
  generated Go runner and browser replay, refreshed after coverage writer integration.
- Composition compilation and generated byte transport; realization and runtime deployment.
- ModelTypes structural realization and model-backed normalization, including each declared
  target/result profile. External Bundle-only import/normalization remains a classified foreign
  input path, not an unmentioned exemption for the model-backed edge.

InfraIr and its infra packages remain a separate model domain. Shared primitives do not import
InfraIr into this gate. External AEP evidence consumers are outside the repository boundary.
Optional native TypeScript is a distinct profile with its actual fixed compiler/Node/V8 contract;
default workspace execution does not qualify it. Generic browser replay does not establish full
view evaluation or missing literal-assignment fidelity. Composition transport does not establish
typed payload validation. Every such limit must be reflected in its cells, not only this prose.

## Closed cell accounting and adoption

Implement closed Rust values for matrix cells, case identities, execution requirements and
results. Each cell has exactly one discovered model ID and shape fingerprint, one consumer ID
and profile fingerprint, and one disposition:

- Supported: an exact executable case and a concrete behavior or intentionally unchanged-effect
  assertion. A changed/control input must make a no-effect claim meaningful.
- Refused: an exact executable case asserting a named refusal or obligation and its boundary.
  A panic, arbitrary nonzero exit or unexamined diagnostic is not refusal evidence.
- BaselineUnknown: an exact pair/shape/profile in the separately accepted initial baseline, with
  an owner and existing follow-up artifact plus a precise unproven/broken behavior statement.

The initial baseline is a finite manifest pinned to the source commit and extraction profile
after coverage integration. The future unit has two explicit stages. First, implement and review
only the extractor, source/profile identity and unaccepted baseline output under this accepted
policy. Root then inspects its identities, counts, classifications and grouped unknown explanations
and freezes exact baseline eligibility in a recorded checkpoint. Only afterward may the unit's
enforcement stage admit BaselineUnknown. Neither bootstrap nor an ordinary gate may create
Supported/Refused evidence from enumeration. Tool-assisted enumeration is allowed once; checking the
gate never regenerates baseline eligibility from today's source. New fields, variants, changed
shapes, consumers or profiles require Supported/Refused execution for their new obligations.
An old BaselineUnknown does not transfer to a changed obligation. Reject duplicates, missing or
stale cells, unknown consumer/model IDs, contradictory dispositions and missing follow-up owners.
No future wildcard, prefix expansion, automatic NotApplicable or package-wide test alias closes
an obligation. Removal of a model/consumer produces stale cells to reconcile explicitly.

The baseline manifest may compactly group an explicit finite list of exact IDs under one shared
owner/explanation. The gate expands only the pinned list, never a live prefix. It must print total
discovered obligations and Supported/Refused/BaselineUnknown counts, including exact unaccounted
IDs on failure, without calling unknown coverage complete support.

## Actual behavior execution

The xtask gate itself executes the finite deduplicated linked cases after discovery/accounting
passes. A case names the Cargo package, test-target kind/name, full libtest name, exact features
and target profile, tool preconditions, semantic attribution and any nested runtime contract.
Do not consume arbitrary external green receipts or infer individual cases from a previous
aggregate workspace log. Build selected test targets using locked offline Cargo and identify
their actual executable artifacts. Preserve current source/fixture/manifest/lock/profile and
binary identities for the run; detect source changes during execution.

Stable libtest execution uses exact case selection. First list the selected binary's tests and
require one exact non-ignored named test. Execute that binary with --exact, one test thread and
the exact name; require its own exit zero, exactly one executed/passed case and zero failed,
ignored or filtered-to-zero success. Bind a tested parser for the installed stable libtest text;
an unfamiliar format fails rather than inventing structured output. Missing/renamed cases,
setup failure, signal termination, ignored cases and false result fixtures fail the gate.
Cargo compilation is not case execution. Preserve each subprocess exit and timing. The compiled
runtime schema provider, scanned declarations and behavioral binaries must describe the same
source/profile checkpoint. Passing a different source-root path to an already compiled xtask
does not rebind schemars or owner binaries. Include the provider's compiled dependency identity
in the receipt and refuse a mismatch before admitting matrix results.

If a linked case needs Go, browser or native TypeScript, its profile must establish actual tool
identity and that the nested runtime branch executed with its asserted scenario/count contract.
An outer Rust pass after an early return is insufficient. If an existing case cannot provide the
required current execution evidence, keep only an eligible baseline unknown or scope a permanent
owner test before assigning it as Supported/Refused. No incidental tool download or silent skip.
Optional profiles do not become generic default claims through a passing Rust wrapper.

Pure extractor/accounting/executor tests stay inside ess-xtask and use controlled fixtures and
a fake command runner for failure injection. Ordinary workspace tests never recursively launch
the full gate. Taskfile runs the real finite case lane separately; all existing check tasks remain
unchanged in meaning and order. The added lane does not replace workspace tests or targeted native
lanes. New package-local parser/serialization dependencies require measured locked resolution;
no root Cargo.toml change is implied by the current scope.

## Initial evidence and mutation requirements

The initial qualifying cells must include concrete F01 behaviors for relation aspects, component
reach, CLI exposure/grouping, outcome assignment/refusal, parameter naming/order/types, ranking,
reusable rows and residual owner relocation where the current exact cases establish them.
Use the scope report's case table as read-only candidates. For fields a case does not actually
assert, record the gap. F01 diff assertions do not qualify every producer. Include the current
real network-view and reusable-row OpenAPI output/invalidation cases, not just change-kind tests.
Refresh all exact test identities against the final integrated source before accepting cells.

Retain meaningful red and green proof for: a new optional field absent from inhabited Billing;
a changed string-backed semantic alternative; a new consumer/target; removed/stale/duplicate
cells; missing/renamed/ignored/zero-selected/failing cases; source/profile drift; unknown macro,
cfg, alias or schema shape; a swallowed panic or skipped nested runtime; and a falsely attributed
F01 behavior. Source-copy mutations use the same extraction path as production and identify the
owner/member/consumer causing failure. Distinguish discovery-only probes from semantic red/green
proof. A discovery-only AST copy may establish a missing-ID or changed-shape failure; it cannot
claim that schema or behavior used that changed model. For schema/behavior proof, compile the
schema provider and linked cases against the same complete isolated source copy, including its
fixtures and path dependencies, or use an explicitly compiled fixture model with all three
observations tied to that same fixture source. A fixture proves the extractor/executor mechanism,
not production consumer semantics. An uncompilable mutation is setup failure and is not counted
as the required matrix-accounting red. Record and compare all three source/profile identities.

At least one representative compile-valid production optional-field mutation must fail actual
accounting at the named new obligation; a string-backed fixture probe separately checks that
schema-only discovery cannot hide semantic structure. The concrete F01 attribution mutation must
change the reviewed production behavior with its meaningful control and run its actual attributed
owner case in the same copied source. Restore the real behavior and matching qualifying execution
for green; changing labels, adding unknown or running an old binary is insufficient. Fixture-only
result injection validates executor mechanics and is never presented as that causal F01 proof.

The measured Go system-level-type panic is a known broken baseline, never Refused. Its concrete
repair is being scoped separately by the fuzz story. Historical 33-command seed evidence may
inform attribution but cannot substitute for this gate's same-run results.

## Integration and remaining freeze decisions

Before selection: refresh scope after wave 11, freeze the model source and finite extraction
profiles, and accept this policy and its two-stage unit contract. During the first selected stage,
review the extractor, exact consumer identities, initial qualifying cases and baseline output;
root freezes the exact eligibility manifest before the enforcement stage can admit any unknowns.
This draft chooses bounded discovery, exact cells and same-run execution. It does not preapprove
an unknown baseline that has not been produced. Private filenames are implementation choices;
identity and fingerprint profiles must be fixed and reviewed before baseline output is accepted.
An unsupported current declaration shape is a concrete design decision to return, not permission
to silently skip it or enlarge scope.

The final source must pass package checks and every existing task check lane plus the added
consumer lane. Taskfile is validation workflow source, so task site-build also runs. There is no
public documentation change in this scope; Website/Atlas delivery applies only if actual public
source is later added through an explicit scope decision. No persisted ESS product format changes.

## Selected first stage, 2026-09-07

The coordinator accepts the preceding policy and selects the finite first-stage work order in
`docs/plan/2026-09-07-review-boundaries-18.md`. Stage 1 implements extraction, explicit consumer
classification, exact case candidates and unaccepted finite eligibility output. It stops at a
named source/output checkpoint for root inspection before any BaselineUnknown can qualify.
The later actual-case runner, same-source mutation proofs, independent source review and complete
repository gate remain required before the story can be implemented or published as complete.

The concrete initial model profile is production Rust for x86_64-unknown-linux-gnu, with the
current default workspace features. The three model packages declare no local feature table;
normal cfg(test) declarations are excluded through tested parser rules. Unknown production
conditions still refuse extraction. Rust 1.98.1 comes from the independently verified retained
bin/lib snapshot; all four Rust wrapper variables are explicitly empty, builds use two jobs,
locked offline Cargo and each managed unit's own default target. Full manifests, source/fixture
bytes, provider identity and effective build configuration are frozen before extraction.
Current consumers and their alternative profiles remain an explicit finite inventory, not a
claim that all alternatives executed in this first stage.

The current consumer refresh includes manifest/legacy/direct/omitted authored acquisition;
qualified release and observed-binding model callers; suite/5 and paired replay; browser
unknown-state presentation; realization v1/v2; namespace observation; cache and support checks;
and the actual release-action/build inputs. Committed suite run/select bypasses stay distinct.
InfraIR remains a separate classified domain. Browser display does not qualify full evaluation,
and release consistency checks do not establish execution or producer authentication. Required
native runtimes and optional TypeScript profiles remain separately attributed later obligations.

Root will inspect complete identities, counts, classifications, mandatory F01 exclusions and
owned explanations at the Stage 1 checkpoint. No accepted baseline is generated from the current
source by an ordinary gate. Any additional write owner is reserved before editing it. This
selection introduces no new ESS public wire format or product semantics.

## Stage 1 diagnostic macro classification

During Stage 1, actual production extraction exposed two additional diagnostic macros. The
coordinator explicitly classifies only `ess_compiler::resolve::codes::codes!` and
`ess_primitives::error::validation_codes!` as diagnostic surfaces. The former generates named
Code-valued constants and ALL; the latter generates ValidationCode and its ALL/as_str methods.
Both require fixed reviewed definition and invocation fingerprints, explicit generated-symbol
ownership and concrete consumer inventory entries. Their classification does not establish
nonreachability from the selected roots. If retained-model traversal reaches a generated
declaration, extraction must refuse with its declaration and macro identity until a supported
expansion grammar is separately implemented. They are not external opaque leaves. Changed,
additional, relocated or unknown macros and invocations still refuse. Guard-drift and a selected
root field reaching ValidationCode require meaningful probes. The final Stage 1 checkpoint must
record the concrete pins and actual graph result before any baseline eligibility is considered.

The selected canonical pins use syn 2.0.119 token-stream bytes, including multiline token
contents. Root independently recomputed these hashes from the complete preserved producer
output. They are fixed reviewed inputs, not values regenerated by an ordinary check.

| Exact diagnostic macro | Definition SHA256 | Invocation SHA256 |
|---|---|---|
| ess_compiler::resolve::codes::codes | 207896d6f5692c5ab10d28b8dc63c0b05045a387ccecee10430c65cfeb345a2f | e08bfceb0f997ae8e83ac7ad8596bd76eb16e1cbcaebf8c67c8e59799f21656b |
| ess_primitives::error::validation_codes | 71a0a5b3d5725632de53cc7ca460f20fa88e9d90ef7bd46e642631ed46e7b546 | 94f2a40c4d1a8a343afd1cf208f1166d5a9311096e9701d0f4d56719a0598299 |

Provisional production-discovery4 returned zero with 809 Rust obligations across 148 reached
declarations and no edge to either macro's generated declarations. Root checked closure of
the complete emitted graph and the 17 generated-symbol identities. This observation alone
does not prove parser correctness or accept baseline eligibility. The complete guarded
reachability probes, final profile and consumer classifications remain Stage 1 obligations.

## Stage 1 profile identity checkpoint decision

Consumer eligibility identity is the canonical declared bounded profile, its exact entrypoint
identities and declaration/signature shapes, and the fixed semantic execution requirements
(target, features, compiler and flags). Entry visibility, cfg alternatives and the variants or
fields of a bound type declaration participate. The complete source map, implementation bodies,
fixtures, Cargo configuration receipts and executable hashes remain separate execution-checkpoint
evidence. They must agree for the provider and cases executed in a run; they are not themselves
the consumer eligibility fingerprint.

This implements the preceding distinction between declaration changes and evaluator-body changes.
Formatting, comments, unrelated package edits and body-only edits cannot invalidate every existing
unknown cell. New concrete entries still require explicit classification, and changed model shapes,
entrypoint declaration shapes or execution profiles cannot inherit old eligibility. Body-only
changes still require meaningful behavior tests and ordinary review. Tests must establish both
stable identities for the former edits and invalidation or refusal for the latter changes.

The exact initial extraction uses the retained Rust 1.98.1 environment recorded above. Its private
Cargo-home path and storage settings are execution evidence, not requirements silently imposed on
all other xtask commands. Build-time source stamping must support Cargo's normal home convention.
The later consumer Taskfile lane must explicitly establish its admitted build and tool profile.

## Accepted initial eligibility and selected enforcement stage

Root accepted the Stage 1 source/output checkpoint at local source commit
7a6d76855e28d280138d2d439eb6f1e7c15a58d9 after complete source, inventory, attribution,
package-result and native-evidence readback. The independently produced final inventories
agree on 1,806 model obligations, 87 behavioral profiles and 157,122 exact pairs.

Only the finite 157,068 nonmandatory pairs in
crates/edge/ess-xtask/src/consumer_coverage/initial-baseline.json are initially eligible
for BaselineUnknown. The accepted file SHA256 is
e005a2e74e067ad51315e594151643b702381dc174bb9af5c355c3eeeb17ad51. Its exact model
shape dictionary and finite per-profile lists are fixed data; ordinary checks cannot add
entries, expand a prefix or transfer eligibility to a changed shape/profile. Concrete
package owners and their unproven boundaries are recorded in each group. The independent
follow-up epic:qualify-initial-consumer-baseline remains draft and unimplemented. Its
qualification work is outside the original 31 remediation stories; the current gate
implementation cannot satisfy or own its own unknown follow-up.

All 54 mandatory F01 pairs are excluded from eligibility. Seventeen existing source cases
provide candidates for 49 pairs; the additional reserved ess-diff integration-test owner
supplies five isolated relation kind/target/carrier and parameter order/type witnesses.
No candidate becomes Supported or Refused until the gate builds and executes its exact
nonignored case under the bound same-source profile. The known Go system-level-type panic
remains broken, with its concrete repair separately owned by the existing fuzz story.

Stage 2 now implements enforcement, actual exact-case execution and the required same-source
production mutations. Source review by the independent adversary and all repository check
lanes plus site-build still precede implementation completion and publication.

## Enforcement workflow profile

`task consumer-check` declares the reviewed Rust 1.98.1 profile, two build jobs, offline Cargo,
empty Rust wrappers, disabled incremental compilation and debug builds without debug information.
The existing CI Rust setup step provisions 1.98.1 before the offline repository gate. Local
callers provision the same version beforehand; the lane selects it through `RUSTUP_TOOLCHAIN`
without choosing a private home, temporary directory or target directory. A directly selected
compiler still must satisfy the checker’s measured identity and source/profile checks.

The separately named lane follows `support-check`; every existing lane retains its relative
order. Each invocation creates a fresh retained result directory under `target/consumer-coverage`
and prints that path. The explicit `--output` option also requires a fresh directory. Repeated
checks preserve earlier results. A future toolchain or target change requires a reviewed profile
migration; a moving CI default cannot silently transfer the initial unknown eligibility.
