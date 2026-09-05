---
format: aep.planning-md/1
id: review-result:review-boundaries-3-conformance-design-adversary-pass-1
kind: review-result
status: active
title: Wave 3 conformance design adversary pass 1
relations:
- reviews: story:review-conformance-format-design
revision: 1
---
unit: story:review-conformance-format-design — commit 5148543b57c855cb4ccca92e2368e566801e9c36, adversary pass 1
verdict: NEEDS-CHANGE
cases: not applicable — document-only source and matrix review; no runtime cases executed
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: route the binding-design findings for correction; no source patch or lifecycle action performed

```console
git --no-pager diff --stat
```

```text
```

Exit: `0`.

## 2. Cases and execution boundary

No executable case was added. The assignment is a binding-design review with all tracked files read-only; the brief explicitly prohibits manufacturing prose tests and compatibility execution claims. The findings below are source-backed judgement findings, not test-runner evidence. No 0→0 count is offered as verification.

## 3. Observed review commands and matrix coverage

```console
git rev-parse HEAD
```

```text
5148543b57c855cb4ccca92e2368e566801e9c36
```

Exit: `0`.

The complete new-file diff was read with:

```console
git --no-pager diff 45832cc885377b2d61845ee33af14f0293d99e67...5148543b57c855cb4ccca92e2368e566801e9c36 -- docs/design/review-conformance-coverage.md
```

The actual reviewed target is the 40-character commit in the header; the command transcript immediately below records the exact executed command and output header.

```text
diff --git a/docs/design/review-conformance-coverage.md b/docs/design/review-conformance-coverage.md
new file mode 100644
index 0000000..be48b5a
--- /dev/null
+++ b/docs/design/review-conformance-coverage.md
@@ -0,0 +1,246 @@
```

Exit: 0. The diff adds the entire 246-line document, which was also read in numbered sections covering lines 1–246. The contradictions are introduced in that document; the source behavior used as a counterexample predates the unit.

The complete 36-row matrix was inspected. These are review observations, not executed outcomes:

| Matrix rows | Ground examined | Result of this review |
| --- | --- | --- |
| M01–M04 | Rust v1 reader/writer, Go v1 writer, both AEP readers, frozen version dispatch | Existing v1 semantics and differing readers are identified; the missing v5/v1 pairing is F2. |
| M05–M08 | Rust per-check versus per-scenario aggregation, Go skipped/error and teardown | Source supports the specified profile split, including unsupported plus separate error and teardown overriding a skip. |
| M09–M11 | Closed report shapes, count partitions, exact scenario membership, missing Go subtests | New invariants are specified; no execution claim was made. The current Go status initialization explains the filtered-subtest risk. |
| M12–M18 | SuiteFormat parser, Runner entry, reduced Go struct, browser fetch/replay | Reader-first admission and retained old-runtime cases are necessary and correctly distinguished from an automatic refusal caused by a version increment. |
| M19–M26 | Empty, component, authored/generated, explicit selection and refusals | Empty/incomplete qualification is separated from execution. F1 prevents representing existing partial-check and duplicate-source producer states. |
| M27–M31 | Exact original bytes, metadata coverage, canonical writer rules and scalar implementations | Raw-byte identity avoids lossy Go reserialization; deterministic encoder vectors remain explicitly future implementation work. |
| M32 | Detailed CLI versus standalone output | Current main.rs has separate report.standalone() output and detailed report serialization; the design treats them separately. |
| M33 | Impact provenance and dependency intersection | The consumer is real and not an execution verifier; F3 identifies the incorrect persisted version. |
| M34 | Realization ConformanceEvidence | Source establishes artifact references, not a byte-verifying report parser; the design preserves this distinction. |
| M35 | AEP adapter, planning reader, closed domain result and facts | Both readers and the domain/predicate migration are named; old scenarios.failed meanings are not silently narrowed. |
| M36 | Legacy format selections after new defaults | Legacy preservation is stated, but the version-pairing/strict-mode table must be completed under F2. |

```console
rg -n '^\| M[0-9][0-9] \|' docs/design/review-conformance-coverage.md
```

```text
183:| M01 | Valid existing Rust v1 passed/failed/error/unsupported reports | Admit unchanged under v1 rules and preserve bytes; do not infer coverage | R, reader stage |
184:| M02 | Valid existing Go v1 passed/failed/skipped reports | Admit unchanged; scenarios_failed remains all non-passes | R/A, reader stage |
185:| M03 | A v1 report with category fields added without changing format | Closed v1 reader refuses | R/A |
186:| M04 | v2 presented to an existing v1 report reader | Refuse report/2; no count reinterpretation | R/A fixtures using retained old reader |
187:| M05 | Valid paired v2 Rust unsupported + separate error scenarios | unsupported=1, error=1, failed=0; execution_status and conformance_status failed | R |
188:| M06 | Rust error + unsupported checks within one scenario | One error result; execution_status inconclusive | R |
189:| M07 | Go skipped-only run | skipped equals total, failed=0; execution_status/conformance_status inconclusive; strict process fails | R |
190:| M08 | Go ordinary error or skipped followed by teardown error | One final failed result; no error/unsupported/skipped contribution for it | R |
191:| M09 | Unknown profile, outcome word, report field, or report/3 | Refuse before adaptation; never coerce to failed/skipped/passed | R/A |
192:| M10 | Counts do not sum, overflow, duplicate IDs, list/count disagreement, or mixed profile vocabulary | Refuse v2 report | R/A |
193:| M11 | Paired v2 with an omitted or extra scenario result | Refuse complete report, including results fabricated for filtered Go subtests | R/C |
194:| M12 | Valid old suite/1–4 with its own supported vocabulary | New readers admit legacy execution; v2 exact report remains unknown coverage | C |
195:| M13 | New suite/5 with complete inventory and every selected scenario passed | Paired v2 conformance_status passed for that selection | C/R |
196:| M14 | Future suite/6 or /99 using only familiar steps | New readers refuse before any target callback or browser replay | C |
197:| M15 | New vocabulary mislabeled with an earlier suite major | New readers refuse before execution | C |
198:| M16 | Unknown typed suite field/step/coverage code or duplicate JSON key | New readers refuse before execution; payload-map keys remain data | C |
199:| M17 | Today's Rust runner or generated Go runtime receives suite/5 | Do not claim version-based refusal: these sources lack the check; deployment pairing is prohibited until upgraded/regenerated | C/O retained-runtime fixture |
200:| M18 | Today's browser player receives new suite metadata | Do not claim coverage/digest verification; publish the updated player with the matching suite first | C/O browser fixture |
201:| M19 | Empty suite and zero outcomes | execution_status passed may remain; conformance_status inconclusive; strict refuses qualification | R/C/A |
202:| M20 | Whole-system generated_and_authored, complete inventory, no refusals | Nonempty all-pass execution qualifies for that exact selection | C/R |
203:| M21 | Component selection, complete in-scope inventory, known outside scenarios | Can qualify for the named component; cannot satisfy a system expectation | C/A |
204:| M22 | Authored-only or generated-only selection | Can qualify only for the same declared origin selection; combined requirement refuses it | C/A |
205:| M23 | Explicit subset with admitted parent, requested IDs and accounted omissions | Can qualify only for that explicit selection; broader requirement refuses it | C/A |
206:| M24 | Explicit subset hides a refusal or loses parent/ID binding | Refuse suite admission; cannot claim complete inventory | C |
207:| M25 | Nonempty all-pass selected suite with an in-scope synthesis/authored refusal | execution_status passed, conformance_status inconclusive; strict fails | C/R |
208:| M26 | Legacy suite rebuilt without original inventory, or unknown coverage | Diagnostic execution allowed; complete qualification refuses | C/R/A |
209:| M27 | Same spec_digest but changed authored step, dependency, selection, refusal or coverage list | Exact suite digest changes; old expected/report binding refuses | C/R/A |
210:| M28 | Correct report with a different exact suite, missing suite, or model digest mismatch | No qualifying evidence; paired mismatch refuses, missing bytes remains unverified | C/R/A |
211:| M29 | Whitespace/key order/default omission/number or Unicode escape/newline rewrite | Exact byte digest changes even if decoded values compare equal | C/R |
212:| M30 | Go reduced execution struct is marshaled to calculate digest | Fixture must fail; only retained complete suiteJSON may be hashed | C/R |
213:| M31 | Numeric precision edge, signed zero, combining Unicode, escaped controls | Golden canonical bytes and raw-byte hash match the specified profile; no silent normalization | C/R |
214:| M32 | Detailed CLI JSON/YAML treated as standalone report | Format/surface admission refuses the mismatch | R/A |
215:| M33 | Impact on a v5 component/subset suite with matching model provenance | Compute scenario invalidation within that selection; do not claim whole-system execution coverage | C |
216:| M34 | Existing realization digest reference without referenced bytes/profile bridge | Remain an artifact reference, not verified exact-suite conformance | C/O |
217:| M35 | New report sent through old AEP domain result/predicates | Refuse v2 adaptation until the new domain/policy route exists; do not narrow the old failed fact | A |
218:| M36 | Report/1 and suite/1–4 explicitly requested after default movement | Preserve documented legacy behavior/bytes and mark coverage unavailable | R/C/O |
```

Exit: `0`.

AEP was inspected only through Git objects at 00c742e4179593738a2e8aa69e2ecc07d3c89402, including:
- crates/observe/aep-ess-evidence/src/lib.rs:15, :20, :138–167;
- crates/edge/aep-cli/src/planning.rs:5907–5985;
- crates/govern/aep-domain/src/evidence.rs:1006–1051 and :1862–1889.

The following read-only comparison emitted no diff and exited 0:

```console
git -C /home/timo/beyond10x/aep --no-pager diff cc321f31fa0120b32a5b9f5e7b8c8fdfa55f69f9 00c742e4179593738a2e8aa69e2ecc07d3c89402 -- crates/observe/aep-ess-evidence/src/lib.rs crates/edge/aep-cli/src/planning.rs
```

```text
```

Atlas's stated ADR/version/shipped-log requirement was read from Git object 6035d6e1209686ca474a3f43975fde7d8621ba48:AGENTS.md, its Cross-repo changes section. No current deployed-version assertion, AEP execution, Atlas change or dirty sibling working-tree authority was used.

```console
git --no-pager diff --check
```

```text
```

Exit: `0`.

## 4. Findings

All findings cover the new binding design at commit 5148543b57c855cb4ccca92e2368e566801e9c36.

| ID | File:line | Category / severity | Verdict | Origin | Finding |
| --- | --- | --- | --- | --- | --- |
| F1 | docs/design/review-conformance-coverage.md:95 | acceptance / blocker | NEEDS-CHANGE | introduced | The refusal-ID disjointness rule cannot preserve existing synthesis results that retain a runnable scenario beside a refusal for an unimplemented check, or an accepted authored scenario beside a duplicate-source refusal. |
| F2 | docs/design/review-conformance-coverage.md:167 | boundary / blocker | NEEDS-CHANGE | introduced | Independent suite-v5 and report-v1 selections have no specified writer outcome even though the frozen report-v1 reader rejects suite major 5, leaving the advertised opt-in rollout without a complete version-pairing contract. |
| F3 | docs/design/review-conformance-coverage.md:31 | contract-drift / warning | CONFIRMED | introduced | The impact compatibility rules name impact/1 although the cited ESS baseline writes ess-impact/2, so the frozen contract and its migration target are identified incorrectly. |

### F1 — The coverage shape excludes current partial-check results

**What was measured:** design line 95 requires every present refused scenario ID to be absent from selected/outside IDs. However, synthesize.rs:2818 records RefusalUndeclared with the scenario's actual ID, and :2846 returns that runnable scenario. The existing source test at tests/synthesis.rs:1305 explicitly requires both for quiet.orders.Order/state/Shipped/refuses/quiet.orders.ShipOrder. This is an existing supported result, not a hand-built unreachable state.

**What reaches it:** ordinary synthesis of a lifecycle command without a declared wrong_state answer. CLI synthesize calls that producer and emits its executable suite while reporting refusals separately (main.rs:2500–2527 and :2555–2562); v5 is meant to preserve both halves. Component synthesis additionally keeps whole-system refusals while moving runnable scenarios to outside (synthesize.rs:997–1018), so the outside exclusion is affected too.

**Class and other members:** a refusal can describe an omitted assertion within an emitted scenario, not just the absence of the entire scenario. ViewUndecidable (:2068) and OrderUnwitnessed (:2099) record the same kind of partial-check gap. DuplicateScenario preserves the first scenario and records the colliding ID (:2697). Authored compile retains an accepted source's scenario (authored.rs:1427–1433) while a later duplicate source returns a refusal naming the same ID (:1473–1480). Thus fixing only RefusalUndeclared would leave the same representational defect.

**Required correction:** bind a representation that distinguishes an omitted check/rejected candidate from the absence of a selected scenario, and explicitly permits the existing supported coexistence while retaining the exact scenario/source identities. State its execution count and incomplete-coverage consequences and add it to the future matrix. Do not make implementation discard the useful scenario, hide the refusal, erase its known ID, or refuse every such diagnostic suite merely to satisfy the new disjointness rule.

Source excerpts and exact outputs:

```console
sed -n '2815,2827p;2837,2847p' crates/verify/ess-conformance/src/synthesize.rs
```

```text
    } else {
        // Beside the scenario, never instead of it: what the scenario asserts is real, and it is
        // less than §19 asks for, and only one of those two facts is visible in a passing run.
        refusals.push(Refusal::about(
            id,
            RefusalCause::RefusalUndeclared {
                entity: entity.clone(),
                state: state.clone(),
                command: command_ref,
            },
        ));
        None
    };
    let text = match (&reported, accepted) {
        (Some(error), _) => format!(
            "`{command}` does not move a `{entity}` that is in `{state}`, and reports `{error}`"
        ),
        (None, true) => format!(
            "`{command}` is accepted on a `{entity}` that is in `{state}`, and does not move it"
        ),
        (None, false) => format!("`{command}` does not move a `{entity}` that is in `{state}`"),
    };
    Some(ConformanceScenario::new(clipped(&text), steps, source))
}
```

Exit: `0`.

```console
sed -n '1304,1332p' crates/verify/ess-conformance/tests/synthesis.rs
```

```text
#[test]
fn a_command_that_declares_no_wrong_state_answer_is_refused_by_name_beside_its_scenario() {
    // The other half, and the reason the refusal stays in the code: a specification that has not
    // adopted the construct still gets the scenario, and still gets told what its suite is not
    // checking. §36's rule is about silence, not about absence — a reader of a passing run cannot
    // otherwise tell a scenario that asserts everything the section asks for from one that asserts
    // what was left.
    let synthesis = synthesize(&fixture(NO_DECLARED_REFUSAL));
    let id = "quiet.orders.Order/state/Shipped/refuses/quiet.orders.ShipOrder";

    assert!(
        ids(&synthesis).iter().any(|known| known == id),
        "the scenario is still produced; what it cannot assert is not a reason to drop it: {:?}",
        ids(&synthesis)
    );
    assert!(
        !shape(&synthesis, id).contains(&"error"),
        "and it asserts no error, because the command names none for this"
    );

    let refusal = synthesis
        .refused(code(12))
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .unwrap_or_else(|| panic!("the mechanism refuses: {:?}", refused(&synthesis)));
```

Exit: `0`.

### F2 — Independent format opt-ins lack a complete pairing rule

**What was measured:** line 167 offers suite-format 5 and report-format 2 as independent opt-ins, with suite/4 and report/1 defaults until the final movement. Lines 9, 23 and 61 freeze v1 admission/meaning; its reader explicitly permits suite majors 1–4. Lines 163 and 238 define v2 reports over legacy suites, but neither the CLI rules nor the 36 rows choose an outcome for a v5 suite executed with report-format 1 or the still-default report/1.

**What reaches it:** at writer stage 5, a user follows the advertised suite-format 5 opt-in, then uses conform run --suite on that artifact without selecting report-format 2; regenerated Go packages similarly have an embedded v5 suite while ESS_REPORT_FORMAT remains unset. This uses the document's published staging and defaults. The existing Rust writer copies the actual suite version into v1 (evidence.rs:149), and Go does the same (runtime.go:635); a v1 document carrying suite major 5 is rejected by the frozen Rust reader at evidence.rs:84.

**Required correction:** add an explicit supported suite/report/strict pairing table for both Rust and generated Go. Choose and document the pre-execution outcome for suite/5 with explicit or default report/1—normally refusal with a required report/2 choice, or a precisely stated coupled default. Do not extend v1's admitted suite range or silently downcast away v5 coverage. Include the related legacy-format opt-out behavior after strict becomes default. The finding is missing binding policy; it does not claim that the future writer already exists or that a compatibility test ran.

Source excerpts and exact output:

```console
sed -n '79,86p;134,154p' crates/verify/ess-conformance/src/evidence.rs
```

```text
impl StandaloneConformanceReport {
    fn validate(&self) -> Result<(), &'static str> {
        if self.format != STANDALONE_REPORT_FORMAT {
            return Err("unsupported standalone conformance report format");
        }
        if !SuiteFormat::parse(&self.suite_version).is_ok_and(SuiteFormat::is_supported) {
            return Err("suite_version is not a supported conformance-suite format");
        }
impl ConformanceReport {
    /// Publishes this run as a standalone ESS report with no workflow-system coupling.
    pub fn standalone(&self) -> StandaloneConformanceReport {
        StandaloneConformanceReport {
            format: STANDALONE_REPORT_FORMAT.to_owned(),
            specification: format!("{}/{}", self.suite.system, self.suite.specification_version),
            spec_digest: self.suite.spec_digest.clone(),
            implementation: self.implementation.to_string(),
            status: report_status(self.status),
            scenarios_total: self.scenarios.len(),
            scenarios_failed: self
                .scenarios
                .iter()
                .filter(|result| result.status != Status::Passed)
                .count(),
            suite_version: self.suite.suite_version.to_string(),
            failed_scenarios: self
                .failures()
                .map(|result| format!("{} {}", result.status, result.scenario))
                .collect(),
            completed_at: self.completed_at,
```

Exit: `0`.

The design's relevant rules are lines 163–175 and its stages 234–239; all were read in full. The v5/v1 pairing is absent from matrix rows M01–M36 above.

### F3 — The impact contract is named at the wrong version

**What was measured:** design S9 at line 31 and the movement section at line 228 promise to preserve/add nothing silently to impact/1. The cited source defines IMPACT_FORMAT as ess-impact/2 at impact.rs:96; the preceding comment explains that /2 added artifacts and optional suite/invalidation. This is not a speculative new impact format.

**What reaches it:** ESS impact CLI reads a suite at main.rs:2007–2013 and calls ess_diff::impact at :2014. The returned persisted result sets format: IMPACT_FORMAT and copies SuiteProvenance at impact.rs:841–848. The v5 internal-view migration must preserve or intentionally evolve that actual current envelope.

**Required correction:** name ess-impact/2 consistently and distinguish any historical /1 profile from the current producer. Keep the proposed future exact-suite binding envelope decision separate; do not reinterpret the existing /2 bytes.

```console
rg -n 'impact/1' docs/design/review-conformance-coverage.md
rg -n 'pub const IMPACT_FORMAT' crates/verify/ess-diff/src/impact.rs
```

```text
31:| S9 | [impact.rs](../../crates/verify/ess-diff/src/impact.rs), lines 775–851 and 893–930 | Impact compares system/model/contract provenance and intersects scenario dependencies. It does not verify an execution report or establish full coverage. Preserve existing impact/1 bytes; admit v5 suites through an internal view, carry selection in diagnostics, and conservatively refuse evidence reuse when the exact selected suite is unavailable. |
228:Impact continues using semantic dependencies for invalidation. A v5 admitted-suite view supplies the same dependency graph plus its selection; missing coverage cannot be interpreted as an empty dependency set. No new field is added silently to impact/1 or its serialized SuiteProvenance. To issue a future persisted impact result that binds exact suites, the impact owner must version that envelope separately; until then callers pair the original suite externally and cannot reuse a report solely from model/contract provenance. [S9]
96:pub const IMPACT_FORMAT: &str = "ess-impact/2";
```

Exit: `0`.

## 5. Examined ground and limits

- Read the full design, all 36 matrix rows, original brief, implementation report, story acceptance, complete new-file diff and ESS AGENTS.
- Checked Rust scenario status precedence and run aggregation directly in report.rs:62–80 and :556–568, plus standalone category/list derivation and validation in evidence.rs.
- Checked Go report construction, skipped/error handling, BeginScenario and EndScenario, host subtest filtering, reduced JSON decoding and embedded source bytes. The specified five-way v2 partition retains producer differences instead of relabeling skipped or unsupported.
- Checked suite major syntax versus admission, runner callbacks, CLI --path synthesis/authoring, standalone versus detailed serialization, authored-only web output and browser replay inputs. Retained old runtimes need actual rollout/regeneration; the document does not claim otherwise.
- Checked exact-byte digest rules against the current Rust suite serializer, Node/Number finite binary64 boundary and Go's lossy execution view. The proposed raw-byte verification strategy is implementable without requiring Go to reproduce Rust's formatter. Canonical vectors, duplicate-key admission and full new reader behavior remain future execution work.
- Checked AEP's two independent report readers and closed EssConformanceResult/facts at the advertised Git objects. Existing planning recording is descriptive and independently checks zero totals; it is not made into a qualifying predicate merely by updated prose. The design names this separate migration.
- Checked realization only as the cited artifact-reference holder; no unestablished suite/report parser was added to the inventory.
- The read-only source checks do not prove implementation compatibility, discovery completeness, actual adopter deployment or exhaustive future malformed-input behavior. No runtime, package test, formatter, Clippy, browser gate, network service, cluster, release or full gate was run.
- No tracked file was modified. The only authored file in this review is the assigned adversary report. No implementation correction, test mutation, store operation, staging, commit or cleanup occurred.

```console
git status --short
```

```text
```

Exit: `0`.

```console
df -B1 .
```

```text
Filesystem        1B-blocks         Used    Available Use% Mounted on
/dev/nvme0n1p2 910126964736 723515469824 140304142336  84% /
```

Exit: `0`.

## 6. Outside writes

None. Report path: /home/timo/.local/state/worktree/trees/b10x/ess/review-conformance-format-design/target/review-boundaries-3/adversary-pass-1.md. No separate scratch fixture or log was created; command excerpts are retained in this report.

## 7. Findings for the coordinator

```findings
- file: docs/design/review-conformance-coverage.md
  line: 95
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The refusal-ID disjointness rule cannot preserve existing synthesis results that retain a runnable scenario beside a refusal for an unimplemented check, or an accepted authored scenario beside a duplicate-source refusal.
- file: docs/design/review-conformance-coverage.md
  line: 167
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Independent suite-v5 and report-v1 selections have no specified writer outcome even though the frozen report-v1 reader rejects suite major 5, leaving the advertised opt-in rollout without a complete version-pairing contract.
- file: docs/design/review-conformance-coverage.md
  line: 31
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: The impact compatibility rules name impact/1 although the cited ESS baseline writes ess-impact/2, so the frozen contract and its migration target are identified incorrectly.
```

