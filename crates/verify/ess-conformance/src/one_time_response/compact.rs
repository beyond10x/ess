//! Rules a suite states once rather than per scenario (suite/52 and /53, beyond10x/ess#570).
//!
//! Two things a synthesized scenario spells out grow with the number of events a model declares,
//! whatever the scenario touches:
//!
//! - a one-time trace repeats the whole declared event catalog, one observation window per event
//!   per operation per window length, and the complete response authority of every marked origin;
//! - an act that publishes nothing (or one event of many) is followed by one `expect_no_event`
//!   step for every declared event it does not publish.
//!
//! Every one of those is a rule over the declared event catalog. From suite/52 the catalog is
//! written once at the suite root (`events`), with the one-time response authority of every marked
//! command (`one_time_response`). A scenario then keeps only what is its own: for a trace, the
//! origins it must observe and its longest wait; for a run of absent events, where it stands and
//! which catalog events it leaves out (`absent_events`); for a refused send whose `unpublished`
//! names the catalog, which written step it is and which events it leaves out
//! (`unpublished_events`).
//!
//! The compact spelling is a spelling, never a meaning: every reader expands it into the complete
//! scenario before admission, and the writer uses it only where that expansion reproduces the
//! in-memory scenario exactly, writing the complete form otherwise.
use super::{EventAuthority, EventWindow, Origin, Response, Trace};
use crate::scenario::{
    CommandRef, ConformanceScenario, ConformanceSuite, EssSemanticRef, EventRef, OutcomeRef,
    ScenarioId, ScenarioPurpose, ScenarioStep,
};
use ess_domain::OutcomeName;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

/// The ordinary vocabulary whose root may carry the catalogs.
pub const ORDINARY: u32 = 52;
/// Its inventory-bearing counterpart.
pub const COVERAGE: u32 = 53;

/// The ordinary major a fresh suite needs at least: [`ORDINARY`] when a scenario carries a trace,
/// so its rules are written once.
pub fn ordinary_floor(suite: &ConformanceSuite) -> Option<u32> {
    traced(suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`COVERAGE`] when a scenario carries
/// a trace.
pub fn coverage_floor(suite: &ConformanceSuite) -> Option<u32> {
    traced(suite).then_some(COVERAGE)
}

fn traced(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .values()
        .any(|scenario| scenario.one_time_response.is_some())
}

/// Suite-level one-time authority: each marked command's response, shared by every compact trace.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    /// Each marked command a compact trace invokes, by command name.
    pub commands: Vec<CatalogCommand>,
}

/// One marked command: its complete response authority and its marked outcomes in source order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogCommand {
    /// The command whose invocation produces the response.
    pub command: CommandRef,
    /// The complete declared response, shared by every marked outcome of the command.
    pub response: Response,
    /// Every marked outcome of the command, in source order.
    pub outcomes: Vec<CatalogOutcome>,
}

/// One marked outcome and its disclosure positions.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogOutcome {
    /// The successful outcome.
    pub outcome: OutcomeName,
    /// Its marked response fields.
    pub fields: Vec<String>,
}

/// What a scenario keeps of its trace once the suite root carries the rest.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactTrace {
    /// Origins this witness must actually observe successfully.
    pub required_origins: Vec<OutcomeRef>,
    /// The scenario's longest wait: every catalog event is scanned after each operation
    /// immediately and, when this is not zero, until it has elapsed.
    pub within_ms: u64,
}

/// The `unpublished` of one written `expect_not_granted` step: every catalog event outside
/// `except`, in catalog order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unpublished {
    /// The position of the step in the written `steps`.
    pub step: usize,
    /// The catalog events it leaves out, in catalog order.
    pub except: Vec<EventRef>,
}

/// A run of `expect_no_event` steps, one per catalog event outside `except`, in catalog order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbsentEvents {
    /// The position in the written `steps` the run stands before.
    pub before: usize,
    /// The catalog events the run leaves out, in catalog order.
    pub except: Vec<EventRef>,
}

fn operation(step: &ScenarioStep) -> bool {
    matches!(
        step,
        ScenarioStep::ExecuteCommand { .. }
            | ScenarioStep::ExecuteCommandWithoutInput { .. }
            | ScenarioStep::QueryView { .. }
            | ScenarioStep::EventuallyView { .. }
    )
}

fn invoked(steps: &[ScenarioStep]) -> BTreeSet<&CommandRef> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { command, .. } => Some(command),
            _ => None,
        })
        .collect()
}

/// The complete trace a compact one stands for, over the catalog `events`, in a scenario with
/// these (expanded) steps.
///
/// Origins are the catalog's marked outcomes of every command the steps invoke, in catalog order;
/// every catalog event is observed with `within_ms`; and a window opens after each operation for
/// each event, immediately and at `within_ms`.
pub fn expand_trace(
    events: &[EventRef],
    catalog: &Catalog,
    compact: &CompactTrace,
    steps: &[ScenarioStep],
) -> Result<Trace, String> {
    let invoked = invoked(steps);
    let mut origins = Vec::new();
    for entry in catalog
        .commands
        .iter()
        .filter(|entry| invoked.contains(&entry.command))
    {
        for outcome in &entry.outcomes {
            origins.push(Origin {
                command: entry.command.clone(),
                outcome: OutcomeRef::new(entry.command.clone(), outcome.outcome.clone()),
                response: entry.response.clone(),
                fields: outcome.fields.clone(),
            });
        }
    }
    let operations: Vec<_> = steps
        .iter()
        .enumerate()
        .filter(|(_, step)| operation(step))
        .map(|(index, _)| index)
        .collect();
    let lengths = BTreeSet::from([0, compact.within_ms]);
    if events
        .len()
        .saturating_mul(operations.len())
        .saturating_mul(lengths.len())
        > 65_536
    {
        return Err("one-time trace authority bound".into());
    }
    let mut event_windows = Vec::new();
    for event in events {
        for &after_step in &operations {
            for &within_ms in &lengths {
                event_windows.push(EventWindow {
                    event: event.clone(),
                    after_step,
                    within_ms,
                });
            }
        }
    }
    Ok(Trace {
        origins,
        required_origins: compact.required_origins.clone(),
        events: events
            .iter()
            .map(|event| EventAuthority {
                event: event.clone(),
                within_ms: compact.within_ms,
            })
            .collect(),
        event_windows,
    })
}

/// Whether `except` names catalog events in catalog order and leaves at least one out.
fn leaves_out(events: &[EventRef], except: &[EventRef]) -> bool {
    let mut last = None;
    for event in except {
        let Some(at) = events.iter().position(|declared| declared == event) else {
            return false;
        };
        if last.is_some_and(|last| at <= last) {
            return false;
        }
        last = Some(at);
    }
    except.len() < events.len()
}

/// The steps a compact scenario stands for: each run inserted before its written position, runs at
/// one position in the order they are listed.
pub fn expand_steps(
    events: &[EventRef],
    written: Vec<ScenarioStep>,
    runs: &[AbsentEvents],
) -> Result<Vec<ScenarioStep>, String> {
    if runs.windows(2).any(|pair| pair[1].before < pair[0].before)
        || runs.last().is_some_and(|run| run.before > written.len())
    {
        return Err("absent-event runs stand at ascending positions within the steps".into());
    }
    let mut steps = Vec::with_capacity(written.len() + runs.len() * events.len());
    let mut written = written.into_iter().enumerate().peekable();
    for run in runs {
        while written.peek().is_some_and(|(index, _)| *index < run.before) {
            steps.push(written.next().expect("peeked").1);
        }
        if !leaves_out(events, &run.except) {
            return Err(
                "an absent-event run leaves out catalog events in catalog order and keeps one"
                    .into(),
            );
        }
        steps.extend(
            events
                .iter()
                .filter(|event| !run.except.contains(event))
                .map(|event| ScenarioStep::ExpectNoEvent {
                    event: event.clone(),
                }),
        );
    }
    steps.extend(written.map(|(_, step)| step));
    Ok(steps)
}

/// The written steps and runs of `steps`: each maximal run of `expect_no_event` steps in catalog
/// order that is shorter to name by what it leaves out becomes one run.
fn compact_steps<'a>(
    events: &[EventRef],
    steps: &'a [ScenarioStep],
) -> (Cow<'a, [ScenarioStep]>, Vec<AbsentEvents>) {
    let position: BTreeMap<_, _> = events.iter().enumerate().map(|(i, e)| (e, i)).collect();
    let absent = |step: &ScenarioStep| match step {
        ScenarioStep::ExpectNoEvent { event } => position.get(event).copied(),
        _ => None,
    };
    let mut written = Vec::new();
    let mut runs = Vec::new();
    let mut index = 0;
    while index < steps.len() {
        let Some(mut last) = absent(&steps[index]) else {
            written.push(steps[index].clone());
            index += 1;
            continue;
        };
        let start = index;
        index += 1;
        while let Some(at) = steps.get(index).and_then(absent).filter(|at| *at > last) {
            last = at;
            index += 1;
        }
        let run = &steps[start..index];
        let kept: BTreeSet<_> = run.iter().filter_map(absent).collect();
        let except: Vec<_> = events
            .iter()
            .enumerate()
            .filter(|(at, _)| !kept.contains(at))
            .map(|(_, event)| event.clone())
            .collect();
        if except.len() < run.len() {
            runs.push(AbsentEvents {
                before: written.len(),
                except,
            });
        } else {
            written.extend(run.iter().cloned());
        }
    }
    if runs.is_empty() {
        (Cow::Borrowed(steps), runs)
    } else {
        (Cow::Owned(written), runs)
    }
}

/// The catalog events outside `named`, when `named` lists catalog events in catalog order.
fn except(events: &[EventRef], named: &[EventRef]) -> Option<Vec<EventRef>> {
    let mut rest = events.iter();
    let mut except = Vec::new();
    for event in named {
        loop {
            let next = rest.next()?;
            if next == event {
                break;
            }
            except.push(next.clone());
        }
    }
    except.extend(rest.cloned());
    Some(except)
}

/// Each written `expect_not_granted` step whose `unpublished` is shorter to name by what it leaves
/// out is written without it, and listed.
fn compact_unpublished(
    events: &[EventRef],
    steps: &mut Cow<'_, [ScenarioStep]>,
) -> Vec<Unpublished> {
    let mut patches = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        if let ScenarioStep::ExpectNotGranted { unpublished, .. } = step {
            if let Some(except) = except(events, unpublished)
                .filter(|except| !unpublished.is_empty() && except.len() < unpublished.len())
            {
                patches.push(Unpublished {
                    step: index,
                    except,
                });
            }
        }
    }
    if !patches.is_empty() {
        let steps = steps.to_mut();
        for patch in &patches {
            if let ScenarioStep::ExpectNotGranted { unpublished, .. } = &mut steps[patch.step] {
                unpublished.clear();
            }
        }
    }
    patches
}

/// Give each listed written `expect_not_granted` step, written without one, its `unpublished`.
fn expand_unpublished(
    events: &[EventRef],
    steps: &mut [ScenarioStep],
    patches: &[Unpublished],
) -> Result<(), String> {
    if patches.windows(2).any(|pair| pair[1].step <= pair[0].step) {
        return Err("unpublished-event lists name written steps in ascending order".into());
    }
    for patch in patches {
        let Some(ScenarioStep::ExpectNotGranted { unpublished, .. }) = steps.get_mut(patch.step)
        else {
            return Err("an unpublished-event list names an expect_not_granted step".into());
        };
        if !unpublished.is_empty() || !leaves_out(events, &patch.except) {
            return Err(
                "an unpublished-event list completes a step written without one, leaving out \
                 catalog events in catalog order and keeping one"
                    .into(),
            );
        }
        *unpublished = events
            .iter()
            .filter(|event| !patch.except.contains(event))
            .cloned()
            .collect();
    }
    Ok(())
}

/// The event catalog compact spellings refer to: every event a trace observes or an
/// `expect_no_event` step names, in name order.
fn events_of(suite: &ConformanceSuite) -> Vec<EventRef> {
    let mut events = BTreeSet::new();
    for scenario in suite.scenarios.values() {
        if let Some(trace) = &scenario.one_time_response {
            events.extend(trace.events.iter().map(|authority| &authority.event));
        }
        for step in &scenario.steps {
            if let ScenarioStep::ExpectNoEvent { event } = step {
                events.insert(event);
            }
            if let ScenarioStep::ExpectNotGranted { unpublished, .. } = step {
                events.extend(unpublished);
            }
        }
    }
    events.into_iter().cloned().collect()
}

/// Each marked command's authority as the first trace invoking it states it, by command name.
fn catalog_of(suite: &ConformanceSuite) -> Catalog {
    let mut commands: BTreeMap<&CommandRef, CatalogCommand> = BTreeMap::new();
    for trace in suite
        .scenarios
        .values()
        .filter_map(|scenario| scenario.one_time_response.as_ref())
    {
        let mut seen: BTreeMap<&CommandRef, CatalogCommand> = BTreeMap::new();
        for origin in &trace.origins {
            seen.entry(&origin.command)
                .or_insert_with(|| CatalogCommand {
                    command: origin.command.clone(),
                    response: origin.response.clone(),
                    outcomes: Vec::new(),
                })
                .outcomes
                .push(CatalogOutcome {
                    outcome: origin.outcome.outcome.clone(),
                    fields: origin.fields.clone(),
                });
        }
        for (command, entry) in seen {
            commands.entry(command).or_insert(entry);
        }
    }
    Catalog {
        commands: commands.into_values().collect(),
    }
}

/// How one scenario is written.
#[derive(serde::Serialize)]
pub(crate) struct ScenarioOut<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    one_time_response: Option<TraceOut<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    absent_events: Vec<AbsentEvents>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    unpublished_events: Vec<Unpublished>,
    purpose: &'a ScenarioPurpose,
    steps: Cow<'a, [ScenarioStep]>,
    source: Cow<'a, BTreeSet<EssSemanticRef>>,
}

#[derive(serde::Serialize)]
#[serde(untagged)]
enum TraceOut<'a> {
    Complete(&'a Trace),
    Compact(CompactTrace),
}

/// The written form of a suite.
pub(crate) struct Written<'a> {
    /// The event catalog, where a compact spelling refers to it.
    pub events: Option<Vec<EventRef>>,
    /// The one-time authority, where a compact trace refers to it.
    pub one_time_response: Option<Catalog>,
    /// Every scenario.
    pub scenarios: BTreeMap<&'a ScenarioId, ScenarioOut<'a>>,
}

/// The written form of `suite`.
///
/// Below suite/52 every scenario is written complete, as it always was. From suite/52 a trace the
/// catalogs reproduce is written compact, and its scenario's `source` omits the catalog events,
/// which the reader puts back; a trace they do not reproduce is written complete. A run of
/// `expect_no_event` steps is written as an absent-event run where that is shorter.
pub(crate) fn written(suite: &ConformanceSuite) -> Written<'_> {
    fn complete(scenario: &ConformanceScenario) -> ScenarioOut<'_> {
        ScenarioOut {
            one_time_response: scenario.one_time_response.as_ref().map(TraceOut::Complete),
            absent_events: Vec::new(),
            unpublished_events: Vec::new(),
            purpose: &scenario.purpose,
            steps: Cow::Borrowed(&scenario.steps),
            source: Cow::Borrowed(&scenario.source),
        }
    }
    if suite.provenance.suite_version.major() < ORDINARY {
        return Written {
            events: None,
            one_time_response: None,
            scenarios: suite
                .scenarios
                .iter()
                .map(|(id, scenario)| (id, complete(scenario)))
                .collect(),
        };
    }
    let events = events_of(suite);
    let mut catalog = catalog_of(suite);
    let mut used = BTreeSet::new();
    let mut referred = false;
    let mut scenarios = BTreeMap::new();
    for (id, scenario) in &suite.scenarios {
        let mut out = complete(scenario);
        if let Some(trace) = &scenario.one_time_response {
            let compact = CompactTrace {
                required_origins: trace.required_origins.clone(),
                within_ms: trace.events.first().map_or(0, |event| event.within_ms),
            };
            if expand_trace(&events, &catalog, &compact, &scenario.steps)
                .is_ok_and(|expanded| &expanded == trace)
            {
                used.extend(invoked(&scenario.steps).into_iter().cloned());
                referred = true;
                let mut source = scenario.source.clone();
                for event in &events {
                    source.remove(&EssSemanticRef::from(event.clone()));
                }
                out.one_time_response = Some(TraceOut::Compact(compact));
                out.source = Cow::Owned(source);
            }
        }
        let (mut steps, absent_events) = compact_steps(&events, &scenario.steps);
        let unpublished_events = compact_unpublished(&events, &mut steps);
        referred |= !absent_events.is_empty() || !unpublished_events.is_empty();
        out.steps = steps;
        out.absent_events = absent_events;
        out.unpublished_events = unpublished_events;
        scenarios.insert(id, out);
    }
    catalog
        .commands
        .retain(|entry| used.contains(&entry.command));
    Written {
        events: referred.then_some(events),
        one_time_response: (!used.is_empty()).then_some(catalog),
        scenarios,
    }
}

/// A trace as read: complete (every suite from /34), or compact (from /52, beside the catalogs).
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TraceIn {
    #[serde(default)]
    origins: Option<Vec<Origin>>,
    required_origins: Vec<OutcomeRef>,
    #[serde(default)]
    events: Option<Vec<EventAuthority>>,
    #[serde(default)]
    event_windows: Option<Vec<EventWindow>>,
    #[serde(default)]
    within_ms: Option<u64>,
}

impl TraceIn {
    /// The complete trace, or the compact one; never a mixture.
    fn spelling(self) -> Result<Result<Trace, CompactTrace>, String> {
        match self {
            Self {
                origins: Some(origins),
                required_origins,
                events: Some(events),
                event_windows: Some(event_windows),
                within_ms: None,
            } => Ok(Ok(Trace {
                origins,
                required_origins,
                events,
                event_windows,
            })),
            Self {
                origins: None,
                required_origins,
                events: None,
                event_windows: None,
                within_ms: Some(within_ms),
            } => Ok(Err(CompactTrace {
                required_origins,
                within_ms,
            })),
            _ => Err(
                "a one-time trace is either complete (origins, events, event_windows) or compact \
                 (required_origins, within_ms)"
                    .into(),
            ),
        }
    }
}

/// A list that is either left out or written: `null` is neither, and is refused as the Go and
/// TypeScript runtimes refuse it (review 2026-10-19).
fn listed<'de, D, T>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    <Vec<T> as serde::Deserialize>::deserialize(deserializer).map(Some)
}

/// A scenario as read, before it is expanded.
#[derive(serde::Deserialize)]
pub(crate) struct ScenarioIn {
    #[serde(default)]
    one_time_response: Option<TraceIn>,
    #[serde(default, deserialize_with = "listed")]
    absent_events: Option<Vec<AbsentEvents>>,
    #[serde(default, deserialize_with = "listed")]
    unpublished_events: Option<Vec<Unpublished>>,
    purpose: ScenarioPurpose,
    steps: Vec<ScenarioStep>,
    source: BTreeSet<EssSemanticRef>,
}

/// Expand every compact scenario of a suite as read against its catalogs.
pub(crate) fn read(
    major: u32,
    events: Option<&[EventRef]>,
    catalog: Option<&Catalog>,
    scenarios: BTreeMap<ScenarioId, ScenarioIn>,
) -> Result<BTreeMap<ScenarioId, ConformanceScenario>, String> {
    if (events.is_some() || catalog.is_some()) && major < ORDINARY {
        return Err("a suite-level event or one-time catalog requires suite/52 or /53".into());
    }
    if events.is_some_and(|events| !events.windows(2).all(|pair| pair[0] < pair[1])) {
        return Err("the event catalog lists each event once, in name order".into());
    }
    if catalog.is_some_and(|catalog| {
        !catalog
            .commands
            .windows(2)
            .all(|pair| pair[0].command < pair[1].command)
            || catalog
                .commands
                .iter()
                .any(|entry| entry.outcomes.is_empty())
    }) {
        return Err(
            "the one-time catalog lists each command once, in name order, with an outcome".into(),
        );
    }
    let mut used = BTreeSet::new();
    let mut referred = false;
    let mut result = BTreeMap::new();
    for (id, scenario) in scenarios {
        let mut source = scenario.source;
        let mut steps = scenario.steps;
        if let Some(patches) = scenario.unpublished_events {
            let events =
                events.ok_or("an unpublished-event list requires the suite event catalog")?;
            if patches.is_empty() {
                return Err("an empty unpublished_events is written as none".into());
            }
            referred = true;
            expand_unpublished(events, &mut steps, &patches)?;
        }
        let steps = match scenario.absent_events {
            None => steps,
            Some(runs) => {
                let events =
                    events.ok_or("an absent-event run requires the suite event catalog")?;
                if runs.is_empty() {
                    return Err("an empty absent_events is written as none".into());
                }
                referred = true;
                expand_steps(events, steps, &runs)?
            }
        };
        let trace = match scenario
            .one_time_response
            .map(TraceIn::spelling)
            .transpose()?
        {
            None => None,
            Some(Ok(trace)) => Some(trace),
            Some(Err(compact)) => {
                let (Some(events), Some(catalog)) = (events, catalog) else {
                    return Err(
                        "a compact one-time trace requires the suite event and one-time catalogs"
                            .into(),
                    );
                };
                referred = true;
                used.extend(invoked(&steps).into_iter().cloned());
                for event in events {
                    source.insert(event.clone().into());
                }
                Some(expand_trace(events, catalog, &compact, &steps)?)
            }
        };
        let mut built = ConformanceScenario::new(scenario.purpose, steps, source);
        built.one_time_response = trace;
        result.insert(id, built);
    }
    if events.is_some() && !referred {
        return Err("the suite event catalog is written only where a scenario refers to it".into());
    }
    if catalog.is_some_and(|catalog| {
        catalog
            .commands
            .iter()
            .any(|entry| !used.contains(&entry.command))
    }) {
        return Err("the one-time catalog carries authority no compact trace uses".into());
    }
    Ok(result)
}
