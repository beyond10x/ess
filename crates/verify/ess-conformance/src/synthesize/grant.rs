//! Who may send what, on a served surface (beyond10x/ess#265).
//!
//! A served component's surface checks the caller's grant before a command runs, and refuses by
//! default: a request authenticated as no actor, or as an actor the specification does not grant
//! the command, gets the served contract's standard refusal. Three things follow for the suite,
//! and this module does all three, for commands a served component (`reached_by: network`)
//! accepts and for no other.
//!
//! - **Who is refused.** One `<command>/grant/denied` per command some declared actor lacks the
//!   grant for, sent as the lowest-named such actor: any of them is refused, and choosing by name
//!   is the choice that does not move when an unrelated actor is declared. The send reuses the
//!   arrangement of one of the command's own scenarios whose send takes an accepting branch —
//!   preferring one that snapshots the subject it acts on — so the refusal is asked of a request
//!   that would otherwise have been accepted. The log may then hold no more of anything the send
//!   could have published than it held before it. Where no scenario sends the command into an
//!   accepting branch, the refusal is withheld and a [`Note::GrantDeniedUnwitnessed`] names it;
//!   where every declared actor holds the grant there is nobody to refuse, and a
//!   [`Note::GrantedToEveryActor`] says so.
//! - **Who nobody is.** A command no declared actor is granted is refused to every caller. No
//!   scenario may send it as no actor and expect it to run: those are withheld, its refusal is the
//!   only witness, and a [`Note::GrantedToNoActor`] names them.
//! - **Who is admitted.** Every granted actor sends each command it is granted at least once: the
//!   command's scenarios are sent by its granted actors in turn, and only where it has fewer
//!   scenarios than granted actors is a `<command>/grant/admitted/<actor>` added, a copy of its
//!   simplest scenario sent as that actor. A command an actor carrying attributes holds keeps the
//!   actor synthesis chose, because its scenarios' expectations are read for that actor's values;
//!   a [`Note::GrantRotationSkipped`] names each such command.
//!
//! A model that declares actors and serves nothing gets one [`Note::GrantEnforcedByCaller`]
//! instead: no surface answers the refusal, and enforcing a grant is the caller's. A model that
//! declares no actor says nothing about who may invoke what, and gets none of this.

use super::{
    clipped, insert, not_emitted, ActorRef, BTreeSet, CommandRef, ConformanceScenario,
    ConformanceSuite, EssIr, EssSemanticRef, Note, QualifiedName, Refusal, ResolvedCommand,
    ScenarioId, ScenarioStep,
};

/// Every grant scenario and note the model's served surfaces owe, in that order: the commands no
/// actor may send, the actors that may, and the refusals.
pub(super) fn denied(
    ir: &EssIr,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
    notes: &mut Vec<Note>,
) {
    if ir.actors().is_empty() {
        return;
    }
    let served = served_commands(ir);
    if served.is_empty() {
        notes.push(Note::GrantEnforcedByCaller);
        return;
    }
    admit_every_granted_actor(ir, &served, suite, refusals, notes);
    // Every refusal is built before anything is withheld, so a command no actor is granted can still
    // borrow the accepting scenario that is about to be withheld for it.
    let mut denied = Vec::new();
    for command in ir.commands().values() {
        if !served.contains(&command.name) {
            continue;
        }
        let command_ref = CommandRef::new(command.name.clone());
        let Some(sender) = ir
            .actors()
            .values()
            .find(|actor| !grants(actor, &command.name))
        else {
            notes.push(Note::GrantedToEveryActor {
                command: command_ref,
            });
            continue;
        };
        let sender = ActorRef::new(sender.name.clone());
        match refused_from_template(ir, command, &sender, suite) {
            Some(scenario) => denied.push((
                ScenarioId::Grant {
                    command: command_ref,
                },
                scenario,
            )),
            None => notes.push(Note::GrantDeniedUnwitnessed {
                command: command_ref,
            }),
        }
    }
    withhold_ungranted(ir, &served, suite, notes);
    for (id, scenario) in denied {
        insert(suite, id, scenario, refusals);
    }
}

/// The commands whose scenarios [`admit_every_granted_actor`] sends in turn as each actor granted
/// them: served, held by two actors or more, none of them carrying attributes. Which actor one such
/// scenario is sent as depends on every other scenario of the suite sending the command, so a
/// synthesis that writes only some scenarios (`Focus::About`) cannot say it for them.
pub(super) fn rotated(ir: &EssIr) -> BTreeSet<QualifiedName> {
    if ir.actors().is_empty() {
        return BTreeSet::new();
    }
    served_commands(ir)
        .into_iter()
        .filter(|command| {
            let holders: Vec<&ess_compiler::ir::ResolvedActor> = ir
                .actors()
                .values()
                .filter(|actor| grants(actor, command))
                .collect();
            holders.len() >= 2 && holders.iter().all(|actor| actor.attributes.is_empty())
        })
        .collect()
}

/// The commands a served component accepts.
fn served_commands(ir: &EssIr) -> BTreeSet<QualifiedName> {
    ir.components()
        .values()
        .filter(|component| component.reached_by == ess_domain::component::Reach::Network)
        .flat_map(|component| {
            component
                .accepts
                .iter()
                .map(|command| command.name().clone())
        })
        .collect()
}

/// Whether `actor` may invoke `command`.
fn grants(actor: &ess_compiler::ir::ResolvedActor, command: &QualifiedName) -> bool {
    actor.may.iter().any(|granted| granted.name() == command)
}

/// The command a step sends, and as whom; `None` for a step that sends nothing.
fn sent(step: &ScenarioStep) -> Option<(&CommandRef, &Option<ActorRef>)> {
    match step {
        ScenarioStep::ExecuteCommand { command, actor, .. }
        | ScenarioStep::ExecuteCommandWithoutInput { command, actor, .. } => Some((command, actor)),
        _ => None,
    }
}

/// Whether `outcome` is a branch that accepts the command.
fn accepts(outcome: &ess_compiler::ir::ResolvedOutcome) -> bool {
    // An accepting branch reports no declared error; a `wrong_state:` branch accepts only where it
    // says so (`refuses` is read nowhere else).
    outcome.error.is_none()
        && (outcome.condition != ess_compiler::ir::ResolvedCondition::WrongState
            || !outcome.refuses)
}

/// A grant scenario, which this module owns and nothing it does rewrites.
fn is_grant(id: &ScenarioId) -> bool {
    matches!(
        id,
        ScenarioId::Grant { .. } | ScenarioId::GrantAdmitted { .. }
    )
}

/// Withholds every scenario that sends a served command no declared actor is granted, and says so.
///
/// Such a command is refused to every caller, so a scenario sending it as no actor and requiring
/// it to run requires what no conforming surface does.
fn withhold_ungranted(
    ir: &EssIr,
    served: &BTreeSet<QualifiedName>,
    suite: &mut ConformanceSuite,
    notes: &mut Vec<Note>,
) {
    for command in ir.commands().values() {
        if !served.contains(&command.name)
            || ir
                .actors()
                .values()
                .any(|actor| grants(actor, &command.name))
        {
            continue;
        }
        let withheld: Vec<ScenarioId> = suite
            .scenarios
            .iter()
            .filter(|(id, scenario)| {
                !is_grant(id)
                    && scenario
                        .steps
                        .iter()
                        .filter_map(sent)
                        .any(|(sent, _)| sent.name() == &command.name)
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in &withheld {
            suite.scenarios.remove(id);
        }
        notes.push(Note::GrantedToNoActor {
            command: CommandRef::new(command.name.clone()),
            withheld,
        });
    }
}

/// Sends each served command as each actor granted it, at least once.
fn admit_every_granted_actor(
    ir: &EssIr,
    served: &BTreeSet<QualifiedName>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
    notes: &mut Vec<Note>,
) {
    for command in ir.commands().values() {
        if !served.contains(&command.name) {
            continue;
        }
        let holders: Vec<&ess_compiler::ir::ResolvedActor> = ir
            .actors()
            .values()
            .filter(|actor| grants(actor, &command.name))
            .collect();
        if holders.len() < 2 {
            continue;
        }
        // A command an attributed actor holds is read for that actor's values: its scenarios keep
        // the actor synthesis chose, and a note says so.
        let attributed: Vec<ActorRef> = holders
            .iter()
            .filter(|actor| !actor.attributes.is_empty())
            .map(|actor| ActorRef::new(actor.name.clone()))
            .collect();
        if !attributed.is_empty() {
            notes.push(Note::GrantRotationSkipped {
                command: CommandRef::new(command.name.clone()),
                attributed,
            });
            continue;
        }
        let granted: Vec<ActorRef> = holders
            .iter()
            .map(|actor| ActorRef::new(actor.name.clone()))
            .collect();
        let sending: Vec<ScenarioId> = suite
            .scenarios
            .iter()
            .filter(|(id, scenario)| {
                !is_grant(id)
                    && scenario
                        .steps
                        .iter()
                        .filter_map(sent)
                        .any(|(sent, actor)| sent.name() == &command.name && actor.is_some())
            })
            .map(|(id, _)| id.clone())
            .collect();
        for (turn, id) in sending.iter().enumerate() {
            let actor = &granted[turn % granted.len()];
            if let Some(scenario) = suite.scenarios.get_mut(id) {
                send_as(scenario, &command.name, actor);
            }
        }
        // The simplest of them, sent as each actor no scenario is sent as yet.
        let Some(template) = sending
            .iter()
            .filter_map(|id| suite.scenarios.get(id))
            .min_by_key(|scenario| scenario.steps.len())
            .cloned()
        else {
            continue;
        };
        for actor in granted.iter().skip(sending.len()) {
            let mut scenario = template.clone();
            send_as(&mut scenario, &command.name, actor);
            scenario.purpose = clipped(&format!(
                "`{}` sent as `{actor}`, which the specification grants it, is accepted",
                command.name
            ));
            let id = ScenarioId::GrantAdmitted {
                command: CommandRef::new(command.name.clone()),
                actor: actor.clone(),
            };
            insert(suite, id, scenario, refusals);
        }
    }
}

/// Every step of `scenario` that sends `command` as some actor sends it as `actor` instead.
fn send_as(scenario: &mut ConformanceScenario, command: &QualifiedName, actor: &ActorRef) {
    let mut replaced = BTreeSet::new();
    for step in &mut scenario.steps {
        if let ScenarioStep::ExecuteCommand {
            command: sent,
            actor: Some(sender),
            ..
        }
        | ScenarioStep::ExecuteCommandWithoutInput {
            command: sent,
            actor: Some(sender),
            ..
        } = step
        {
            if sent.name() == command && sender != actor {
                replaced.insert(sender.clone());
                *sender = actor.clone();
            }
        }
    }
    let still_sent: BTreeSet<ActorRef> = scenario
        .steps
        .iter()
        .filter_map(sent)
        .filter_map(|(_, actor)| actor.clone())
        .collect();
    for gone in replaced.difference(&still_sent) {
        scenario.source.remove(&EssSemanticRef::from(gone.clone()));
    }
    scenario.source.insert(actor.clone().into());
}

/// The refusal, asked of a request one of the command's own scenarios would have had accepted:
/// its arrangement kept, its last send of the command sent as `sender`, and of what followed only
/// the claims that something is unchanged and the queries they read.
///
/// Only a scenario whose send then takes an accepting branch is borrowed: a send the command would
/// have refused anyway shows nothing about the grant. Among those, one that snapshots the subject
/// before the send is preferred, so the refusal is shown to have left it as it was; then the one
/// with fewest steps. `None` where no scenario sends the command into an accepting branch, and the
/// denied scenario is withheld with [`Note::GrantDeniedUnwitnessed`].
fn refused_from_template(
    ir: &EssIr,
    command: &ResolvedCommand,
    sender: &ActorRef,
    suite: &ConformanceSuite,
) -> Option<ConformanceScenario> {
    // The last send of the scenario, where it is this command's and the scenario then requires
    // one of the command's accepting branches of it: a send the command would have refused anyway
    // asks nothing of the grant.
    let accepted_send = |scenario: &ConformanceScenario| {
        let at = scenario
            .steps
            .iter()
            .rposition(|step| sent(step).is_some())?;
        let (sent_command, _) = sent(&scenario.steps[at])?;
        if sent_command.name() != &command.name {
            return None;
        }
        let required = scenario.steps[at + 1..]
            .iter()
            .find_map(|step| match step {
                ScenarioStep::ExpectOutcome { outcome }
                    if outcome.command.name() == &command.name =>
                {
                    Some(outcome)
                }
                _ => None,
            })?;
        command
            .outcomes
            .iter()
            .any(|declared| declared.name == required.outcome && accepts(declared))
            .then_some(at)
    };
    let snapshots = |step: &ScenarioStep| {
        matches!(
            step,
            ScenarioStep::SnapshotSubject { .. }
                | ScenarioStep::SnapshotCompleteSubject { .. }
                | ScenarioStep::SnapshotView { .. }
        )
    };
    let (template, at) = suite
        .scenarios
        .iter()
        .filter(|(id, _)| !is_grant(id))
        .filter_map(|(_, scenario)| accepted_send(scenario).map(|at| (scenario, at)))
        .min_by_key(|(scenario, at)| {
            let snapshotted = scenario.steps[..*at].iter().any(snapshots);
            (!snapshotted, scenario.steps.len())
        })?;
    // The arrangement as it was, external answers included: a surface that ran the refused send
    // would take the accepting branch the template took.
    let mut steps: Vec<ScenarioStep> = template.steps[..at].to_vec();
    let mut send = template.steps[at].clone();
    if let ScenarioStep::ExecuteCommand { actor, .. }
    | ScenarioStep::ExecuteCommandWithoutInput { actor, .. } = &mut send
    {
        *actor = Some(sender.clone());
    }
    steps.push(send);
    steps.push(ScenarioStep::ExpectNotGranted {
        actor: sender.clone(),
        unpublished: not_emitted(ir, &[]),
    });
    // Of what followed, only the claims that something is unchanged, each with the query of its
    // view it reads.
    let after = &template.steps[at + 1..];
    let unchanged_view = |step: &ScenarioStep| match step {
        ScenarioStep::ExpectSubjectUnchanged { view }
        | ScenarioStep::ExpectCompleteSubjectUnchanged { view }
        | ScenarioStep::ExpectViewUnchanged { view } => Some(view.clone()),
        _ => None,
    };
    for (offset, step) in after.iter().enumerate() {
        let kept = match step {
            ScenarioStep::QueryView { view, .. } => after[offset + 1..]
                .iter()
                .any(|later| unchanged_view(later).as_ref() == Some(view)),
            other => unchanged_view(other).is_some(),
        };
        if kept {
            steps.push(step.clone());
        }
    }
    let mut source = template.source.clone();
    source.insert(CommandRef::new(command.name.clone()).into());
    source.insert(sender.clone().into());
    source.extend(not_emitted(ir, &[]).into_iter().map(EssSemanticRef::from));
    Some(ConformanceScenario::new(
        purpose(command, sender),
        steps,
        source,
    ))
}

/// What a denied scenario proves.
fn purpose(command: &ResolvedCommand, sender: &ActorRef) -> crate::scenario::ScenarioPurpose {
    clipped(&format!(
        "`{}` sent as `{sender}`, which the specification does not grant it, is refused before \
         it runs",
        command.name
    ))
}
