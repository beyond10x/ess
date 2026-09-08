---
format: aep.planning-md/1
id: story:review-output-ownership
kind: story
status: implemented
title: Make generated output replacement recoverable and ownership-aware
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-output-containment
scope:
- confidence: inferred
  path: .github/workflows/ci.yml
- confidence: inferred
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli
- confidence: inferred
  path: crates/edge/ess-xtask/Cargo.toml
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: cited
  path: crates/edge/ess-xtask/src/main.rs
- confidence: inferred
  path: docs/design/review-format-catalog.md
- confidence: cited
  path: docs/design/review-output-ownership.md
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
- confidence: cited
  path: models/output-ownership
- confidence: inferred
  path: website/docs/guides/generate-artifacts.md
- confidence: inferred
  path: website/docs/reference/cli.md
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 15
---
## Finding and source

F10 (P1) from `docs/reviews/2026-09-05-architecture-review.md:365`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/main.rs:2016`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A repeated generation updates only owned outputs as one recoverable operation while preserving authored files across stale-file retirement and injected failure.

## Implementation boundary

Design ownership and recovery against existing generated artifact paths before implementing staged writes. Define first-run adoption of legacy output, collision refusal, stale generated-file removal, interrupted staging/recovery and preservation of unowned authored additions. Add typed ownership data only after its design and compatibility policy are recorded.

## Validation

Inject mid-write/rename failures, rerun after interrupted staging, remove a formerly generated artifact and keep an unrelated authored file; prove either the previous complete output or recoverably staged new output, never an unexplained mixture.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Input discovery policy is separate; do not delete unknown files or infer ownership from an extension.

## Scope

Closure scope confirmed against the complete final source, consumer qualification and native
integration results at f6ebe903. Opening inferences remain visible below. The opening scoper report at1e618d2 remains
SHA2566a8a354085aa204cfc2f716c97cb6a8fd7cc0436811d5e870b23f3991b6c9974 with28 independently
verified inputs. The implementor's original confirmation table is retained in
target/review-output-ownership-wave19/scope-confirmation.md; root owns shared-file confirmation.

| Path | Opening confidence | Confirmed implementation surface |
|---|---|---|
| crates/edge/ess-cli | cited | Fixed generated-output owners and all actual caller routes, native filesystem/state/admission adapters, explicit adoption/recovery, ordinary CLI exposure, preservation and interruption tests. |
| crates/edge/ess-xtask/src/main.rs | cited | Compatible native directory locks held across projection-sync preflight, writes and pruning; intersecting ownership/reserved-state and planned-alias refusal. The original preflight-only hypothesis was corrected after a measured lock-bypass failure. |
| crates/edge/ess-xtask/Cargo.toml | inferred, added at revision12 | Confirmed rustix fs dependency for the same descriptor-lock protocol used by the CLI. |
| crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json | cited | Exact finite classifications for 7902 actual production declarations; final consumer-check verifies source accounting and current attributed behavior. |
| docs/design/review-output-ownership.md | cited | Selected fixed-owner, private checkpoint, native admission, platform and recovery contract; records the tested permission refinement. |
| docs/design/source-pinned-data-normalization.md | cited | Reconciled generated-library stale retirement, ownership, recovery and nonwriting checks. |
| models/output-ownership | cited | Validated Anchor/Transaction typed model and relation, with Prepare/Commit/Restore outcomes; static modeling does not substitute for native runtime evidence. |
| website/docs/guides/generate-artifacts.md | inferred | Confirmed adopter workflow for authored preservation, first generation, reference adoption, stale retirement, interruption and explicit recovery. |
| website/docs/reference/cli.md | inferred | Confirmed visible generate/output and hidden output alias, exact adoption selectors and explicit enclosing compose anchor. |
| docs/design/review-format-catalog.md | inferred | Confirmed new private ess-output-state/1 reader/writer and canonical-byte/compatibility policy. |
| website/docs/reference/formats.md | inferred | Confirmed unreleased private checkpoint format and recovery authority documentation. |
| Cargo.lock | inferred | Confirmed only errno0.3.14, linux-raw-sys0.12.1 and rustix1.1.4 added, with no existing package replacement. |
| .github/workflows/ci.yml | inferred | Confirmed both shipped macOS architecture witnesses and retained Linux gate; actual platform tests are required, and historical release backfills keep their own source authority. |

The opening consumer profiles.json and reviewed-candidates.json paths were conditional inferred
reservations. Their current bytes are unchanged from the published baseline (SHA256
6e6787c7ea5b643b98658aafbab8d6ab6017a9e727499fca09c29e12cf0520ba and
9c7d6ec69e6ed4c792cd9cc60461f68789736994be9a8fe897c4c27a81d5f120 respectively). Final consumer-check confirms unchanged reviewed eligibility; their two conditional typed
reservations are retired, with this opening history retained. No e005 expansion or baseline regeneration is authorized.

The independently published mainb7a0303 entity-snapshot allocation change was incorporated as
49c193d. Its synthesis files and two helper classifications are already-main work and do not
become output-ownership scope by appearing in this integration tree.

Final confidence: high for these 13 confirmed surfaces. Both review findings are resolved;
final native, integration and consumer evidence is recorded below.

## Selected output-ownership binding and model — 2026-09-08

The coordinator selects `docs/design/review-output-ownership.md` under the existing standing implementation approval. Ordinary generation keeps its real signatures and fixed default roots. Legacy adoption is a separate operation comparing a settled, unmodified generated reference with exact target bytes; first enrollment for the selected owner or exact idempotence cannot discard another owner's or prior stale inventory. Compose uses one explicit enclosing root. The fixed G01–G14 owners, authored preservation, selected stale retirement, no-write checks and explicit recovery are retained.

The anchor owns at most one current transaction, via its UUID. `models/output-ownership/system.yaml` and `models/output-ownership/domains/ownership.yaml` give those nouns typed identities, the explicit relation and Prepare/Commit/Restore outcomes. The current retained ESS0.20.0 CLI (SHA25678976b7f5ac11dedd6da1649833415afc06d3f15c0de88d3dd1ddf26cd94f442) executed validation and compilation; both actual0. Validation output: `output v1 — 2 file(s), valid`. This is static model evidence, not executed filesystem recovery.

The selected protocol uses anchor-local, same-mount staging; top-down shared ancestor/exclusive anchor directory locks; Staging/Prepared/Committed/Restored checkpoints and settled Idle state; immutable preimages; explicit recovery; and retained decisions through cleanup. The later adjacent-stage/cross-filesystem alternative was considered and not selected. Linux and macOS support remain required; an empty hardware admission registry and ext4-only restriction were withdrawn. Process interruption and injected IO failures are tested; unconditional hardware power-loss survival and simultaneous multi-directory visibility are not promised.

All original W01–W17 obligations remain with the adoption/platform/cross-mount revisions recorded in the binding. Changing each of four bound flat consumer signatures would affect1806 cells (7224 combined); changing either nine-profile signature would affect16254. The chosen ordinary defaults avoid unnecessary declaration changes. Actual signature/profile changes still require new behavioral accounting; helper/body edits still need review and direct same-source execution. No dead wrapper, hidden context or baseline expansion is allowed.

Wave19 uses one implementation unit. Four original stories and the separate recovery implementation were outstanding after wave18; this selects only output ownership. The earlier consumer remediation is published at1e618d2 with CI34194913016 and documentation validation34194912907 successful; all three owned wave18 worktrees and their exact outputs were retired after complete evidence retention. No tag, version bump, release or downstream publication is selected.

## Wave19 implementation checkpoint

Root selected the concrete adoption spelling: --owner FAMILY; standalone owners additionally require --file NAME as one native filename. Tree/compose owners reject --file. Output management is visible under ess generate output and keeps its hidden ess output flat alias. This refines the selected binding without widening ordinary bound generator signatures.

At source base11bf599fa4680ee4adb8d93e3523f86ec0b30cab, the CLI implementor observed three new behavioral failures: unowned destination overwritten with exit0, withdrawn site publication left stale, and no enrollment state. The runner printed 0 passed/3 failed/0 ignored and exited101. Original streams are retained in the unit's target/review-output-ownership-wave19/first-behavioral-red; root separately read all three assertions and retained stream hashes. They establish the current defects, not a completed fix.

The initial CLI package baseline had five environment panics across four targets because root assigned a long TMPDIR inside Git. The corrected short external TMPDIR /home/timo/.cache/e19-tmp is on the same SSD. The implementor then observed all114 cases in those four targets pass, actual0; the original baseline is preserved and its assertions are unchanged.

Root added repository projection-sync preflight with four meaningful preservation tests. The initial xtask baseline observed105 unit passes and4/5 layout passes: the new wave page's verbatim command output quoted14 future paths where the checker requires current paths. Root moved that raw output unchanged into the established review-evidence location and linked it from the plan; the layout rerun observed5/5 passes. The bypass mutation then observed2 passed/3 failed among5 sync cases, exit101. Restoring the guard gave109 unit+5 layout=114 passes, package/Clippy/fmt each0. Final source main.rs SHA256: 1ebb62860c73f3d878474ff69648df05e4aa286d1f6e8fed056b1bf1ddf61ade. Actual commands, statuses, streams and producer-PID absence are retained in the unit's target/review-boundaries-19-root-checks. The four new helpers have explicit finite classifications; no baseline eligibility or e005 change was made.

Root's documentation and two-architecture native macOS CI additions remain integration drafts. The native CI lane is branch/PR validation; release backfills keep the checked-out historical Taskfile and existing native packaging authority. CLI ownership/adoption/recovery implementation, its complete fault/process-cut witnesses, native macOS results, independent source review and full integration gate remain owed. This checkpoint is not story completion or source publication.

## Final source review and integration preparation

The corrected source and final additive tests are published to PR14 at
f6ebe903e5753b42c817835204245f4780e7bd44. The complete final adversary report is recorded
unchanged as review-result:output-ownership-adversary-wave19-pass2, SHA256
eb51852d6dcbca41e6eb3b83f5017dc29bf6c8d116fac62947e7b7b0e4beace3.
Its three added cases passed their first isolated executions; the complete touched target
ran7 cases with0 failures, preserving the original230-line prefix. Clippy and formatting
also passed. The computed comparison reports0 carried,0 new and2 resolved findings:
the pass1 count fell from2 to0. No third attack is selected.

Root read the complete report and additive diff, verified78 sealed payloads,1234 source
pins and both native executables. The completed adversary tree and assigned TMP were archived
in full and compared before removal. Exact managed cleanup and branch retirement succeeded.
Reports and archives are retained at
/home/timo/.cache/ess-review/2026-09-06-resume/wave19-adversary-final-retention;
its completion receipt SHA256 is60a317cfdddc30dd7d0902ce3b7c71fc3fbe72b21e142ddd4a9375398608e334.

At f6ebe903, both native Mac jobs passed all7 adversary cases, including the new actual case
and normalization alias branches and exact255-byte filename adoption in a read-only root.
ARM explicitly reports38 ownership+7 adversary+5 correction+6 sync cases. Intel explicitly
reports38 ownership+7 adversary+6 sync cases; its correction step succeeded and announced5
cases, but the ending runner summary is absent from both identical job-log downloads.
No missing runner summary is invented. The actual synthetic checkout52fb1d982f1797e8906c9517ad11e5f29c36e21c
has a tree identical to f6ebe903. Documentation run34227509027 executed task site-build
successfully. This validates source; it is not documentation deployment.

## Final implementation evidence — 2026-09-08 13:49:28 UTC

All required Taskfile lanes are verified at f6ebe903. CI34227509228 actually ran the complete
task check with exit0 and2372 passing cases across202 runner summaries. Local formatting,
strict Clippy, rustdoc, examples, projections, support, consumer coverage, release consistency
and action checks each returned0. The original local workspace test attempt remains201:
its scratch monitor stopped it on an ENOENT while scanning a changing temporary directory.
The corrected scratch monitor tolerated only disappearing descendants; source and test
assertions were unchanged. The retained inventory reconciles38 completed native targets,
22 initially passed ownership cases plus17 continuation passes,143 pending native targets
and20 doctest targets. Fifteen incidental repeated targets are not counted twice. All182
unique native targets plus20 doctest targets account for2356 native+16 doctest=2372 passes,
with no failed or ignored cases. This is explicit continuation, not a relabeled original exit.

Final consumer qualification executed22 cases through72 commands, all actual0, with87 profiles,
1806 models and157122 cells:54 Supported,0 Refused,157068 BaselineUnknown. The source digest is
bc67910a754bd394993cbee4faf27f4242df95b24a28bd56d080ce9efaa21868, identical to the successful CI
source observation. Accepted initial gaps remain unproven; this is not complete consumer support.
Profiles, reviewed candidates and the e005 initial eligibility bytes remain unchanged.
The final consumer report crossed the10GiB scratch cap by43917312 bytes; the precise raw sample
is10781335552 bytes, the monitor attempted INT, and the child still returned0 with the complete
qualified result. No no-resource-stop claim is made. Root verified quiescence and removed only
81571840 bytes of reproducible target/doc output before the last two successful gate lanes.

The exact per-step exits, elapsed times, executable manifest digest, consumer result,
platform observations, resource event and unavailable aggregate agent-cost counters are in
docs/reviews/2026-09-08-output-ownership-closure.json. Retained raw records are selected for
/home/timo/.cache/ess-review/2026-09-06-resume/wave19-final-retention before owned cleanup.
Publication and cleanup follow this implementation evidence; no release or deployment is selected.
