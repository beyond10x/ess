---
format: aep.planning-md/1
id: verification-report:coverage-writer-actual-correspondence
kind: verification-report
status: draft
title: 'Coverage writer: integrated gate and 43 actual AEP correspondence cases'
relations:
- verifies: story:review-conformance-coverage
revision: 1
---
# Coverage writer integrated gate and actual downstream correspondence

ESS source `a333949e6581e151f2c3b154d7df30e125d07375` integrates the independently reviewed
writer and the root-verified final test disposition at `d3c2c1dd09ad279f5bf6b4c1050b589a4a7ad761`.
All ten integration lanes returned their own exit 0 with frozen source unchanged. The complete
workspace test lane executed 2,058 passing cases, zero failed or ignored, across 187 summaries.
Its duration was 224.876846674 seconds. Command receipts, raw logs, original source snapshots,
producer binaries and exports remain in `target/review-boundaries-11/gate-a333949e6581`.

| Lane | Direct exit | Seconds | Raw log SHA256 |
| --- | --- | --- | --- |
| fmt-check | 0 | 1.502595180994831 | e96f1cab699f6a4bb3c4dd64f88e7ce3dfb5aaaa299891416e29335e005cd893 |
| clippy | 0 | 43.64588203292806 | df2875c35ab10ded692ebf8252179743d363ec7acaa77c3c2b77cc808a3442f9 |
| test | 0 | 224.8768466740148 | 41bd7b04d86153dafc77175d0b937273af8cedd81b4b104383bf1013ab8567f5 |
| doc-check | 0 | 23.823037813068368 | bf999d4bd5420f29a763ccfb1b299a22410e122ce37b779474f7c0e4675fb24f |
| example-check | 0 | 12.111435408005491 | ed589817ee6baa31cfa20701f4bdc592332a146236d0eb890353581a9e7876f5 |
| projection-check | 0 | 10.611125690978952 | d67703f04c862e265e9b0d6276196cbaa9e98ec58d32cc62b88b08fff8e94f66 |
| release-check | 0 | 0.1013244380010292 | a0ddaec10557806a60282302c856a4ee919bbb5ff393481d2fb750a7b8fe7b10 |
| action-check | 0 | 0.10130824404768646 | ce62e1f35e7bcdfebdd7ed28735f5d7e206b8f32f6f5fc8e2172e216eac900b9 |
| site-build | 0 | 16.41941811295692 | 3deb5f6ede57ae9e4c3d6ca1375b256bd3f1ee6d1a7972ffe375f9d08694f749 |
| planning | 0 | 22.422477285959758 | c13d9e06b2753d0acf602cbfc0cbee1530c74eaee3fc47242df7d840c25f6043 |

## Independent producer correspondence

The pre-writer semantic plan remains SHA256
`5a378b14f7747ce7b3f1eac9b6d6e8c5962c02f81e4e116f7e74d478bcae6ef1`, and D1–D6 resolutions remain
`6feb6754888aefd7fbd9b371fdc09ed42ec30bbcbef3962176d7e1a9f8ca9f8b`.
Root source preinspection preceded the full gate. After execution, the wrapper compared all
17 complete suite definitions with their legacy oracles and the independently fixed inventories.
Three producer profiles retain byte-identical original selected suites and parent chains.
Every one of 39 actual report instances has independently checked original clock inputs,
callback counts/IDs, retained compiled inputs and binaries. The clock/definition checkpoint was
persisted before opening report bytes; the unchanged Rust mapper only hashes report bytes.
Go completion timestamps are actual fixed-clock samples; the wrapper does not claim a separate
Go start measurement. Rust terminal counts rely on the reviewed sequential runner returning,
not on an invented terminal callback. The CLI pair-refusal control has no callback recorder;
source ordering and the same-gate target-closure regression provide its bounded evidence.

The first actual wrapper attempt (v3) stopped with exit 1 because it required one matching process
observation and found two. All six original /proc observations remain retained. Both complete
executable/hash/argv matches had identical cwd and complete relevant environment evidence.
Root's v4 correction requires all full matches to agree, preserves every observation and makes
no claim to identify a unique producing PID. Non-atomic fork/exec observations are an inference,
not an established lineage. No report expectation or production source changed. v4 passed in
2.387360938 seconds; the separate four no-report refusal controls passed in 0.132974188 seconds.
Original v3 output and both complete v4 diffs remain alongside the receipts.

The frozen mapper `9c478fe4ca22fb5dd1bd5ac79033148d661eace370e10d9c824307a0dcc4efb7`
returned 0 in 160.513133737 seconds, producing 39 instance expectations and four task variants.
The unchanged AEP helper `90b3072b095486420bda7de199511bd7c00b7d73293d9d79ccc3c31248627262`
then ran all 43 cases: each returned 0, taking 29.035928187 seconds in total. Both AEP readers,
checked recording/submission and typed replay/restore matched. Compiled inputs match published
reader `658cf76e6371b1628f6de69548e724b52803f5c2`; AEP document HEAD
`70ec336c00fa67b3d83f5c05bfbe71189462c01e` and all tracked inputs stayed unchanged.
These are actual ESS Rust/generated Go reports, not synthetic mapper self-checks. The helper
consumes those exports; it does not execute ESS or independently authenticate a producer.

Root independently rehashed all 43 raw logs and expectations and compared each complete result's
inventory, outcomes, counts, clock, qualification and transport hashes with its expectation.
Every raw log parses exactly to its result JSON. All cases recorded the same 16 checks:
- original bytes
- independent full inventory and outcomes
- exhaustive typed facts
- raw admission loss
- re-admission clearing
- typed JSON and YAML
- direct Execution recording
- Engine submission
- snapshot restore
- source readback
- qualification
- evidence.missing
- changed-suite no mutation
- missing-reader no mutation
- frozen count suite5 refusal
- both reader setter orders

The 43 expected qualifications are 18 qualified, 17 unknown and eight contradictions. Unknown
and contradiction cases are successful negative checks, not 25 qualifying conformance runs.
Qualified cases reduce evidence.missing from nine to eight; diagnostic cases retain nine.
The four separate no-report controls cover host filtering, negative Go clock, legacy Go report
pairing and legacy CLI report pairing; all retain exit 1 and absent report output as expected.

| Fixture | Qualification | Reason | Completion clock |
| --- | --- | --- | --- |
| S01-combined-go-passed | qualified | — | 0 |
| S01-combined-rust-passed | qualified | — | 114 |
| S02-generated-rust-passed | qualified | — | 110 |
| S03-authored-rust-passed | qualified | — | 5 |
| S04-invoice-component-go-passed | qualified | — | 0 |
| S04-invoice-component-rust-passed | qualified | — | 95 |
| S05-child-rust-passed | qualified | — | 11 |
| S06-grandchild-go-passed | qualified | — | 0 |
| S06-grandchild-rust-passed | qualified | — | 5 |
| S07-empty-go-passed | unknown | EmptySelection | 0 |
| S07-empty-rust-passed | unknown | EmptySelection | 1 |
| S08-unknown-go-passed | unknown | UnknownCoverage | 0 |
| S08-unknown-rust-passed | unknown | UnknownCoverage | 3 |
| S09-unknown-child-rust-passed | unknown | UnknownCoverage | 3 |
| S10-repeated-gap-rust-passed | unknown | InScopeRefusal | 110 |
| S11-gap-survivor-child-go-passed | unknown | InScopeRefusal | 0 |
| S11-gap-survivor-child-rust-passed | unknown | InScopeRefusal | 5 |
| S12-gap-filtered-away-rust-passed | unknown | InScopeRefusal | 5 |
| S13-accepted-duplicate-go-passed | unknown | InScopeRefusal | 0 |
| S13-accepted-duplicate-rust-passed | unknown | InScopeRefusal | 3 |
| S14-refused-then-accepted-go-passed | unknown | InScopeRefusal | 0 |
| S14-refused-then-accepted-rust-passed | unknown | InScopeRefusal | 3 |
| S15-single-complete-go-clock-9007199254740993 | qualified | — | 9007199254740993 |
| S15-single-complete-go-clock-9223372036854775807 | qualified | — | 9223372036854775807 |
| S15-single-complete-go-failed | contradiction | ExecutionFailed | 0 |
| S15-single-complete-go-passed | qualified | — | 0 |
| S15-single-complete-go-skip-then-teardown-error | contradiction | ExecutionFailed | 0 |
| S15-single-complete-go-skipped-strict | unknown | ExecutionInconclusive | 0 |
| S15-single-complete-go-skipped | unknown | ExecutionInconclusive | 0 |
| S15-single-complete-rust-clock-18446744073709551615 | qualified | — | 18446744073709551615 |
| S15-single-complete-rust-clock-9007199254740993 | qualified | — | 9007199254740993 |
| S15-single-complete-rust-clock-9223372036854775807 | qualified | — | 9223372036854775807 |
| S15-single-complete-rust-clock-9223372036854775808 | qualified | — | 9223372036854775808 |
| S15-single-complete-rust-error | unknown | ExecutionInconclusive | 0 |
| S15-single-complete-rust-failed | contradiction | ExecutionFailed | 0 |
| S15-single-complete-rust-passed | qualified | — | 0 |
| S15-single-complete-rust-unsupported | contradiction | ExecutionFailed | 0 |
| S16-final-merge-rust-passed | unknown | InScopeRefusal | 3 |
| S17-known-origin-selection-rust-passed | qualified | — | 5 |
| V01-component-as-system | contradiction | SelectionMismatch | 95 |
| V02-generated-as-combined | contradiction | SelectionMismatch | 110 |
| V03-authored-as-combined | contradiction | SelectionMismatch | 5 |
| V04-explicit-as-all | contradiction | SelectionMismatch | 5 |

## Retained receipts

- `target/review-boundaries-11/gate-a333949e6581/complete.json` — SHA256 `7e40e22b24b9a2a198584f3ef1e4282ae9362d189a6d33c93fbe422f784ae8e6`.
- `target/review-boundaries-11/gate-a333949e6581/actual-correspondence-root-readback.json` — SHA256 `c9521a105a692c5c24f0dd0f708d5b328e0f3dab00f6d97565ad3a86f32d4f98`.
- `target/review-boundaries-11/gate-a333949e6581/wrapper-v4-command.json` — SHA256 `6e315983a74eb5d2a7d2e362b591e0fb9b3089ad0ed29c7d1f12f5405b78c417`.
- `target/review-boundaries-11/gate-a333949e6581/no-report-controls-v4.json` — SHA256 `ed060a0fe0e70414102ee46af8c155f622631661a36b45430de9b9b761bab6a4`.
- `target/review-boundaries-11/gate-a333949e6581/actual-mapper-command.json` — SHA256 `99bb204cd633711e259226447f5badec68e7df6c069f699e1e5f1328ac9486ee`.
- `target/review-boundaries-11/gate-a333949e6581/actual-aep-correspondence-command.json` — SHA256 `5d1a7b35661db5c226945ad5f69424e99aa7f089ef24736f75c74482375ea55a`.
- `target/review-boundaries-11/producer-mapping-preparation/correction-01/actual-a333949-wrappers-02/root-d1-d6-readback.json` — SHA256 `29154209e403e9c41b70f7bd77bee6870ee286fb4ccd4dc0c16cdc87daf102ef`.
- `target/review-boundaries-11/producer-mapping-preparation/correction-01/actual-a333949-wrappers-02/actual-bundle.json` — SHA256 `0b535dc26383c1f2b897f3f8d10da6e0b183f02bad6dd793ede9e98ffdcf0dda`.

The AEP case directory is its coordinator's
`target/ess-conformance-coverage/producer-compat/preparation/actual-ess-a333949-01`.
The bot wrapper published exact gated ESS source `a333949e6581e151f2c3b154d7df30e125d07375`;
fresh remote readback matched. All 11 directly introduced commits have verified bot author and
committer. Remote CI, documentation delivery and owned cleanup are subsequent observed outcomes,
not claims supplied by this record. Suite4/report1 defaults, installed binaries and releases
remain unchanged. The separate browser-fidelity and recovery obligations remain open.
