---
format: aep.planning-md/1
id: story:a-skipped-scenario-is-not-a-failed-one
kind: story
status: implemented
title: A conformance report counts skipped scenarios as failed
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-conformance-format-design
- depends_on: story:review-report-reader-validation
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: inferred
  path: website/docs
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 23
---
## What is wrong

`ess-conformance-report/1` carries `scenarios_failed`, and the Go runner counts **skipped**
scenarios in it.

`crates/ess-conformance/src/go/runtime.go:465-476`:

```go
failed := make([]string, 0)
for _, result := range results {
    switch result.status {
    case statusPassed:
        continue
    case statusFailed:
        anyFailed = true
    case statusSkipped:
        anySkipped = true
    }
    failed = append(failed, result.status+" "+result.id)
}
```

Both non-passing statuses land in `failed`, and `ScenariosFailed: len(failed)` publishes the count
under a name that says one of them.

Measured on `sbf/acd`, 2026-09-03, ess 0.11.0:

```json
{"status": "inconclusive", "scenarios_total": 24, "scenarios_failed": 9}
```

Nothing failed. Nine scenarios were skipped, all of them `acd.routing.DispatchCall`, which that
implementation exposes no surface for.

## Why it is worth fixing rather than documenting

`failed_scenarios` — the **list** — is honest: every entry is prefixed with its own status, so a
reader who opens it sees `skipped …`. The count is what consumers read, and a consumer cannot
recover the split from it.

AEP's `ess-conformance` principle
(`principles/verification/ess-conformance.yaml:28`) asserts:

```yaml
predicates:
  - ess_conformance.passed
  - ess_conformance.scenarios.failed == 0
```

The second reads this number. As it stands, a suite with any skip can never satisfy it — which may
even be the wanted rule, but it is being enforced by the wrong fact. `ess_conformance.passed` is
already false for an `inconclusive` run and already carries that claim; `scenarios.failed` is
supposed to carry a different one and does not.

Recorded downstream in `sbf/acd` at
`.engineering/planning/executable-system-specification/acd-v3.md`, where the consequence is that
the artifact cannot honestly be moved to `conforming` and the reason is two-thirds wording.

## Historical requested checks

- `scenarios_failed` counts scenarios whose status is `failed`.
- The report carries `scenarios_skipped` beside it, so the split is readable without parsing
  `failed_scenarios`.
- `status` is unchanged: `passed`, `failed`, `inconclusive` already distinguish the three cases and
  that part was right.
- A test that a run with a skip and no failure reports `scenarios_failed: 0`.
- The Rust runner is checked for the same defect rather than assumed clean.

## Scope

Derived 2026-09-06 by independent story-scoper at ESS 1667d022ed2041342c8928250ab8bddfe9c988b9 and story revision 11, refreshed against accepted docs/design/review-conformance-coverage.md, Atlas ADR0039 and published AEP 30aeef2c9985613f5283764bdff3eec12160e43d — cited. Coordinator condensed the returned scope; inspection ran no tests or adoption checks — cited.

- **Primary surface:** crates/verify/ess-conformance, including evidence.rs, report.rs, scenario.rs, runner.rs, go/mod.rs, go/runtime.go, lib.rs and package tests — cited. Add explicit report/2 and separate detailed run/2 for admitted original suite/1–4 only, coverage exactly unknown; preserve all old defaults and bytes. Suite5, inventory, filtering and complete qualification follow in review-conformance-coverage.
- **Wire admission:** evidence.rs:12–169 owns the frozen v1 reader — cited. Add a separate closed report2 with bound producer profile, exact original-byte SuiteReference, six exact u64 counts, five sorted/distinct/disjoint outcome partitions, execution/conformance statuses, complete-selection/1 and exact completed_at. Refuse decoded duplicate keys, unknown fields/enums, malformed scalar tokens, contradictory counts/statuses and missing/mismatched selected IDs. Give report1 a permanent explicit suite1–4 allowlist independent of future supported-suite helpers.
- **Original suites:** scenario.rs:101–107,190–205,236–269,308–427 and runner.rs:286 establish acquisition/execution — cited. Preserve every original UTF-8 byte including final newline for sha256-json-bytes/1; validate major-specific vocabulary, closed structural fields, decoded duplicate keys and semantic invariants before identity/callbacks. Payload maps remain open data. In-memory suites serialize once and execute that admitted value.
- **Admitted API:** initially inferred; confirmed by implementor report and correction1be4 at admission.rs, runner.rs and counts.rs — cited. AdmittedSuite owns private original bytes, checked value and digest; ExecutedRun carries immutable actual execution identity into CountReport/CountRun. No unchecked Deserialize or mutable escape is added. Historical ConformanceSuite parsing remains unadmitted and legacy report construction/wire stays separate.
- **Rust execution:** report.rs:46–86,556–584 establishes failed > error > unsupported > passed within a scenario; failed or unsupported dominates a separate error at run level — cited. Rust skipped stays zero. Preserve those semantics; unknown coverage never yields conformance passed, including empty/all-pass runs. Every new arithmetic conversion is checked; legacy virtual-clock saturation and shared Timestamp are outside this change.
- **Generated Go:** go/mod.rs:49–63 and runtime.go:499–649 own embedded source/suite bytes and execution/reporting — cited. Add ESS_REPORT_FORMAT=1|2 plus independently resolved ESS_CONFORMANCE_STRICT and ESS_CONFORMANCE_ALLOW_INCOMPLETE; preserve ordinary error→failed, unsupported→skipped, teardown error→failed. V2 tracks actual invoked terminal callbacks, refusing missing selected outcomes under host filters/cancellation/abnormal termination even without a report destination. Report IO failure cannot produce success. Retain a real old generated runtime as a separate control.
- **Scalar/canonical contract:** new counts and report/run timestamps use exact unsigned decimal u64 tokens — cited. Cover zero, 9007199254740993, i64::MAX, i64::MAX+1, u64::MAX; reject negative, signed, quoted, fraction, exponent and overflow. Go checks nonnegative UnixMilli before uint64 conversion, without v1 fallback. New output obeys UTF-8 key ordering, two-space layout and final LF; detailed payload golden vectors separately cover integral float, signed zero, exponent boundaries, Unicode and controls under the binding's finite-number rules. Old producer timestamp domains/bytes remain frozen.
- **CLI:** crates/edge/ess-cli, main.rs:460–480,2413–2478 and existing tests/go_conformance.rs/fixtures — cited. Resolve --report-format 1|2, --strict, --allow-incomplete, pairing and configuration before target construction/identity/callbacks/writes. Execute applicable P1–P4 and configuration refusals; suite5 remains unsupported even for report2 and always refused with report1. Diagnostic exits follow execution; strict v2 requires conformance passed, therefore legacy unknown cannot succeed. Go strict inconclusive must fail even when all subtests skip.
- **Detailed output:** report2 JSON/YAML uses closed ess-conformance-run/2 with exactly format, summary, started_at and scenarios — cited. Detailed IDs/statuses agree with summary partitions, timestamps retain full u64 and standalone --report-out stays JSON. Go produces standalone only.
- **Execution evidence:** package/CLI tests must execute actual Rust and newly generated Go, skip-only, errors, teardown override, omitted subtests, no destination, invalid settings, negative time, scalar/partition/paired-ID invariants, detailed admission and unchanged legacy bytes/defaults — cited. API setup failures are not behavioral red evidence; no claim of all 75 rows or suite5 P5–P10 at this stage.
- **Public guide:** website/docs/guides/verify-conformance.md currently describes report1 — cited. Explain explicit opt-in, detailed/standalone, pairing, strict/diagnostic and unknown coverage without release/adopter claims.
- **Public format reference:** website/docs/reference/formats.md currently calls report2/run2/exact-suite planned only — cited. Refresh only conformance count-stage status, retaining planned suite5 coverage.
- **Internal catalog:** docs/design/review-format-catalog.md has the same planned-only conformance/digest rows — cited. Maintain those bounded status rows without rewriting the accepted coverage design or other formats.
- **Public collision token:** website/docs retains the existing literal parent scope used by the wave calculator — inferred.
- **Root lock:** initially inferred; confirmed by the implementor Cargo-produced patch and frozen source manifest — cited. Root adopted exactly the ess-conformance sha2 dependency entry using already-locked sha2 0.11. No root Cargo.toml or extra package was needed. Preserve concurrent normalization lock additions during integration.
- **Published reader seam:** AEP30aeef2 supplies adapt_json_v2/CountStageReader, planning --from/--suite and typed checked recording/restore — cited. Both routes admit original report/suite bytes; suite5 and detailed run2 are refused. Planning preserves exact timestamp text but explicitly refuses valid values outside its calendar backend. Descriptive history and unknown-coverage evidence cannot qualify complete conformance.
- **Coordinator compatibility:** freeze final actual Rust/Go producers, pass original pairs through both published AEP readers and typed snapshot replay before opt-in publication, per ADR0039 — cited. Record source/runtime identities, byte hashes, exact commands/exits/categories/time, replay identity, legacy controls, unknown-coverage qualification refusal and narrower-calendar refusal. The harness remains coordinator-owned; ESS gains no AEP dependency.
- **Sequencing/adoption:** reader source is published; installed old CLI, direct callers, retained generated packages and detailed-output automation remain separate adoption facts — cited. Protocol adp-ess-conformance/1, evidence ess_conformance_v2 and report/2 are independent versions. No tag/default/suite5 movement follows.
- **Excluded:** browser/player, impact, realization digest consumers, SDK and shared primitives have no count-stage edit established — cited. Coverage shares conformance/CLI and follows this stage.
- **Confidence:** high for producer/CLI/docs/AEP boundaries and now-confirmed internal capabilities/root lock — cited. The literal website/docs parent token remains inferred collision bookkeeping, not a broader implementation write grant.
- **Would collide with:** the two Rust packages, confirmed Cargo.lock addition and cited guide/catalog paths — cited. The parent docs token remains inferred. Root owns planning, compatibility harness, shared changelog, Atlas, publication and lifecycle.

Coordinator confirmation after implementation and two independent attacks: the initial root-lock and admitted-API inferences are resolved above, with their original status visible. The initial report SHA25642301e05ff1818c52b93cbb6bd287c731e7102334928f713014642f1456327bd, correction SHA256b5197b11aa13bf152a09dd03ea6d896ca1d19a485b3f2055f919c8db94796296 and manifests are the sources. All actual public prose changes remain the three assigned guide/catalog paths; raw-value support already existed. Final Go leaf-admission correction is frozen at2d2c391. Root independently verified every inherited assertion and the complete191-input fixture, then the full integrated gate passed1,918cases on64f6f07. Actual source/report details and limits are recorded in verification-report:review-boundaries-8-integrated.

## Acceptance

A skip-only execution emits a versioned report with zero actual failures and explicit non-pass category accounting while preserving its inconclusive execution verdict.

## Remediation ownership and compatibility

Owns the F03 count split, not exact-suite coverage binding. The conformance-format design must settle Rust error/unsupported versus Go skipped mapping, list semantics and old/current reader behavior before this implementation. Valid v1 bytes and meanings stay frozen; a new count meaning is not retrofitted to v1. Downstream AEP migration requires separate governed work and an Atlas ADR before enabling a new default writer. The narrow v1 reader story lands first. For computed collisions, the guide's parent `website/docs` token is also recorded, inferred, matching the public-doc stories' chosen granularity.

## Legacy DTO and admitted execution clarification

The historical ConformanceSuite DTO, from_json and generic Deserialize remain unadmitted value parsing. Existing suite.rs fixtures at the wave8 base parse discarded provenance metadata and newer vocabulary under a legacy marker; their parser behavior alone is not version/vocabulary admission or original-byte proof. Do not weaken those tests or reinterpret the DTO as an admitted capability.

Original-byte CLI, generated-Go and report-pair inputs enter the strict AdmittedSuite path directly. The admitted value has no unchecked deserialization, public constructor or mutable escape. Every Runner entry checks supported membership and retained typed vocabulary before target identity/callbacks. A checked try_run-style API reports refusal; a compatibility nonfallible run wrapper must document its pre-execution refusal behavior and cannot fabricate a complete report on invalid input.

The separate in-memory path serializes its supplied typed suite once, admits that immutable buffer and executes the corresponding value. If a historical DTO parser previously discarded fields from some other JSON, the new buffer is a different issued suite document with its own digest. It cannot be called admission of, or paired as, the discarded original bytes. Original unknown structural fields remain refused by direct admitted input; newer vocabulary under a falsely older major remains refused before execution even if the DTO parser accepted it.

Required control: retain the old DTO parsing assertions, directly refuse the unknown-field original at AdmittedSuite/CLI, prove retained invalid-major vocabulary never reaches callbacks, and show any admitted reconstructed in-memory document has its own exact digest and cannot pair with the rejected original. This is the S3/S4 distinction between value compatibility, original-byte admission and serialize-once in-memory execution, not a permissive original-byte bypass. No source-breaking raw-parser migration is selected in the count stage.

## First review correction boundary

The first independent attack at a46bd7ff46ec8553bef4f48d4021514c8f175e82 is preserved byte-for-byte in review-result:ess-count-writer-adversary-pass-1 (report SHA25632563d1fcbcc00489f7d6f897a0c6f471d2774ef99e65ad4a861e69c959b84ff). Three retained reachable introduced cases fail: exact executed-suite rebinding, permissive Go predicate admission, and Goexit during teardown incorrectly counted terminal. Complete package selection executed405:402passed3failed0ignored33summaries; all401inherited cases still pass. The same implementor receives all three for correction; fixed outcomes wait for actual correction evidence.

Count-source construction must carry identity from the exact immutable admitted suite actually executed. Same provenance and selected IDs do not establish this. The opt-in producer receives an immutable execution result/capability with no unchecked construction, mutable escape or raw Deserialize. A legacy ConformanceReport clone or independently parsed DTO is not that capability. Preserve historical ConformanceReport public construction and wire bytes rather than installing a source-breaking private field there. Newly introduced, unpublished opt-in APIs may be refactored to carry the execution binding; narrowly mechanical adaptations of new count/adversary helper signatures must preserve every assertion and fixture meaning. This changes no legacy default or report wire contract.

Generated Go original admission must match the accepted structural/predicate grammar and limits before target construction. A callback interrupted by Goexit is not a completed teardown and cannot support a complete report2, including when no destination is configured. Retain the actual valid quantifier, ordinary skip/returned teardown error, strictness and opaque-ID controls. No AEP reader relaxation is part of correction.

## Implementation evidence

Source2d2c391 integrates atbfc2c0b and passes the complete gate at64f6f07:1,918workspace cases, zero failures or ignored cases, all10recorded lanes exited0 with the exact compiler-input carry described in verification-report:review-boundaries-8-integrated. Both independent reviews are corrected and root verified the final correction without a third attack. All19actual Rust/Go producer fixtures passed the independent published AEP reader/planning/replay correspondence, including expected negative controls. Count-only unknown coverage never qualifies complete conformance; legacy defaults and bytes remain frozen. Source/public documentation publication and task-owned cleanup are recorded separately in the wave page.
