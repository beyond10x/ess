//! An event binding reads the delivery context its event arrived with (beyond10x/ess#195, `ess/18`).
//!
//! The shape of the issue: an event delivered per account on an external channel names the
//! message, not the account, and the account is the subscription it arrived on. The binding
//! declares that context under `when:` — `context_fields`, with `context_authority` naming the
//! external channel whose authority binds it — and reads it as `context.<field>`. A missing context
//! is a refusal, never a lookup in the payload, and `event.<channel>` stays unspelled.
use ess_domain::{
    binding::{BindingCause, MappingSource},
    spec::RawSpecFile,
    system::Source,
    Specification,
};
use ess_primitives::error::{ValidationCode, ValidationErrors};

/// The issue's shape, declared.
const INBOX: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/delivery-context.yaml");

/// The binding's `when:` block as the fixture writes it.
const WHEN: &str = "    when:\n      event: demo.inbox.MessageReceived\n      context_authority: account-messages\n      context_fields:\n        - {name: account_id, type: demo.inbox.AccountId}\n";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("inbox.yaml"), raw)])
}

fn accepted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn refused(text: &str) -> ValidationErrors {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
}

fn has(errors: &ValidationErrors, code: ValidationCode, fragment: &str) -> bool {
    errors
        .as_slice()
        .iter()
        .any(|error| error.code == code && error.to_string().contains(fragment))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// The same document as JSON text, which the reader takes through the same path.
fn as_json(text: &str) -> String {
    let value: serde_json::Value = serde_yaml::from_str(text).expect("the fixture is YAML");
    serde_json::to_string_pretty(&value).expect("JSON")
}

#[test]
fn the_issue_shape_validates_under_ess_18() {
    let spec = accepted(INBOX);
    let binding = spec
        .bindings()
        .values()
        .find(|binding| binding.name.as_str() == "received")
        .expect("the binding is declared");
    // Read through `Debug` and `Display`, so this case says what it requires of a build that does
    // not have the construct yet rather than failing to compile against it.
    let cause = format!("{:?}", binding.cause);
    assert!(
        cause.starts_with("External(") && cause.contains("account-messages"),
        "a delivery context makes the cause an external delivery naming its channel: {cause}"
    );
    assert!(cause.contains("account_id"), "{cause}");
    assert_eq!(
        binding.cause.event().map(ToString::to_string).as_deref(),
        Some("demo.inbox.MessageReceived"),
        "the cause is still an event"
    );
    let source = &binding.mapping["account_id"];
    assert!(
        format!("{source:?}").starts_with("DeliveryContext"),
        "`context.account_id` reads the delivery context: {source:?}"
    );
    assert_eq!(source.to_string(), "context.account_id");
    assert!(matches!(
        binding.mapping["message_id"],
        MappingSource::EventField { .. }
    ));
    assert!(!matches!(binding.cause, BindingCause::Periodic(_)));
}

#[test]
fn the_issue_shape_validates_as_json_too() {
    accepted(&as_json(INBOX));
}

#[test]
fn a_delivery_context_below_ess_18_is_refused_in_yaml_and_json() {
    for format in ["ess/17", "ess/3"] {
        let older = replaced(INBOX, "format: ess/18\n", &format!("format: {format}\n"));
        for text in [older.clone(), as_json(&older)] {
            let errors = refused(&text);
            assert!(
                has(
                    &errors,
                    ValidationCode::UnsupportedFormatVersion,
                    "requires specification format ess/18"
                ),
                "{format}: {errors}"
            );
        }
    }
}

#[test]
fn a_context_mapping_below_ess_18_without_a_declaration_is_refused_and_names_ess_18() {
    let older = replaced(INBOX, "format: ess/18\n", "format: ess/17\n");
    let undeclared = replaced(
        &older,
        WHEN,
        "    when:\n      event: demo.inbox.MessageReceived\n",
    );
    // Below ess/18 `context.account_id` is no longer literal text: it is refused, for reading a
    // context nothing declares, and the refusal names the format that admits one.
    let errors = refused(&undeclared);
    assert!(
        has(
            &errors,
            ValidationCode::UnobservableFact,
            "`context.account_id` reads a delivery context"
        ) && errors.to_string().contains("ess/18"),
        "{errors}"
    );
}

#[test]
fn reading_context_without_declaring_it_is_refused_and_not_looked_up_in_the_payload() {
    let undeclared = replaced(
        INBOX,
        WHEN,
        "    when:\n      event: demo.inbox.MessageReceived\n",
    );
    let errors = refused(&undeclared);
    assert!(
        has(
            &errors,
            ValidationCode::UnobservableFact,
            "`context.account_id` reads a delivery context"
        ),
        "{errors}"
    );
}

#[test]
fn a_context_field_the_declaration_does_not_name_is_refused() {
    let text = replaced(
        INBOX,
        "account_id: context.account_id",
        "account_id: context.tenant",
    );
    let errors = refused(&text);
    assert!(
        has(
            &errors,
            ValidationCode::UndeclaredReference,
            "declares no delivery context field `tenant`"
        ),
        "{errors}"
    );
}

#[test]
fn a_context_field_of_the_wrong_type_is_refused() {
    let text = replaced(
        INBOX,
        "        - {name: account_id, type: demo.inbox.AccountId}\n    invoke",
        "        - {name: account_id, type: Integer}\n    invoke",
    );
    let errors = refused(&text);
    assert!(
        has(
            &errors,
            ValidationCode::TypeMismatch,
            "context field `account_id`"
        ),
        "{errors}"
    );
}

#[test]
fn context_fields_without_an_authority_are_refused() {
    let text = replaced(INBOX, "      context_authority: account-messages\n", "");
    let errors = refused(&text);
    assert!(
        has(
            &errors,
            ValidationCode::MissingDeclaration,
            "context_authority"
        ),
        "{errors}"
    );
}

#[test]
fn an_authority_without_context_fields_is_refused() {
    let text = replaced(
        INBOX,
        "      context_fields:\n        - {name: account_id, type: demo.inbox.AccountId}\n",
        "",
    );
    let errors = refused(&text);
    assert!(
        has(
            &errors,
            ValidationCode::MissingDeclaration,
            "context_fields"
        ),
        "{errors}"
    );
}

#[test]
fn an_empty_context_is_refused() {
    let text = replaced(
        INBOX,
        "      context_fields:\n        - {name: account_id, type: demo.inbox.AccountId}\n",
        "      context_fields: []\n",
    );
    let errors = refused(&text);
    assert!(
        has(&errors, ValidationCode::EmptyDeclaration, "context_fields"),
        "{errors}"
    );
}

#[test]
fn a_context_field_declared_twice_is_refused() {
    let text = replaced(
        INBOX,
        "        - {name: account_id, type: demo.inbox.AccountId}\n    invoke",
        "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: account_id, type: String}\n    invoke",
    );
    let errors = refused(&text);
    assert!(
        has(&errors, ValidationCode::DuplicateDeclaration, "context"),
        "{errors}"
    );
}

/// "Admitted only for events delivered by an external channel": an event some command of the
/// specification publishes arrives from inside the system, with no channel to bind a context.
#[test]
fn a_delivery_context_on_an_event_the_system_publishes_is_refused() {
    let text = replaced(
        INBOX,
        "        emits: [demo.inbox.MessageRecorded]\n",
        "        emits: [demo.inbox.MessageRecorded, demo.inbox.MessageReceived]\n",
    );
    let text = replaced(
        &text,
        "          demo.inbox.MessageRecorded: {account_id: input.account_id, message_id: input.message_id}\n",
        "          demo.inbox.MessageRecorded: {account_id: input.account_id, message_id: input.message_id}\n          demo.inbox.MessageReceived: {message_id: input.message_id, from: input.peer}\n",
    );
    let errors = refused(&text);
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "only for an event an external channel delivers"
        ),
        "{errors}"
    );
}

/// The issue's two spellings stay refused: `host_context.` is the periodic host, and
/// `event.channel` is not a payload field.
#[test]
fn the_issue_spellings_that_were_refused_stay_refused() {
    let host = replaced(
        INBOX,
        "account_id: context.account_id",
        "account_id: host_context.account_id",
    );
    assert!(has(
        &refused(&host),
        ValidationCode::UnobservableFact,
        "host mappings require a periodic cause"
    ));
    let channel = replaced(
        INBOX,
        "account_id: context.account_id",
        "account_id: event.channel",
    );
    assert!(has(
        &refused(&channel),
        ValidationCode::UnobservableFact,
        "`event.channel` is not a field"
    ));
}

/// A periodic host already has a context of its own, read as `host_context.<field>`.
#[test]
fn a_delivery_context_beside_a_periodic_cause_is_refused() {
    let periodic = include_str!("fixtures/periodic.yaml");
    let text = replaced(
        &replaced(periodic, "format: ess/3\n", "format: ess/18\n"),
        "    when:\n      periodic:\n",
        "    when:\n      context_authority: account-messages\n      context_fields: [{name: account_id, type: String}]\n      periodic:\n",
    );
    let errors = refused(&text);
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "not to a periodic cause"
        ),
        "{errors}"
    );
}
