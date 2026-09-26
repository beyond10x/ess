//! Outcome groups: one outcome declared once for many commands (beyond10x/ess#105).
//!
//! `docs/design/outcome-groups.md` is the binding design. C2, C4 and C5 are its deciding checks of
//! the same names, and each `g<n>_` case is one row of its G1–G16 table, asserting the
//! `ValidationCode`, the location and how many refusals the rule raises. The stable
//! `ESS-COMMAND-…` codes and the cited lines are asserted in `ess-compiler`'s test of the same name.
use ess_domain::{
    command::RawCommandSpec, outcome_group, spec::RawSpecFile, system::Source, Specification,
};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const SYSTEM: &str = include_str!("fixtures/outcome-groups/system.yaml");
const CALLS: &str = include_str!("fixtures/outcome-groups/domains/calls.yaml");
const SESSION: &str = include_str!("fixtures/outcome-groups/domains/session.yaml");

/// A file's text with its `outcome_groups:` section, which the fixture writes last, removed.
fn without_groups(text: &str) -> &str {
    text.split_once("outcome_groups:\n")
        .map_or(text, |(head, _)| head)
}

/// The fixture with its two groups replaced: `calls` goes in the calls file, `session` in the
/// session file, each written as the body of an `outcome_groups:` list (or empty for none).
fn tree(system: &str, calls: &str, session: &str) -> Vec<(Source, RawSpecFile)> {
    let mut files = Vec::new();
    for (label, text, groups) in [
        ("system.yaml", system, ""),
        ("domains/calls.yaml", without_groups(CALLS), calls),
        ("domains/session.yaml", without_groups(SESSION), session),
    ] {
        let text = if groups.is_empty() {
            text.to_owned()
        } else {
            format!("{text}outcome_groups:\n{groups}")
        };
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
        files.push((Source::new(label), raw));
    }
    files
}

/// The fixture exactly as written.
fn fixture() -> Vec<(Source, RawSpecFile)> {
    [
        ("system.yaml", SYSTEM),
        ("domains/calls.yaml", CALLS),
        ("domains/session.yaml", SESSION),
    ]
    .into_iter()
    .map(|(label, text)| {
        (
            Source::new(label),
            RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
        )
    })
    .collect()
}

/// The fixture's calls file with `groups` as its only group list and no group in the session file.
fn groups(body: &str) -> Vec<(Source, RawSpecFile)> {
    tree(SYSTEM, body, "")
}

fn refused(files: Vec<(Source, RawSpecFile)>) -> ValidationErrors {
    Specification::assemble(files).expect_err("the specification must be refused")
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Exactly one refusal in the whole specification, with this code at this location.
fn only(errors: &ValidationErrors, code: ValidationCode, at: &str) -> String {
    assert_eq!(
        errors.as_slice().len(),
        1,
        "expected exactly one refusal, {code:?} at `{at}`, got:\n{}",
        listed(errors)
    );
    let error = &errors.as_slice()[0];
    assert_eq!(
        (error.code, error.location.as_str()),
        (code, at),
        "{}",
        listed(errors)
    );
    error.message.clone()
}

fn hint(errors: &ValidationErrors) -> String {
    errors.as_slice()[0].hint.clone().unwrap_or_default()
}

/// Each command's outcome names, in order, after `expand` ran over `files`.
fn expanded(files: &mut [(Source, RawSpecFile)]) -> (Vec<(String, Vec<String>)>, ValidationErrors) {
    let mut errors = ValidationErrors::new();
    outcome_group::expand(files, &mut errors);
    let mut commands: Vec<(String, Vec<String>)> = files
        .iter()
        .flat_map(|(_, file)| file.commands.iter())
        .map(|command: &RawCommandSpec| {
            (
                command.name.to_string(),
                command
                    .outcomes
                    .iter()
                    .map(|outcome| outcome.name.to_string())
                    .collect(),
            )
        })
        .collect();
    commands.sort();
    (commands, errors)
}

fn outcomes_of(spec: &Specification, command: &str) -> Vec<String> {
    spec.commands()[&command.parse().unwrap()]
        .outcomes
        .iter()
        .map(|outcome| outcome.name.to_string())
        .collect()
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

/// C2's shape, as `expanded` reports it.
fn c2_shape() -> Vec<(String, Vec<String>)> {
    vec![
        (
            "tel.calls.Dial".to_owned(),
            strings(&["dialled", "session-expired"]),
        ),
        (
            "tel.calls.Hold".to_owned(),
            strings(&["held", "session-expired", "credential-rejected"]),
        ),
        (
            "tel.calls.Park".to_owned(),
            strings(&["parked", "credential-rejected"]),
        ),
        (
            "tel.calls.Resume".to_owned(),
            strings(&["resumed", "credential-rejected"]),
        ),
    ]
}

// C2 -----------------------------------------------------------------------------------------------

#[test]
fn c2_each_member_gains_its_groups_outcomes_after_its_own_in_group_name_order() {
    let spec = Specification::assemble(fixture()).unwrap_or_else(|errors| panic!("{errors}"));
    assert_eq!(
        outcomes_of(&spec, "tel.calls.Hold"),
        strings(&["held", "session-expired", "credential-rejected"]),
        "`audited` sorts before `remote-backed`, although it is declared in the later file"
    );
    assert_eq!(
        outcomes_of(&spec, "tel.calls.Resume"),
        strings(&["resumed", "credential-rejected"])
    );
    assert_eq!(
        outcomes_of(&spec, "tel.calls.Dial"),
        strings(&["dialled", "session-expired"])
    );
    assert_eq!(
        outcomes_of(&spec, "tel.calls.Park"),
        strings(&["parked", "credential-rejected"]),
        "the excepted command keeps only its own outcome"
    );
    let park = &spec.commands()[&"tel.calls.Park".parse().unwrap()];
    assert_eq!(
        park.outcomes[1].error.as_ref().map(ToString::to_string),
        Some("tel.session.Expired".to_owned()),
        "Park's own `credential-rejected`, not the group's"
    );
    let hold = &spec.commands()[&"tel.calls.Hold".parse().unwrap()];
    assert_eq!(
        hold.outcomes[2].error.as_ref().map(ToString::to_string),
        Some("tel.session.Unauthenticated".to_owned())
    );
    assert_eq!(
        hold.outcomes[2].summary.as_deref(),
        Some("The remote service refused the session.")
    );
    assert_eq!(hold.outcomes[2].refs.len(), 1);
}

#[test]
fn c2_expansion_leaves_no_group_behind_in_any_file() {
    let mut files = fixture();
    let (commands, errors) = expanded(&mut files);
    assert!(errors.is_empty(), "{}", listed(&errors));
    assert_eq!(commands, c2_shape());
    assert!(files.iter().all(|(_, file)| file.outcome_groups.is_empty()));
}

#[test]
fn a_group_may_be_declared_in_a_file_with_no_domain() {
    let system = format!(
        "{SYSTEM}outcome_groups:\n{}",
        CALLS.split_once("outcome_groups:\n").unwrap().1
    );
    let spec = Specification::assemble(tree(
        &system,
        "",
        SESSION.split_once("outcome_groups:\n").unwrap().1,
    ))
    .unwrap_or_else(|errors| panic!("{errors}"));
    assert_eq!(
        outcomes_of(&spec, "tel.calls.Hold"),
        strings(&["held", "session-expired", "credential-rejected"])
    );
}

#[test]
fn a_domain_selector_selects_the_commands_its_files_declare() {
    let spec = Specification::assemble(groups(
        "  - name: everywhere\n    domain: tel.calls\n    except: [tel.calls.Park]\n    outcomes:\n      \
         - {name: credential-rejected, external: the service rejects it, error: tel.session.Unauthenticated}\n",
    ))
    .unwrap_or_else(|errors| panic!("{errors}"));
    for command in ["tel.calls.Hold", "tel.calls.Resume", "tel.calls.Dial"] {
        assert_eq!(
            outcomes_of(&spec, command).last().map(String::as_str),
            Some("credential-rejected"),
            "{command}"
        );
    }
    assert_eq!(
        outcomes_of(&spec, "tel.calls.Park"),
        strings(&["parked", "credential-rejected"])
    );
}

#[test]
fn an_actor_declared_twice_is_selected_by_its_first_declaration() {
    let calls = CALLS.replace(
        "  - name: tel.calls.Supervisor\n",
        "  - name: tel.calls.Agent\n    may: [tel.calls.Dial]\n  - name: tel.calls.Supervisor\n",
    );
    let text = format!(
        "{}outcome_groups:
  - name: agents
    actor: tel.calls.Agent
{OUTCOME}",
        without_groups(&calls)
    );
    let mut files = tree(SYSTEM, "", "");
    files[1].1 = RawSpecFile::parse(&text).unwrap();
    let (commands, errors) = expanded(&mut files);
    assert!(errors.is_empty(), "{}", listed(&errors));
    let of = |name: &str| {
        commands
            .iter()
            .find(|(command, _)| command == name)
            .unwrap()
            .1
            .clone()
    };
    assert_eq!(
        of("tel.calls.Dial"),
        strings(&["dialled"]),
        "the second `Agent` grants nothing to a group"
    );
    assert_eq!(of("tel.calls.Hold"), strings(&["held", "lost"]));
}

// Reader errors ------------------------------------------------------------------------------------

fn parse_error(groups: &str) -> String {
    let text = format!("{}outcome_groups:\n{groups}", without_groups(CALLS));
    RawSpecFile::parse(&text)
        .err()
        .unwrap_or_else(|| panic!("the reader must refuse:\n{text}"))
        .to_string()
}

#[test]
fn a_group_key_the_format_does_not_have_is_a_reader_error() {
    let error = parse_error("  - name: g\n    commands: [tel.calls.Hold]\n    when: {}\n");
    assert!(error.contains("unknown field `when`"), "{error}");
}

#[test]
fn a_group_outcome_is_only_an_external_refusal() {
    for key in [
        "when: {}",
        "emits: [tel.calls.CallHeld]",
        "moves: tel.calls.X.y",
        "wrong_state: true",
    ] {
        let error = parse_error(&format!(
            "  - name: g\n    commands: [tel.calls.Hold]\n    outcomes:\n      - name: o\n        \
             external: it fails\n        error: tel.session.Expired\n        {key}\n"
        ));
        let field = key.split(':').next().unwrap();
        assert!(
            error.contains(&format!("unknown field `{field}`")),
            "{key}: {error}"
        );
    }
}

#[test]
fn a_group_outcome_without_external_or_error_is_a_reader_error() {
    let no_external = parse_error(
        "  - name: g\n    commands: [tel.calls.Hold]\n    outcomes:\n      - {name: o, error: tel.session.Expired}\n",
    );
    assert!(
        no_external.contains("missing field `external`"),
        "{no_external}"
    );
    let no_error = parse_error(
        "  - name: g\n    commands: [tel.calls.Hold]\n    outcomes:\n      - {name: o, external: it fails}\n",
    );
    assert!(no_error.contains("missing field `error`"), "{no_error}");
}

#[test]
fn a_malformed_group_name_is_a_reader_error() {
    let error = parse_error("  - name: Remote_Backed\n    commands: [tel.calls.Hold]\n");
    assert!(error.contains("Remote_Backed"), "{error}");
}

// G1–G16 -------------------------------------------------------------------------------------------

const OUTCOME: &str = "    outcomes:\n      - {name: lost, external: the service drops it, error: tel.session.Expired}\n";

#[test]
fn g1_a_group_name_declared_twice_is_refused_once_and_the_later_copy_not_expanded() {
    let errors = refused(tree(
        SYSTEM,
        &format!("  - name: twice\n    commands: [tel.calls.Hold]\n{OUTCOME}"),
        // The later copy would collide with Hold's own `held` if it were expanded.
        "  - name: twice\n    commands: [tel.calls.Hold]\n    outcomes:\n      - {name: held, external: x, error: tel.session.Expired}\n",
    ));
    let message = only(
        &errors,
        ValidationCode::DuplicateDeclaration,
        "outcome_groups.twice",
    );
    assert!(message.contains("domains/session.yaml"), "{message}");
}

#[test]
fn g2_a_group_with_no_membership_is_refused() {
    let errors = refused(groups(&format!("  - name: g\n{OUTCOME}")));
    only(
        &errors,
        ValidationCode::MissingDeclaration,
        "outcome_groups.g",
    );
}

#[test]
fn g3_a_group_with_two_membership_forms_is_refused_naming_both() {
    let errors = refused(groups(&format!(
        "  - name: g\n    actor: tel.calls.Agent\n    commands: [tel.calls.Hold]\n{OUTCOME}"
    )));
    let message = only(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "outcome_groups.g",
    );
    assert!(
        message.contains("`commands`") && message.contains("`actor`"),
        "{message}"
    );
}

#[test]
fn g4_a_listed_command_nobody_declares_is_refused() {
    let errors = refused(groups(&format!(
        "  - name: g\n    commands: [tel.calls.Hold, tel.calls.Nope]\n{OUTCOME}"
    )));
    only(
        &errors,
        ValidationCode::UndeclaredReference,
        "outcome_groups.g.commands",
    );
    assert!(
        hint(&errors).contains("tel.calls.Dial"),
        "{}",
        hint(&errors)
    );
}

#[test]
fn g5_an_actor_selector_naming_no_actor_is_refused() {
    let errors = refused(groups(&format!(
        "  - name: g\n    actor: tel.calls.Nobody\n{OUTCOME}"
    )));
    only(
        &errors,
        ValidationCode::UndeclaredReference,
        "outcome_groups.g",
    );
}

#[test]
fn g6_a_domain_selector_naming_no_domain_is_refused() {
    let errors = refused(groups(&format!(
        "  - name: g\n    domain: tel.nowhere\n{OUTCOME}"
    )));
    only(
        &errors,
        ValidationCode::UndeclaredReference,
        "outcome_groups.g",
    );
}

#[test]
fn g7_except_beside_an_explicit_list_is_refused() {
    let errors = refused(groups(&format!(
        "  - name: g\n    commands: [tel.calls.Hold]\n    except: [tel.calls.Dial]\n{OUTCOME}"
    )));
    only(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "outcome_groups.g.except",
    );
    assert!(hint(&errors).contains("leave the command out of `commands:`"));
}

#[test]
fn g8a_an_exception_nobody_declares_is_refused() {
    let errors = refused(groups(&format!(
        "  - name: g\n    actor: tel.calls.Agent\n    except: [tel.calls.Park, tel.calls.Nope]\n{OUTCOME}"
    )));
    only(
        &errors,
        ValidationCode::UndeclaredReference,
        "outcome_groups.g.except",
    );
}

#[test]
fn g8b_an_exception_the_selector_does_not_select_is_refused() {
    let errors = refused(groups(&format!(
        "  - name: g\n    actor: tel.calls.Agent\n    except: [tel.calls.Park, tel.calls.Dial]\n{OUTCOME}"
    )));
    let message = only(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "outcome_groups.g.except",
    );
    assert!(message.contains("tel.calls.Dial"), "{message}");
}

#[test]
fn g9_a_group_whose_membership_is_empty_is_refused() {
    let observer = CALLS.replace("actors:\n", "actors:\n  - name: tel.calls.Observer\n");
    for (case, calls, group) in [
        ("an empty list", CALLS.to_owned(), "    commands: []\n"),
        (
            "an actor granted nothing",
            observer,
            "    actor: tel.calls.Observer\n",
        ),
        (
            "a domain with no command",
            CALLS.to_owned(),
            "    domain: tel.session\n",
        ),
        (
            "everything excepted",
            CALLS.to_owned(),
            "    actor: tel.calls.Supervisor\n    except: [tel.calls.Dial]\n",
        ),
    ] {
        let text = format!(
            "{}outcome_groups:\n  - name: g\n{group}{OUTCOME}",
            without_groups(&calls)
        );
        let mut files = tree(SYSTEM, "", "");
        files[1].1 = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{case}: {error}"));
        let errors = refused(files);
        assert_eq!(errors.as_slice().len(), 1, "{case}:\n{}", listed(&errors));
        let error = &errors.as_slice()[0];
        assert_eq!(
            (error.code, error.location.as_str()),
            (ValidationCode::EmptyDeclaration, "outcome_groups.g"),
            "{case}"
        );
    }
}

#[test]
fn g10_a_group_with_no_outcomes_is_refused() {
    for outcomes in ["", "    outcomes: []\n"] {
        let errors = refused(groups(&format!(
            "  - name: g\n    commands: [tel.calls.Hold]\n{outcomes}"
        )));
        only(
            &errors,
            ValidationCode::EmptyDeclaration,
            "outcome_groups.g.outcomes",
        );
    }
}

#[test]
fn g11_two_outcomes_of_one_name_in_a_group_are_refused() {
    let errors = refused(groups(
        "  - name: g\n    commands: [tel.calls.Hold]\n    outcomes:\n      \
         - {name: lost, external: one, error: tel.session.Expired}\n      \
         - {name: lost, external: two, error: tel.session.Expired}\n",
    ));
    only(
        &errors,
        ValidationCode::DuplicateDeclaration,
        "outcome_groups.g.outcomes.lost",
    );
}

#[test]
fn g12_an_external_cause_that_is_blank_is_refused() {
    let errors = refused(groups(
        "  - name: g\n    commands: [tel.calls.Hold]\n    outcomes:\n      \
         - {name: lost, external: '   ', error: tel.session.Expired}\n",
    ));
    only(
        &errors,
        ValidationCode::UnexplainedDecision,
        "outcome_groups.g.outcomes.lost.external",
    );
}

#[test]
fn g13_c4_an_undeclared_error_is_refused_once_not_once_per_member() {
    let errors = refused(groups(
        "  - name: g\n    commands: [tel.calls.Hold, tel.calls.Resume, tel.calls.Dial]\n    outcomes:\n      \
         - {name: lost, external: the service drops it, error: tel.session.Nope}\n",
    ));
    only(
        &errors,
        ValidationCode::UndeclaredReference,
        "outcome_groups.g.outcomes.lost.error",
    );
}

#[test]
fn g14_a_member_declaring_the_same_outcome_is_refused_once_at_the_group() {
    let errors = refused(tree(
        SYSTEM,
        &CALLS
            .split_once("outcome_groups:\n")
            .unwrap()
            .1
            .replace("    except: [tel.calls.Park]\n", ""),
        SESSION.split_once("outcome_groups:\n").unwrap().1,
    ));
    let message = only(
        &errors,
        ValidationCode::DuplicateDeclaration,
        "outcome_groups.remote-backed.outcomes.credential-rejected",
    );
    assert!(message.contains("tel.calls.Park"), "{message}");
    assert!(
        hint(&errors).contains("list `tel.calls.Park` under `except:`"),
        "{}",
        hint(&errors)
    );
}

#[test]
fn g14_the_colliding_command_gets_none_of_the_group_and_the_others_do() {
    let mut files = tree(
        SYSTEM,
        &CALLS
            .split_once("outcome_groups:\n")
            .unwrap()
            .1
            .replace("    except: [tel.calls.Park]\n", ""),
        SESSION.split_once("outcome_groups:\n").unwrap().1,
    );
    let (commands, errors) = expanded(&mut files);
    assert_eq!(errors.as_slice().len(), 1, "{}", listed(&errors));
    assert_eq!(commands, c2_shape());
}

#[test]
fn g14_under_an_explicit_list_the_hint_says_leave_it_out() {
    let errors = refused(groups(
        "  - name: g\n    commands: [tel.calls.Hold, tel.calls.Park]\n    outcomes:\n      \
         - {name: parked, external: x, error: tel.session.Expired}\n",
    ));
    only(
        &errors,
        ValidationCode::DuplicateDeclaration,
        "outcome_groups.g.outcomes.parked",
    );
    assert!(
        hint(&errors).contains("leave `tel.calls.Park` out of `commands:`"),
        "{}",
        hint(&errors)
    );
}

#[test]
fn g15_two_groups_giving_one_command_the_same_outcome_are_refused_at_the_later() {
    let session = SESSION
        .split_once("outcome_groups:\n")
        .unwrap()
        .1
        .replace("name: session-expired", "name: credential-rejected");
    let calls = CALLS.split_once("outcome_groups:\n").unwrap().1;
    let errors = refused(tree(SYSTEM, calls, &session));
    let message = only(
        &errors,
        ValidationCode::DuplicateDeclaration,
        "outcome_groups.remote-backed.outcomes.credential-rejected",
    );
    assert!(
        message.contains("`audited`") && message.contains("tel.calls.Hold"),
        "{message}"
    );

    let mut files = tree(SYSTEM, calls, &session);
    let (commands, _) = expanded(&mut files);
    let of = |name: &str| {
        commands
            .iter()
            .find(|(command, _)| command == name)
            .unwrap()
            .1
            .clone()
    };
    assert_eq!(
        of("tel.calls.Hold"),
        strings(&["held"]),
        "neither group wins"
    );
    assert_eq!(
        of("tel.calls.Dial"),
        strings(&["dialled", "credential-rejected"])
    );
    assert_eq!(
        of("tel.calls.Resume"),
        strings(&["resumed", "credential-rejected"])
    );
}

#[test]
fn g16_c5_a_group_below_the_format_is_refused_per_group_and_still_expanded() {
    let system = SYSTEM.replace("format: ess/12", "format: ess/10");
    let mut files = fixture();
    files[0].1 = RawSpecFile::parse(&system).unwrap();
    let errors = refused(files.clone());
    let mut found: Vec<(ValidationCode, String, String)> = errors
        .as_slice()
        .iter()
        .map(|error| (error.code, error.location.clone(), error.message.clone()))
        .collect();
    found.sort();
    let message = "outcome groups require specification format ess/12".to_owned();
    assert_eq!(
        found,
        vec![
            (
                ValidationCode::UnsupportedFormatVersion,
                "outcome_groups.audited".to_owned(),
                message.clone()
            ),
            (
                ValidationCode::UnsupportedFormatVersion,
                "outcome_groups.remote-backed".to_owned(),
                message
            ),
        ]
    );
    let (commands, _) = expanded(&mut files);
    assert_eq!(commands, c2_shape());
}

#[test]
fn g16_does_not_fire_at_the_format_that_admits_groups() {
    assert!(Specification::assemble(fixture()).is_ok());
}
