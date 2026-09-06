---
format: aep.planning-md/1
id: verification-report:coverage-writer-correction-and-gate-preparation
kind: verification-report
status: draft
title: Coverage writer correction and root gate preparation
relations:
- verifies: story:review-conformance-coverage
revision: 1
---
# Corrected writer and root gate preparation

The source correction is frozen at 8560854dae5998a3b4f1b03abb60b75af4783893, with both bot identities
verified. All four unchanged first-review cases pass. The complete package lane recorded 646 passed,
zero failed or ignored, 62 summaries, exit 0 in 42.390921502 seconds. Report SHA256:
a6c65b16dc7fc90b0f100bb144033c48ba51cdc766f6ff68cd6bf1b646abc1df. Seal SHA256:
903d4023c20e00b96f7123a9e509865311d86a8e8a3a68d8a7cf7efe31d76d1c.

Root verified 21 seal records, 14,879 evidence entries, 2,953 cache entries, 951 external entries,
and all 1,103 final source snapshots. All 13 new and four inherited raw logs occur verbatim in the
report. The 39 report and four control index paths were checked as opaque bytes; each profile's
554 source inputs matches the full source snapshot. The new helper indentation assumption and lint
failure remain in the failed attempts. Source pass 1's fixed outcome was recorded after correction.
Final source attack 2 is active. Integration, mapping, helper execution and publication remain pending.

A separate preparation audit reviewed only root ignored orchestration and frozen contracts and
receipt shapes. It is not a writer source attack. Root read the complete audit below and verified
all 17 input hashes. O1 was corroborated with an owned Task/sleep launcher whose child belonged to
a nonleader TID; the extracted v5 traversal detected that child. O4 was checked using an actual
true child exit 0 and the v6 AST-extracted post-step block with a deliberately missing process
observation: results.json preserved exit 0 and a separate orchestration refusal. Neither probe ran
an ESS gate, producer, mapper or AEP helper.

The current preparation below supersedes the reviewed versions while preserving their bytes.
Root's source readback records O1–O4 corrections. This is coordinator review, not another agent's
approval or proof of successful actual correspondence. The final gate and all correspondence steps
remain required. Both the D1 map and independent semantic plan are unchanged.

{
  "at": "2026-09-06T20:47:46.133856+00:00",
  "audit_report_sha256": "911bf60bebdd2f789f4f5ba51a282d0bb924147de2ae7b904d02bdf1031bb15d",
  "audit_inputs_verified": 17,
  "new_files": {
    "run_integration_gate_v6.py": "4dda4eb49a8dfcc28de53397fe9eab988b886f22d2e68de52e61a5adfd89e331",
    "preparation/prepare_actual_wrappers_v3.py": "c5da717268c2c976623cb5855ff41d664ee22b83ec10436d9c7a2d7171148a3f",
    "preparation/read_no_report_controls_v3.py": "b5691d24672e6767188aec5ba0ebe71eedd482b1ef7125440e1edc4f7ada6355",
    "preparation/run_actual_aep_correspondence_v2.py": "1ff02b6092e65610fbd8960198e65e04315a1a78bc3c89a7d36bb079bd1e54fe"
  },
  "O1": "all TID traversal with owned Task/sleep red/green probe",
  "O2": "enumerated relevant environment key census plus explicit positive Go configuration comparisons; retains source proof for Rust",
  "O3": "source identities and exact expectation/bundle producer pins bound before controls/helper; all bundle transports pinned before/after",
  "O4": "direct child row persisted before post-step checks, separate refused status; real true-child controlled absence probe retained0",
  "stage": "syntax checks and process/receipt-only probes; no integration gate/ESS producer/actual mapper/AEP helper execution",
  "full_audit_read": true
}

Preexecution source checkpoint: producer-source-preinspection-frozen-02.json, 17 inputs,
SHA256 3c9306c70dca9c0d79b0862e9b21198a52f4979b24d128cd80268504a7ec3963.
It retains prior source observations, compares 11 unchanged inputs, reviews the two changed inputs,
and adds the refusal builder, fixed D1 fixture and CLI acquisition/target boundary. It must match
final integrated bytes before execution. Source pass 2 may add tests but no implementation.

## Complete preparation audit, unabridged

# Orchestration preparation audit 01

**Result: four bounded findings in the supplied orchestration.** O1 can block a valid actual run; O2–O4 concern environment provenance, cross-run source attribution and preserving direct lane receipts. This is preparation review, not approval, producer correspondence, a source attack on the coverage writer, or an executed gate result.

Only the four named scripts, their fixed mapper/input contracts, root's separately authorized Task/sleep probe result and six historical receipt-shape files were inspected. No audited script was imported or executed. No report.json was read.

All relative paths below start at the ESS coordinator /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba. Script paths are abbreviated only after the following exact inventory.

| Abbreviation | Exact reviewed path | SHA-256 |
|---|---|---|
| Gate | target/review-boundaries-11/run_integration_gate_v4.py | 72299e3adaf5af381720ec2194deb3c292d855922788c1b86afc4498fc8ef1cf |
| Wrapper | target/review-boundaries-11/preparation/prepare_actual_wrappers_v2.py | de558b6eeebb23191667682ac4ae3ddb8f1b93ed02329878c2f1f4c135cec2c1 |
| Controls | target/review-boundaries-11/preparation/read_no_report_controls.py | 09bed61a26a71390b2ebeb26a5c5ed5c78c5977e37935470eaa874a4b38c42f8 |
| Helper runner | target/review-boundaries-11/preparation/run_actual_aep_correspondence.py | d3c3183492cafc55a2814cf7250728e28a6f370a21542cf6f248bdd0a73c2cb2 |

## O1 — High: leader-only child traversal misses Task's launched process

**Locations:** Gate:87–94,97–103,136–142; Wrapper:148–150.

**Source proof:** descendants reads only /proc/PID/task/PID/children. It does not visit other TIDs' children. observe_producer_process therefore cannot reach a Cargo/test descendant launched by a nonleader Task thread. If the test lane exits zero without the required observation, line 142 asserts before its direct lane result is retained. If only some relevant processes were observed, the Wrapper still requires one exact executable/hash/argv match for each profile.

**Root-owned corroboration:** preparation/process-observer-probe-01/result.json:10–21,366–367 reports PID 1303666, nonleader TID 1303685 and child 1303687, with an empty leader-only child list; the owned task/sleep command exited zero in approximately 2.07 seconds. This reviewer read that result, not the probe implementation or a product run. The probe establishes the launcher/thread counterexample, not an ESS gate outcome.

**Small correction:** enumerate children for every current TID under each visited process, deduplicate reachable PIDs and tolerate vanished threads/processes. Preserve direct /proc executable/hash/argv/cwd correlation. Keep an explicit refusal when observations are missing; never reconstruct an observed process solely from producer.json.

**Status:** root reports a separate v5 traversal correction and another owned probe. Neither v5 source nor probe02 was inspected by this audit. O1 remains attached to the frozen v4 bytes; this report does not approve the reported correction.

## O2 — Medium: a partial parent environment is treated as the reconstruction base

**Locations:** Gate:109–112; Wrapper:203,209–217,238–241,296–306; Controls:75–87.

**Source proof:** Gate records a fixed selected environment containing Cargo/temp/export/cache keys, but omits inherited ESS_CONFORMANCE_*, ESS_REPORT_* and ESS_FIXTURE_* settings. Wrapper copies that selected map and applies explicit Rust Command Debug removals/overrides; Controls does likewise and tests absence of STRICT or REPORT_OUT in the resulting map. Omitted from the observation and absent in the real process are different facts. A relevant setting that was inherited and neither overridden nor explicitly removed is missing from this reconstructed record.

**Observed historical shape, limited to receipt text:** the authorized R01-go-host-filter/go-run.command:1 explicitly contains env -u ESS_CONFORMANCE_STRICT and overrides ALLOW_INCOMPLETE, callback/clock paths, clock, target, target mode, report format and output path. Thus the historical R01 command supplies positive evidence for those settings even with the partial parent map; it is incorrect to claim this omission by itself invalidates R01. Its paired receipt supplies command/elapsed_ns/exit/success only, not the missing inherited environment. The R03 Rust CLI command has no explicit cwd/env changes in its Debug spelling and the parent producer.json has no environment or cwd fields.

**Small correction:** retain the known relevant inherited ESS keys at the directly observed parent process, apply explicit removals/overrides and record which values are inherited, overridden or removed. When absence matters, require complete observation for that relevant key or a recorded explicit removal. Continue treating the result as a selected relevant environment, not a dump of every inherited variable. The positive Go path should compare the actual recorded configuration with the independently fixed target/format/incomplete/strict/clock contract rather than filling those semantics only from the plan.

**Rust limit:** in-process Rust target/clock configuration is not proved by environment absence. Its authority remains the exact reviewed constructor/runner source plus its logged inputs and callbacks, as Wrapper:275–276 states. Capturing more environment must not replace that source proof.

**Status:** root reports relevant-prefix capture in v5; this audit did not inspect that correction or any current writer source.

## O3 — Medium: separately supplied runs are not bound to one source identity

**Locations:** Controls:17–38,48–54,126–129; Helper runner:37–58,66–85,107–110.

There are two concrete joins missing:

1. **Controls gate versus wrapper checkpoint.** Controls validates the wrapper's internal bundle/checkpoint pins and matches the supplied gate's own process/producer/source receipts. It never requires summary.source_commit, bundle.source_commit and that gate's source.json/complete.json head to agree. Its final source_commit comes from the wrapper summary. A wrapper from source A and controls from source B, with unchanged suite bytes/IDs and passing receipts, can satisfy these checks while the final controls result names A.

2. **Mapped expectation receipt versus supplied bundle.** The helper runner reads a producer receipt from each mapped expectation, uses its fixture ID to find a separate bundle instance, and checks that instance's report/input bytes against expectation hashes. It does not require the mapped producer_receipt pin to equal bundle.instances[id].producer_receipt, or receipt.source_commit to equal bundle.source_commit. Preexecution and completion metadata nevertheless name bundle.source_commit. If two source revisions produce identical report/input bytes, expectations and receipt A can be paired with transport B and attributed to B without an identity check linking those records.

**Evidence boundary:** these are conditional acceptance paths established by reading source; no mismatched bundle or gate was constructed or executed. Hash-equal bytes remain hash-equal, but that does not establish which execution/source receipt owns the claim. Fixed D1–D6 semantics do not remove this provenance join.

**Small correction:** before reading control outcomes or invoking the helper, compare the gate/summary/bundle source identities and require the exact mapped receipt pin to match the selected bundle instance. Validate those named pins, not only the receipt's kind. Pin the selected bundle as part of preexecution input and recheck that same bundle identity at completion. Preserve the existing variant behavior: V01–V04 intentionally reuse a base receipt/fixture, so compare via receipt.fixture as the runner already does rather than requiring a variant ID to be an instance key.

**Why the mapper does not repair this later join:** fixed correction-01/mapper.rs:202–221 checks bundle/source/receipt identities during mapping and :243–246 carries the exact producer_receipt pin into output. Helper runner's separately supplied bundle is read later; it must be tied back to that pin rather than relying solely on content-equal report/input files.

## O4 — Medium: a post-step orchestration refusal loses the direct lane receipt

**Locations:** Gate:140–142,145–170.

**Source proof:** a completed child's real returncode is available at line 140, but rows are appended and results.json written only after the no-observation assertion, source-equality assertion and producer/binary retention reads/checks. A missing process observation, source-drift refusal or missing/changed producer binary raises before that lane row is durable. The lane log survives, but the structured direct exit, finish time, duration and counts do not. The outer Python failure is not the child's direct status.

**Concrete relation to O1:** in the confirmed Task-thread counterexample, a zero test exit can be followed by line 142 failure without a retained test result row. Correcting traversal reduces that trigger but leaves the ordering problem for other post-step checks.

**Small correction:** persist the direct command result and log hash immediately after child completion, then record source/process/binary verification as separately named post-step results or append an explicit orchestration-refusal record. Keep the gate incomplete on any failed verification. Do not convert a failed orchestration check into a fabricated nonzero product exit, and do not claim all checks passed solely because the child returned zero.

## D1–D6 and receipt interpretation retained

The findings require orchestration corrections, not new semantic choices or modifications to the fixed mapper, oracle or plan.

| Contract | Source inspection outcome |
|---|---|
| D1 refusal messages and multiplicity | Wrapper:99–110 copies the independently fixed inventory and substitutes messages from pinned resolutions while retaining its refusal list. No report-derived message/count expectation was introduced. |
| D2 acquisition/origin and D3 final merge | Wrapper:111–138 compares selected IDs, complete selected definitions, provenance and parent chain against fixed cases/oracles. Whether the actual source performs the promised acquisition/final merge remains the root's exact-source preinspection obligation; these receipts cannot prove an API call by themselves. |
| D4 source/tool/binary/target | Gate:19–23,54–81 retains reviewed source and pre-lane tool identities. Wrapper:145–181 correlates source snapshots, binaries and direct test-process receipts. This supports source attribution only with O1–O3 addressed and the source review tied to final source; this audit does not renew that source review. |
| D5 original bytes and independent semantics | Wrapper:95–138 checks suite/parent/carrier bytes and legacy definitions. Report files are first hashed at :294 after the :270 checkpoint and are not parsed by this wrapper. Fixed mapper:232–256 likewise uses report bytes only for hashing. No report semantics were inspected here. |
| D6 exact clock inputs | Wrapper:242–268 uses Python exact integers, checks advancing or fixed inputs, writes the clock transcript, and seals its checkpoint before report hashing. Mapper:131–144 enforces clock mode, uint64 range, Go int64 limit and evaluation time. The assertion that a clock value was logged before return remains a source fact; transcript values alone cannot prove that ordering. |
| Rust terminal accounting | Wrapper:252 and :276 explicitly derive terminal count from logged begin count plus reviewed sequential runner return/final-clock behavior. This is source-supported inference, not a separately observed Rust terminal-event transcript. Do not relabel it as direct terminal observation. |
| Go completion/start distinction | Wrapper:277 explicitly says Go has one completion sample and started_at repeats it without claiming an independently measured start. Mapper:220 does not impose the Rust/advancing start rule on fixed Go cases. No correction to the independent clock domain is needed. |
| No-report controls | Controls:121–124 correctly limits the Rust CLI's zero-callback claim to source ordering and a named green regression, because that CLI has no callback recorder. Historical receipt text confirms the anticipated Debug command and command/elapsed_ns/exit/success shape; it supplies no callback trace. |
| Expected output cardinality | The pinned plan has 17 structures, 39 report instances and four refusal controls. Mapper:260–274 adds four task-mismatch variants to yield 43 expectations, matching Helper runner:56–57. No cardinality mismatch was found. |

No JSON-schema/type contradiction was found between Wrapper's emitted receipt shapes and the fixed correction-01/receipt-schemas.json / mapper structs in the inspected fields. This is a read-only contract comparison; the mapper and helper remain unexecuted by this audit. The helper runner's fixed AEP/helper/source checks were read, but their actual binary/runtime state was not revalidated here.

## Input scope, seal and relinquishment

The historical-shape exception was explicitly supplied by root after the opening task. Its exact prefix is:

/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/coverage-writer/final-exports-attempt2/controls

Read only R01-go-host-filter's go-run.command/receipt, R03-rust-suite5-report1-no-destination's cli-run.command/receipt, profile producer.json and source-manifest.json. The latter was used for manifest shape/count/hash, not to inspect the source paths it lists. No new writer report or active unit source was opened. The initial prefix without -writer did not exist and yielded only absent-path observations.

All 17 pinned inputs (532,765 bytes) were unchanged at seal. Input manifest: **input-manifest.json**, 9934 bytes, SHA-256 **63e39a71bf1396c4fd5dc821b40d3319d537ddd9c5a1619a22db6b44d81b5a2f**. The report's own hash is supplied at handoff to avoid self-reference. Structured finding identities and exact input hashes are included in that manifest.

No tests, builds, ESS/AEP/mapper/helper/gate calls, network, store/Git changes or cleanup occurred. No reviewed script was edited. Writes are confined to this new report and input manifest. Parent's reported v5 work is outside this audit and has not been accepted by it.

All writes relinquished at handoff.
