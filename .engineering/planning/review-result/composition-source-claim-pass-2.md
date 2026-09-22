---
format: aep.planning-md/1
id: review-result:composition-source-claim-pass-2
kind: review-result
status: active
title: Composition source and claim admission, final pass
relations:
- reviews: task:consumer-accounting-composition
revision: 1
---
approve

The corrected S6 source-to-claim unit is admissible at its two stated consumer boundaries. The
correction resolves every finite finding from the first review, represents the 14 upstream reading
coordinates per profile as entrypoint gaps rather than composition support, and preserves the exact
five aggregate candidates. This remains a source-only `NON_AUTHORITY` result; it does not perform
aggregate adoption, reconciliation, baseline mutation, or native/global qualification.

## Reviewed unit and complete accounting

This is the second and final permitted whole-unit review of the same 1,372 rows. The reviewed
artifacts are:

- `correction-1/non-authority-composition-compile.json`, SHA-256
  `43280096202cc86c08eb1854ca94aae26f845340c668e51ea7a1f1d92cdf367f`.
- `correction-1/non-authority-composition-rust-byte-transport.json`, SHA-256
  `10b6d075da82fd140d7334874a8b8d88d14ee61dfc0585d79e9791d6e07d2c32`.
- Integration source, SHA-256
  `9cb7481f6eea68b7f7e409be85d304c4ae519b6f0086cabc57e0fb2172cb50f1`.
- Transport harness, SHA-256
  `97a400d5689fc18652bfd87a328266e840e21a061679485471b0bbffcd15a0d1`.
- Production `crates/specify/ess-composition/src/lib.rs`, SHA-256
  `db71d32fd00681c3047b1efd41759898ee4e3923ba3373b0580430a81c309afb`, unchanged.
- The 28-file fixture manifest, SHA-256
  `5302a19f708ee16854d7dde38af8db604e51f5005e92cd3aa1a19618e3e73c4e`.

Each corrected ledger has 621 A rows and 65 C rows, with unique continuous indexes. Their
model/set/index/current-shape/old-shape coordinates are identical across the two profiles and retain
the original 686-coordinate obligation set. Profile hashes and the immutable initial baseline
`3dd8dff59335c8a77c93c2734118566fd1b2d5165c0590d0aa9be397374a47de`
remain unchanged.

| Disposition | Compile | Rust byte transport | Combined |
| --- | ---: | ---: | ---: |
| Supported | 665 | 665 | 1,330 |
| UnsupportedAtThisEntrypoint | 16 | 16 | 32 |
| AggregateClosureCandidate | 5 | 5 | 10 |
| Total | 686 | 686 | 1,372 |

The 16 unsupported rows in each profile comprise the two preserved graph gaps and the 14 corrected
reading gaps. The graph gaps remain A0, `DependencyRelation/variant/HostedBy`, and C0,
`DependencyRelation`; neither identity is present at the `EssIr` composition entrypoint and no S5
evidence is borrowed.

The five aggregate candidates remain exactly C21, C28, C29, C55, and C56. They are the accepted
`RawOutcome` parent identities and continue to make no direct parent support claim.

## Outcomes for every first-review finding

### Finite accessor and reading arms

All 58 affected coordinates per profile now have an honest finite disposition. Forty-four are
Supported with relevant observations, and 14 are `UnsupportedAtThisEntrypoint`.

The accessor coordinates A60, A65, and A70-A72 now use an admitted tagged union. The fixture and
case inspect the compiled accessor plan, including the `Operation::Union` tag and variants and the
`Operation::Missing` node. Its independent emitted-artifact and executed-transport lane preserves
the distinction between compile, emission, and runtime evidence.

The admitted reading cases cover the finite reachable alternatives: `offset_date_time_text` and
`encoded_offset`; `unix_seconds` and `encoding_defined_epoch`;
`local_date_time_millis_literal_z` with `requires_observation`; and the unknown role and authority.
The compile case inspects encoding, origin count, role, and offset on the resolved named type. The
three added valid variants each have their own digest-emission observation and generated-client
runtime lane. Together with the retained base reading case, these observations support the 39
reachable reading coordinates from the original finding without group qualification.

The 14 finite gaps are exactly:

`A345-A350, C34-C35, C38-C43`.

These are the `reading` properties and parents on the struct, enum, and union `NamedType` schema
arms. `NamedType::check_shape` accepts a reading only on a directly String/Integer-backed newtype
whose primitive matches the encoding. The invalid fixture authors readings on all three other arms
and observes three exact validation errors during `Specification::assemble`; no `EssIr` or
`EssClientPlan` is produced. Both ledgers therefore leave `cases`, `control_cases`, and
`downstream_cases` empty, name the upstream owner, and record the refusal case only as the reason
the input cannot reach the reviewed entrypoint. They do not borrow that refusal as composition or
transport support.

### Descendant-family parent unions

The four changed parents now cite the complete finite union of their affected descendants:

| Coordinate | Complete compile case union | Complete transport case union |
| --- | --- | --- |
| C1 `ResolvedBinding` | periodic + selection | emission and runtime for periodic + selection |
| C6 `ResolvedMappingValue` | accessor + periodic + selection | emission and runtime for accessor + periodic + selection |
| C9 `BindingSpec` | periodic + selection | emission and runtime for periodic + selection |
| C11 `MappingSource` | accessor + periodic + selection | emission and runtime for accessor + periodic + selection |

No parent relies on a single descendant family after correction.

### Payload and sets

Payload coordinates A489 and C57-C59 now cite the response fixture, which actually changes the
`ListCreated.receipt` payload source from response ownership to generated ownership. Sets
coordinates A490 and C60-C61 cite a separately authored `sets.details: input.details` occurrence.
The sets case inspects the resolved outcome and exact target before asserting unchanged selected
closure; its transport profile separately observes digest-only emission and runs the six runtime
cases. None of these rows continues to cite the unrelated subject-state variant.

### Isolated binding and external names

A238-A240 now cite the binding-only overlay, where `poll-status` becomes `poll-status-v2` while
`jira:DEV-630` remains fixed. C25 and C32-C33 cite the external-reference-only overlay, where the
reference becomes `jira:DEV-901` while the binding name remains fixed. The compile case asserts
both separations directly. Each overlay also has its own emission and executed-transport lane, so
one changed value can no longer mask omission of the other.

### Executed transport controls

All 665 Supported transport rows now bind their control text to the retained C2 payload-drop and
C3 transport-error evidence: red exit 101 with 27 passed and 11 failed, restored exit 0 with 38
passed. The text names the runtime cases that fail. No Supported row describes those controls as
planned, and the 14 new gap rows make no transport-control claim.

## Boundary and evidence assessment

The compile boundary remains the existing `compile` through `resolved_service` path. It selects a
component's commands, owned-domain queries, events, errors, and recursively reached named types,
and carries `SourceDigest::of(ir)`. The correction adds fixture observations only. It does not add
response traversal, setting-type traversal, view-parameter traversal, or any new product semantic.
The coordinator's scoped response-only omission therefore remains intact.

The Rust byte-transport boundary still starts from the selected `EssClientPlan`.
`rust_artifacts` emits the plan JSON, manifest, and Rust library; the generated client locates the
emitted service descriptor, forwards request bytes unchanged, and returns response bytes or the
transport error. Corrected rows distinguish the model observation, emitted bytes, standalone
build, and executed runtime behavior rather than treating composition compilation as transport
evidence.

The retained correction evidence is internally complete and hash-consistent. Every ledger case,
control case, downstream case, and refusal case names a test present in the retained source or
transport harness. All 28 fixture manifest entries and all 54 generated-lane log manifest entries
match. The correction target records 57 passed cases; the package records 69 integration cases;
strict package Clippy and both formatting checks exit zero. Eighteen generated Rust lanes each
record a library build, harness build, and six passing runtime cases. The four-fault correction
control records 0 passed/4 failed at exit 101 and 4 passed/0 failed after restoration at exit 0.
The original C1/C2/C3 controls remain valid.

This final examination did not rerun Cargo or mutate source, production, planning, the baseline,
or either ledger. No source-to-claim gap remains within the stated S6 boundaries. Root retains the
separate decisions for adoption, reconciliation, aggregate handling, and eventual native/global
qualification.

```findings
[]
```
