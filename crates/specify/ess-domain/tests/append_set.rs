//! `sets: {items: {append: input.item}}` — an outcome storing the subject's list with one element
//! added last (`ess/31`).
//!
//! An adopter's command added one element to a stored list and could not say so: a list holding
//! anything was no `sets:` source, and the expression language holds no list. From `ess/31` the
//! `{append: input.<path>}` source stores the list the subject held before the outcome, absent
//! read as empty, with the input's element last. The element is typed against the list's element
//! type; everything else a value source refuses, this one refuses by name.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("jobs.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// A job holding lists, created plainly and grown by `AddTag`, whose outcome sets `target` to
/// `value` from an input `item` of type `item`.
fn model(format: &str, target: &str, value: &str, item: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.jobs
types:
  - {{name: demo.jobs.JobId, kind: newtype, of: Uuid}}
  - name: demo.jobs.Label
    kind: struct
    fields:
      - {{name: append, type: String}}
entities:
  - name: demo.jobs.Job
    identity: {{name: job_id, type: demo.jobs.JobId}}
    fields:
      - {{name: title, type: String}}
      - {{name: tags, type: List<String>}}
      - {{name: notes, type: Optional<List<String>>}}
      - {{name: label, type: Optional<demo.jobs.Label>}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
events:
  - name: demo.jobs.JobAdded
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
  - name: demo.jobs.TagAdded
    fields: [{{name: job_id, type: demo.jobs.JobId}}, {{name: tag, type: String}}]
commands:
  - name: demo.jobs.AddJob
    input: [{{name: title, type: String}}, {{name: tags, type: List<String>}}]
    outcomes:
      - name: added
        creates: demo.jobs.Job
        instance: job_id
        sets: {{title: input.title, tags: input.tags}}
        emits: [demo.jobs.JobAdded]
        payload: {{demo.jobs.JobAdded: {{job_id: {{generated: true}}}}}}
  - name: demo.jobs.AddTag
    input:
      - {{name: job_id, type: demo.jobs.JobId}}
      - {{name: item, type: {item}}}
    outcomes:
      - name: tagged
        updates: demo.jobs.Job
        instance: job_id
        sets:
          {target}: {value}
        emits: [demo.jobs.TagAdded]
        payload: {{demo.jobs.TagAdded: {{job_id: input.job_id, tag: {{generated: true}}}}}}
"
    )
}

fn add_tag(spec: &ess_domain::Specification) -> &ess_domain::command::CommandSpec {
    spec.commands()
        .values()
        .find(|c| c.name.to_string() == "demo.jobs.AddTag")
        .expect("the command")
}

#[test]
fn an_update_may_append_an_input_to_a_stored_list() {
    for target in ["tags", "notes"] {
        let parsed = spec(&model("ess/31", target, "{append: input.item}", "String"))
            .unwrap_or_else(|error| panic!("{target}: `{{append: …}}` fills a list:\n{error}"));
        let command = add_tag(&parsed);
        let source = &command.outcomes[0].sets[target];
        assert_eq!(
            format!("{source:?}"),
            "Append { field: \"item\" }",
            "{target}: the source appends the input"
        );
        let encoded = ess_yaml::to_string(command).unwrap();
        assert!(
            encoded.contains("append: input.item"),
            "{target}: written back as authored:\n{encoded}"
        );
        let raw: ess_domain::command::RawCommandSpec = ess_yaml::from_str(&encoded).unwrap();
        let roundtrip = ess_domain::command::CommandSpec::try_from(raw).unwrap();
        assert_eq!(
            &roundtrip, command,
            "{target}: the construct survives a round trip"
        );
    }
}

#[test]
fn an_element_of_another_type_is_refused_as_type_mismatch() {
    for item in ["Integer", "Optional<String>"] {
        let error = spec(&model("ess/31", "tags", "{append: input.item}", item))
            .expect_err("the element must be the list's element type");
        for required in ["type_mismatch", "item", item, "List<String>", "String"] {
            assert!(
                error.contains(required),
                "{item}: {required:?} missing from:\n{error}"
            );
        }
    }
}

#[test]
fn append_over_a_field_that_is_no_list_is_refused() {
    let error = spec(&model("ess/31", "title", "{append: input.item}", "String"))
        .expect_err("only a list is appended to");
    for required in ["type_mismatch", "demo.jobs.Job.title", "List<"] {
        assert!(
            error.contains(required),
            "{required:?} missing from:\n{error}"
        );
    }
}

#[test]
fn append_on_a_creating_outcome_is_refused() {
    let text = model("ess/31", "title", "input.item", "String")
        .replace("tags: input.tags}", "tags: {append: input.title}}");
    let error = spec(&text).expect_err("a created row holds no list before it");
    for required in ["conflicting_declaration", "`{append: …}`", "creates:"] {
        assert!(
            error.contains(required),
            "{required:?} missing from:\n{error}"
        );
    }
}

#[test]
fn append_in_an_event_payload_is_refused() {
    let text = model("ess/31", "tags", "{append: input.item}", "String")
        .replace("tag: {generated: true}", "tag: {append: input.item}");
    let error = spec(&text).expect_err("`{append: …}` is a `sets:` source only");
    for required in ["conflicting_declaration", "`{append: …}`", "`sets:`"] {
        assert!(
            error.contains(required),
            "{required:?} missing from:\n{error}"
        );
    }
}

#[test]
fn an_element_that_is_no_input_is_refused() {
    let error = spec(&model("ess/31", "tags", "{append: urgent}", "String"))
        .expect_err("the element is read from the input");
    assert!(error.contains("input.<path>"), "{error}");
}

#[test]
fn append_below_ess_31_is_refused_naming_ess_31() {
    for format in ["ess/27", "ess/30"] {
        let error = spec(&model(format, "tags", "{append: input.item}", "String"))
            .expect_err("below ess/31 there is no `{append: …}`");
        for required in ["unsupported_format_version", "tags", "ess/31"] {
            assert!(
                error.contains(required),
                "{format}: {required:?} missing from:\n{error}"
            );
        }
    }
}

#[test]
fn a_struct_member_named_append_below_ess_31_is_a_nested_mapping() {
    let text = model("ess/30", "label", "{append: input.item}", "String");
    let parsed = spec(&text).unwrap_or_else(|error| panic!("a nested mapping:\n{error}"));
    let source = &add_tag(&parsed).outcomes[0].sets["label"];
    assert!(
        format!("{source:?}").starts_with("Struct"),
        "below ess/31 the shape is the nested mapping it was: {source:?}"
    );
}
