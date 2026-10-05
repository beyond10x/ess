//! Typed text operands (beyond10x/ess#200) in an authored specification: from `ess/22` a view
//! filter's `starts_with`, `ends_with` or `contains` takes a parameter, `{param: <name>}`, and a
//! command guard's — a plain `when:`, a `when_subject:` and a `when_related:` predicate — takes an
//! input, `{input: <name>}`. Both sides are `String` or a newtype of it, `Optional` admitted.
//!
//! Below `ess/22` the mapping is refused as it always was and the spelling `param.query` stays the
//! literal it always was, with the parameter diagnostic's old words. From `ess/22` that spelling,
//! where it names a declared parameter or input of the place it is written in, is refused with the
//! repair. `docs/design/expression-family-source22.md`, "Typed text operands (#200)" and final
//! review decision 12, is the design.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

const DIRECTORY: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/typed-text-operands.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("directory.yaml"), raw)])
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| {
            format!(
                "{:?} {} {} (hint: {:?})",
                error.code, error.location, error.message, error.hint
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn validates(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{}\n---\n{text}", listed(&errors)))
}

/// Every refusal of `text` with `code` whose message holds `needle`.
fn refusals(text: &str, code: ValidationCode, needle: &str) -> Vec<ValidationError> {
    let errors = assemble(text).expect_err("refused");
    let found: Vec<ValidationError> = errors
        .as_slice()
        .iter()
        .filter(|error| error.code == code && error.message.contains(needle))
        .cloned()
        .collect();
    assert_ne!(
        found.len(),
        0,
        "expected {code:?} containing {needle:?}:\n{}",
        listed(&errors)
    );
    found
}

/// One contact entity, one command whose refusal is guarded by `when`, an invariant `invariant`
/// (none where empty), and one view declaring `params` and filtered by `filter`.
fn model(format: u32, when: &str, invariant: &str, params: &str, filter: &str) -> String {
    let invariants = if invariant.is_empty() {
        String::new()
    } else {
        format!("    invariants:\n      - {invariant}\n")
    };
    format!(
        "format: ess/{format}
system: directory
version: v1
domain: directory.people
types:
  - {{name: directory.people.Phone, kind: newtype, of: String}}
entities:
  - name: directory.people.Contact
    identity: {{name: contact_id, type: Uuid}}
    fields:
      - {{name: name, type: String}}
      - {{name: note, type: Optional<String>}}
      - {{name: phone, type: directory.people.Phone}}
      - {{name: calls, type: Integer}}
{invariants}    lifecycle: {{initial: Listed, states: [Listed], terminal: [Listed]}}
errors:
  - {{name: directory.people.Refused, summary: Refused., fields: []}}
events:
  - name: directory.people.Checked
    fields: []
commands:
  - name: directory.people.Check
    input:
      - {{name: caller, type: directory.people.Phone}}
      - {{name: prefix, type: String}}
      - {{name: carrier, type: Optional<String>}}
      - {{name: digits, type: Integer}}
    outcomes:
      - name: refused
        when: {when}
        error: directory.people.Refused
      - name: checked
        emits: [directory.people.Checked]
views:
  - name: directory.people.Search
    source: directory.people.Contact
    consistency: read_your_writes
    params: {params}
    filter: {filter}
    fields:
      - {{name: contact_id, type: Uuid}}
      - {{name: name, type: String}}
"
    )
}

const GUARD: &str = "{caller: {starts_with: {input: prefix}}}";
const PARAMS: &str = "[{name: q, type: String}]";
const FILTER: &str = "{name: {contains: {param: q}}}";

#[test]
fn t200_every_site_and_operator_validates_from_ess_22() {
    validates(DIRECTORY);
    for keyword in ["starts_with", "ends_with", "contains"] {
        validates(&model(
            22,
            &format!("{{caller: {{{keyword}: {{input: prefix}}}}}}"),
            "",
            PARAMS,
            &format!("{{name: {{{keyword}: {{param: q}}}}}}"),
        ));
        // `Optional` on either side; a newtype of `String` on either side.
        validates(&model(
            22,
            &format!("{{caller: {{{keyword}: {{input: carrier}}}}}}"),
            "",
            "[{name: q, type: Optional<directory.people.Phone>}]",
            &format!("{{note: {{{keyword}: {{param: q}}}}}}"),
        ));
    }
}

#[test]
fn t200_below_ess_22_the_mapping_is_refused_as_it_always_was() {
    for format in ["ess/20", "ess/21"] {
        let relabelled = DIRECTORY.replace("format: ess/22", &format!("format: {format}"));
        // In the words it always was, at the declaration that wrote it (beyond10x/ess#448).
        let refused = assemble(&relabelled).expect_err(format);
        assert!(
            refused.as_slice().iter().any(|error| error.code
                == ValidationCode::UnparsablePredicate
                && error
                    .message
                    .contains("a comparison operand must be a scalar")),
            "{format}: {refused}"
        );
    }
}

#[test]
fn t200_a_parameter_reads_only_in_a_view_and_an_input_only_in_a_guard() {
    // An input in a view filter: a view is read with parameters, never a command's input.
    let found = refusals(
        &model(
            22,
            GUARD,
            "",
            PARAMS,
            "{all: [{name: {contains: {param: q}}}, {name: {ends_with: {input: q}}}]}",
        ),
        ValidationCode::UnobservableFact,
        "{input: q}",
    );
    assert!(
        found[0]
            .location
            .contains("view.directory.people.Search.filter"),
        "{found:#?}"
    );
    // A parameter in a guard: a command is decided by its input, and the guard's read of the
    // undeclared root `param` is refused where every read of the input is checked.
    let found = refusals(
        &model(
            22,
            "{caller: {starts_with: {param: prefix}}}",
            "",
            PARAMS,
            FILTER,
        ),
        ValidationCode::UnobservableFact,
        "`param.prefix` reads `param`",
    );
    assert!(found[0].location.contains("Check"), "{found:#?}");
    // An invariant reads neither.
    for operand in ["{param: q}", "{input: q}"] {
        refusals(
            &model(
                22,
                GUARD,
                &format!("{{name: {{contains: {operand}}}}}"),
                PARAMS,
                FILTER,
            ),
            ValidationCode::UnobservableFact,
            operand,
        );
    }
}

#[test]
fn t200_the_operand_names_a_declared_string() {
    // Undeclared.
    refusals(
        &model(
            22,
            GUARD,
            "",
            PARAMS,
            "{all: [{name: {contains: {param: q}}}, {name: {contains: {param: nope}}}]}",
        ),
        ValidationCode::UndeclaredReference,
        "nope",
    );
    refusals(
        &model(
            22,
            "{caller: {starts_with: {input: nope}}}",
            "",
            PARAMS,
            FILTER,
        ),
        ValidationCode::UnobservableFact,
        "`input.nope` reads `input`",
    );
    // Not text: the operand, then the fact.
    refusals(
        &model(
            22,
            "{caller: {starts_with: {input: digits}}}",
            "",
            PARAMS,
            FILTER,
        ),
        ValidationCode::TypeMismatch,
        "digits",
    );
    refusals(
        &model(22, GUARD, "", "[{name: q, type: Integer}]", FILTER),
        ValidationCode::TypeMismatch,
        "{param: q}",
    );
    refusals(
        &model(22, GUARD, "", PARAMS, "{calls: {contains: {param: q}}}"),
        ValidationCode::TypeMismatch,
        "calls",
    );
}

#[test]
fn t200_from_ess_22_the_literal_spelling_of_a_declared_name_is_refused_with_the_repair() {
    let found = refusals(
        &model(22, GUARD, "", PARAMS, "{name: {contains: param.q}}"),
        ValidationCode::TypeMismatch,
        "{param: q}",
    );
    assert!(found[0].message.contains("\"param.q\""), "{found:#?}");
    let found = refusals(
        &model(
            22,
            "{caller: {starts_with: input.prefix}}",
            "",
            PARAMS,
            FILTER,
        ),
        ValidationCode::TypeMismatch,
        "{input: prefix}",
    );
    assert!(found[0].message.contains("\"input.prefix\""), "{found:#?}");
    // A stored row's predicate reads the input under `input.` too.
    let subject = DIRECTORY.replace(
        "- {name: {starts_with: {input: initial}}}\n                - {phone: {ends_with: {input: tail}}}",
        "- {name: {starts_with: input.initial}}\n                - {phone: {ends_with: {input: tail}}}",
    );
    assert_ne!(subject, DIRECTORY);
    refusals(&subject, ValidationCode::TypeMismatch, "{input: initial}");
}

#[test]
fn t200_an_undeclared_namespace_looking_literal_stays_the_literal() {
    // No parameter `query`, no input `nothing`: the texts they spell, as before.
    validates(&model(
        22,
        "{caller: {starts_with: input.nothing}}",
        "",
        PARAMS,
        "{all: [{name: {contains: {param: q}}}, {name: {contains: param.query}}]}",
    ));
}

#[test]
fn t200_below_ess_22_the_literal_spelling_keeps_its_meaning_and_its_words() {
    // `input.prefix` is the text it spells.
    validates(&model(
        21,
        "{caller: {starts_with: input.prefix}}",
        "",
        "[]",
        "{name: {contains: param.q}}",
    ));
    // `param.q` is read by nothing, and the diagnostic keeps its words byte for byte.
    let found = refusals(
        &model(
            21,
            "{caller: {starts_with: input.prefix}}",
            "",
            PARAMS,
            "{name: {contains: param.q}}",
        ),
        ValidationCode::UnobservableFact,
        "declares the parameter `q` and no filter reads it",
    );
    assert_eq!(
        found[0].hint.as_deref(),
        Some(
            "read it in `filter:` as `param.q`, or drop it — a parameter nothing selects on \
             makes the view look narrower than it is"
        )
    );
}

#[test]
fn t200_the_unread_parameter_hint_names_the_text_operand_from_ess_22() {
    let found = refusals(
        &model(
            22,
            GUARD,
            "",
            "[{name: q, type: String}, {name: r, type: String}]",
            FILTER,
        ),
        ValidationCode::UnobservableFact,
        "declares the parameter `r` and no filter reads it",
    );
    let hint = found[0].hint.as_deref().unwrap_or_default();
    assert!(hint.contains("`param.r`"), "{hint}");
    assert!(
        hint.contains("`{param: r}`")
            && hint.contains("starts_with")
            && hint.contains("ends_with")
            && hint.contains("contains"),
        "{hint}"
    );
}

const SELECTION: &str =
    include_str!("../../../generate/ess-synth/tests/fixtures/binding-selection.yaml");

#[test]
fn t200_a_selection_plan_refuses_a_typed_text_operand() {
    // A binding's selection decides over its bounded inputs with literal operands only (final
    // review decision 16): a parameter or an input is refused rather than read as text.
    let base = SELECTION.replacen("format: ess/3", "format: ess/22", 1);
    validates(&base);
    for operand in ["{input: agent_id}", "{param: agent_id}"] {
        let text = base.replacen(
            "where: 'item.id != \"\"'",
            &format!("where: {{item.id: {{starts_with: {operand}}}}}"),
            1,
        );
        assert_ne!(text, base);
        let errors =
            assemble(&text).map_or_else(|errors| listed(&errors), |spec| listed(&spec.validate()));
        assert!(
            errors.contains("selection admits only"),
            "{operand}: {errors}"
        );
    }
}

const SET_EFFECTS: &str = include_str!("../../ess-compiler/tests/fixtures/set-effects.yaml");

#[test]
fn t200_a_set_effects_filter_reads_no_typed_text_operand() {
    // A set effect's filter reads `input.<path>` as a value, and no typed text operand: only a
    // command's guards take `{input: <name>}`.
    let base = SET_EFFECTS.replacen("format: ess/16", "format: ess/22", 1);
    validates(&base);
    let text = base.replacen(
        "instances: {where: team == input.team}",
        "instances: {where: {team: {starts_with: {input: team}}}}",
        1,
    );
    assert_ne!(text, base);
    let found = refusals(&text, ValidationCode::UnobservableFact, "{input: team}");
    assert!(
        found[0].message.contains("names nothing here"),
        "{found:#?}"
    );
}
