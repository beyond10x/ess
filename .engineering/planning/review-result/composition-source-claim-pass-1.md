---
format: aep.planning-md/2
id: review-result:composition-source-claim-pass-1
kind: review-result
status: active
title: Composition source and claim admission, first pass
relations:
- reviews: task:consumer-accounting-composition
revision: 1
---
needs-revision

composition-compile — 58 Supported rows claim accessor or reading arms that no retained fixture authors or varies, so their direct rows and affected parents need arm-specific evidence or a non-Supported disposition — execution/non-authority-composition-compile.json:1638
composition-rust-byte-transport — The same 58 accessor/reading rows borrow generated-byte and runtime observations from inputs that never construct those arms, so they need arm-specific emission/runtime evidence or a non-Supported disposition — execution/non-authority-composition-rust-byte-transport.json:2110
composition-compile — Changed parents C1, C6, C9 and C11 cite one family although their changed descendants span two or three families, so each parent needs the complete finite union of its descendant-family cases — execution/non-authority-composition-compile.json:16800
composition-rust-byte-transport — Changed parents C1, C6, C9 and C11 cite one family although their changed descendants span two or three families, so each parent needs the complete finite union of emission and runtime cases for those families — execution/non-authority-composition-rust-byte-transport.json:21760
composition-compile — RawOutcome rows A489, A490 and C57-C61 cite a subject-state state swap that changes no payload or sets input; payload rows need the response case and sets rows need an authored sets observation or a non-Supported disposition — execution/non-authority-composition-compile.json:13221
composition-rust-byte-transport — RawOutcome rows A489, A490 and C57-C61 cite a subject-state state swap that changes no payload or sets input; payload rows need the response emission/runtime cases and sets rows need an authored sets observation or a non-Supported disposition — execution/non-authority-composition-rust-byte-transport.json:17125
composition-compile — Name rows A238-A240, C25, C32 and C33 use one variant that changes both the binding id and ExternalRef, so the digest movement cannot establish either coordinate independently and the inputs need to be isolated — execution/non-authority-composition-compile.json:6444
composition-rust-byte-transport — Name rows A238-A240, C25, C32 and C33 use one variant that changes both the binding id and ExternalRef, so the emitted digest movement and unchanged runtime cannot establish either coordinate independently and the inputs need to be isolated — execution/non-authority-composition-rust-byte-transport.json:8340
composition-rust-byte-transport — Every Supported row still calls the payload-drop and transport-error controls planned even though the candidate claims completed execution, so the current ledger must bind those statements to the retained C2/C3 red/restored evidence — execution/non-authority-composition-rust-byte-transport.json:72

## Reviewed unit and finite accounting

This review covers the original complete S6 unit at retained working tree
`home-path:sha256:e0f4900fa8e88669162147cee17d3ab076a134f683c4aaba895154accad7faec`, base
`f1af8280338b97d862a6c474ec50f78d5157d71c`, with production `lib.rs`
`db71d32fd00681c3047b1efd41759898ee4e3923ba3373b0580430a81c309afb`, integration source
`05f12e07a78467e39a065f735a5646312dae0560aa898c93d3cd61610edf98ae`, downstream harness
`97a400d5689fc18652bfd87a328266e840e21a061679485471b0bbffcd15a0d1`, and all 19 fixture hashes
matching `fixture-manifest.sha256`.

The reviewed candidates are exactly:

- `execution/non-authority-composition-compile.json`, SHA-256
  `de7a1949e652d604e8ca5cd7f7acd2ada200b330f5a3300ab9fd2dc6310dd004`.
- `execution/non-authority-composition-rust-byte-transport.json`, SHA-256
  `4a8ddbb74954d7769b0cc08c4d7d08118aeada03e8bc74f94f88590a54bd0bc9`.

All 1,372 rows were accounted for. Each ledger has 621 A rows and 65 C rows with unique,
continuous set indexes and exact model/set/index/current-shape/old-shape equality to obligations
source `8222c62c803881d9f6865dfa95850a0125adbd8ca14db9b1ff925647b8335763`.
The only family changes from that source are the three RawOutcome parents moved into the already
accepted aggregate set.

| Per profile | Rows | Review result |
| --- | ---: | --- |
| Supported candidates with no finding | 604 | Source-to-claim evidence is relevant at this boundary. This is not authority adoption or native/global qualification. |
| Supported candidates needing revision | 75 | Four finite defects below. |
| UnsupportedAtThisEntrypoint | 2 | Correct: A0 and C0 are graph-only `DependencyRelation` identities absent from `EssIr`; S5 evidence is not borrowed. |
| AggregateClosureCandidate | 5 | Correct: C21, C28, C29, C55 and C56 are the exact accepted five identities and make no direct parent claim. |
| Total | 686 | Complete. |

Across both profiles this is 1,208 Supported rows with no finding, 150 Supported rows needing
revision, four correct graph refusals, and ten correct aggregate candidates: 1,372 rows total.

## Finding 1: unexercised arms and their incomplete parents

The affected coordinates are identical in both profiles, 58 rows each:

`A60, A65, A70-A72, A167, A169-A171, A175-A176, A178, A182, A185,
A345-A350, A353-A354, A359-A370, A529-A530, A535-A542, A558-A559,
A568-A571, C34-C35, C38-C43`.

The accessor fixture changes `event.details.title` to `event.details.note`. The retained model has
no union declaration, so it cannot construct `Operation::Union` or the associated `Missing` branch;
the rows nevertheless assign those arms and their `Operation` parent to the accessor case.

The reading fixture has one newtype with `offset_date_time_text`, `encoded_offset`, and a role that
moves from `producer_process` to `consumer_process`. It never authors
`local_date_time_millis_literal_z`, `unix_seconds`, `encoding_defined_epoch`,
`requires_observation`, either `unknown` alternative, or reading on the struct, enum and union
NamedType arms. The source declares those finite alternatives, while `NamedType::check_shape`
additionally refuses a reading attached to a non-newtype. A moved whole-model digest from the one
newtype role change does not exercise the absent arms. The parent rows listed above therefore also
lack the complete arm union.

The bounded correction is to retain the existing cases for the coordinates they reach, add exact
valid or refusal observations for these finite arms where the original S6 boundary can receive
them, and change any unreachable coordinate's disposition rather than inferring support from the
family count. No new product behavior or traversal is required.

## Finding 2: changed parents omit descendant-family unions

The affected coordinates are `C1`, `C6`, `C9`, and `C11` in both profiles:

| Coordinate | Parent | Current cited family | Required descendant-family union |
| --- | --- | --- | --- |
| C1 | `rust:ess_compiler::ir::ResolvedBinding` | periodic | periodic + selection |
| C6 | `rust:ess_compiler::ir::ResolvedMappingValue` | selection | accessor + periodic + selection |
| C9 | `rust:ess_domain::binding::BindingSpec` | selection | periodic + selection |
| C11 | `rust:ess_domain::binding::MappingSource` | selection | accessor + periodic + selection |

For `composition-compile`, each row needs the listed union of closure cases. For
`composition-rust-byte-transport`, each row needs both the emission and executed transport case for
every family in that union. The cases already exist and passed; this correction is finite ledger
attribution, not another test run.

## Finding 3: payload and sets rows cite an irrelevant input

The affected coordinates are `A489`, `A490`, and `C57-C61` in both profiles. They are schema
positions under `RawOutcome.properties.payload` and `.sets`, but their `varied_input` is the swap of
the two `when_subject_state` values on `Touch`. Those outcomes contain neither `payload` nor `sets`.

`A489` and `C57-C59` are payload positions and the existing response variant actually changes a
payload source from `{response: receipt}` to `{generated: true}`; those rows can cite the response
cases. `A490` and `C60-C61` are sets positions, and no retained fixture authors `sets:` at all; they
need an actual sets occurrence and relevant observation or a non-Supported disposition. The five
RawOutcome aggregate identities remain unchanged by this correction.

## Finding 4: the names variant confounds two coordinates

The affected coordinates are `A238-A240`, `C25`, and `C32-C33` in both profiles. One fixture overlay
changes `poll-status` to `poll-status-v2` and `jira:DEV-630` to `jira:DEV-901` together. The cases
only establish that the combined edit moves the digest and leaves the selected closure/emitted
transport behavior unchanged. If either value stopped reaching the IR, the other change would keep
the case green. Separate binding-name-only and ExternalRef-only overlays, or equivalent decisive
controls, are required for the six row claims.

## Warning: executed controls are still described as planned

All Supported transport rows repeat a `control` value saying the payload-drop and transport-error
mutations are planned. They were executed: C2 and C3 exited 101 red and 0 restored, and their
generated-client lanes failed the named runtime cases. The candidate's execution status and
`execution/implementation-result.md` correctly report that evidence. Replace the stale row text
with the exact retained C2/C3 evidence; no rerun is needed.

## Boundary and retained evidence

The actual `composition-compile` boundary is `compile` through `resolved_service`: it selects one
component's commands, owned-domain queries, events, errors, and recursively reached named types,
while carrying `SourceDigest::of(ir)`. The response-only named type, setting-only type, and view
parameter omissions are the scoped current contract from `coordinator-boundary-disposition.md`;
this review does not request a new traversal.

The actual `composition-rust-byte-transport` boundary begins with the already selected
`EssClientPlan`. `rust_artifacts` emits `ess-client-plan.json`, `Cargo.toml`, and `src/lib.rs`; the
generated `Client::execute` resolves the emitted service descriptor, then forwards request bytes
and returns response or transport error. The retained evidence separates emission, standalone
build, and six-case runtime execution for all 11 lanes. Composition compilation alone is not used
as transport evidence.

The retained execution evidence is sufficient as execution evidence: 38 new and 12 existing tests
passed in the 50-case package, Clippy and formatting exited zero, every final generated-client lane
compiled and ran six cases, and controls C1/C2/C3 went red 101 and restored green 0. This review did
not rerun them and makes no production, baseline, native-authority, planning, or aggregate adoption
change. Root still owns aggregate/reconciliation adoption and later native/global qualification.

What could not be established: none beyond the 75 exact rows per profile identified above.

```findings
- file: execution/non-authority-composition-compile.json
  line: 1638
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 58 Supported rows claim accessor or reading arms that no retained fixture authors or varies, so their direct rows and affected parents need arm-specific evidence or a non-Supported disposition
- file: execution/non-authority-composition-rust-byte-transport.json
  line: 2110
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: The same 58 accessor/reading rows borrow generated-byte and runtime observations from inputs that never construct those arms, so they need arm-specific emission/runtime evidence or a non-Supported disposition
- file: execution/non-authority-composition-compile.json
  line: 16800
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Changed parents C1, C6, C9 and C11 cite one family although their changed descendants span two or three families, so each parent needs the complete finite union of its descendant-family cases
- file: execution/non-authority-composition-rust-byte-transport.json
  line: 21760
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Changed parents C1, C6, C9 and C11 cite one family although their changed descendants span two or three families, so each parent needs the complete finite union of emission and runtime cases for those families
- file: execution/non-authority-composition-compile.json
  line: 13221
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: RawOutcome rows A489, A490 and C57-C61 cite a subject-state state swap that changes no payload or sets input; payload rows need the response case and sets rows need an authored sets observation or a non-Supported disposition
- file: execution/non-authority-composition-rust-byte-transport.json
  line: 17125
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: RawOutcome rows A489, A490 and C57-C61 cite a subject-state state swap that changes no payload or sets input; payload rows need the response emission/runtime cases and sets rows need an authored sets observation or a non-Supported disposition
- file: execution/non-authority-composition-compile.json
  line: 6444
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Name rows A238-A240, C25, C32 and C33 use one variant that changes both the binding id and ExternalRef, so the digest movement cannot establish either coordinate independently and the inputs need to be isolated
- file: execution/non-authority-composition-rust-byte-transport.json
  line: 8340
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Name rows A238-A240, C25, C32 and C33 use one variant that changes both the binding id and ExternalRef, so the emitted digest movement and unchanged runtime cannot establish either coordinate independently and the inputs need to be isolated
- file: execution/non-authority-composition-rust-byte-transport.json
  line: 72
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: Every Supported row still calls the payload-drop and transport-error controls planned even though the candidate claims completed execution, so the current ledger must bind those statements to the retained C2/C3 red/restored evidence
```
