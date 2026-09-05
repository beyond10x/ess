unit: story:review-conformance-format-design — final bounded conformance design correction
verdict: green
cases: not applicable — document-only checks; no runtime cases executed
origin: n/a
wrote-outside-worktree: none
needs-coordinator: no

## 1. Unit and acceptance

Correct only the two new findings from immutable adversary-pass-2.md at HEAD dc746cfab0e628ad28911cbee32c1031a9efb6dd: preserve every refusal occurrence and specify exact unsigned v2 timestamps, while retaining all earlier design decisions and assertions. Original story base: 45832cc885377b2d61845ee33af14f0293d99e67. This is the final bounded implementor correction under correction-brief-2.md, not a third full review.

The tracked edit is confined to docs/design/review-conformance-coverage.md. This report is the only new scratch artifact authored in this correction.

## 2. Actual change and correction classes

Command (worktree root):
```text
git --no-pager diff --stat
```
Output (verbatim):
```text
 docs/design/review-conformance-coverage.md | 66 ++++++++++++++++++++++--------
 1 file changed, 48 insertions(+), 18 deletions(-)
```
Exit: 0.


F1 class: multiplicity of producer diagnostics is distinct from scenario, source-map and outcome identity. The design now requires a sorted refusal list with one element per producer occurrence, including byte-identical records. counts.refused is the occurrence-list length. No deduplication, added identity, ordinal message suffix or extra execution result is permitted. Equal full-field sort keys yield identical bytes. Component/origin selections, explicit filters, report coverage and paired parent/suite equality preserve full multiplicity. Any in_scope occurrence keeps the completeness predicate inconclusive; bytes alone cannot prove the producer originally enumerated every occurrence.

Enumeration: generated repeated unobservable invariants (M59), authored repeated undeclared-view assertions (M60), component/origin/filter retention (M61), and child/report deduplication or altered-message refusal (M62). Existing selected/outside scenario, source-map and outcome uniqueness remains unchanged. Existing partial-check and rejected-candidate retention rules remain intact.

F2 class: timestamp wire domains and exactness across producers/readers/adaptation. Legacy Rust Timestamp(u64) and Go int64/UnixMilli remain distinct and frozen. New report/2 and run/2 timestamp fields use exact nonnegative epoch-ms u64 integer tokens, including detailed started_at and summary.completed_at. Go v2 uses uint64 wire fields with checked nonnegative clock conversion; a negative clock refuses production without wrapping, clamping or automatic v1 fallback. New browser/AEP and other readers use exact integer parsing and checked arithmetic. A narrower internal date/time domain requires an explicit adaptation refusal, never reinterpretation as negative, truncation or false qualification. The checked-reader rules do not rewrite the existing AdvancingClock implementation or its saturating advancement.

Enumeration: 0 (M63), i64::MAX (M64), i64::MAX+1 (M65), u64::MAX (M66), negative input (M67), overflow (M68), fractional/exponent/signed/quoted tokens (M69), binary64 precision loss (M70), negative Go clock (M71), narrower consumer adaptation (M72), both detailed timestamp fields (M73), frozen legacy domains (M74), and exact checked reader/adaptation arithmetic (M75).

The original M01–M58 and P1–P10 rows remain byte-for-byte unchanged. The suite/report/strict pairing policy still freezes report/1 to suites 1–4; the actual persisted impact format remains ess-impact/2. Exact-suite digest and payload-number profiles, all earlier scenario/refusal coexistence decisions, source identity and rollout ownership remain unchanged. New timestamp and occurrence rules are carried into their reader/writer rollout steps.

## 3. Pre-correction finding evidence; no runtime red run

No runtime red run was performed: the coordinator explicitly assigned a document-only correction and forbade code/tests/builds. Immutable adversary-pass-2.md contains the two source-backed document findings; its recorded comparison was carried 0, new 2, resolved 3. That report and adversary-pass-1.md remain unchanged. The following reads substantiate the correction classes; these are source reads, not executed examples.

Command (worktree root):
```text
sed -n '25,61p' crates/specify/ess-primitives/src/time.rs
```
Output (verbatim):
```text

/// Milliseconds since the Unix epoch, UTC.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(transparent)]
pub struct Timestamp(u64);

impl Timestamp {
    /// The epoch itself, useful as a deterministic default in tests.
    pub const EPOCH: Self = Self(0);

    /// Builds a timestamp from milliseconds since the Unix epoch.
    pub const fn from_epoch_millis(millis: u64) -> Self {
        Self(millis)
    }

    /// Milliseconds since the Unix epoch.
    pub const fn epoch_millis(self) -> u64 {
        self.0
    }

    /// The instant as ISO-8601 in UTC, to the second: `2026-08-28T14:03:07Z`.
    ///
    /// The date half is [`CivilDate::from_timestamp`] — Hinnant's `civil_from_days`, pure integer
    /// arithmetic, no date library — and the time half is the remainder of the day. Sub-second
    /// precision is dropped: this is the spelling a journal, an event envelope or a person reads,
```
Exit: 0.

Command (worktree root):
```text
sed -n '116,156p' crates/verify/ess-conformance/src/runner.rs
```
Output (verbatim):
```text
pub struct AdvancingClock {
    next: u64,
    step: u64,
}

impl AdvancingClock {
    /// The instant the default clock starts at: 2023-11-14T22:13:20Z, as
    /// `aep_conformance::harness::Harness` uses.
    pub const DEFAULT_START_MS: u64 = 1_700_000_000_000;

    /// How far the default clock moves per read.
    ///
    /// With [`RunnerConfig::DEFAULT_EVENTUAL_TIMEOUT_MS`] this bounds an eventual assertion at fifty
    /// asks, which is a budget rather than a delay: no wall-clock time passes.
    pub const DEFAULT_STEP_MS: u64 = 100;

    /// A clock starting at `start_ms` and advancing `step_ms` per read.
    ///
    /// A step of zero is raised to one. A clock that does not move cannot bound anything, and
    /// silently accepting one would trade a compile-time guarantee for a hang.
    pub fn new(start_ms: u64, step_ms: u64) -> Self {
        Self {
            next: start_ms,
            step: step_ms.max(1),
        }
    }
}

impl Default for AdvancingClock {
    fn default() -> Self {
        Self::new(Self::DEFAULT_START_MS, Self::DEFAULT_STEP_MS)
    }
}

impl Clock for AdvancingClock {
    fn now(&mut self) -> Timestamp {
        let now = self.next;
        self.next = self.next.saturating_add(self.step);
        Timestamp::from_epoch_millis(now)
    }
}
```
Exit: 0.

Command (worktree root):
```text
rg -n 'completed_at:|started_at:|CompletedAt|UnixMilli' crates/verify/ess-conformance/src/evidence.rs crates/verify/ess-conformance/src/report.rs crates/verify/ess-conformance/src/go/runtime.go
```
Output (verbatim):
```text
crates/verify/ess-conformance/src/go/runtime.go:591:	CompletedAt     int64    `json:"completed_at"`
crates/verify/ess-conformance/src/go/runtime.go:637:		CompletedAt:     time.Now().UnixMilli(),
crates/verify/ess-conformance/src/report.rs:541:    pub started_at: Timestamp,
crates/verify/ess-conformance/src/report.rs:543:    pub completed_at: Timestamp,
crates/verify/ess-conformance/src/evidence.rs:39:    pub completed_at: ess_primitives::time::Timestamp,
crates/verify/ess-conformance/src/evidence.rs:58:            completed_at: ess_primitives::time::Timestamp,
crates/verify/ess-conformance/src/evidence.rs:72:            completed_at: wire.completed_at,
crates/verify/ess-conformance/src/evidence.rs:154:            completed_at: self.completed_at,
crates/verify/ess-conformance/src/evidence.rs:235:            started_at: ess_primitives::time::Timestamp::from_epoch_millis(1_700_000_000_000),
crates/verify/ess-conformance/src/evidence.rs:236:            completed_at: ess_primitives::time::Timestamp::from_epoch_millis(1_700_000_001_000),
```
Exit: 0.

Command (worktree root):
```text
sed -n '1557,1569p' crates/specify/ess-domain/src/entity.rs
```
Output (verbatim):
```text
    type Error = ValidationErrors;

    fn try_from(raw: RawEntitySpec) -> Result<Self, Self::Error> {
        let spec = Self {
            name: raw.name,
            identity: raw.identity,
            fields: raw.fields,
            relations: raw.relations,
            states: raw.states.into(),
            invariants: raw.invariants.into_iter().map(Invariant::from).collect(),
            naming: raw.naming,
        };
        let location = format!("entity {}", spec.name);
```
Exit: 0.

Command (worktree root):
```text
sed -n '2943,2962p' crates/verify/ess-conformance/src/synthesize.rs
```
Output (verbatim):
```text
    let mut named: BTreeSet<ViewRef> = BTreeSet::new();
    for invariant in &entity.invariants {
        let witnesses = witnesses_for(ir, invariant, views, &state, &run.settled);
        if witnesses.is_empty() {
            refusals.push(Refusal::about(
                id,
                RefusalCause::InvariantUnobservable {
                    entity: entity_ref.clone(),
                    invariant: invariant.statement.clone(),
                    unpublished: unpublished(ir, invariant, views),
                    state: state.clone(),
                },
            ));
            continue;
        }
        for view in witnesses {
            steps.extend(assert_satisfied(
                view,
                bound(view, &run.settled),
                invariant.predicate.clone(),
```
Exit: 0.

Command (worktree root):
```text
sed -n '1541,1550p;1598,1606p;1891,1902p' crates/verify/ess-conformance/src/authored.rs
```
Output (verbatim):
```text
impl Compiler<'_> {
    /// Records a refusal about the scenario being compiled.
    fn refuse(&mut self, cause: Cause) {
        self.refusals.push(Refusal {
            origin: self.origin.to_owned(),
            scenario: Some(self.id.clone()),
            cause,
        });
    }

            self.act(act);
        }
        for assertion in &document.assert {
            self.assertion(assertion);
        }
        if document.timeline.is_empty() {
            self.refuse(Cause::NothingHappens);
        }
    }

    /// One claim about one view.
    fn assertion(&mut self, assertion: &Assertion) {
        let Some((name, declared)) = self.declared(&assertion.view, self.ir.views(), |view| {
            Cause::UndeclaredView { view }
        }) else {
            return;
        };
        let view = ViewRef::new(name);
        let fields = declared.fields.clone();
        let params = declared.params.clone();
        let order_by = declared.order_by.clone();
```
Exit: 0.


## 4. Final document checks

The following commands ran from /home/timo/.local/state/worktree/trees/b10x/ess/review-conformance-format-design. No test lane ran, so there is no runner-derived before/after case count. The 75 compatibility rows and 10 pairing rows are specified future implementation obligations, not passing runtime cases.

Command (worktree root):
```text
git --no-pager diff --name-only
```
Output (verbatim):
```text
docs/design/review-conformance-coverage.md
```
Exit: 0.

Command (worktree root):
```text
git --no-pager diff --check
```
Output (verbatim):
```text
```
Exit: 0.

Command (worktree root):
```text
rg -n '[ \t]+$' docs/design/review-conformance-coverage.md
```
Output (verbatim):
```text
```
Exit: 1.

Command (worktree root):
```text
rg -c '^\| M[0-9][0-9] \|' docs/design/review-conformance-coverage.md
```
Output (verbatim):
```text
75
```
Exit: 0.

Command (worktree root):
```text
rg -c '^\| P[0-9]+ \|' docs/design/review-conformance-coverage.md
```
Output (verbatim):
```text
10
```
Exit: 0.

Command (worktree root):
```text
df -B1 --output=avail .
```
Output (verbatim):
```text
       Avail
140237901824
```
Exit: 0.

Command (worktree root):
```text
sha256sum docs/design/review-conformance-coverage.md target/review-boundaries-3/adversary-pass-1.md target/review-boundaries-3/adversary-pass-2.md target/review-boundaries-3/correction-report.md
```
Output (verbatim):
```text
4d3c0db04da9e63eab3e581fbd5f9f3214311f4df72a9cb3c2e7d4da43244db3  docs/design/review-conformance-coverage.md
3f6b96ba5ee6f6191106392f93e64f01ef09befa30288de60f7745b7d9c9d572  target/review-boundaries-3/adversary-pass-1.md
1953d4df8bc407c6eb8588a0b22515072b123544b9fe984dd5b0a56cb4a3efbf  target/review-boundaries-3/adversary-pass-2.md
68669a1575f68f0fa6af8f981dae979242e86ee4ce8d2ef6c652016c9460a92d  target/review-boundaries-3/correction-report.md
```
Exit: 0.

Command (worktree root):
```text
git status --short
```
Output (verbatim):
```text
 M docs/design/review-conformance-coverage.md
```
Exit: 0.


The trailing-whitespace rg exit 1 means no matching lines. The measured free space exceeds the required 8 GiB reserve; no build output was produced.

Tool-orchestration comparison of the complete document snapshot read before this correction and the complete edited document:
```javascript
const rows = s => s.split("\n").filter(x => /^\| (M\d\d|P\d+) \|/.test(x));
({
  priorRows: rows(before).length,
  unchanged: rows(before).every(r => rows(after).includes(r)),
  matrix: rows(after).filter(r => r.startsWith("| M")).length,
  pairings: rows(after).filter(r => r.startsWith("| P")).length
})
```
Actual result:
```json
{"priorRows":68,"unchanged":true,"matrix":75,"pairings":10}
```

Full bounded diff and affected prose were inspected against the source evidence and coordinator choices. The new matrix has explicit outcomes and owners for both correction classes. Document SHA256: 4d3c0db04da9e63eab3e581fbd5f9f3214311f4df72a9cb3c2e7d4da43244db3. The old report hashes above match their pre-edit values.

## 5. Retained boundaries and handoff

No production source, tests, planning artifacts, sibling repositories, Git lifecycle, build, formatter, Clippy, browser, AEP, deployment or downstream publication operation was changed or executed. The unit is design-only; migration implementation and the Atlas ADR remain separately owned. No third full adversary review was attempted. The coordinator owns the personal bounded inspection, integration and repository gates. No extra coordinator patch is required.

Writes are relinquished after saving this report and final read-only hash/status verification.

## 6. Outside-worktree paths

None. Authored paths are only the assigned tracked design and target/review-boundaries-3/correction-report-2.md inside the assigned worktree. Previous reports were not overwritten.

