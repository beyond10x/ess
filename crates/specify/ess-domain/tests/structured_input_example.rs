//! `example:` on a command input of struct, `Map` or `Json` type (ess/33, a downstream report,
//! 2026-10-30).
//!
//! From `ess/33` an input whose terminal type is a struct (open or closed), a `Map<…>` or `Json`,
//! through any newtype or `Optional`, may carry an example, held as a precondition's literal is. An
//! open struct's example may name members it does not declare, which are kept as written; a closed
//! one refuses them. A `Json` example is any value but `null`. The open-struct rule opens list
//! example elements and precondition literals too. Below `ess/33` each keeps its refusal, its hint
//! naming `ess/33`; a union example stays refused at every format.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/structured-input-example.yaml");
const CREATE: &str = "command.demo.docs.CreateDoc.input";
const IMPORT: &str = "command.demo.docs.ImportDoc.input";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{text}"));
    Specification::assemble([(Source::new("spec.yaml"), raw)])
}

fn admitted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("must be admitted: {errors}\n{text}"))
}

fn refused(text: &str) -> ValidationErrors {
    match assemble(text) {
        Ok(_) => panic!("must refuse:\n{text}"),
        Err(errors) => errors,
    }
}

fn with(model: &str, from: &str, to: &str) -> String {
    assert!(model.contains(from), "the model carries {from:?}");
    model.replacen(from, to, 1)
}

/// `model` with the `example:` of the input `field` replaced by `value`.
fn example_in(model: &str, field: &str, value: &str) -> String {
    let prefix = format!("      - {{name: {field}, type: ");
    let line = model
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| panic!("the model declares the input {field}"));
    let written = match line.find(", example: ") {
        Some(at) => format!("{}, example: {value}}}", &line[..at]),
        None => format!("{}, example: {value}}}", line.trim_end_matches('}')),
    };
    with(model, line, &written)
}

/// `model` with the `example:` of the input `field` left out.
fn without_example(model: &str, field: &str) -> String {
    let prefix = format!("      - {{name: {field}, type: ");
    let line = model
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| panic!("the model declares the input {field}"));
    let at = line
        .find(", example: ")
        .expect("the input carries an example");
    with(model, line, &format!("{}}}", &line[..at]))
}

fn example(field: &str, value: &str) -> String {
    example_in(MODEL, field, value)
}

fn below_ess_33(model: &str) -> String {
    with(model, "format: ess/33", "format: ess/32")
}

fn refusal<'e>(
    errors: &'e ValidationErrors,
    code: ValidationCode,
    site: &str,
    says: &str,
) -> &'e ess_primitives::error::ValidationError {
    errors
        .as_slice()
        .iter()
        .find(|error| {
            error.code == code && error.location.contains(site) && error.message.contains(says)
        })
        .unwrap_or_else(|| panic!("expected {code:?} at …{site} saying {says:?}, got:\n{errors}"))
}

fn example_site(command: &str, field: &str) -> String {
    format!("{command}.{field}.example")
}

fn yaml(text: &str) -> Node {
    ess_yaml::from_str(text).unwrap_or_else(|error| panic!("{error}: {text}"))
}

#[test]
fn a_struct_map_or_json_example_is_admitted_at_ess_33_and_round_trips() {
    let spec = admitted(MODEL);
    for (command, field, expected) in [
        (
            "demo.docs.CreateDoc",
            "body",
            "{version: \"1\", nodes: [{id: start, kind: entry}]}",
        ),
        (
            "demo.docs.ImportDoc",
            "header",
            "{version: \"2\", note: draft, origin: upload}",
        ),
        ("demo.docs.ImportDoc", "limits", "{low: 1, high: 5}"),
        ("demo.docs.ImportDoc", "labels", "{team: core}"),
        (
            "demo.docs.ImportDoc",
            "settings",
            "{retries: 3, mode: fast}",
        ),
        ("demo.docs.ImportDoc", "parts", "[{kind: intro}]"),
    ] {
        let declared = &spec.commands()[&command.parse().unwrap()];
        assert_eq!(
            declared.examples[field],
            yaml(expected),
            "{command}.{field}"
        );
        let written = ess_yaml::to_string(declared).unwrap();
        let reread: ess_domain::command::RawCommandSpec = ess_yaml::from_str(&written).unwrap();
        let reread = ess_domain::command::CommandSpec::try_from(reread).unwrap();
        assert_eq!(&reread, declared, "{command} round-trips");
    }
    // A `Json` example is any value but `null`, through its newtype; a map example names keys of
    // its key type and values of its value type.
    for (field, value) in [
        ("settings", "[1, two]"),
        ("settings", "\"text\""),
        ("settings", "7"),
        ("settings", "{}"),
        ("labels", "{}"),
        ("limits", "{low: 2, high: 2}"),
    ] {
        admitted(&example(field, value));
    }
}

#[test]
fn an_open_struct_example_keeps_undeclared_members_and_a_closed_one_refuses_them() {
    // Open: the undeclared member is kept verbatim; a declared one keeps its type and its
    // required-ness.
    let spec = admitted(MODEL);
    let import = &spec.commands()[&"demo.docs.ImportDoc".parse().unwrap()];
    assert_eq!(
        import.examples["header"].as_map().unwrap().get("origin"),
        Some(&Node::Text("upload".to_owned()))
    );
    let header = example_site(IMPORT, "header");
    refusal(
        &refused(&example("header", "{note: draft, origin: upload}")),
        ValidationCode::MissingDeclaration,
        &header,
        "requires the field `version`",
    );
    refusal(
        &refused(&example("header", "{version: 2, origin: upload}")),
        ValidationCode::TypeMismatch,
        &header,
        "field `version`",
    );
    // Closed: an undeclared member is refused, and so is a value its invariant is false for.
    let limits = example_site(IMPORT, "limits");
    refusal(
        &refused(&example("limits", "{low: 1, high: 5, middle: 3}")),
        ValidationCode::UndeclaredReference,
        &limits,
        "declares no field `middle`",
    );
    let ordered = with(
        MODEL,
        "      - {name: high, type: Integer}\n",
        "      - {name: high, type: Integer}\n    invariants: [low <= high]\n",
    );
    admitted(&ordered);
    refusal(
        &refused(&example_in(&ordered, "limits", "{low: 6, high: 5}")),
        ValidationCode::ConflictingDeclaration,
        &limits,
        "low <= high",
    );
    // A `List<Open>` element naming a member it does not declare: refused below ess/33, admitted
    // from it.
    let only_parts = ["body", "header", "limits", "labels", "settings"]
        .into_iter()
        .fold(MODEL.to_owned(), |model, field| {
            without_example(&model, field)
        });
    admitted(&only_parts);
    refusal(
        &refused(&below_ess_33(&only_parts)),
        ValidationCode::UndeclaredReference,
        &example_site(IMPORT, "parts"),
        "element 0: `demo.docs.Body` declares no field `kind`",
    );
    // A precondition literal of an open struct, the same: admitted from ess/33 only. Preconditions
    // are held once every command validates, so the model carries no other example.
    let precondition = with(
        &without_example(&only_parts, "parts"),
        "domain: demo.docs\n",
        "domain: demo.docs\nactors:\n  - {name: demo.docs.Editor, may: [demo.docs.CreateDoc]}\n\
         preconditions:\n  - command: demo.docs.CreateDoc\n    as: demo.docs.Editor\n    \
         input: {name: first, body: {version: \"1\"}}\n",
    );
    admitted(&precondition);
    let errors = refused(&below_ess_33(&precondition));
    assert!(
        errors.as_slice().iter().any(|error| error.message.contains(
            "precondition 0, input `body`: `Optional<demo.docs.Body>` declares no field `version`"
        )),
        "{errors}"
    );
}

#[test]
fn a_structured_example_below_ess_33_keeps_its_refusal_and_names_ess_33() {
    let errors = refused(&below_ess_33(MODEL));
    for (command, field, says) in [
        (
            CREATE,
            "body",
            "`Optional<demo.docs.Body>` reaches `demo.docs.Body`, which is not a scalar",
        ),
        (
            IMPORT,
            "header",
            "`demo.docs.Header` reaches `demo.docs.Header`, which is not a scalar",
        ),
        (
            IMPORT,
            "limits",
            "`demo.docs.Limits` reaches `demo.docs.Limits`, which is not a scalar",
        ),
        (IMPORT, "labels", "`Map<String, String>` is not a scalar"),
        (
            IMPORT,
            "settings",
            "is not a value of `demo.docs.Settings` (Json)",
        ),
    ] {
        let error = refusal(
            &errors,
            ValidationCode::TypeMismatch,
            &example_site(command, field),
            says,
        );
        assert!(
            error
                .hint
                .as_deref()
                .is_some_and(|hint| hint.contains("ess/33")),
            "{field}: {error:?}"
        );
    }
    // A union example stays refused at every format, with its own reason and no format to move
    // to.
    let union = with(
        &with(
            MODEL,
            "types:\n",
            "types:\n  - name: demo.docs.Source\n    kind: union\n    tag: kind\n    variants:\n      \
             file: String\n      link: String\n",
        ),
        "      - {name: parts,",
        "      - {name: source, type: demo.docs.Source, example: {kind: file, value: a}}\n      \
         - {name: parts,",
    );
    let source = example_site(IMPORT, "source");
    for model in [union.clone(), below_ess_33(&union)] {
        let errors = refused(&model);
        let error = refusal(
            &errors,
            ValidationCode::TypeMismatch,
            &source,
            "demo.docs.Source",
        );
        assert!(
            !error
                .hint
                .as_deref()
                .is_some_and(|hint| hint.contains("ess/33")),
            "{error:?}"
        );
    }
    let at_33 = refused(&union);
    refusal(
        &at_33,
        ValidationCode::TypeMismatch,
        &source,
        "reaches the union `demo.docs.Source`",
    );
    assert_eq!(
        at_33.as_slice().len(),
        1,
        "only the union is refused at ess/33: {at_33}"
    );
}
