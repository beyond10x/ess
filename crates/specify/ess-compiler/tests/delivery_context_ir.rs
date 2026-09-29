//! The delivery context of an event binding, resolved (beyond10x/ess#195, `ess/18`).
//!
//! The compiler used to refuse any context on an event binding a second time ("event bindings
//! cannot use periodic host inputs"). That refusal stays for `host_context.` and `host_read.`; the
//! declared delivery context resolves into the IR beside the event cause, and `context.<field>`
//! into a typed mapping value.
use ess_compiler::ir::{EssIr, ResolvedBindingCause, ResolvedMappingValue, ResolvedTypeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const INBOX: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/delivery-context.yaml");

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("inbox.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

#[test]
fn the_delivery_context_resolves_beside_the_event_cause() {
    let ir = ir_of(INBOX);
    let binding = ir
        .bindings()
        .values()
        .find(|binding| binding.name.as_str() == "received")
        .expect("compiled");
    let ResolvedBindingCause::Event(event) = &binding.cause else {
        panic!("still an event cause: {:?}", binding.cause)
    };
    assert_eq!(event.name().to_string(), "demo.inbox.MessageReceived");
    let context = binding.context.as_ref().expect("the context is in the IR");
    assert_eq!(context.authority.as_str(), "account-messages");
    assert_eq!(context.fields.len(), 1);
    assert_eq!(context.fields[0].name, "account_id");
    assert!(matches!(
        &context.fields[0].type_ref,
        ResolvedTypeRef::Declared { name } if name.name().to_string() == "demo.inbox.AccountId"
    ));

    let account = binding
        .mapping
        .iter()
        .find(|mapping| mapping.target == "account_id")
        .expect("account_id is mapped");
    assert!(
        matches!(
            &account.value,
            ResolvedMappingValue::DeliveryContext { field, .. } if field == "account_id"
        ),
        "{:?}",
        account.value
    );
    assert!(account.conversion.is_none());
}

#[test]
fn the_context_serialises_beside_the_event_and_only_where_declared() {
    let ir = ir_of(INBOX);
    let binding = &ir.bindings().values().next().expect("one binding");
    let json = serde_json::to_value(binding).expect("serialises");
    assert_eq!(json["event"], "demo.inbox.MessageReceived", "{json:#}");
    assert_eq!(json["context"]["authority"], "account-messages", "{json:#}");
    assert_eq!(
        json["mapping"][0]["value"]["kind"], "delivery_context",
        "{json:#}"
    );

    let plain = INBOX
        .replace("      context_authority: account-messages\n", "")
        .replace(
            "      context_fields:\n        - {name: account_id, type: demo.inbox.AccountId}\n",
            "",
        )
        .replace("account_id: context.account_id", "account_id: account-a");
    let ir = ir_of(&plain);
    let json = serde_json::to_value(ir.bindings().values().next().unwrap()).unwrap();
    assert!(
        json.get("context").is_none(),
        "a binding without a context keeps its bytes: {json:#}"
    );
}

/// The periodic host's context stays the periodic host's.
#[test]
fn host_context_on_an_event_binding_is_still_refused() {
    let text = INBOX.replace(
        "account_id: context.account_id",
        "account_id: host_context.account_id",
    );
    let raw = RawSpecFile::parse(&text).unwrap();
    let errors = Specification::assemble([(Source::new("inbox.yaml"), raw)]).expect_err("refused");
    assert!(
        errors
            .to_string()
            .contains("host mappings require a periodic cause"),
        "{errors}"
    );
}
