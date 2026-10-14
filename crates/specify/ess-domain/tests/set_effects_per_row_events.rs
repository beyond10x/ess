//! One event per changed row (ess/25, beyond10x/ess#531,
//! `docs/design/set-effects-over-filtered-instances.md`, "One event per changed row").
//!
//! An `emits:` entry `{event: <name>, each: changed, when_state: <states>}` on an outcome with
//! `instances:` publishes the event once per row the outcome changed that held one of those states
//! before it, its payload reading the row with `{changed: <field>}` and `{changed: state}`.

use ess_domain::command::{PayloadSource, RawOutcome};
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/set-effects-per-row.yaml");

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|error| {
        ValidationErrors::from(ess_primitives::error::ValidationError::new(
            ValidationCode::ConflictingDeclaration,
            "parse",
            error.to_string(),
        ))
    })?;
    Specification::assemble([(ess_domain::system::Source::new("desk.yaml"), raw)])
}

fn admitted(body: &str) -> Specification {
    assemble(body).unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{body}"))
}

fn refused(body: &str) -> ValidationErrors {
    match assemble(body) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

fn edited(from: &str, to: &str) -> String {
    assert!(MODEL.contains(from), "the fixture holds:\n{from}");
    MODEL.replacen(from, to, 1)
}

/// Exactly one error, with `code`, at a location containing `at`.
fn alone(errors: &ValidationErrors, code: ValidationCode, at: &str) {
    let all = errors.as_slice();
    assert_eq!(all.len(), 1, "one refusal, alone:\n{errors}");
    assert_eq!(all[0].code, code, "{errors}");
    assert!(
        all[0].location.contains(at),
        "refused at `{at}`, got:\n{errors}"
    );
}

fn has(errors: &ValidationErrors, code: ValidationCode, needle: &str) {
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == code && error.to_string().contains(needle)),
        "expected {code:?} mentioning `{needle}`, got:\n{errors}"
    );
}

const ENTRY: &str = "{event: demo.desk.Left, each: changed, when_state: Busy}";

#[test]
fn per_row_event_with_state_filter_validates_at_ess_25() {
    let spec = admitted(MODEL);
    let command = spec
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.desk.CloseAll")
        .expect("CloseAll is declared");
    let outcome = &command.outcomes[0];
    assert_eq!(
        outcome
            .emits
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["demo.desk.TeamClosed", "demo.desk.Left"],
        "the per-row event is one of the outcome's events, in the place written"
    );
    let set = outcome
        .set_effects
        .instances
        .as_ref()
        .expect("`instances:` is kept");
    let [event] = set.events.as_slice() else {
        panic!("one per-row event: {:#?}", set.events)
    };
    assert_eq!(event.event.to_string(), "demo.desk.Left");
    assert_eq!(
        event
            .when_state
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["Busy"]
    );
    let left = outcome
        .payload
        .iter()
        .find(|(name, _)| name.to_string() == "demo.desk.Left")
        .map(|(_, fields)| fields)
        .expect("Left's payload block");
    assert_eq!(
        left.get("user_id"),
        Some(&PayloadSource::ChangedField {
            field: "user_id".into()
        })
    );
    assert_eq!(
        left.get("was"),
        Some(&PayloadSource::ChangedField {
            field: "state".into()
        })
    );

    // Without a state filter every changed row publishes it, and the written form round-trips.
    let unfiltered = admitted(&edited(ENTRY, "{event: demo.desk.Left, each: changed}"));
    for spec in [&spec, &unfiltered] {
        for command in spec.commands().values() {
            for outcome in &command.outcomes {
                let written = ess_yaml::to_string(&RawOutcome::from(outcome.clone()))
                    .expect("an outcome serializes");
                let read: RawOutcome = ess_yaml::from_str(&written)
                    .unwrap_or_else(|error| panic!("{error}\n{written}"));
                let back = ess_domain::command::Outcome::try_from(read)
                    .unwrap_or_else(|errors| panic!("{errors}\n{written}"));
                assert_eq!(&back, outcome, "{written}");
            }
        }
    }

    // A list of states, as `when_subject_state:` spells one.
    admitted(&edited(
        ENTRY,
        "{event: demo.desk.Left, each: changed, when_state: [Open, Busy]}",
    ));
}

#[test]
fn per_row_event_is_refused_below_ess_25_and_on_a_single_instance_outcome() {
    // Below ess/25: the entry alone, at its place, naming ess/25.
    let below = refused(&MODEL.replacen("format: ess/25", "format: ess/24", 1));
    alone(
        &below,
        ValidationCode::UnsupportedFormatVersion,
        "outcomes.closed-all.emits[1]",
    );
    has(&below, ValidationCode::UnsupportedFormatVersion, "ess/25");

    // On an outcome with one subject: refused by name, alone.
    let single = edited(
        "      - name: taken
        moves: demo.desk.Seat.take
        instance: user_id
        emits: [demo.desk.CallTaken]",
        "      - name: taken
        moves: demo.desk.Seat.take
        instance: user_id
        emits: [{event: demo.desk.CallTaken, each: changed}]",
    );
    alone(
        &refused(&single),
        ValidationCode::UnsupportedConstruct,
        "outcomes.taken.emits[0]",
    );
}

#[test]
fn the_state_filter_and_the_changed_source_are_checked() {
    // A state the entity does not declare.
    has(
        &refused(&edited(
            ENTRY,
            "{event: demo.desk.Left, each: changed, when_state: Away}",
        )),
        ValidationCode::UndeclaredReference,
        "Away",
    );
    // A state the move never starts from: a row there is skipped, never changed.
    has(
        &refused(&edited(
            ENTRY,
            "{event: demo.desk.Left, each: changed, when_state: Closed}",
        )),
        ValidationCode::ConflictingDeclaration,
        "Closed",
    );
    // Listed twice in one outcome.
    has(
        &refused(&edited(
            "          - demo.desk.TeamClosed\n",
            "          - demo.desk.TeamClosed\n          - demo.desk.Left\n",
        )),
        ValidationCode::ConflictingDeclaration,
        "demo.desk.Left",
    );
    // A field the row does not hold.
    has(
        &refused(&edited("{changed: call_ref}", "{changed: phone}")),
        ValidationCode::UndeclaredReference,
        "phone",
    );
    // A field of another type.
    has(
        &refused(&edited(
            "user_id: {changed: user_id}",
            "user_id: {changed: team}",
        )),
        ValidationCode::TypeMismatch,
        "team",
    );
    // `{changed: …}` on an event published once.
    has(
        &refused(&edited(
            "{team: input.team, closed: {count: changed}}",
            "{team: {changed: team}, closed: {count: changed}}",
        )),
        ValidationCode::UnsupportedConstruct,
        "{changed: …}",
    );
}
