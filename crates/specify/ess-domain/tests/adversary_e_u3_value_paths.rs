//! Adversary pass 1 over E-U3 (Family F A4, beyond10x/ess#233,
//! `docs/design/expression-family-source22.md`): "An input fallback must be statically required
//! along its complete route and assignable to the target; a second optional fallback is refused
//! because `else:` promises a value." A newtype declared over an `Optional` admits absence as
//! surely as an `Optional` does; the route's own walk already counts one as optional
//! (`TypeRegistry::newtype_layers`). These cases ask whether the fallback's terminal does too, and
//! probe the path rules the section lists (no traversal of a List, Map or Union; a newtype in the
//! middle; three segments).
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationErrors;

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("probe.yaml"), raw)])
}

fn refused(text: &str) -> String {
    match RawSpecFile::parse(text) {
        Err(error) => error.to_string(),
        Ok(raw) => Specification::assemble([(Source::new("probe.yaml"), raw)])
            .err()
            .unwrap_or_else(|| panic!("must refuse:\n{text}"))
            .to_string(),
    }
}

/// `label` holds a `MaybeLabel`, a newtype over `Optional<String>`. The primary
/// `previous.label` crosses the `Optional` `previous`; the fallback reads `FALLBACK`.
const MODEL: &str = r"format: ess/22
system: probe
version: v1
domain: probe.maybe

types:
  - {name: probe.maybe.NoteId, kind: newtype, of: Uuid}
  - {name: probe.maybe.MaybeLabel, kind: newtype, of: Optional<String>}
  - name: probe.maybe.Previous
    kind: struct
    fields:
      - {name: label, type: probe.maybe.MaybeLabel}
  - name: probe.maybe.Settings
    kind: struct
    fields:
      - {name: label, type: probe.maybe.MaybeLabel}
      - {name: tags, type: List<probe.maybe.Previous>}
      - {name: wrapped, type: probe.maybe.Wrapped}
  - {name: probe.maybe.Wrapped, kind: newtype, of: probe.maybe.Previous}
  - name: probe.maybe.Choice
    kind: union
    tag: kind
    variants:
      previous: probe.maybe.Previous
      settings: probe.maybe.Wrapped

entities:
  - name: probe.maybe.Note
    identity: {name: note_id, type: probe.maybe.NoteId}
    fields:
      - {name: label, type: probe.maybe.MaybeLabel}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}

events:
  - name: probe.maybe.Opened
    fields:
      - {name: note_id, type: probe.maybe.NoteId}

commands:
  - name: probe.maybe.Open
    input:
      - {name: note_id, type: probe.maybe.NoteId}
      - {name: previous, type: Optional<probe.maybe.Previous>}
      - {name: settings, type: probe.maybe.Settings}
      - {name: maybe, type: probe.maybe.MaybeLabel}
      - {name: choice, type: probe.maybe.Choice}
    outcomes:
      - name: opened
        creates: probe.maybe.Note
        instance: note_id
        emits: [probe.maybe.Opened]
        payload:
          probe.maybe.Opened:
            note_id: input.note_id
        sets:
          label: SOURCE
";

fn with(source: &str) -> String {
    MODEL.replace("SOURCE", source)
}

/// Control: a required fallback of the target's type is admitted, so the refusals below are about
/// the fallback alone.
#[test]
fn adv_control_a_path_of_the_targets_type_is_admitted() {
    assemble(&with("input.settings.wrapped.label")).unwrap_or_else(|errors| {
        panic!("a newtype in the middle of a three-segment path is admitted: {errors}")
    });
}

#[test]
fn adv_a_fallback_path_ending_in_a_newtype_over_optional_is_refused() {
    let text = with("{input: previous.label, else: input.settings.label}");
    match assemble(&text) {
        Ok(_) => panic!(
            "`else: input.settings.label` reads a `MaybeLabel`, a newtype over \
             `Optional<String>` that may be absent, and is admitted as a fallback that promises \
             a value"
        ),
        Err(errors) => {
            let errors = errors.to_string();
            assert!(errors.contains("`else:` promises a value"), "{errors}");
        }
    }
}

#[test]
fn adv_a_top_level_fallback_of_a_newtype_over_optional_is_refused() {
    let text = with("{input: previous.label, else: input.maybe}");
    match assemble(&text) {
        Ok(_) => panic!(
            "`else: input.maybe` reads a `MaybeLabel`, a newtype over `Optional<String>` that \
             may be absent, and is admitted as a fallback that promises a value"
        ),
        Err(errors) => {
            let errors = errors.to_string();
            assert!(errors.contains("`else:` promises a value"), "{errors}");
        }
    }
}

#[test]
fn adv_a_path_through_a_list_or_a_union_is_refused() {
    for through in ["input.settings.tags.label", "input.choice.previous.label"] {
        let error = refused(&with(through));
        assert!(error.contains("has no members"), "{through}: {error}");
    }
}
