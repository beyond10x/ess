//! A binding does not read stored state (beyond10x/ess#440, declined): the command it invokes
//! does, under that command's one pre-branch snapshot. The guide states the idiom with a model
//! that validates, and a `{related: …}` mapping value is still refused as it was.

use std::path::PathBuf;

use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_domain::Specification;

const GUIDE: &str = "website/docs/guides/specify/bindings-and-components.md";
const HEADING: &str = "## Read stored state in the command a binding invokes";

fn guide() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(GUIDE);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {GUIDE}: {error}"))
}

/// The section under `HEADING`, up to the next `## ` heading.
fn section(page: &str) -> Option<&str> {
    let start = page.find(&format!("\n{HEADING}\n"))? + 1;
    let body = &page[start + HEADING.len()..];
    let end = body.find("\n## ").unwrap_or(body.len());
    Some(&page[start..start + HEADING.len() + end])
}

/// The first fenced `yaml` block of the section.
fn fenced_model(section: &str) -> Option<&str> {
    let open = section.find("```yaml\n")? + "```yaml\n".len();
    let close = section[open..].find("\n```")?;
    Some(&section[open..=(open + close)])
}

fn assemble(document: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(document).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("calls.yaml"), raw)]).map_err(|errors| errors.to_string())
}

#[test]
fn binding_stored_read_guide_section_states_the_idiom() {
    let page = guide();
    let section = section(&page).unwrap_or_else(|| panic!("{GUIDE} has no heading `{HEADING}`"));
    for phrase in [
        "`{related:",
        "`when_related`",
        "one snapshot",
        "redelivery",
        "no store",
    ] {
        assert!(
            section.contains(phrase),
            "the section `{HEADING}` does not contain {phrase:?}:\n{section}"
        );
    }
    let envelope = page
        .find("\n## Read a field inside an event envelope\n")
        .expect("the guide keeps its envelope section");
    assert!(
        page.find(&format!("\n{HEADING}\n")).unwrap() > envelope,
        "`{HEADING}` comes after `## Read a field inside an event envelope`"
    );
    let model = fenced_model(section)
        .unwrap_or_else(|| panic!("the section `{HEADING}` has no fenced yaml model"));
    if let Err(errors) = assemble(model) {
        panic!("the section's model does not validate:\n{errors}\n{model}");
    }
    assert!(
        model.contains("{related: {via:"),
        "the model reads the stored row in the invoked command: {model}"
    );
}

#[test]
fn binding_mapping_related_shape_still_refused() {
    let page = guide();
    let model = section(&page)
        .and_then(fenced_model)
        .expect("the guide section's model")
        .to_owned();
    let refused = model.replace(
        "      leg_id: event.leg_id\n",
        "      leg_id: event.leg_id\n      call_bridged: {related: {via: call_id, field: bridged}}\n",
    );
    assert_ne!(refused, model, "the binding's mapping was rewritten");
    let error = assemble(&refused).expect_err("a binding does not read stored state");
    assert!(
        error.contains("unknown field `related`, expected `selection` or `path`"),
        "today's message: {error}"
    );
}
