---
format: aep.planning-md/1
id: review-result:review-boundaries-3-conformance-design-adversary-pass-2
kind: review-result
status: active
title: Wave 3 conformance design adversary pass 2
relations:
- reviews: story:review-conformance-format-design
revision: 1
---
unit: story:review-conformance-format-design — commit dc746cfab0e628ad28911cbee32c1031a9efb6dd, adversary pass 2
verdict: NEEDS-CHANGE
cases: not applicable — document-only source and matrix review; no runtime cases executed
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: record this final full pass and route the two design corrections; no third full attack

```console
git --no-pager diff --stat
```

```text
```

Exit: `0`.

2. Cases and execution boundary

No executable case was added or run. The brief makes every tracked file read-only, limits this unit to document/source review, and prohibits builds and meaningless prose tests. The findings are source-backed judgement findings, not test-runner evidence. No before/after runtime count, red execution result or compatibility execution is claimed.

3. Observed scope, full review and source checks

```console
git rev-parse HEAD
```

```text
dc746cfab0e628ad28911cbee32c1031a9efb6dd
```

Exit: `0`.

```console
git --no-pager diff --name-only 45832cc885377b2d61845ee33af14f0293d99e67...dc746cfab0e628ad28911cbee32c1031a9efb6dd
```

```text
docs/design/review-conformance-coverage.md
```

Exit: `0`.

The complete new-file diff was read in three contiguous slices, then captured through the unpiped Git command below. It contains only the 313-line new design; the original base is 45832cc885377b2d61845ee33af14f0293d99e67. The directly executed command exited 0 and returned 60,133 characters. Its opening is retained here; the full document was read, including all 58 matrix rows and P1–P10.

```console
git --no-pager diff 45832cc885377b2d61845ee33af14f0293d99e67...dc746cfab0e628ad28911cbee32c1031a9efb6dd -- docs/design/review-conformance-coverage.md
```

```text
diff --git a/docs/design/review-conformance-coverage.md b/docs/design/review-conformance-coverage.md
new file mode 100644
index 0000000..ca1bd67
--- /dev/null
+++ b/docs/design/review-conformance-coverage.md
@@ -0,0 +1,313 @@
```

Exit: `0`.

Read completely: the exact adversary charter, original unit brief, pass-2 brief, story acceptance and scope, ESS AGENTS, original implementation report, immutable first review and correction report. Earlier reports were not changed.

```console
rg -c '^\| M[0-9][0-9] \|' docs/design/review-conformance-coverage.md
```

```text
58
```

Exit: `0`.

```console
rg -c '^\| P[0-9]+ \|' docs/design/review-conformance-coverage.md
```

```text
10
```

Exit: `0`.

These counts measure document rows, not executed cases.

| Reviewed rows | Source/contract ground | Observation |
| --- | --- | --- |
| M01–M04, M09–M10, M35, M55 | Rust evidence.rs, both exact AEP readers, closed AEP domain/facts | Frozen non-pass meaning, version dispatch and new category partition are explicit. Report/1 remains restricted to suite/1–4. |
| M05–M08 | report.rs scenario/run aggregation; Go Run, writeReport and execute | Rust same-scenario error outranks unsupported while separate unsupported scenarios fail the run; Go skips and teardown failures retain their actual meanings. |
| M11, M19 | Runner::run and Go subtest result initialization | Complete v2 outcomes must cover all selected IDs; host filtering cannot fabricate passes; empty execution cannot qualify. |
| M12–M18 | SuiteFormat/from_json, Runner entry, Go reduced decoding, browser response handling | Reader-first admission and old-runtime regeneration are explicit; a version increment is not claimed to stop old code. |
| M20–M26 | Selection, synthesis/component partitions, author compilation | Component/origin/explicit scopes remain distinct; incomplete and unknown inventories cannot qualify broader expectations. |
| M27–M31 | Suite serializer, finite Number, pinned serde_json/zmij and raw Go embedding | Exact original-byte hashing avoids lossy reserialization. The remaining scalar contradiction is F2 below. |
| M32 | CLI --report-out and detailed JSON/YAML branches | Standalone report/2 and detailed run/2 are separate surfaces with reader rollout obligations. |
| M33–M34, M58 | impact.rs and realization ConformanceEvidence | Current impact format is correctly ess-impact/2; realization fields remain artifact references without an invented byte-verification claim. |
| M37–M44 | All 14 generated cause variants, authored Cause/compile, partial-check and duplicate-survivor call sites | Original coexistence defect is corrected. Repeated identical cause occurrences remain unspecified under F1 below. |
| M45–M49 | synthesize_for, refusal scope, retained identity and filter rules | Component exclusion needs the refused obligation/candidate's own justification; filters retain source/refusal inventory and cannot hide an in-scope gap. |
| M36, M50–M57, P1–P10 | Actual suite version, independent defaults, strict/diagnostic resolution and pre-target pairing | Supported cross-product now has explicit outcomes, including v5/default-v1 refusal, strict/v1 refusal and defaults pinned to a generated runtime's release. |

First-pass disposition:

- F1 at old line 95: the original ID-disjointness contradiction is corrected at current lines 90–123 and M37–M49. Partial checks, surviving duplicates, authored source identities and component/filter scope are represented. F1 below is new residual ground: multiplicity of identical producer records.
- F2 at old line 167: the missing version-pairing policy is corrected by P1–P10 and M50–M57. No carry of that original finding.
- F3 at old line 31: S9, M33, M58 and line 295 now consistently identify ess-impact/2. No carry of the stale impact citation finding.

AEP reads used Git objects at 00c742e4179593738a2e8aa69e2ecc07d3c89402, including the complete optional adapter, planning.rs:5907–5985, and domain evidence.rs:1006–1051 and 1862–1889. AEP's planning reader independently parses JSON Value and records prose; it is not silently promoted to the adapter's domain evidence path. The two report-reader files remain unchanged from the earlier advertised object:

```console
git -C /home/timo/beyond10x/aep --no-pager diff cc321f31fa0120b32a5b9f5e7b8c8fdfa55f69f9 00c742e4179593738a2e8aa69e2ecc07d3c89402 -- crates/observe/aep-ess-evidence/src/lib.rs crates/edge/aep-cli/src/planning.rs
```

```text
```

Exit: `0`.

Atlas AGENTS was read from the stated 6035d6e1209686ca474a3f43975fde7d8621ba48 object. It requires an ADR naming relying parties/order, a contract version and a shipped log. The design names these as future obligations and does not claim approval, deployment or migration completion. No network or sibling working-tree authority was used.

```console
git --no-pager diff --check 45832cc885377b2d61845ee33af14f0293d99e67...dc746cfab0e628ad28911cbee32c1031a9efb6dd
```

```text
```

Exit: `0`.

```console
git --no-pager diff --check
```

```text
```

Exit: `0`.

4. Judgement findings

Both findings cover the final design at dc746cfab0e628ad28911cbee32c1031a9efb6dd. They are introduced in this wholly new document; the source behavior they must account for predates this unit.

| ID | File:line | Category / severity | Verdict | Origin | Finding |
| --- | --- | --- | --- | --- | --- |
| F1 | docs/design/review-conformance-coverage.md:116 | acceptance / blocker | NEEDS-CHANGE | introduced | The design forbids duplicate refusal records and omitted refusals without defining how to preserve repeated identical refusals that current generated and authored producers emit. |
| F2 | docs/design/review-conformance-coverage.md:167 | contract-drift / warning | CONFIRMED | introduced | The report timestamp is described as an existing signed value although Rust uses u64 and Go uses int64, while the shared new writer rules require unsigned integer fields, leaving the v2 timestamp range and conversion contract contradictory. |

F1 — Identical producer refusals have no bound inventory mapping

What was observed: line 103 prohibits omitting a refusal, line 116 says the count measures records and forbids exact duplicates, and the closed record at line 90 has no occurrence identity or multiplicity. Existing producers accumulate a Vec and can emit identical Refusal values more than once. This is a source-derived counterexample, not an executed red case.

What reaches it, generated: repeat the existing `weight_grams >= 0` invariant at examples/oracle-fixture/domains/order.yaml:59. RawEntitySpec keeps invariants in a Vec; TryFrom at entity.rs:1566 copies every element; validate at :838 checks each predicate but does not prohibit repetitions. The existing test at tests/synthesis.rs:2100 establishes that this fixture's weight invariant is unobservable. For each state-changing outcome, holds_after at synthesize.rs:2945–2959 pushes the same InvariantUnobservable for each identical occurrence. The invariant statement, entity, state, unpublished paths, scenario ID and rendered cause are equal; generated source is null. With a system selection, both projected records also have the same effect, retained value, scope and needs.

What reaches it, authored: one requested scenario file can repeat the same assertion naming an undeclared view. Compiler::run at authored.rs:1600 walks each assertion, assertion at :1893 refuses the same undeclared view, and Compiler::refuse at :1543 appends the same source/ScenarioId/Cause without a source-position field or deduplication. This reaches the normal authored compile entry, which intentionally returns every refusal even when no scenario is accepted. It is not necessary to construct malformed IR.

The writer therefore needs an additional policy to reconcile preserving these producer occurrences with rejecting duplicate records. Arbitrarily adding ordinal text to message could make records distinct, but that would be a new unbound identity/count convention rather than a rule supplied by the design. Specify the intended multiplicity/identity/count behavior and a corresponding generated and authored matrix row. Do not leave the mapper to invent message suffixes, silently coalesce records, or refuse this diagnostic inventory under incompatible rules.

Source excerpts, read-only:

```console
sed -n '50,62p' examples/oracle-fixture/domains/order.yaml
```

```text
        type: oracle.order.Email
      - name: weight_grams
        type: Integer

    # What design §20 evaluates after a state-changing command. `AmendOrder` below is the reason
    # this fixture exists for that section: it changes the field this invariant reads *without*
    # moving the entity along its lifecycle, which is the case the normative example has no
    # instance of.
    invariants:
      - weight_grams >= 0

    # `Shipped` is terminal, and `cancel` cannot start there. That is the illegal transition design
    # §19 asks for a negative check on, and it is reachable: `PlaceOrder` then `ShipOrder` puts an
```

Exit: `0`.

```console
sed -n '1553,1614p' crates/specify/ess-domain/src/entity.rs
```

```text
    pub states: RawStateMachine,
}

impl TryFrom<RawEntitySpec> for EntitySpec {
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

        let mut errors = spec.states.validate_at(&location);

        // A duplicate field is document-local, and it has to be caught here: the second declaration
        // would be invisible to every later lookup, so an invariant reading it would be checked
        // against a field nobody can see.
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        for field in &spec.fields {
            *seen.entry(field.name.as_str()).or_default() += 1;
        }
        for (name, count) in seen {
            if count > 1 {
                errors.push(ValidationError::new(
                    ValidationCode::DuplicateDeclaration,
                    format!("{location}.fields.{name}"),
                    format!("`{name}` is declared {count} times"),
                ));
            }
        }

        // Nor may a field shadow the identity. `observable_fields` offers both, so a view resolving
        // the name would get whichever came first — and the two declarations need not agree on a
        // type.
        for field in &spec.fields {
            if field.name == spec.identity.name {
                errors.push(
                    ValidationError::new(
                        ValidationCode::DuplicateDeclaration,
                        format!("{location}.fields.{}", field.name),
                        format!("`{}` is already the entity's identity", field.name),
                    )
                    .with_hint(
                        "an identity is observable under its own name; a field of that name would \
                         give one name two types",
                    ),
                );
            }
        }

        errors.into_result(spec)
    }
}

/// Lets a set of entities answer what a view may project.
///
```

Exit: `0`.

```console
sed -n '2890,2980p' crates/verify/ess-conformance/src/synthesize.rs
```

```text
            if ir.entity(&subject.entity).invariants.is_empty() {
                continue;
            }
            let id = ScenarioId::Invariant {
                entity: EntityRef::from(&subject.entity),
                after: OutcomeRef::new(CommandRef::new(command.name.clone()), outcome.name.clone()),
            };
            let Some(scenario) =
                holds_after(ir, command, outcome, &projections, actors, &id, refusals)
            else {
                continue;
            };
            insert(suite, id, scenario, refusals);
        }
    }
    value_object_invariants(ir, actors, suite, refusals);
}

/// The scenario that runs one branch and then reads the entity's invariants off a view.
///
/// `None` where the branch cannot be run at all, and where **every** invariant was refused: a
/// scenario that executes a command and asserts nothing about what it was written to check is the
/// shape of green this milestone exists to rule out. A scenario that could assert some of them is
/// still worth having, and the ones it could not appear as refusals beside it.
#[allow(clippy::too_many_arguments)]
fn holds_after(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    projections: &BTreeMap<&EntityHandle, Vec<&ResolvedView>>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> Option<ConformanceScenario> {
    let subject = outcome.subject.as_ref()?;
    let entity = ir.entity(&subject.entity);
    let entity_ref = EntityRef::from(&subject.entity);
    let run = match run(ir, command, outcome, actors) {
        Ok(run) => run,
        Err(cause) => {
            refusals.push(Refusal::about(id, cause));
            return None;
        }
    };
    // The state the branch leaves the instance in, which is what decides whether a filtered view
    // holds a row to read the invariant off. A branch with a subject always has one.
    let state = run.after.clone()?;
    let views = projections
        .get(&subject.entity)
        .map(Vec::as_slice)
        .unwrap_or_default();

    let mut steps = run.steps();
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
            ));
            named.insert(ViewRef::new(view.name.clone()));
        }
    }
    if named.is_empty() {
        return None;
    }

    let mut source: BTreeSet<EssSemanticRef> = run.source.clone();
    let command_ref = CommandRef::new(command.name.clone());
    source.insert(command_ref.clone().into());
    source.insert(OutcomeRef::new(command_ref, outcome.name.clone()).into());
    source.insert(entity_ref.clone().into());
    if let Some(actor) = run.actor.clone() {
        source.insert(actor.into());
    }
    source.extend(named.into_iter().map(EssSemanticRef::from));
    source.extend(
```

Exit: `0`.

```console
sed -n '1536,1554p;1572,1611p' crates/verify/ess-conformance/src/authored.rs
```

```text
    steps: Vec<ScenarioStep>,
    source: BTreeSet<EssSemanticRef>,
    types: BTreeSet<DeclaredTypeRef>,
}

impl Compiler<'_> {
    /// Records a refusal about the scenario being compiled.
    fn refuse(&mut self, cause: Cause) {
        self.refusals.push(Refusal {
            origin: self.origin.to_owned(),
            scenario: Some(self.id.clone()),
            cause,
        });
    }

    /// Resolves a written name, refusing it by name when the model declares nothing under it.
    fn declared<'ir, T>(
        &mut self,
        written: &str,
        }
    }

    /// Compiles the whole document.
    fn run(&mut self, document: &Document) {
        for arrangement in &document.arrange {
            let Some((name, _)) =
                self.declared(&arrangement.entity, self.ir.entities(), |entity| {
                    Cause::UndeclaredEntity { entity }
                })
            else {
                continue;
            };
            let entity = EntityRef::new(name);
            self.source.insert(entity.clone().into());
            self.arranged.insert(arrangement.instance.clone(), entity);
        }

        let mut previous: Option<Moment> = None;
        for act in &document.timeline {
            if let Some(after) = previous {
                if act.at <= after {
                    self.refuse(Cause::UnorderedTimeline { at: act.at, after });
                }
            }
            previous = Some(act.at);
            self.act(act);
        }
        for assertion in &document.assert {
            self.assertion(assertion);
        }
        if document.timeline.is_empty() {
            self.refuse(Cause::NothingHappens);
        }
    }

    /// One act: the command, what it must answer, and what it binds.
    fn act(&mut self, act: &Act) {
        // The windows come first, and that is the semantics rather than a filing decision: a claim
        // about the time before an act is a claim the act is not allowed to be the cause of. Emitted
```

Exit: `0`.

```console
sed -n '1891,1910p' crates/verify/ess-conformance/src/authored.rs
```

```text

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
        let style = declared.assertion_style;
        self.reach(&fields);
        self.source.insert(view.clone().into());

        let stated = assertion.stated();
        if stated.len() != 1 {
            self.refuse(Cause::AmbiguousClaim {
                view: view.clone(),
```

Exit: `0`.

F2 — Signed/unsigned timestamp rules disagree

What was observed: the new report field at line 167 says “Existing signed epoch-millisecond timestamp meaning.” ESS's actual Timestamp is u64, used directly by the Rust report; Go's report CompletedAt is int64. Line 140 requires unsigned integer spellings bounded to u64, and line 149 applies those scalar rules to report and detailed-report writers. These statements do not identify one v2 timestamp domain.

What reaches it: every Rust/Go report writer and reader must implement completed_at, and detailed run/2 also includes started_at. The public Rust AdvancingClock::new(start_ms: u64, step_ms: u64) and Timestamp constructor admit the upper u64 range, including values above i64::MAX; the existing Rust report copies those values. The Go wire type is int64. A negative signed timestamp and a positive value above i64::MAX expose different admission/conversion decisions; the document currently gives inconsistent instructions for both. These boundary examples were not executed.

Required correction: describe the two existing wire domains accurately, choose the new report/run timestamp range and spelling, and specify checked producer/consumer conversions at its boundaries. Exempt signed timestamps explicitly from the unsigned-count rule if that is the chosen new contract. Preserve frozen v1 behavior; do not call one shared signed meaning pre-existing.

```console
sed -n '25,69p' crates/specify/ess-primitives/src/time.rs
```

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
    /// and every backend that stamps an instant should stamp the same one rather than each carrying
    /// its own copy of the calendar.
    #[must_use]
    pub fn iso_8601(self) -> String {
        let seconds_into_day = (self.0 / 1000) % 86_400;
        format!(
            "{}T{:02}:{:02}:{:02}Z",
            CivilDate::from_timestamp(self),
```

Exit: `0`.

```console
sed -n '104,156p' crates/verify/ess-conformance/src/runner.rs
```

```text
pub trait Clock {
    /// The current time, advanced by this read.
    fn now(&mut self) -> Timestamp;
}

/// A clock that starts at a fixed instant and advances by a fixed step on every read.
///
/// The shape §37 asks for and `aep_conformance::harness::Harness` already uses: "a timestamp that
/// advances by a second per call, so ordering is observable without sleeping". Under one of these a
/// run is reproducible — the report's `started_at`, every `duration_ms` and the number of times a
/// bounded assertion asked are all functions of how many times the clock was read.
#[derive(Debug, Clone, PartialEq, Eq)]
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

Exit: `0`.

The Go report declaration at runtime.go:591 declares `CompletedAt int64`, populated with `time.Now().UnixMilli()` at :637. The Rust standalone declaration at evidence.rs:39 uses ess_primitives::time::Timestamp, and standalone() copies self.completed_at at :154. These exact locations were checked with `rg -n 'completed_at:|CompletedAt'` against both files (exit 0). The relevant conflicting design lines are:

```console
sed -n '114,118p;140p;149p;167p' docs/design/review-conformance-coverage.md
```

```text
1. generated and authored are disjoint and their union is exactly the scenario-map keys. Every selected scenario has one origin; adding an authored candidate under an already retained ID is a refusal, not replacement.
2. IDs in outside are unique and absent from selected IDs. **Refusal scenario IDs need not be disjoint from either set.** For every nonnull refusal ID, retained is nonnull exactly when the ID is selected or outside, and identifies that surviving scenario and its accepted source. Otherwise retained is null. Null scenario requires null retained. A check_not_emitted with no survivor is valid, including the all-invariants-unobservable case.
3. Multiple distinct refusal records may concern one ID or source; their count measures records. Exact duplicate records are refused. Sort outside by scenario then origin/reason; sort refused by origin, source, subject, scenario, code, effect, retained.origin, retained.source, scope, needs, message, with null before text and needs compared lexicographically.
4. counts.generated + counts.authored equals the selected scenario-map size. A retained selected scenario contributes exactly one terminal execution result regardless of how many refusals name it; a retained outside scenario contributes none. Rejected candidates and missing checks contribute no execution result or outcome category. counts.refused includes all refusal records, including outside_component/outside_origin; it is never an additional scenario or failure count.
5. Complete coverage requires complete_inventory and zero refusals with scope in_scope. An all-pass selected scenario with an in_scope omitted check or rejected candidate leaves conformance_status inconclusive. Outside scenarios and correctly justified outside-component/origin refusals do not make a complete named selection incomplete; they still prevent its result from satisfying a broader expectation.
| Counts and integer fields | Existing valid spellings preserved | Unsigned decimal integers without exponent, sign or leading zero; bounded to u64 and checked before arithmetic |
New report and detailed-report writers use the same v5 layout/scalar rules for their own bytes, but their hashes, if any, are report-artifact hashes and never substitutes for SuiteReference.digest. Existing realization artifact digests retain their original profile. [S10]
| completed_at | Existing signed epoch-millisecond timestamp meaning |
```

Exit: `0`.

5. Examined ground and limits

- Read every row of the final matrix and pairing table; checked the new retained/effect/scope/source rules against actual generated and authored producer control flow.
- Checked Rust scenario and aggregate status precedence, Go skipped/error/teardown behavior and uninvoked-subtest risk.
- Checked all identified suite/report consumers: Rust admission/runner, Go emitter/runtime, browser emitter/player, CLI standalone/detailed routes, impact, realization references, both AEP report readers and AEP's existing closed domain/fact mapping.
- Checked version movement, legacy opt-outs, strict mode, empty/unknown coverage, explicit-parent retention and exact-suite mismatch rules without finding another contradiction in those examined rules.
- Checked raw-byte binding against canonical suite output and the lossy Go execution struct; golden scalar vectors and new cross-language compatibility remain future implementation obligations.
- This review cannot establish runtime compatibility, actual complete-inventory enumeration, deployed adopter readiness, or passing repository gates. No test, build, formatter, Clippy, browser execution, service/cluster call, release, store command or lifecycle operation ran.
- All tracked files remained read-only. No existing review/report was rewritten. This is the second and final full attack; no third full pass was performed or requested.

```console
git status --short
```

```text
```

Exit: `0`.

6. Outside writes

None. The sole authored path is /home/timo/.local/state/worktree/trees/b10x/ess/review-conformance-format-design/target/review-boundaries-3/adversary-pass-2.md. No external scratch, fixture, cache or build directory was created. No cleanup was performed.

7. Findings for the coordinator

```findings
- file: docs/design/review-conformance-coverage.md
  line: 116
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The design forbids duplicate refusal records and omitted refusals without defining how to preserve repeated identical refusals that current generated and authored producers emit.
- file: docs/design/review-conformance-coverage.md
  line: 167
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: The report timestamp is described as an existing signed value although Rust uses u64 and Go uses int64, while the shared new writer rules require unsigned integer fields, leaving the v2 timestamp range and conversion contract contradictory.
```
