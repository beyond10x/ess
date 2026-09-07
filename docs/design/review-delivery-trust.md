# Delivery evidence trust

Accepted binding for the existing F11 delivery-trust story. Root adopts B01–B07 after the recorded first binding review found no concrete issue and after browser replay closed at published ESS 95ef5be70dbce966056d0484b66db7ed836fa406. The integrated source refresh at 3408bbf049d10215487c50f6f7b5597486b14127 preserves all fourteen reservations and T01–T18; the closing commit changes only planning and the prior wave record. Runtime interfaces remain unimplemented at adoption. The next implementation selection is recorded separately in the wave page. No new model, envelope, format version or remediation objective is introduced. I/N citations refer to integrated source pins and L citations to retained historical receipts, identified below. Original candidate, complete refresh and the mechanical adoption receipt remain under target/review-boundaries-15/preparation/delivery-trust-integration-refresh and target/review-boundaries-16/preparation/opening. This adoption changes source/status annotations only; it is not a second binding review.

The implementation in this change supplies the B02 interfaces and B06 action contract below.
The adoption and source-refresh statements describe their recorded historical stage. They are
not runtime evidence. Runtime results are retained by the implementor and integration coordinator;
the B01–B07 guarantees and T01–T18 obligations remain the acceptance contract.

## B01 — Existing wire and claims

Keep ess-release/1, ess-release-bundle/1, their canonical JSON/YAML behavior, four required EvidenceKind entries, every current reader rule and all historical digests unchanged. Newly produced releases naturally have different digests when their actual attachment references/digests differ; “preserve digests” does not mean reuse an old digest for new bytes. No new evidence kind, persisted result, envelope, authentication status or arbitrary context map is introduced. [I08; I09; I35]

Plain verify, bundle, verify-bundle and publish establish the existing metadata/graph/digest consistency checks. Fetch additionally checks the existing OCI manifest/descriptor/blob content identity under its admitted cache profile. None independently validates the referenced evidence attachment's contents, the producer's authority, a signature, or execution of the referenced runtime/chart artifact. No successful route may say evidence-verified, authenticated conformance, verified SLSA or verified attestation. Library API names may remain for compatibility; document their actual guarantees. [I08:168–243; I09:258–402; I15:1400–1527; I16:117–236; I32]

| Required kind | Policy after candidate adoption |
|---|---|
| provenance | Declared attachment. Content, completeness, subject claims and producer origin unverified. The action's summary is not a verified SLSA attestation. |
| sbom | Declared attachment. No independent SBOM content/completeness validation. |
| signature | Declared attachment. Signature verification and trust-root/issuer policy unsupported. Existing signing can continue under the caller's existing tool authority. |
| conformance | Existing release metadata remains a declared attachment. Legacy logs are not retroactively reports. New action publication requires a separately supplied locally qualified report/2; even that establishes only its exact declared selection, not authenticated execution or remote attachment contents. |

Evidence.digest continues to mean the OCI attachment manifest digest returned by publication. SHA256 of the raw report is a separate local byte identity. Never compare those as if they named the same bytes. [I30:129–163]

## B02 — Concrete CLI adapters

The grouped spellings below have the existing flat `ess release …` aliases with identical program-produced streams/status. The original preparation did not execute them; this change implements them.

1. `ess generate release check-conformance`: offline positive qualification only.
2. `ess generate release publish-conformance`: the same qualification followed by a narrow ORAS upload of the admitted original report bytes. This second adapter is necessary to avoid a shell uploader reopening a mutable caller path after a separate check.
3. Existing `ess generate release publish`: preserve plain consistency-only behavior; add an optional all-or-none conformance qualification group. The release action always supplies this group. The qualified branch admits the report/model/input and bundle before any ORAS invocation and stages the same checked bundle value.

Shared local-report options:

| Option | Meaning |
|---|---|
| `--spec PATH` | Explicit authored ESS model root, loaded/compiled once through the existing CLI owner. Not a hash inferred from the submitted report. |
| `--report PATH` | Original UTF-8 standalone ess-conformance-report/2 JSON bytes. |
| Exactly one `--expected-suite PATH` or `--expected-suite-input PATH` | Independently supplied expected selection: an original unfiltered suite/5, or an original ess-conformance-input/1 carrier including all parent suites. No automatic document guessing or report-derived expectation. |
| `--report-sha256 sha256:…` | Optional full raw-report byte pin. The action always supplies it. It is not Evidence.digest. |
| `--expected-input-sha256 sha256:…` | Optional full raw selected-suite-file/carrier-file byte pin. The action always supplies it. For a carrier, this is not the selected inner suite digest. |

For check-conformance and publish-conformance also require `--component-ir PATH --build-ir PATH --runtime-ir PATH`. These are existing canonical compiled JSON inputs, read and admitted once, with exact canonical-byte checks on this new profile. For publish-conformance require `--to TAGGED_OCI_DESTINATION`; no digest destination. Qualified existing publish instead obtains component/build/runtime and release context from its admitted canonical `--path BUNDLE`, and retains its existing `--to`. Any partial qualification group or pins without qualification is a usage refusal; there is no allow-failed, allow-legacy, skip-check or evidence-authenticated switch.

The extra publish-conformance command adds no file reservation beyond main.rs/release_evidence.rs and their tests. It is explicitly an external publication operation; check-conformance remains offline. Existing five routes retain their positional/options behavior; the two new routes make seven release subcommands. The main/help and public-page source refresh is complete at 3408bbf049d10215487c50f6f7b5597486b14127; root still performs the post-browser replan before selection and refreshes again if relevant source changes.

Examples:

```sh
ess generate release check-conformance \
  --spec ess/model --report evidence/report.json \
  --expected-suite-input policy/expected-input.json \
  --component-ir target/release/component.ir.json \
  --build-ir ess/compiled/build.ir.json --runtime-ir ess/compiled/runtime.ir.json

ess generate release publish-conformance \
  --spec ess/model --report target/release/conformance-report.json \
  --expected-suite-input target/release/expected-input.json \
  --component-ir target/release/component.ir.json \
  --build-ir ess/compiled/build.ir.json --runtime-ir ess/compiled/runtime.ir.json \
  --report-sha256 "$report_bytes_digest" --expected-input-sha256 "$input_bytes_digest" \
  --to "$EVIDENCE_REPOSITORY:$COMPONENT_VERSION-conformance"

ess generate release publish \
  --path target/release/ess-release-bundle.json --to "$BUNDLE_REPOSITORY:$COMPONENT_VERSION" \
  --spec ess/model --report target/release/conformance-report.json \
  --expected-suite-input target/release/expected-input.json \
  --report-sha256 "$report_bytes_digest" --expected-input-sha256 "$input_bytes_digest"
```

The independently supplied expected document is the policy input: its exact admitted scope, origins and all/explicit filter (including IDs and parent reference) constitute the expected selection. No separate free-form Selection envelope is necessary. For an unfiltered suite substitute --expected-suite. The caller must choose that document independently of the report. ESS can enforce equality to supplied bytes, not prove the administrative independence or approval of whoever supplied them. Supplying a report's own claims back as “expected” does not establish independent authority.

## B03 — Admission, model and selection checks

Run in this order within each process, using the same owned values thereafter:

1. Read original report and expected input as UTF-8 buffers once; check any raw-byte pins. Admit a direct suite with AdmittedSuite::from_json then AdmittedInput::from_suite, or a carrier with AdmittedInput::from_json. Keep the selected suite and every original parent string. No reserialization before identity checks, no SuiteInputDocument-only shortcut, and no fallback from a malformed carrier to a permissive parser. [I22:60–116; I23:710–835; N04:151–175]
2. Load/compile the explicit model once through existing resolved/load logic. Derive SuiteProvenance::of(&ir), not a new digest algorithm. Require exact system and specification-version equality, full 64-lowercase-hex spec_digest and contract_digest equality against that model provenance. A legal legacy short SpecDigest cannot satisfy full identity. Resolve any selected model component in ir.components(); require its name to agree with suite provenance and admitted coverage scope. Do not equate the delivery descriptor's component identifier with a modeled component name without a declared relation. [I43; N02:1334–1341,1575–1587; N03:235–305]
3. Check existing component/build/runtime owners and relationships. Require component system and semantic_version to equal the compiled model identity; require runtime.semantic_digest() == Digest::new("sha256:" + ir.source_digest()); use runtime.validate_against_build and existing bundle/release validation. Release SemVer remains independent from semantic vN. This checks a supplied model's equality with declared release context; it does not recompile or certify the physical realization. [I09:123–134,314–402; N01:435–447,482–520]
4. Parse original report bytes with CountReport::from_json(report, admitted.selected()). Its owner enforces exact suite reference/profile, model identity, sorted/disjoint/exhaustive outcome membership, exact-token counts, producer outcome semantics and coverage equality. Then require conformance_status() == CountStatus::Passed. This entails nonempty selection, complete inventory, no in-scope refusal and all-pass execution. Do not rebuild its verdict from a permissive JSON value or accept an independently parsed detailed run summary as a standalone report. [I21:210–345]
5. In qualified bundle publish, use the same verified ReleaseBundle for assessment, diagnostics and staging. Validate every runtime/chart release; print its exact declared unit, output/kind, reference/digest and platforms, source_commit, build/runtime/model identities. A changed but internally consistent artifact digest cannot be rejected as “not executed” by this report: execution binding is unverified in both cases. Never convert local model/selection equality into a claim that either artifact ran.
6. Record only the local scope actually checked. Whole-system, component-scoped, authored-only, generated-only and explicitly filtered expectations can each qualify for their own nonempty complete selection. None implies a larger selection. Existing count-stage/unknown coverage, empty selection, in-scope refusals and negative/inconclusive outcomes refuse these positive gates. Valid report/1 summaries retain their existing readers and meanings but always refuse this new exact-selection gate. [I20; I21; I23:358–365]

No regeneration of suites from the model is claimed to prove honest inventory enumeration. CompleteInventory is an admitted producer claim under the existing contract. Model equality, byte pairing and checked inventory arithmetic do not authenticate its origin.

## B04 — Same-process byte ownership and publication limit

The assessment helper receives owned original report/input buffers, one compiler-produced model value and admitted deployment values. After successful admission, no helper, diagnostic builder or publisher reopens a caller report/input/model/bundle path and treats the new contents as already checked.

For publish-conformance: create a private temporary staging directory; write the exact original admitted report buffer to a fixed file there; invoke ORAS with that staged file from the same owning process; keep staging alive until the child terminates. Use the existing artifact type application/vnd.beyond10x.ess.evidence.conformance.v1 and layer media type application/json. Check ORAS status, UTF-8 and Digest syntax using existing owners. Do not canonicalize, summarize or replace the report bytes before upload. No expected-input/model source bytes need be published. Their locally admitted identity is used for checking, not added to the attachment envelope.

For qualified bundle publish: read/canonically admit the bundle once, perform the final local report/input/model checks, then stage the canonical bytes of that same admitted bundle as the existing publisher does. The accepted canonical-input equality makes those staged bytes equal the original checked bundle bytes. Do not call verified_bundle a second time after assessment. Report/input buffers and the one compiled model value remain alive for the whole operation. Model source files are loaded once through the existing compiler edge; this establishes the identity of that loaded model, not an atomic filesystem snapshot or an attestation of source files. [I15:1474–1513,3162–3183]

These operations establish which local bytes ESS supplied to the invoked child. ORAS's reported manifest digest is syntactically checked, not independently reconstructed or remotely authenticated. The final bundle's conformance attachment reference/digest remains a declared OCI attachment relation. Even when the action obtains it from the immediately preceding publish-conformance invocation and assigns it to both releases, /1 gives a receiver no manifest/blob proof linking that reference to the supplied local report. Never claim receiver-side attachment verification.

The action retains original report and expected-input byte snapshots and their raw digests before its generic check. Pass those pins to every later gate so a changed snapshot between invocations refuses. The same-process rule protects use after admission within an invocation; it is not a new hostile-local-writer, tool-binary-admission or credential-authority protocol. Concurrent hostile mutation of the executor's private staging and dishonest external tools are not newly supported guarantees. Offline tests must nevertheless mutate caller files after admission and verify that the staged payload remains the admitted buffer.

## B05 — Observable results and compatibility

Existing JSON/YAML stdout and output-file bytes remain unchanged for all existing routes. Change misleading text success wording to “consistency checked” or, for fetch, “bundle content identity and consistency checked”. Keep existing publish stdout exactly `<destination> — published at <OCI manifest digest>\n`; final token remains the manifest digest consumed by release.sh. New publish-conformance uses the same one-line shape with its evidence destination/digest. All qualification text goes to stderr; never append it after the publish digest on stdout.

Deterministic stderr, in fixed order:

- Existing consistency routes: `release consistency: checked`; fetch additionally `bundle content identity: checked`; other routes must not claim that independent content check.
- Without supplied qualification: `conformance: not assessed`.
- Positive local qualification: `conformance: passed for the supplied exact declared selection`, followed by the selected suite digest, compact existing Selection serialization, full model/contract identity and raw report hash. Initial check additionally names supplied component/build/runtime identities; final bundle publish names each release/artifact/platform context in ordered map order.
- Always: `attachment binding: unverified`; `producer origin: unverified`; `artifact execution: unverified`; `signature verification: unsupported`.

check-conformance success is exit 0, empty stdout and the qualification text on stderr. Any data/admission/model/selection/qualification failure is exit 1, no success line and no external process; CLI shape errors retain Clap usage refusal semantics. Existing consistency-only success/failure statuses are preserved. A coherent negative report may remain readable in its existing reader but fails these new positive operations. No machine-readable assessment document is added, and the human diagnostics are not an authority receipt.

## B06 — Release-action migration and sequence

Mandatory new caller contract:

| Action input | Requirement / environment |
|---|---|
| `spec-path` | Required explicit authored ESS root → SPEC_PATH. |
| `conformance-report` | Required pre-existing separately supplied report/2 file → CONFORMANCE_REPORT. No fallback to CHECK_COMMAND stdout. |
| `conformance-suite` | Original expected unfiltered suite/5 → CONFORMANCE_SUITE. |
| `conformance-suite-input` | Original expected input/1 carrier with complete parents → CONFORMANCE_SUITE_INPUT. Exactly one of these last two is required; both default empty individually, and release.sh enforces XOR before publication. |

The caller must provide the report and independent expected selection before this action runs. Existing check-command remains a generic repository check and cannot satisfy the conformance input by printing JSON or returning zero. If a caller wants that command to produce the report, it must move report production to a prior step and supply the resulting file explicitly. This is a breaking action-input contract even though the bounded consumer searches returned zero.

Action order:

1. Check required inputs/XOR and installed ESS executable; compile component IR and retain its existing system/service comparison. Copy the supplied report/expected document bytes to distinct action scratch paths, capture their raw SHA256 pins, and invoke check-conformance with the supplied compiled IR and model before CHECK_COMMAND, Buildx execution, image adoption inspection, Helm push, signing or evidence publication. Missing, malformed, wrong-model or nonqualifying input stops here.
2. Run CHECK_COMMAND with its existing failure propagation and retain `target/release/check.log` as a generic local/job check log, outside the four-kind evidence map. It is not published as conformance or smuggled into a renamed conformance file.
3. Continue the existing build-image true/false, chart, signing and SBOM steps. Keep their limits: fake tools in tests establish routing only, and real tools do not establish ESS consumer-side authenticity.
4. Immediately before any evidence upload, re-run the same offline check with original snapshot pins and current supplied model/deployment context. Failure blocks all evidence and bundle uploads from that point; already completed image/chart operations need not be rolled back. This is not a transaction/recovery contract.
5. Upload provenance/SBOM/signature with their existing declared-kind publisher; upload conformance only through publish-conformance using the retained snapshots/pins. Construct the exact existing four-kind map from returned manifest references/digests and use it for runtime/chart releases. Capture child failures reliably; do not rely on a process-substitution `read` hiding the publisher's nonzero status. Generic evidence JSON is not subjected to new unowned attestation verifiers.
6. Keep existing release verification and canonical bundle creation. Invoke existing publish with its required-for-action qualification group. The process must perform final local model/report/input/bundle checks before staging and ORAS. Failure after earlier uploads leaves possible unreferenced artifacts; no rollback guarantee is added. Keep the final stdout digest parsing and summary's exact runtime/chart/bundle references; add conservative qualification wording to the summary without an authenticated claim.

Callers update both the action revision and its pinned ESS revision to versions containing these interfaces, then supply the new inputs. A new action with an older ESS executable must fail on the missing command before release work, not downgrade. An old action with a new ESS executable still produces old declared logs and cannot claim the new gate; existing /1 readers remain conservative. No concrete future commit/tag is fabricated in examples.

The retained local search covered 38 exact local Git heads; the App-authenticated organization code search returned total_count 0 and incomplete_results false for its recorded literal query. Neither proves the absence of other consumers, aliases, wrappers, private/out-of-scope repositories or future callers. Record this bounded no-known-caller disposition and document the required migration. Do not repeat network discovery. [L05–L08]

## B07 — Model/scope boundary

Use existing CountReport, CountStatus, AdmittedInput, AdmittedSuite, Selection, SuiteProvenance, Digest, ComponentIr, BuildIr, RuntimeIr and ReleaseBundle capabilities. Private edge argument/grouping code and ownership of admitted buffers do not create a new persisted evidence model. No new field, relation, signer, issuer, authority object or cross-crate assessment API is selected. If implementation needs such a contract or remote attachment proof, stop that extension and return the exact design/model/version decision to root.

The complete 14-path candidate scope is unchanged. The only refinement from the original scoper is the narrow publish-conformance operation within already reserved edge/action/test files. No conformance-reader, cache, format catalog, Taskfile, support-matrix generator, primitive, recovery or sibling source change is required. The existing support rows do not inventory release subcommands, so new release help does not itself justify a support.rs/status-page write. [I41:482–504]

All T01–T18 obligations remain mandatory in the unchanged acceptance/fixture section below and in work-order-draft.md. Root accepted this design direction before implementation. None of these success/refusal tests was executed by the historical source refresh; implementation evidence is retained separately.

## T01–T18 acceptance and fault matrix

“Accept” below means the named consistency or local selection operation succeeds, never that the evidence is authenticated. Zero cases were executed during this preparation.

| ID | Finite variants / required result | Executable owner |
|---|---|---|
| T01 | All four arbitrary placeholder evidence references/digests in otherwise matching release/build/runtime remain consistency-valid through JSON, YAML and nested bundles. Every success leaves attachment/origin/artifact execution unverified and signature verification unsupported. | Existing deployment fixtures + new delivery_trust integration. |
| T02 | Consistently rehashed invented/tampered evidence remains only consistency-valid. Rehashed invalid nested IR, changed owned output/kind and duplicate map entries refuse before bundle publication. Neither hash replacement nor nested inclusion can yield an authenticated status. | deployment.rs; persisted_delivery controls; delivery_trust. |
| T03 | Plain log, exit-zero generic check output, non-report JSON, unsupported /99 and detailed run/2 passed as standalone report all refuse both new positive routes and qualified bundle publish before ORAS. Generic check execution remains available independently. | Real CountReport reader through each CLI adapter. |
| T04 | Existing valid report/1 passed/failed/error/unsupported/skipped/empty summaries retain current reader semantics and bytes. Every /1 refuses the new exact-selection gate, including a legacy “passed” summary. | report_reader_adversary and delivery_trust. |
| T05 | /1 unsupported version, failed greater than total, list/count disagreement, contradictory status, unknown fields and duplicate identity keys still refuse through the actual reader; no bespoke detector replaces it. | Existing report_reader_adversary regressions. |
| T06 | Nonempty complete unfiltered suite/5 + paired report/2 passes the strict check. Repeat with admitted filtered carrier and each Rust/Go producer profile's all-pass terminal vocabulary. Outputs name only the exact selection/model and retain all unverified qualifiers. | count_reports; coverage_admission; delivery_trust. |
| T07 | Whitespace-only changed original suite bytes, wrong suite digest/profile/version/specification, changed full spec_digest or contract_digest, substituted model and short legacy digest refuse pairing/model gates. Do not normalize before hashing. | Production admission + model compilation in delivery_trust. |
| T08 | Report for a different component/system, authored-only/generated-only/both origin expectation or explicit subset cannot satisfy the independent expected input. The supplied narrow expectation can pass for itself; an empty explicit subset cannot. Unknown model component refuses. Delivery-component label is not silently treated as a modeled-component relation. | Actual admitted selection/model helpers. |
| T09 | Full original parent chain succeeds; missing, wrong, reordered, duplicate, cyclic/repeated or extra parent fails. Carrier-only structural parse is insufficient; altered retained payloads/dependencies or reserialized inner bytes refuse. A filtered raw suite without its carrier refuses. | coverage_admission + CLI integration. |
| T10 | Zero scenarios, unknown coverage, in-scope refusal, failed/error/unsupported/skipped outcomes, contradictory producer profile and inconclusive status block both report upload and qualified bundle publication. Out-of-scope exclusions retain their existing meaning; no new whole-system coverage requirement replaces exact-selection qualification. | CountReport::from_json/status; all three qualified routes. |
| T11 | Overflow, fractional/exponent/negative scalar tokens, repeated/overlapping IDs, incorrect total/membership/coverage summary and duplicate keys fail the exact-token production reader. Also reject partial CLI groups, both/neither expected-input variants and malformed raw-byte pin syntax before effects. | count_reports controls + command_surface/delivery_trust. |
| T12 | Matching compiled model/component semantic identity/build/runtime passes; changed full model or incompatible build/runtime fails early. Final bundles identify both runtime and chart contexts, output names/kinds, reference/digests/platforms and source commit. A consistently changed artifact/platform/source context still never establishes that artifact execution occurred. Keep release SemVer distinct from semantic version. | Real deployment/model owners, bundle_fixture and delivery_trust. |
| T13 | Raw report SHA and OCI manifest SHA deliberately differ. Fake ORAS receives the exact admitted original report bytes with application/json and the existing evidence artifact type, then returns an unrelated valid manifest digest that becomes Evidence.digest. Bad digest text/non-UTF8/nonzero child status fails publication. No remote manifest/blob proof or authenticity claim follows. Mutate caller report/input/bundle paths after admission: staging must retain admitted buffers/values; tests must not reopen the mutated paths as expected data. | Rust fake_release_component fixture + actual publish-conformance/qualified publish. |
| T14 | Successful fake cosign signing/triangulation and opaque signature/provenance/SBOM contents never produce signature/SLSA/producer verification claims. Plain validators/check-conformance launch no ORAS, docker, helm, cosign, syft or network process. Existing signing is separate action activity. | Invocation log and actual CLI/action text assertions. |
| T15 | Run actual release.sh with actual ESS and Rust external-child fixtures. Missing/invalid required report/model/expected input stops before CHECK_COMMAND, builds/adoption inspection and publication. Generic check log never reaches a conformance upload. Negative policy is refusal. Snapshot-byte changes after the first check are caught by raw pins before evidence upload/final bundle publish. A publisher failure propagates even when the old process-substitution pattern would mask it. | delivery_trust; fake_release_component.rs. |
| T16 | New action input/env/XOR contract, absent report fallback, old-ESS/missing-command failure, check-command failure, build-image true/false, image child-platform fallback, chart digest extraction, runtime/chart reference assignment and final summary digest remain explicit. When failure occurs after earlier chart/image uploads, assert no later evidence/bundle call; do not assert rollback of earlier effects. | Actual action script + bounded external-tool fixture. |
| T17 | Seven release help routes/aliases agree. Existing canonical JSON/YAML stdout/output files are byte-identical; text clarifies consistency; new check stdout is empty; both publishers end stdout with their manifest digest; qualification goes only to stderr. Exercise grouped/flat status+stream parity, qualified and plain publish, real cold/warm fetch with the existing offline OCI fixture. Preserve existing content proof and all cache refusal lanes. | command_surface; delivery_trust; cache_origin read-only controls. |
| T18 | Existing mutable-field, nested reader, duplicate-key, report/coverage/cache gates pass. Source comments, action description, design, CLI reference, component-delivery guide and formats reference agree on B01–B07 and migration. Legacy release/bundle canonical bytes/digests are unchanged. No new envelope/signature verifier/authority/cross-repo code appears. | deployment tests; existing gates; focused documentation review. |

The new primary owner is crates/edge/ess-cli/tests/delivery_trust.rs. The only new external process fixture is crates/edge/ess-cli/tests/support/fake_release_component.rs, compiled as a test-only Rust executable using the existing process-fixture pattern. It handles only the finite docker buildx bake/imagetools, helm show/push, cosign sign/triangulate, syft output and ORAS publication/read calls exercised by this matrix; every unexpected invocation refuses. Record argv, status, exact staged payload bytes/media types and call ordering. Use an explicit synchronization point for T13 rather than sleep-based timing. No shipped bypass, external credentials or fake replacement for ESS parsers, model loading, consistency checks or qualification code.

Keep actual Bash/jq and the action's ordinary local file/text utilities real. Missing offline executable prerequisites are a lane failure or explicit unavailable result, never an executed-case pass. Native images/charts/signatures/registries are simulated: this proves CLI/action routing and local admission/staging only. Existing bundle_fixture.rs and fake_delivery.rs remain read-only reusable evidence/test helpers unless root first admits an exact new reservation.


## Source-reference key

I/N entries below are Git blobs at 3408bbf049d10215487c50f6f7b5597486b14127. Readback binds each complete byte count/hash and distinguishes fresh semantic extents from inherited hash-only inputs. L05–L08 are historical discovery receipts, also summarized in the recorded review-result at .engineering/planning/review-result/delivery-trust-binding-pass1.md. They do not establish current remote state or runtime evidence.

| ID | Exact owner or retained receipt |
|---|---|
| I08 | crates/generate/ess-deployment/src/release.rs |
| I09 | crates/generate/ess-deployment/src/component.rs |
| I15 | crates/edge/ess-cli/src/main.rs |
| I16 | crates/edge/ess-cli/src/oci_cache.rs |
| I20 | crates/verify/ess-conformance/src/evidence.rs |
| I21 | crates/verify/ess-conformance/src/counts.rs |
| I22 | crates/verify/ess-conformance/src/admission.rs |
| I23 | crates/verify/ess-conformance/src/coverage.rs |
| I30 | .github/actions/release-component/release.sh |
| I32 | docs/design/review-cache-origin.md |
| I35 | docs/design/review-format-catalog.md |
| I41 | crates/edge/ess-xtask/src/support.rs |
| I43 | crates/specify/ess-primitives/src/evidence.rs |
| N01 | crates/generate/ess-deployment/src/runtime.rs |
| N02 | crates/specify/ess-compiler/src/ir.rs |
| N03 | crates/verify/ess-conformance/src/scenario.rs |
| N04 | crates/edge/ess-cli/src/coverage.rs |
| L05 | /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-15/preparation/delivery-trust-scoper/root-local-consumer-inventory.json |
| L06 | /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-15/preparation/delivery-trust-scoper/remote-consumer-discovery/command.json |
| L07 | /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-15/preparation/delivery-trust-scoper/remote-consumer-discovery/search.stdout |
| L08 | /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-15/preparation/delivery-trust-scoper/remote-consumer-discovery/search.stderr |
