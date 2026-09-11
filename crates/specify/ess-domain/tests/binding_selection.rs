//! Ordered selectors are typed sources, not producer fields or expression-shaped literals.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("fixtures/binding-selection.yaml");

fn admit(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("selection.yaml"), raw)])
        .map_err(|errors| errors.to_string())
}

#[test]
fn declared_ordered_selectors_admit_without_inventing_event_fields() {
    let spec = admit(MODEL).expect("the declared first/exclusion/fallback selection contract");
    let binding = serde_json::to_value(spec.bindings().values().next().unwrap()).unwrap();
    assert_eq!(binding["selection_inputs"][0]["name"], "legs");
    assert_eq!(binding["selections"].as_array().unwrap().len(), 5);
    assert_eq!(binding["mapping"]["agent_id"]["kind"], "selection");
}

#[test]
fn selectors_reject_forward_references_and_undeclared_members() {
    let forward = MODEL.replace("excluding: [first_agent]", "excluding: [agent]");
    let error = admit(&forward).unwrap_err();
    assert!(error.contains("earlier"), "{error}");
    let absent = MODEL.replace("item.domain == Internal", "item.missing == Internal");
    let error = admit(&absent).unwrap_err();
    assert!(error.contains("missing"), "{error}");
}

#[test]
fn optional_selection_is_not_implicitly_unwrapped_for_a_required_target() {
    let text = MODEL.replace(
        "name: agent_id\n        type: Optional<String>",
        "name: agent_id\n        type: String",
    );
    let error = admit(&text).unwrap_err();
    assert!(error.contains("Optional"), "{error}");
}

#[test]
fn existing_selection_prefixed_literal_retains_its_old_meaning() {
    let (head, _) = MODEL.split_once("    selection_inputs:").unwrap();
    let text = format!("{}    mapping:\n      agent_id: selection.agent.id\n      external_id: plain\n    delivery: at_most_once\n    on_failure: drop\n", head.replace("format: ess/3", "format: ess/1"));
    let spec = admit(&text).unwrap();
    let binding = serde_json::to_value(spec.bindings().values().next().unwrap()).unwrap();
    assert_eq!(binding["mapping"]["agent_id"]["kind"], "literal");
    assert_eq!(
        binding["mapping"]["agent_id"]["value"],
        "selection.agent.id"
    );
}
