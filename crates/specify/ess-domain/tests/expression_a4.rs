//! Family F part A4 (beyond10x/ess#233, `docs/design/expression-family-source22.md`): a value in
//! `sets:`, an event `payload:` or an error `payload:` read from a member of a struct input,
//! `input.<path>`, and an `else:` that falls back to another input path, from source `ess/22`.
//!
//! Below `ess/22` every one of these keeps the refusal it had.
use ess_domain::command::PayloadSource;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationErrors;

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/input-value-paths.yaml");
const EXISTENCE: &str = include_str!("../../ess-compiler/tests/fixtures/upsert-by-existence.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("leases.yaml"), raw)])
}

fn accepted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

/// The refusal, whether the document is refused as it is read or as it is assembled.
fn refused(text: &str) -> String {
    match RawSpecFile::parse(text) {
        Err(error) => error.to_string(),
        Ok(raw) => Specification::assemble([(Source::new("leases.yaml"), raw)])
            .err()
            .unwrap_or_else(|| panic!("must refuse:\n{text}"))
            .to_string(),
    }
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replacen(from, to, 1);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn at(text: &str, format: &str) -> String {
    replaced(text, "format: ess/22\n", &format!("format: {format}\n"))
}

fn open(spec: &Specification) -> &ess_domain::command::CommandSpec {
    &spec.commands()[&"leases.pool.Open".parse().unwrap()]
}

fn input(path: &str) -> PayloadSource {
    PayloadSource::InputField {
        field: path.to_owned(),
    }
}

#[test]
fn a4_dotted_sources_are_read_as_paths_under_ess22() {
    let spec = accepted(MODEL);
    let open = open(&spec);
    let opened = open
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "opened")
        .expect("opened");
    assert_eq!(
        opened.sets.get("generation_id"),
        Some(&input("opening.generation_id"))
    );
    assert_eq!(
        opened.sets.get("previous_generation"),
        Some(&input("previous.generation_id"))
    );
    assert_eq!(
        opened.sets.get("sealed_label"),
        Some(&input("sealed.label"))
    );
    let fallback = PayloadSource::InputOrGenerated {
        field: "previous.label".to_owned(),
        otherwise: Some(Box::new(input("settings.defaults.label"))),
    };
    assert_eq!(opened.sets.get("label"), Some(&fallback));
    let event = opened.payload.values().next().expect("the event payload");
    assert_eq!(event.get("label"), Some(&fallback));
    assert_eq!(
        event.get("previous"),
        Some(&input("previous.generation_id"))
    );
    let rejected = open
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "rejected")
        .expect("rejected");
    assert_eq!(
        rejected.error_payload.get("generation_id"),
        Some(&input("opening.generation_id"))
    );
    assert_eq!(rejected.error_payload.get("label"), Some(&fallback));
    assert_eq!(
        fallback.to_string(),
        "input.previous.label, else input.settings.defaults.label"
    );
}

#[test]
fn a4_declaration_order_does_not_change_the_reading() {
    let reordered = replaced(
        MODEL,
        "      - {name: lease_id, type: leases.pool.LeaseId}
      - {name: generation_id, type: leases.pool.GenerationId}
      - {name: opening, type: leases.pool.Opening}
      - {name: previous, type: Optional<leases.pool.Opening>}
      - {name: sealed, type: leases.pool.Sealed}
      - {name: settings, type: leases.pool.Settings}
      - {name: reject, type: Boolean}",
        "      - {name: reject, type: Boolean}
      - {name: settings, type: leases.pool.Settings}
      - {name: sealed, type: leases.pool.Sealed}
      - {name: previous, type: Optional<leases.pool.Opening>}
      - {name: opening, type: leases.pool.Opening}
      - {name: generation_id, type: leases.pool.GenerationId}
      - {name: lease_id, type: leases.pool.LeaseId}",
    );
    let reordered = replaced(
        &reordered,
        "  - name: leases.pool.Opening
    kind: struct
    fields:
      - {name: generation_id, type: leases.pool.GenerationId}
      - {name: label, type: String}",
        "  - name: leases.pool.Opening
    kind: struct
    fields:
      - {name: label, type: String}
      - {name: generation_id, type: leases.pool.GenerationId}",
    );
    let original = accepted(MODEL);
    let moved = accepted(&reordered);
    let sources = |spec: &Specification| {
        open(spec)
            .outcomes
            .iter()
            .map(|outcome| (outcome.sets.clone(), outcome.payload.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(sources(&original), sources(&moved));
}

/// Below `ess/22` each dotted form keeps the refusal it had: a plain `input.a.b` is an input the
/// command does not declare, `{input: a.b, else: …}` names more than one field, and an input after
/// `else:` is no literal.
#[test]
fn a4_old_source_refuses_each_dotted_form_as_before() {
    let old = at(MODEL, "ess/21");
    let sets_only = replaced(
        &replaced(
            &old,
            "            label: {input: previous.label, else: input.settings.defaults.label}
      - name: opened",
            "            label: input.opening.label
      - name: opened",
        ),
        "            label: {input: previous.label, else: input.settings.defaults.label}
        sets:",
        "            label: input.opening.label
        sets:",
    );
    let sets_only = replaced(
        &sets_only,
        "          label: {input: previous.label, else: input.settings.defaults.label}
          sealed_label",
        "          label: input.opening.label
          sealed_label",
    );
    let error = refused(&sets_only);
    assert!(
        error.contains(
            "`input.opening.generation_id` reads `opening.generation_id`, which \
             `leases.pool.Open` does not declare as input"
        ),
        "{error}"
    );
    assert!(error.contains("undeclared_reference"), "{error}");
    let dotted_primary = old.replace(
        "{input: previous.label, else: input.settings.defaults.label}",
        "{input: previous.label, else: Fixed}",
    );
    let error = refused(&dotted_primary);
    assert!(
        error.contains("`{input: <field>, else: …}` names one field of the command's input"),
        "{error}"
    );
    let literal_else = replaced(
        &old,
        "{input: previous.label, else: input.settings.defaults.label}",
        "{input: reject, else: input.settings.defaults.label}",
    );
    let error = refused(&literal_else);
    assert!(
        error.contains("`else:` admits `{generated: true}` only, or a literal"),
        "{error}"
    );
}

#[test]
fn a4_an_optional_parent_cannot_fill_a_required_target() {
    let required = replaced(
        MODEL,
        "          generation_id: input.opening.generation_id
          previous_generation",
        "          generation_id: input.previous.generation_id
          previous_generation",
    );
    let error = refused(&required);
    assert!(error.contains("type_mismatch"), "{error}");
    assert!(error.contains("previous.generation_id"), "{error}");
    assert!(
        error.contains("Optional<leases.pool.GenerationId>"),
        "the route's absence is the source's type: {error}"
    );
    let event = replaced(
        MODEL,
        "            generation_id: input.opening.generation_id
            previous: input.previous.generation_id",
        "            generation_id: input.previous.generation_id
            previous: input.previous.generation_id",
    );
    let error = refused(&event);
    assert!(error.contains("type_mismatch"), "{error}");
}

#[test]
fn a4_a_path_through_a_value_with_no_members_is_refused() {
    for through in [
        "input.lease_id.value",
        "input.opening.label.text",
        "input.reject.flag",
    ] {
        let model = replaced(
            MODEL,
            "sealed_label: input.sealed.label",
            &format!("sealed_label: {through}"),
        );
        let error = refused(&model);
        assert!(error.contains("type_mismatch"), "{through}: {error}");
        assert!(error.contains("has no members"), "{through}: {error}");
    }
}

#[test]
fn a4_an_undeclared_member_or_root_is_refused() {
    let member = replaced(
        MODEL,
        "sealed_label: input.sealed.label",
        "sealed_label: input.sealed.title",
    );
    let error = refused(&member);
    assert!(error.contains("undeclared_reference"), "{error}");
    assert!(
        error.contains("`title` is not a field of `leases.pool.Sealed`"),
        "{error}"
    );
    assert!(
        error.contains("generation_id, label"),
        "the hint lists the members: {error}"
    );
    let root = replaced(
        MODEL,
        "sealed_label: input.sealed.label",
        "sealed_label: input.nothing.label",
    );
    let error = refused(&root);
    assert!(error.contains("undeclared_reference"), "{error}");
    assert!(error.contains("`nothing`"), "{error}");
}

#[test]
fn a4_an_input_fallback_is_required_along_its_whole_route() {
    let optional = replaced(
        MODEL,
        "          label: {input: previous.label, else: input.settings.defaults.label}
          sealed_label",
        "          label: {input: previous.label, else: input.previous.generation_id}
          sealed_label",
    );
    let error = refused(&optional);
    assert!(error.contains("type_mismatch"), "{error}");
    assert!(error.contains("`else:` promises a value"), "{error}");
}

#[test]
fn a4_a_required_primary_never_falls_back() {
    let required = replaced(
        MODEL,
        "          label: {input: previous.label, else: input.settings.defaults.label}
          sealed_label",
        "          label: {input: opening.label, else: input.settings.defaults.label}
          sealed_label",
    );
    let error = refused(&required);
    assert!(
        error.contains("always sent, so `else:` never applies"),
        "{error}"
    );
}

#[test]
fn a4_an_input_fallback_is_type_checked_against_the_target() {
    let mismatch = replaced(
        MODEL,
        "          label: {input: previous.label, else: input.settings.defaults.label}
          sealed_label",
        "          label: {input: previous.label, else: input.reject}
          sealed_label",
    );
    let error = refused(&mismatch);
    assert!(error.contains("type_mismatch"), "{error}");
    assert!(error.contains("input.reject"), "{error}");
}

#[test]
fn a4_a_second_fallback_is_refused() {
    let nested = replaced(
        MODEL,
        "          label: {input: previous.label, else: input.settings.defaults.label}
          sealed_label",
        "          label: {input: previous.label, else: {input: previous.label, else: Fixed}}
          sealed_label",
    );
    let error = refused(&nested);
    assert!(error.contains("`else:`"), "{error}");
}

#[test]
fn a4_aggregate_terminals_and_nested_target_leaves_compile() {
    let model = replaced(
        MODEL,
        "      - {name: copied, type: leases.pool.Defaults}
    lifecycle",
        "      - {name: copied, type: leases.pool.Defaults}
      - {name: defaults, type: leases.pool.Defaults}
    lifecycle",
    );
    let model = replaced(
        &model,
        "          copied: {label: input.opening.label}\n",
        "          copied: {label: input.opening.label}
          defaults: input.settings.defaults\n",
    );
    let spec = accepted(&model);
    let opened = &open(&spec).outcomes[1];
    assert_eq!(
        opened.sets.get("defaults"),
        Some(&input("settings.defaults"))
    );
    let nested = replaced(
        &model,
        "copied: {label: input.opening.label}",
        "copied: {label: input.previous.label}",
    );
    let error = refused(&nested);
    assert!(
        error.contains("type_mismatch"),
        "a leaf is held to the same rule: {error}"
    );
}

// ---- rule 8: an identity only through a required route -----------------------------------------

fn nested_existence(format: &str, slot: &str) -> String {
    let model = EXISTENCE
        .replace("format: ess/16\n", &format!("format: {format}\n"))
        .replace(
            "  - {name: demo.items.Label, kind: newtype, of: String}\n",
            "  - {name: demo.items.Label, kind: newtype, of: String}
  - name: demo.items.Booking
    kind: struct
    fields:
      - {name: slot_id, type: demo.items.ItemId}
  - name: demo.items.Hold
    kind: struct
    fields:
      - {name: slot_id, type: Optional<demo.items.ItemId>}\n",
        )
        .replace(
            "    input:\n      - {name: slot_id, type: demo.items.ItemId}\n      - {name: label, type: demo.items.Label}",
            "    input:\n      - {name: slot_id, type: demo.items.ItemId}\n      - {name: booking, type: demo.items.Booking}\n      - {name: hold, type: Optional<demo.items.Hold>}\n      - {name: label, type: demo.items.Label}",
        )
        .replace(
            "demo.items.SlotBooked: {slot_id: input.slot_id, label: input.label}",
            &format!("demo.items.SlotBooked: {{slot_id: {slot}, label: input.label}}"),
        );
    assert_ne!(model, EXISTENCE);
    model
}

#[test]
fn a4_a_required_route_supplies_a_creation_identity() {
    accepted(&nested_existence("ess/22", "input.booking.slot_id"));
}

#[test]
fn a4_an_optional_route_supplies_no_identity() {
    let error = refused(&nested_existence(
        "ess/22",
        "{input: hold.slot_id, else: {generated: true}}",
    ));
    assert!(
        error.contains("supplies an identity only when its whole route is required"),
        "{error}"
    );
}
