unit: story:review-conformance-format-design — correction round 1 at 5148543b57c855cb4ccca92e2368e566801e9c36 plus document working tree
verdict: green
cases: not applicable — document-only source and matrix validation; no runtime cases executed
origin: n/a
wrote-outside-worktree: none
needs-coordinator: no

1. Unit and acceptance

The binding migration design must specify unambiguous producer outcome, coverage, exact-suite identity and compatibility semantics before a new writer is implemented. This correction answers F1–F3 in the unchanged first review report. The sole editable tracked surface remains docs/design/review-conformance-coverage.md; the inferred scope was checked against the actual existing document and no scope change was needed.

2. Actual diff

```text
$ git --no-pager diff --stat
 docs/design/review-conformance-coverage.md | 129 ++++++++++++++++++++++-------
 1 file changed, 98 insertions(+), 31 deletions(-)
exit: 0
```

```text
$ git --no-pager diff --name-only
docs/design/review-conformance-coverage.md
exit: 0
```

3. Initial red result and correction class

No executable red case was authored or run. The original and correction briefs make this a document-only unit, prohibit executable/source changes, and require source/matrix review instead of invented prose tests. The initial adverse result is the coordinator-recorded adversary pass 1: two NEEDS-CHANGE blockers and one CONFIRMED warning, preserved at target/review-boundaries-3/adversary-pass-1.md. Its SHA256 remains 3f6b96ba5ee6f6191106392f93e64f01ef09befa30288de60f7745b7d9c9d572; it was read fully and never written.

| Finding | Fixed rule | Class enumeration and affected matrix |
| --- | --- | --- |
| F1: refusal IDs incorrectly required disjointness from executable/outside IDs | Refusal effect is orthogonal to scenario membership. check_not_emitted/candidate_not_emitted preserve known IDs and a typed retained survivor reference. Authored inventory binds exact file digest, known ID and final accepted/refused disposition; duplicate source refusal retains both rejected source and accepted first source. Counts distinguish one selected execution result from any number of refusal records. scope/needs determines coverage applicability; filters retain parent refusals and source inventory. | Enumerated all 14 generated cause variants into two effects and all authored Cause variants as candidate_not_emitted. Explicit rows M37–M49 cover RefusalUndeclared; ViewUndecidable plus OrderUnwitnessed; partial and absent InvariantUnobservable; duplicate generated candidate; duplicate authored source; earlier failed authored source plus later accepted same-ID source; merge collisions; component, insufficient-dependency, origin and explicit-filter cases. |
| F2: incomplete suite/report/strict pairing policy | Resolve explicit/default version and mode independently, then validate P1–P10 before target construction, identity, callbacks or report writes. Actual suite/5 with report/1 always refuses, even for a default report option, no report destination or allow-incomplete. Report/1's admitted suites stay exactly 1–4 independently of the execution supported set. | P1–P10 cover the complete supported cross-product, unsupported suite/report and conflicting modes. M36 and M50–M57 cover opt-in/defaulted v5/v1, attempted diagnostic downcast, explicit/default strict with v1, v2 on legacy under strict/diagnostic, frozen report/1 parsing after /5 support grows, invalid flags and defaults pinned in generated runtimes. Both Rust and Go have the same binding rule. |
| F3: current impact contract misnamed | Current persisted envelope is ess-impact/2 at IMPACT_FORMAT. Existing /2 bytes, fields and SuiteProvenance meanings stay frozen; historical /1 is identified only as historical. Future exact-suite impact fields require a separate versioned envelope. | Updated S9, M33, M58 and movement prose; checked every impact-version occurrence against impact.rs:86–96 and the actual result constructor at :841–848. |

The F1 correction is broader than only the reported RefusalUndeclared instance. Source inspection found InvariantUnobservable can retain a partial scenario or emit no scenario when all assertions are unavailable; both are represented without erasing the known ID. An authored compile failure before a later success at the same ID also remains representable without pretending the refusal has no surviving peer. Refused candidate component scope needs the candidate's own dependency justification: a surviving duplicate's outside-component status alone cannot exclude the rejected candidate.

An explicit filter preserves all parent refusal scope and source identities even when its associated scenario moves to selection_filter outside. An in_scope refusal cannot disappear by filtering the ID. Component/origin changes require newly built justified selection; unknown applicability stays in_scope. Outside-component/origin refusals remain visible but do not block the approved narrower selection.

The F2 policy has one chosen outcome, not alternative proposals. During opt-in rollout suite/5 requires explicit report/2 for Rust and Go. After default movement the complete v1 opt-out is actual suite/1–4, explicit report/1 and explicit allow-incomplete. Fresh legacy production selects suite-format 4; historical suite/1–3 input stays unchanged. A v2/legacy strict pairing executes but cannot succeed because coverage remains unknown. An already generated runtime retains its own release defaults, so M57 explicitly refers to the upgraded opt-in runtime, while M17 still records that baseline old code has no version check.

4. Source and matrix validation

These are source-backed design observations and specified future outcomes, not executed compatibility results. The unchanged producer outcome profiles and exact-json-byte digest/canonicalization rules retain the original design decision. No migration implementation, canonical golden or downstream reader deployment is claimed.

| Source checked in the assigned tree | Observation supporting the correction |
| --- | --- |
| synthesize.rs:197–246, 997–1030 | Separate scenario/refusal/outside collections; component synthesis retains the whole refusal collection. |
| synthesize.rs:502–526 and all RefusalCause call sites | Four missing-check causes versus ten candidate causes; mapping is explicitly exhaustive for the baseline. |
| synthesize.rs:2068–2109, 2815–2846 | View/order and undeclared refusal checks can be missing beside an emitted scenario. |
| synthesize.rs:2697–2699 | DuplicateScenario is added after the existing scenario already owns the ID. |
| synthesize.rs:2910–2976 | InvariantUnobservable can coexist with a partial scenario or accompany no scenario when all invariants lack witnesses. |
| authored.rs:1049–1090, 1416–1483 | A failed file contributes refusals, no partial scenario; accepted sources update seen, so duplicate retains first source and a prior non-duplicate failure need not prevent later acceptance. |
| main.rs:2435–2488, 2493–2528, 2640–2705 | Run/synthesis/author paths handle the current collections differently and must preserve the newly bound full inventory; CLI target/report operations are downstream of suite creation. |
| evidence.rs:79–86, 134–154; scenario.rs:365–429 | v1 reader currently calls general suite support while v1 writer copies actual suite_version, requiring a frozen v1 allowlist independent of v5 execution support. |
| go/runtime.go:499–539, 623–641 | Generated runtime currently parses embedded suite then calls newTarget.Identity and writes a v1 report using embedded SuiteVersion; the pairing gate must precede target construction. |
| impact.rs:86–96, 830–850 | Existing envelope is ess-impact/2 and carries the suite provenance, distinct from a future exact-suite envelope. |

Selected exact source-read commands and results are retained below. All exited zero; no code executed from these reads.

```text
$ sed -n '79,86p;134,154p' crates/verify/ess-conformance/src/evidence.rs
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
exit: 0
```

```text
$ sed -n '499,539p;623,641p' crates/verify/ess-conformance/src/go/runtime.go
func Run(t *testing.T, newTarget func() Target) {
	t.Helper()

	var suite Suite
	if err := json.Unmarshal([]byte(suiteJSON), &suite); err != nil {
		t.Fatalf("the embedded suite does not parse, which is a generator defect: %v", err)
	}

	// Sorted, so the order scenarios run in is the order they are written in the report, on every
	// machine. Go randomises map iteration on purpose and a suite that reported in that order would
	// be unreadable across two runs.
	ids := make([]string, 0, len(suite.Scenarios))
	for id := range suite.Scenarios {
		ids = append(ids, id)
	}
	sort.Strings(ids)

	t.Logf(
		"%s %s, %d scenario(s), spec digest %s",
		suite.Provenance.System,
		suite.Provenance.SpecificationVersion,
		len(ids),
		suite.Provenance.SpecDigest,
	)

	// One extra target, for the name the report carries. Asked before any scenario runs so a
	// target that cannot name itself fails the run here, with that message, rather than leaving a
	// report that says nothing about what was tested.
	identity, err := newTarget().Identity()
	if err != nil {
		t.Fatalf("the target does not name itself: %v", err)
	}

	harness := NewHarness(suite.Provenance.System)
	results := make([]scenarioResult, 0, len(ids))
	for _, id := range ids {
		scenario := suite.Scenarios[id]
		// Read after the subtest, not returned from it: Skipf and Fatalf leave through Goexit,
		// and what the run had decided by then is the only record of how it left.
		status := statusPassed
		t.Run(id, func(t *testing.T) {
	case anySkipped:
		status = "inconclusive"
	}

	document := report{
		Format:          "ess-conformance-report/1",
		Specification:   suite.Provenance.System + "/" + suite.Provenance.SpecificationVersion,
		SpecDigest:      suite.Provenance.SpecDigest,
		Implementation:  identity.Name + " " + identity.Version,
		Status:          status,
		ScenariosTotal:  len(results),
		ScenariosFailed: len(failed),
		SuiteVersion:    suite.Provenance.SuiteVersion,
		FailedScenarios: failed,
		CompletedAt:     time.Now().UnixMilli(),
	}
	encoded, err := json.MarshalIndent(document, "", "  ")
	if err != nil {
		t.Errorf("the report does not encode, which is a runner defect: %v", err)
exit: 0
```

```text
$ sed -n '86,103p;830,850p' crates/verify/ess-diff/src/impact.rs
/// A label rather than a parsed type, and that is the difference between this and
/// [`DeltaFormat`](crate::DeltaFormat): a format version earns a parser when something reads the
/// document back and has to refuse a shape it does not understand. Nothing reads an impact report
/// back — see the module documentation — so a parser here would be a refusal that cannot fire, and
/// the word is carried anyway because a consumer keying on it costs nothing.
///
/// `/2` since wave 7: the document gained an `artifacts` section, and `suite` plus
/// `invalidation` became optional — a report can now be about the generated tree alone. The label
/// still has no parser, for the reason above, and it is bumped anyway because a label that stays
/// at `/1` across a shape change is a label that lies to the one consumer who does key on it.
pub const IMPACT_FORMAT: &str = "ess-impact/2";

/// Why a delta and a suite cannot be compared at all.
///
/// Every variant is a state a person can reach by pointing the command at the wrong file, and each
/// one is refused rather than answered — because the plausible answer in each case is the dangerous
/// one. Narrowing against the wrong suite produces a short list of scenarios that looks exactly like
/// a correct short list.
    let artifacts = analyse_artifacts(&delta, &graph, before, &inventory, tree, uncompared_moved);

    let churn = churn(
        &delta,
        &impacts,
        invalidation.as_ref(),
        suite,
        &artifacts,
        inventory.len(),
    );

    Ok(EssImpact {
        format: IMPACT_FORMAT,
        delta,
        suite: suite.map(|suite| suite.provenance.clone()),
        impacts,
        invalidation,
        artifacts,
        churn,
    })
}
exit: 0
```

```text
$ sed -n '2910,2985p;3048,3100p;3419,3485p' crates/verify/ess-conformance/src/synthesize.rs
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
        input_types(ir, command)
            .into_iter()
            .map(EssSemanticRef::from),
    );

    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    for declared in ir.types().values() {
        let invariants = match &declared.body {
            ResolvedBody::Newtype { invariants, .. } | ResolvedBody::Struct { invariants, .. } => {
                invariants
            }
            ResolvedBody::Enum { .. } | ResolvedBody::Union { .. } => continue,
        };
        if invariants.is_empty() {
            continue;
        }
        let positions = positions_of(ir, declared, invariants);
        if positions.is_empty() {
            refusals.push(Refusal {
                subject: DeclaredTypeRef::new(declared.name.clone()).into(),
                scenario: None,
                cause: RefusalCause::ValueInvariantUnwitnessed {
                    value: DeclaredTypeRef::new(declared.name.clone()),
                    invariants: invariants
                        .iter()
                        .map(|invariant| invariant.statement.clone())
                        .collect(),
                    at: None,
                },
            });
            continue;
        }
        for (view, field) in positions {
            let id = ScenarioId::ValueInvariant {
                value: DeclaredTypeRef::new(declared.name.clone()),
                at: ViewRef::new(view.name.clone()),
                field: field.clone(),
            };
            match holds_at(ir, declared, invariants, view, &field, actors) {
                Ok(scenario) => insert(suite, id, scenario, refusals),
                Err(cause) => refusals.push(Refusal::about(&id, cause)),
            }
        }
    }
}

/// Every view field position that can answer this type's invariants: reached through newtypes and
/// structs, and with every rebased path landing on a scalar.
///
/// What is deliberately *not* a position, and why each absence is a reading of the model rather
/// than a shortcut:
///
/// * inside a `List` or a `Map` — a fact path has no index or key selector, the rule
///   [`resolve_path`] already applies to every other surface;
/// * inside a `Union` — which variant a row holds is the row's business, so no single path is one
/// binding reacts to is published by a command that may itself need an instance driven into a state.
///
/// Where the trigger cannot be produced at all, the refusal is about the **binding** rather than
/// about one of its four scenarios: none of the four exists, and four copies of one reason is a
/// diagnostic a reader has to deduplicate by hand.
fn bindings(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    for binding in ir.bindings().values() {
        let subject = BindingRef::new(binding.name.clone());
        let event = EventRef::from(&binding.event);
        let Some((publisher, published_by)) = publisher(ir, &binding.event) else {
            refusals.push(Refusal {
                subject: subject.clone().into(),
                scenario: None,
                cause: RefusalCause::BindingUnobservable {
                    binding: subject,
                    gap: BindingGap::NothingPublishes { event },
                },
            });
            continue;
        };
        let trigger = match run(ir, publisher, published_by, actors) {
            Ok(run) => run,
            Err(cause) => {
                refusals.push(Refusal {
                    subject: subject.clone().into(),
                    scenario: None,
                    cause,
                });
                continue;
            }
        };

        let invoked = ir.command(&binding.command);
        let source = binding_source(ir, binding, publisher, published_by, &trigger);
        for aspect in BindingAspect::ALL.map(|(aspect, _)| aspect) {
            let id = ScenarioId::Binding {
                binding: subject.clone(),
                aspect,
            };
            let built = match aspect {
                BindingAspect::Flow => flow(ir, invoked, &trigger, &event),
                BindingAspect::Mapping => mapping(ir, binding, invoked, &trigger, &event),
                BindingAspect::Delivery => delivery(ir, binding, invoked, &trigger, &event),
                BindingAspect::OnFailure => on_failure(ir, binding, invoked, &trigger, &event),
            };
            let (steps, purpose, extra) = match built {
                Ok(built) => built,
                Err(gap) => {
                    refusals.push(Refusal::about(
                        &id,
                        RefusalCause::BindingUnobservable {
                            binding: subject.clone(),
                            gap,
                        },
                    ));
                    continue;
                }
            };
            let mut depends = source.clone();
            depends.extend(extra);
            insert(
                suite,
exit: 0
```

The complete matrix was read for one outcome and ownership in every row; these row counts measure document coverage, not test execution. Existing36 rows were retained or clarified and22 added, for58 total. The independent pairing table has10 rows.

```text
$ rg -c '^\| M[0-9][0-9] \|' docs/design/review-conformance-coverage.md
58
exit: 0
```

```text
$ rg -c '^\| P[0-9]+ \|' docs/design/review-conformance-coverage.md
10
exit: 0
```

```text
$ rg -n '^\| M[0-9][0-9] \|' docs/design/review-conformance-coverage.md
228:| M01 | Valid existing Rust v1 passed/failed/error/unsupported reports | Admit unchanged under v1 rules and preserve bytes; do not infer coverage | R, reader stage |
229:| M02 | Valid existing Go v1 passed/failed/skipped reports | Admit unchanged; scenarios_failed remains all non-passes | R/A, reader stage |
230:| M03 | A v1 report with category fields added without changing format | Closed v1 reader refuses | R/A |
231:| M04 | v2 presented to an existing v1 report reader | Refuse report/2; no count reinterpretation | R/A fixtures using retained old reader |
232:| M05 | Valid paired v2 Rust unsupported + separate error scenarios | unsupported=1, error=1, failed=0; execution_status and conformance_status failed | R |
233:| M06 | Rust error + unsupported checks within one scenario | One error result; execution_status inconclusive | R |
234:| M07 | Go skipped-only run | skipped equals total, failed=0; execution_status/conformance_status inconclusive; strict process fails | R |
235:| M08 | Go ordinary error or skipped followed by teardown error | One final failed result; no error/unsupported/skipped contribution for it | R |
236:| M09 | Unknown profile, outcome word, report field, or report/3 | Refuse before adaptation; never coerce to failed/skipped/passed | R/A |
237:| M10 | Counts do not sum, overflow, duplicate IDs, list/count disagreement, or mixed profile vocabulary | Refuse v2 report | R/A |
238:| M11 | Paired v2 with an omitted or extra scenario result | Refuse complete report, including results fabricated for filtered Go subtests | R/C |
239:| M12 | Valid old suite/1–4 with its own supported vocabulary | New readers admit legacy execution; v2 exact report remains unknown coverage | C |
240:| M13 | New nonempty suite/5 with complete inventory, no in_scope refusal and every selected scenario passed | Paired v2 conformance_status passed for that selection | C/R |
241:| M14 | Future suite/6 or /99 using only familiar steps | New readers refuse before any target callback or browser replay | C |
242:| M15 | New vocabulary mislabeled with an earlier suite major | New readers refuse before execution | C |
243:| M16 | Unknown typed suite field/step/coverage code or duplicate JSON key | New readers refuse before execution; payload-map keys remain data | C |
244:| M17 | Today's Rust runner or generated Go runtime receives suite/5 | Do not claim version-based refusal: these sources lack the check; deployment pairing is prohibited until upgraded/regenerated | C/O retained-runtime fixture |
245:| M18 | Today's browser player receives new suite metadata | Do not claim coverage/digest verification; publish the updated player with the matching suite first | C/O browser fixture |
246:| M19 | Empty suite and zero outcomes | execution_status passed may remain; conformance_status inconclusive; strict refuses qualification | R/C/A |
247:| M20 | Whole-system generated_and_authored, complete inventory, no refusals | Nonempty all-pass execution qualifies for that exact selection | C/R |
248:| M21 | Component selection, complete in-scope inventory, known outside scenarios and no in_scope refusal | Can qualify for the named component; cannot satisfy a system expectation | C/A |
249:| M22 | Authored-only or generated-only selection | Can qualify only for the same declared origin selection; combined requirement refuses it | C/A |
250:| M23 | Explicit subset with admitted parent, requested IDs and accounted omissions | Can qualify only for that explicit selection; broader requirement refuses it | C/A |
251:| M24 | Explicit subset hides a refusal or loses parent/ID binding | Refuse suite admission; cannot claim complete inventory | C |
252:| M25 | Nonempty all-pass selected suite with an in-scope synthesis/authored refusal | execution_status passed, conformance_status inconclusive; strict fails | C/R |
253:| M26 | Legacy suite rebuilt without original inventory, or unknown coverage | Diagnostic execution allowed; complete qualification refuses | C/R/A |
254:| M27 | Same spec_digest but changed authored step, dependency, selection, refusal or coverage list | Exact suite digest changes; old expected/report binding refuses | C/R/A |
255:| M28 | Correct report with a different exact suite, missing suite, or model digest mismatch | No qualifying evidence; paired mismatch refuses, missing bytes remains unverified | C/R/A |
256:| M29 | Whitespace/key order/default omission/number or Unicode escape/newline rewrite | Exact byte digest changes even if decoded values compare equal | C/R |
257:| M30 | Go reduced execution struct is marshaled to calculate digest | Fixture must fail; only retained complete suiteJSON may be hashed | C/R |
258:| M31 | Numeric precision edge, signed zero, combining Unicode, escaped controls | Golden canonical bytes and raw-byte hash match the specified profile; no silent normalization | C/R |
259:| M32 | Detailed CLI JSON/YAML treated as standalone report | Format/surface admission refuses the mismatch | R/A |
260:| M33 | Impact on a v5 component/subset suite with matching model provenance | Compute invalidation within the named selection and preserve the current ess-impact/2 envelope; do not claim whole-system execution coverage | C |
261:| M34 | Existing realization digest reference without referenced bytes/profile bridge | Remain an artifact reference, not verified exact-suite conformance | C/O |
262:| M35 | New report sent through old AEP domain result/predicates | Refuse v2 adaptation until the new domain/policy route exists; do not narrow the old failed fact | A |
263:| M36 | Suite/1–4 plus explicit report/1 and explicit allow-incomplete after default movement | P1: preserve legacy output/count/exit behavior and bytes; coverage unavailable | R/C/O |
264:| M37 | RefusalUndeclared beside a selected all-pass generated scenario | Admit check_not_emitted with that ID and retained generated identity; count one execution result plus one refusal; conformance_status inconclusive | C/R, inventory/report writers |
265:| M38 | ViewUndecidable and OrderUnwitnessed beside the same selected all-pass scenario | Preserve both missing-check records; count one execution result and two refusals; conformance_status inconclusive | C/R, inventory/report writers |
266:| M39 | InvariantUnobservable with some observable invariants and an all-pass survivor | Retain the useful scenario and each missing check; conformance_status inconclusive | C/R, inventory/report writers |
267:| M40 | InvariantUnobservable with every invariant unobservable | Preserve check_not_emitted records and known ID with retained null; no result for that ID; selected all-pass or empty run remains inconclusive | C/R, inventory/report writers |
268:| M41 | DuplicateScenario with first generated scenario selected and passing | Retain first scenario and candidate_not_emitted refusal at same ID; one execution result; conformance_status inconclusive | C/R, inventory/report writers |
269:| M42 | Duplicate authored source after an accepted source, same selected ID, all-pass execution | Preserve both source digests, rejected source and accepted retained.source from Duplicate.first; one execution result; conformance_status inconclusive | C/R, authored inventory/report writers |
270:| M43 | Authored non-duplicate compile refusal followed by a successful file with the same ID | Preserve refused/accepted source dispositions and known shared ID; retained points to accepted source; one execution result; in_scope refusal keeps conformance_status inconclusive | C/R, authored inventory/report writers |
271:| M44 | Generated/authored merge candidate collides with an already retained ID | Preserve survivor origin, candidate origin and known source identities beside the refusal; no replacement or extra execution result | C, merge writer/admission |
272:| M45 | Partial-check or duplicate-candidate refusal beside a known other-component outside scenario, with justified needs and complete_inventory; selected nonempty scenarios all pass | Retain scenario in outside and refusal with retained identity and outside_component scope; no result for outside ID; named component can qualify if no other in_scope gap exists | C/R, component inventory/report writers |
273:| M46 | Refusal without sufficient dependency/source information to prove component exclusion | Retain as in_scope; selected all-pass execution is inconclusive | C/R, component inventory/report writers |
274:| M47 | Explicit filter omits a parent scenario with an in_scope missing-check or candidate refusal | Move ID to selection_filter outside and retain refusal/identity/scope and source map unchanged; even all-pass retained execution is inconclusive | C/R, filter writer/admission |
275:| M48 | Explicit filter removes a refusal, reclassifies its scope, or drops the accepted/rejected source binding | Refuse child admission against its exact parent before execution | C, filter admission |
276:| M49 | Retained parent refusal has an origin excluded by an explicitly built origin selection | Keep visible as outside_origin; it does not block that origin-only selection but cannot satisfy a combined expectation | C/A, inventory/adaptation |
277:| M50 | Suite/5 with explicit or unset report/1 during opt-in phase, including missing report destination | P5: refuse before any Rust target/Go newTarget or report write; require explicit report/2 | R/C, CLI/Go pairing gate |
278:| M51 | Suite/5 with report/1 and allow-incomplete in either phase | P5: refuse before execution; override cannot discard v5 coverage | R/C, CLI/Go pairing gate |
279:| M52 | Suite/1–4 with explicit/default report/1 and strict resolved explicitly in either phase or by the new default | P2: refuse before execution; no automatic report promotion | R/C, CLI/Go pairing gate |
280:| M53 | Suite/1–4 with report/2 under default/explicit strict after movement | P4: execute with unknown coverage; failed execution yields failed, otherwise inconclusive; strict exits nonzero | R/C, CLI/Go pairing gate |
281:| M54 | Suite/1–4 with report/2 and allow-incomplete in either phase | P3: diagnostic execution exit behavior; coverage remains unknown and cannot qualify | R/C, CLI/Go pairing gate |
282:| M55 | v1 report names suite/5 after general execution support grows to include /5 | Frozen v1 reader refuses; v1 writer pairing prevents creation | R/A, report readers/writers |
283:| M56 | Both strict and allow-incomplete are explicit, or Go strictness variable is neither unset nor 1 | Refuse invalid invocation before execution, regardless of otherwise valid pairing | R/C, CLI/Go pairing gate |
284:| M57 | Upgraded runtime generated during the opt-in phase retains its old defaults after the standalone ESS binary upgrades | Resolve that runtime's actual defaults; embedded v5 with unset report stays refused under P5 until explicitly configured or regenerated | C/O, retained-runtime rollout fixture |
285:| M58 | Current persisted impact result versus a proposed exact-suite impact envelope | Preserve ess-impact/2 format and existing fields/meanings; future exact-suite fields require a separately versioned impact envelope | C/O, impact migration |
exit: 0
```

```text
$ rg -n '^\| P[0-9]+ \|' docs/design/review-conformance-coverage.md
201:| P1 | 1–4 | 1 | Diagnostic | Execute with frozen legacy output/count/exit behavior; no coverage qualification. |
202:| P2 | 1–4 | 1 | Strict | Refuse before execution: strict requires report/2; select report/2 or an explicit diagnostic override. |
203:| P3 | 1–4 | 2 | Diagnostic | Execute paired exact bytes; v2 categories with unknown coverage; conformance_status cannot pass. |
204:| P4 | 1–4 | 2 | Strict | Execute paired exact bytes; unknown coverage prevents success: failed execution yields failed, otherwise inconclusive; strict process exits nonzero. |
205:| P5 | 5 | 1 | Either | Refuse before execution, whether report/1 was explicit or default; require report/2. Neither allow-incomplete nor a missing report destination permits a v5-to-v1 downcast. |
206:| P6 | 5 | 2 | Diagnostic | Execute admitted exact selection; truthful v2 qualification, with producer execution exit behavior. |
207:| P7 | 5 | 2 | Strict | Execute admitted exact selection; process succeeds only for conformance_status passed and the expected selection. |
208:| P8 | Unsupported suite major or vocabulary | Any | Either | Refuse admission before execution, even if report selection itself is supported. |
209:| P9 | Supported suite | Unsupported report major | Either | Refuse report selection before execution. |
210:| P10 | Any | Any | Conflicting strict/allow-incomplete controls | Refuse invocation before execution. |
exit: 0
```

```text
$ rg -n 'ess-impact/[12]|pub const IMPACT_FORMAT' docs/design/review-conformance-coverage.md crates/verify/ess-diff/src/impact.rs
crates/verify/ess-diff/src/impact.rs:96:pub const IMPACT_FORMAT: &str = "ess-impact/2";
docs/design/review-conformance-coverage.md:31:| S9 | [impact.rs](../../crates/verify/ess-diff/src/impact.rs), lines 86–96, 775–851 and 893–930 | Impact compares system/model/contract provenance and intersects scenario dependencies. It does not verify an execution report or establish full coverage. Preserve existing ess-impact/2 bytes; admit v5 suites through an internal view, carry selection in diagnostics, and conservatively refuse evidence reuse when the exact selected suite is unavailable. |
docs/design/review-conformance-coverage.md:260:| M33 | Impact on a v5 component/subset suite with matching model provenance | Compute invalidation within the named selection and preserve the current ess-impact/2 envelope; do not claim whole-system execution coverage | C |
docs/design/review-conformance-coverage.md:285:| M58 | Current persisted impact result versus a proposed exact-suite impact envelope | Preserve ess-impact/2 format and existing fields/meanings; future exact-suite fields require a separately versioned impact envelope | C/O, impact migration |
docs/design/review-conformance-coverage.md:295:Impact continues using semantic dependencies for invalidation. A v5 admitted-suite view supplies the same dependency graph plus its selection; missing coverage cannot be interpreted as an empty dependency set. The current persisted format is **ess-impact/2** (IMPACT_FORMAT at impact.rs:96); /2 introduced artifacts and optional suite/invalidation. Historical ess-impact/1 is not the current producer contract. No new field is added silently to ess-impact/2 or its serialized SuiteProvenance, and its existing bytes/meanings remain frozen. To issue a future persisted impact result that binds exact suites, the impact owner must version that envelope separately; until then callers pair the original suite externally and cannot reuse a report solely from model/contract provenance. [S9]
exit: 0
```

```text
$ git --no-pager diff --check
exit: 0
```

```text
$ rg -n '[ \t]+$' docs/design/review-conformance-coverage.md
exit: 1 (no trailing-whitespace matches)
```

```text
$ git rev-parse HEAD
5148543b57c855cb4ccca92e2368e566801e9c36
exit: 0
```

```text
$ git status --short
 M docs/design/review-conformance-coverage.md
exit: 0
```

```text
$ df -B1 --output=avail .
       Avail
140291465216
exit: 0
```

5. Boundaries and remaining implementation obligations

Only the design document changed: 98 added and 31 removed lines in the correction diff. No production, test, manifest, executable checker, planning/store, sibling repository, source fixture or lifecycle surface was edited. The original implementation and adversary reports were preserved. No package test, Cargo formatter, Clippy, browser check, AEP command, Atlas operation, site build or network execution ran; the assigned document-only brief exempts package/prose tests and leaves full integration gates to the coordinator.

This report's green result is the completed document correction and whitespace checks, not new report/suite compatibility execution or approval. The matrix and P1–P10 must become real Rust/Go/consumer fixtures in the separately owned implementation stories. Atlas ADR, both AEP readers/domain/predicates, adopter runtime regeneration and final default movement remain staged obligations in the design.

The corrected document SHA256 is 84e5519a4aabc80c3f65da00d2f58456d90171837b44aa409a082cc6d4e5e2d0. Available disk at return is 140,291,465,216 bytes, above the 8,589,934,592-byte reserve; no build was performed. Token usage and full duration were not measured.

6. Outside paths

None. The only authored scratch file for this correction is /home/timo/.local/state/worktree/trees/b10x/ess/review-conformance-format-design/target/review-boundaries-3/correction-report.md. No unassigned scratch, cache purge, staging, commit, cleanup or worktree operation occurred. Writes are relinquished for coordinator handoff and the second review.

