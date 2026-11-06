//! An `each:` entry that admits repeated identities (ess/35, beyond10x/ess#748 part 3,
//! story:an-each-write-may-take-repeated-identities-in-list-order).
//!
//! `each: {in: input.<list>, as: <name>, repeated: last_wins}` lifts the requirement that a
//! declared `distinct:` hold the identity member distinct across the list: the elements apply one
//! after another in list order, so a later element's write to a row wins. `last_wins` is the only
//! value. Below `ess/35` the key is refused as `unsupported_format_version` naming `ess/35`;
//! without it the `distinct:` requirement holds as before.

use ess_domain::command::set_effects::Repeated;
use ess_domain::command::RawOutcome;
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const EACH: &str = include_str!("../../ess-compiler/tests/fixtures/set-each.yaml");
const DISTINCT: &str = "      - name: duplicated\n        when: {not: {distinct: {in: applied, as: d, by: d.document_id}}}\n        error: demo.feed.DuplicateDocument\n";
const PLAIN: &str = "each: {in: input.applied, as: doc}";
const REPEATED: &str = "each: {in: input.applied, as: doc, repeated: last_wins}";
const AT: &str = "RunSource.outcomes.ran.affects[0].each";

/// The fixture at `format`, without its `distinct:` refusal, its `each:` written `each`.
fn model(format: u32, each: &str) -> String {
    let edit = |text: String, from: &str, to: &str| {
        assert!(text.contains(from), "the fixture holds:\n{from}");
        text.replacen(from, to, 1)
    };
    let text = edit(
        EACH.to_owned(),
        "format: ess/23",
        &format!("format: ess/{format}"),
    );
    let text = edit(text, DISTINCT, "");
    edit(text, PLAIN, each)
}

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|error| {
        ValidationErrors::from(ess_primitives::error::ValidationError::new(
            ValidationCode::ConflictingDeclaration,
            "parse",
            error.to_string(),
        ))
    })?;
    Specification::assemble([(ess_domain::system::Source::new("feed.yaml"), raw)])
}

fn ran(spec: &Specification) -> &ess_domain::command::Outcome {
    spec.commands()
        .get(&"demo.feed.RunSource".parse().unwrap())
        .expect("RunSource")
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "ran")
        .expect("ran")
}

#[test]
fn repeated_last_wins_validates_without_distinct_at_ess_35() {
    let spec = assemble(&model(35, REPEATED))
        .unwrap_or_else(|errors| panic!("`repeated: last_wins` is admitted at ess/35: {errors}"));
    let outcome = ran(&spec);
    let [entry] = outcome.set_effects.affects.as_slice() else {
        panic!("one entry: {:#?}", outcome.set_effects)
    };
    let each = entry.each.as_ref().expect("the entry reads each element");
    assert_eq!(each.repeated, Some(Repeated::LastWins), "{each:#?}");
    // The key is written back as it was read, and reads back to the same entry.
    let written = ess_yaml::to_string(&RawOutcome::from(outcome.clone())).unwrap();
    assert!(written.contains("repeated: last_wins"), "{written}");
    let read: RawOutcome = ess_yaml::from_str(&written).unwrap();
    let back = ess_domain::command::Outcome::try_from(read)
        .unwrap_or_else(|errors| panic!("{errors}\n{written}"));
    assert_eq!(back.set_effects, outcome.set_effects, "{written}");
    // A `distinct:` beside the key is admitted too: the key lifts the requirement, it does not
    // forbid the refusal.
    let beside = model(35, REPEATED).replacen(
        "      - name: ran\n",
        &format!("{DISTINCT}      - name: ran\n"),
        1,
    );
    assemble(&beside).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn repeated_is_refused_below_ess_35() {
    for format in [23, 27, 34] {
        let errors =
            assemble(&model(format, REPEATED)).expect_err("`repeated:` is refused below ess/35");
        let refusal = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::UnsupportedFormatVersion)
            .unwrap_or_else(|| panic!("ess/{format}: unsupported_format_version, got:\n{errors}"));
        assert!(
            refusal.location.ends_with(&format!("{AT}.repeated")),
            "ess/{format}: {refusal}"
        );
        assert!(
            refusal.message.contains("ess/35"),
            "ess/{format}: {refusal}"
        );
        assert!(
            refusal.message.contains("repeated: last_wins"),
            "ess/{format}: {refusal}"
        );
        assert_eq!(
            errors.len(),
            1,
            "ess/{format}: the refusal comes alone:\n{errors}"
        );
    }
}

#[test]
fn an_unknown_repeated_value_is_refused() {
    for value in ["first_wins", "true", "distinct"] {
        let each = format!("each: {{in: input.applied, as: doc, repeated: {value}}}");
        let errors = assemble(&model(35, &each)).expect_err("only `last_wins` is a value");
        let text = errors.to_string();
        assert!(
            text.contains(value),
            "{value}: names the value written: {text}"
        );
        assert!(
            text.contains("last_wins"),
            "{value}: names the value admitted: {text}"
        );
    }
}

#[test]
fn without_repeated_the_distinct_requirement_holds() {
    let errors = assemble(&model(35, PLAIN)).expect_err("no `distinct:` and no `repeated:`");
    let refusal = errors
        .as_slice()
        .iter()
        .find(|error| {
            error.code == ValidationCode::MissingDeclaration && error.location.ends_with(AT)
        })
        .unwrap_or_else(|| panic!("missing_declaration at `{AT}`, got:\n{errors}"));
    assert!(refusal.message.contains("distinct"), "{refusal}");
    // The hint now names the key as the other way out.
    let hint = refusal.hint.as_deref().unwrap_or_default();
    assert!(hint.contains("repeated: last_wins"), "{refusal}");
    // With the fixture's own `distinct:` refusal the entry is admitted at ess/35 as at ess/23.
    let fixture = EACH.replacen("format: ess/23", "format: ess/35", 1);
    assemble(&fixture).unwrap_or_else(|errors| panic!("{errors}"));
}
