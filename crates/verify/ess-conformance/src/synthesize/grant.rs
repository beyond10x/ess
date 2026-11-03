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
//!   is the choice that does not move when an unrelated actor is declared. A `grant-drop` mutant
//!   of the mutation audit sends it as the actor that lost the grant ([`with_dropped_grant`],
//!   beyond10x/ess#710). The send reuses the
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
//!   a [`Note::GrantRotationSkipped`] names each such command. A `grant-add` mutant of the
//!   mutation audit that gave such a command to an actor carrying no attributes, where the command
//!   reads none, still gets a `grant/admitted/<actor>` for that actor ([`with_added_grant`]).
//!
//! A command the system issues itself (ess/27, beyond10x/ess#542) is awaited rather than sent, so
//! its system actors get no `grant/admitted` scenario and no rotation: the await is their witness.
//! Its refusal is still sent, as the lowest-named actor lacking the grant that is not the system
//! itself where there is one, because an outside caller must not be able to issue the system's
//! step.
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
        // A refusal of the system's own step is asked of an outside caller where one lacks the
        // grant: the system is no caller of its own surface (beyond10x/ess#542).
        let lacking = || {
            ir.actors()
                .values()
                .filter(|actor| !grants(actor, &command.name))
        };
        let Some(sender) = dropped_sender(ir, &command.name)
            .or_else(|| lacking().find(|actor| command.system_issued && !actor.system))
            .or_else(|| lacking().next())
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

/// Each refusal [`denied`] withheld for want of an accepting send, built where a scenario added to
/// the suite since sends the command into an accepting branch: rows arranged by callers chosen for
/// them (`caller::arranged`, downstream report, 2026-10-18). `stamp` sends the refused send as the
/// caller its actor carries; the note goes with the refusal it named.
pub(super) fn denied_since(
    ir: &EssIr,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
    notes: &mut Vec<Note>,
    stamp: &dyn Fn(&mut ScenarioStep),
) {
    let unwitnessed: Vec<CommandRef> = notes
        .iter()
        .filter_map(|note| match note {
            Note::GrantDeniedUnwitnessed { command } => Some(command.clone()),
            _ => None,
        })
        .collect();
    for command_ref in unwitnessed {
        let Some(command) = ir.commands().get(command_ref.name()) else {
            continue;
        };
        let Some(sender) = dropped_sender(ir, &command.name).or_else(|| {
            ir.actors()
                .values()
                .find(|actor| !grants(actor, &command.name))
        }) else {
            continue;
        };
        let sender = ActorRef::new(sender.name.clone());
        let Some(mut scenario) = refused_from_template(ir, command, &sender, suite) else {
            continue;
        };
        // The refused send only: every step before it keeps the caller it was arranged as.
        let refused = scenario
            .steps
            .iter()
            .position(|step| matches!(step, ScenarioStep::ExpectNotGranted { .. }))
            .and_then(|at| at.checked_sub(1));
        if let Some(send) = refused.and_then(|at| scenario.steps.get_mut(at)) {
            stamp(send);
        }
        notes.retain(|note| {
            !matches!(note, Note::GrantDeniedUnwitnessed { command } if *command == command_ref)
        });
        insert(
            suite,
            ScenarioId::Grant {
                command: command_ref,
            },
            scenario,
            refusals,
        );
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
        .filter(|command| !system_issued(ir, command))
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

/// Whether the system issues `command` itself (beyond10x/ess#542).
fn system_issued(ir: &EssIr, command: &QualifiedName) -> bool {
    ir.commands()
        .get(command)
        .is_some_and(|declared| declared.system_issued)
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

/// One grant the mutation audit considers (beyond10x/ess#710): `actor` and a command, whether
/// `actor` holds it, and why the audit leaves it alone where it does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuditedGrant {
    /// The actor, by qualified name.
    pub(crate) actor: String,
    /// The command, by qualified name.
    pub(crate) command: String,
    /// Whether the actor's `may:` names the command.
    pub(crate) held: bool,
    /// Why no mutant changes it, where none does.
    pub(crate) excluded: Option<GrantExclusion>,
}

/// Why the mutation audit leaves a grant alone (beyond10x/ess#710, #711).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GrantExclusion {
    /// A held grant of the system's own actor: it is no caller of a served surface.
    SystemActor,
    /// A held grant of a command no served component accepts: no surface answers its refusal.
    CommandNotServed,
    /// A grant the actor lacks of a command an actor carrying attributes holds, where the actor
    /// carries attributes or the command reads one: [`admit_every_granted_actor`] keeps the actor
    /// synthesis chose for it, and no scenario can be sent as the actor that gained the grant.
    AttributedHolder,
}

/// Every [`AuditedGrant`] of `ir`, in byte order of actor and then command; none where the model
/// declares no actor, as [`denied`] then writes nothing.
///
/// Each actor that is not the system's has one per command a served component accepts, held or
/// not; every other `may:` entry, of the system's actor or of a command nothing serves, is one
/// with its [`GrantExclusion`], so no grant the model declares goes unaccounted for.
pub(crate) fn audited_grants(ir: &EssIr) -> Vec<AuditedGrant> {
    let served = served_commands(ir);
    let mut found: Vec<AuditedGrant> = Vec::new();
    for actor in ir.actors().values() {
        let entry = |command: &QualifiedName, held, excluded| AuditedGrant {
            actor: actor.name.to_string(),
            command: command.to_string(),
            held,
            excluded,
        };
        for granted in &actor.may {
            let command = granted.name();
            if actor.system {
                found.push(entry(command, true, Some(GrantExclusion::SystemActor)));
            } else if !served.contains(command) {
                found.push(entry(command, true, Some(GrantExclusion::CommandNotServed)));
            }
        }
        if actor.system {
            continue;
        }
        for command in &served {
            let held = grants(actor, command);
            let excluded = (!held && !admitted_as_gainer(ir, actor, command))
                .then_some(GrantExclusion::AttributedHolder);
            found.push(entry(command, held, excluded));
        }
    }
    found.sort_by(|left, right| (&left.actor, &left.command).cmp(&(&right.actor, &right.command)));
    found.dedup_by(|left, right| (&left.actor, &left.command) == (&right.actor, &right.command));
    found
}

/// Whether a `grant-add` mutant giving `actor` the served `command` has a scenario sending the
/// command as `actor`: where no holder does yet, the mutant's only holder is `actor`; where no
/// holder, `actor` included, carries attributes, the rotation sends it as every holder
/// ([`admit_every_granted_actor`]); otherwise only if `actor` carries none and the command reads
/// none, so the copy [`admit_added_grant`] sends asks what it asked the holder (beyond10x/ess#710).
fn admitted_as_gainer(
    ir: &EssIr,
    actor: &ess_compiler::ir::ResolvedActor,
    command: &QualifiedName,
) -> bool {
    let holders: Vec<&ess_compiler::ir::ResolvedActor> = ir
        .actors()
        .values()
        .filter(|holder| grants(holder, command))
        .collect();
    let attributed =
        !actor.attributes.is_empty() || holders.iter().any(|holder| !holder.attributes.is_empty());
    holders.is_empty()
        || !attributed
        || (actor.attributes.is_empty() && super::caller::reads(ir, command).is_empty())
}

/// The grant a `grant-add` mutant of the mutation audit gave: `actor` may now invoke `command`
/// (beyond10x/ess#710). Only [`with_added_grant`] names one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AddedGrant {
    /// The actor, by qualified name.
    pub(crate) actor: String,
    /// The command, by qualified name.
    pub(crate) command: String,
}

thread_local! {
    /// The added grant of the `grant-add` mutant being synthesized on this thread, if any.
    static ADDED: std::cell::RefCell<Option<AddedGrant>> = const { std::cell::RefCell::new(None) };
}

/// Runs `synthesize` with `added` as the grant the model under synthesis gained, and restores
/// what was there before, also on a panic.
///
/// Where an actor carrying attributes holds the command, so its scenarios keep the actor
/// synthesis chose, a `<command>/grant/admitted/<actor>` scenario is still sent as the actor that
/// gained the grant ([`admit_added_grant`]). Every synthesis outside the mutation audit runs with
/// none, and its bytes are unchanged.
pub(crate) fn with_added_grant<R>(added: Option<AddedGrant>, synthesize: impl FnOnce() -> R) -> R {
    struct Restore(Option<AddedGrant>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let previous = self.0.take();
            ADDED.with(|it| *it.borrow_mut() = previous);
        }
    }
    let previous = ADDED.with(|it| it.replace(added));
    let _restore = Restore(previous);
    synthesize()
}

/// The actor [`with_added_grant`] names for `command`, where `ir` declares it, it holds the grant,
/// carries no attributes and the command reads none.
fn added_sender<'a>(
    ir: &'a EssIr,
    command: &QualifiedName,
) -> Option<&'a ess_compiler::ir::ResolvedActor> {
    let added = ADDED.with(|it| it.borrow().clone())?;
    if added.command != command.to_string() || !super::caller::reads(ir, command).is_empty() {
        return None;
    }
    ir.actors().values().find(|actor| {
        actor.name.to_string() == added.actor
            && actor.attributes.is_empty()
            && grants(actor, command)
    })
}

/// The grant a `grant-drop` mutant of the mutation audit removed: `actor` may no longer invoke
/// `command` (beyond10x/ess#710). Only [`with_dropped_grant`] names one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DroppedGrant {
    /// The actor, by qualified name.
    pub(crate) actor: String,
    /// The command, by qualified name.
    pub(crate) command: String,
}

thread_local! {
    /// The dropped grant of the `grant-drop` mutant being synthesized on this thread, if any.
    static DROPPED: std::cell::RefCell<Option<DroppedGrant>> =
        const { std::cell::RefCell::new(None) };
}

/// Runs `synthesize` with `dropped` as the grant the model under synthesis no longer holds, and
/// restores what was there before, also on a panic.
///
/// The `<command>/grant/denied` refusal is then sent as the actor that lost the grant wherever it
/// now lacks it, rather than as the lowest-named lacking actor, which a correct target refuses
/// too. Every synthesis outside the mutation audit runs with none, and its bytes are unchanged.
pub(crate) fn with_dropped_grant<R>(
    dropped: Option<DroppedGrant>,
    synthesize: impl FnOnce() -> R,
) -> R {
    struct Restore(Option<DroppedGrant>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let previous = self.0.take();
            DROPPED.with(|it| *it.borrow_mut() = previous);
        }
    }
    let previous = DROPPED.with(|it| it.replace(dropped));
    let _restore = Restore(previous);
    synthesize()
}

/// The actor [`with_dropped_grant`] names for `command`, where `ir` declares it and it lacks the
/// grant.
fn dropped_sender<'a>(
    ir: &'a EssIr,
    command: &QualifiedName,
) -> Option<&'a ess_compiler::ir::ResolvedActor> {
    let dropped = DROPPED.with(|it| it.borrow().clone())?;
    if dropped.command != command.to_string() {
        return None;
    }
    ir.actors()
        .values()
        .find(|actor| actor.name.to_string() == dropped.actor && !grants(actor, command))
}

/// Attribute-free actors still denote distinct principals even when their grants are identical.
/// Choose a declared arranger and another declared actor of the command under test. With no
/// credential-dependent predicates, actor substitution preserves the source-selected outcomes.
pub(super) fn cross_caller(ir: &EssIr, suite: &mut ConformanceSuite, notes: &mut Vec<Note>) {
    for (id, scenario) in &mut suite.scenarios {
        let Some(command) =
            super::caller::under_test(id).and_then(|name| name.parse::<QualifiedName>().ok())
        else {
            continue;
        };
        if scenario.steps.iter().any(|step| matches!(step,
            ScenarioStep::ExecuteCommand { caller, .. } | ScenarioStep::ExecuteCommandWithoutInput { caller, .. } if !caller.is_empty())) {
            continue;
        }
        let commands: BTreeSet<_> = scenario
            .steps
            .iter()
            .filter_map(sent)
            .map(|(sent, _)| sent.name().clone())
            .collect();
        let arranging: BTreeSet<_> = commands.iter().filter(|name| **name != command).collect();
        if !commands.contains(&command)
            || (arranging.is_empty() && scenario.steps.iter().filter_map(sent).count() < 2)
        {
            continue;
        }
        let pair = ir
            .actors()
            .values()
            .filter(|actor| actor.attributes.is_empty())
            .filter(|actor| {
                arranging.iter().all(|name| grants(actor, name))
                    && (!arranging.is_empty() || grants(actor, &command))
            })
            .find_map(|first| {
                ir.actors()
                    .values()
                    .find(|second| {
                        second.name != first.name
                            && second.attributes.is_empty()
                            && grants(second, &command)
                    })
                    .map(|second| (first, second))
            });
        let Some((first, second)) = pair else {
            continue;
        };
        let original = scenario.clone();
        if arranging.is_empty() {
            let first = ActorRef::new(first.name.clone());
            let second = ActorRef::new(second.name.clone());
            split_invocations(scenario, &first, &second);
            let mut reversed = original;
            split_invocations(&mut reversed, &second, &first);
            if let Err(reason) = super::caller::append_independent(ir, scenario, &reversed) {
                notes.push(Note::CrossCallerUnswapped {
                    scenario: id.clone(),
                    mixed: true,
                    reason: reason.to_owned(),
                });
            }
            continue;
        }
        for name in &commands {
            send_as(
                scenario,
                name,
                &ActorRef::new(if name == &command {
                    second.name.clone()
                } else {
                    first.name.clone()
                }),
            );
        }
        if grants(first, &command) && arranging.iter().all(|name| grants(second, name)) {
            let mut reversed = original;
            for name in &commands {
                send_as(
                    &mut reversed,
                    name,
                    &ActorRef::new(if name == &command {
                        first.name.clone()
                    } else {
                        second.name.clone()
                    }),
                );
            }
            if let Err(reason) = super::caller::append_independent(ir, scenario, &reversed) {
                notes.push(Note::CrossCallerUnswapped {
                    scenario: id.clone(),
                    mixed: true,
                    reason: reason.to_owned(),
                });
            }
        } else {
            notes.push(Note::CrossCallerUnswapped {
                scenario: id.clone(),
                mixed: true,
                reason: "the declared grants do not permit the reversed caller assignment"
                    .to_owned(),
            });
        }
    }
}

/// With no credential reads, the first invocation arranges and subsequent invocations act.
fn split_invocations(scenario: &mut ConformanceScenario, first: &ActorRef, second: &ActorRef) {
    let mut ordinal = 0;
    for step in &mut scenario.steps {
        let (ScenarioStep::ExecuteCommand { actor, .. }
        | ScenarioStep::ExecuteCommandWithoutInput { actor, .. }) = step
        else {
            continue;
        };
        *actor = Some(if ordinal == 0 {
            first.clone()
        } else {
            second.clone()
        });
        ordinal += 1;
    }
    scenario.source.insert(first.clone().into());
    scenario.source.insert(second.clone().into());
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
        // A system actor's grant is witnessed by the await of the step it issues, never by a send
        // (beyond10x/ess#542).
        if !served.contains(&command.name) || command.system_issued {
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
            admit_added_grant(ir, &command.name, suite, refusals);
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

/// Under [`with_added_grant`], the `<command>/grant/admitted/<actor>` scenario of the actor that
/// gained the grant of a command whose rotation was skipped: a copy of the simplest scenario that
/// sends the command as some actor, each such send sent as the gaining actor with no caller
/// values, which it declares none of and the command reads none of (beyond10x/ess#710). A target
/// that refuses the actor the specification does not grant the command fails it.
fn admit_added_grant(
    ir: &EssIr,
    command: &QualifiedName,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let Some(gainer) = added_sender(ir, command) else {
        return;
    };
    let Some(mut scenario) = suite
        .scenarios
        .iter()
        .filter(|(id, scenario)| {
            !is_grant(id)
                && scenario
                    .steps
                    .iter()
                    .filter_map(sent)
                    .any(|(sent, actor)| sent.name() == command && actor.is_some())
        })
        .map(|(_, scenario)| scenario)
        .min_by_key(|scenario| scenario.steps.len())
        .cloned()
    else {
        return;
    };
    let actor = ActorRef::new(gainer.name.clone());
    send_as(&mut scenario, command, &actor);
    for step in &mut scenario.steps {
        if let ScenarioStep::ExecuteCommand {
            command: sent,
            caller,
            ..
        }
        | ScenarioStep::ExecuteCommandWithoutInput {
            command: sent,
            caller,
            ..
        } = step
        {
            if sent.name() == command {
                caller.clear();
            }
        }
    }
    scenario.purpose = clipped(&format!(
        "`{command}` sent as `{actor}`, which the specification grants it, is accepted"
    ));
    let id = ScenarioId::GrantAdmitted {
        command: CommandRef::new(command.clone()),
        actor,
    };
    insert(suite, id, scenario, refusals);
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
/// with fewest steps; a template whose denied scenario the suite's admission would refuse is passed
/// over for the next. A scenario whose last send is the command's is preferred to every scenario
/// that sends another command after it, as a `ranked` view check does (beyond10x/ess#712). `None`
/// where no scenario sends the command into an accepting branch, or none
/// that gives an admissible one, and the denied scenario is withheld with
/// [`Note::GrantDeniedUnwitnessed`].
fn refused_from_template(
    ir: &EssIr,
    command: &ResolvedCommand,
    sender: &ActorRef,
    suite: &ConformanceSuite,
) -> Option<ConformanceScenario> {
    // The send at `at`, where the scenario then requires one of the command's accepting branches
    // of it before anything else is sent: a send the command would have refused anyway asks
    // nothing of the grant.
    let accepted_at = |scenario: &ConformanceScenario, at: usize| {
        let required = scenario.steps[at + 1..]
            .iter()
            .take_while(|step| sent(step).is_none())
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
    // The scenario's last send, where it is this command's.
    let last_send = |scenario: &ConformanceScenario| {
        let at = scenario
            .steps
            .iter()
            .rposition(|step| sent(step).is_some())?;
        let (sent_command, _) = sent(&scenario.steps[at])?;
        (sent_command.name() == &command.name).then_some(at)
    };
    // The scenario's last send of this command, where a later send of another follows it: a
    // `ranked` view check appends a second creating send after the command's own
    // (beyond10x/ess#712). `denied_from` drops every later send.
    let last_own_send = |scenario: &ConformanceScenario| {
        let at = scenario
            .steps
            .iter()
            .rposition(|step| sent(step).is_some_and(|(sent, _)| sent.name() == &command.name))?;
        (last_send(scenario) != Some(at)).then_some(at)
    };
    // Today's templates first, so every refusal a scenario ending in the command's send gave keeps
    // its bytes; then those whose send of the command is followed by another's.
    templates(ir, command, sender, suite, |scenario| {
        last_send(scenario).and_then(|at| accepted_at(scenario, at))
    })
    .or_else(|| {
        templates(ir, command, sender, suite, |scenario| {
            last_own_send(scenario).and_then(|at| accepted_at(scenario, at))
        })
    })
}

/// The first admissible denied scenario among the templates `accepted_send` finds, in
/// [`refused_from_template`]'s order of preference.
fn templates(
    ir: &EssIr,
    command: &ResolvedCommand,
    sender: &ActorRef,
    suite: &ConformanceSuite,
    accepted_send: impl Fn(&ConformanceScenario) -> Option<usize>,
) -> Option<ConformanceScenario> {
    let snapshots = |step: &ScenarioStep| {
        matches!(
            step,
            ScenarioStep::SnapshotSubject { .. }
                | ScenarioStep::SnapshotCompleteSubject { .. }
                | ScenarioStep::SnapshotView { .. }
        )
    };
    let mut templates: Vec<_> = suite
        .scenarios
        .iter()
        .filter(|(id, _)| !is_grant(id))
        .filter_map(|(_, scenario)| accepted_send(scenario).map(|at| (scenario, at)))
        .collect();
    // Stable, so templates alike in both keys keep the suite's order, as the first minimum did.
    templates.sort_by_key(|(scenario, at)| {
        let snapshotted = scenario.steps[..*at].iter().any(snapshots);
        (!snapshotted, scenario.steps.len())
    });
    // A template whose kept prefix leaves a retained result or a complete snapshot open — a
    // replay's `capture_command_result`, whose `expect_replay_result` followed the send — gives a
    // denied scenario the suite's own admission refuses; the next template is taken instead.
    templates
        .into_iter()
        .map(|(template, at)| denied_from(ir, command, sender, template, at))
        .find(|denied| crate::replay::validate_steps(&denied.steps).is_ok())
}

/// The denied scenario `template` gives with its send at `at` sent as `sender`, as
/// [`refused_from_template`] describes it.
fn denied_from(
    ir: &EssIr,
    command: &ResolvedCommand,
    sender: &ActorRef,
    template: &ConformanceScenario,
    at: usize,
) -> ConformanceScenario {
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
        actor: Some(sender.clone()),
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
    ConformanceScenario::new(purpose(command, sender), steps, source)
}

/// What a denied scenario proves.
fn purpose(command: &ResolvedCommand, sender: &ActorRef) -> crate::scenario::ScenarioPurpose {
    clipped(&format!(
        "`{}` sent as `{sender}`, which the specification does not grant it, is refused before \
         it runs",
        command.name
    ))
}
