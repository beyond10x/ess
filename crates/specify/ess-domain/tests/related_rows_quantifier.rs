//! A selector tests each candidate's related rows: `exists: {in: {entity: E}, as: b, that: P}`
//! (`ess/27`, beyond10x/ess#551, `docs/design/related-rows-quantifier.md`).
//!
//! What validation decides:
//! * the rows quantifier is admitted in the `where:` of `{related: {entity, where, field}}` and of
//!   `{related: {entity, where, order_by, first}}`, and in a row-set `when_related:`'s `where:` and
//!   `forall:`, with `b.<field>` typed as a row of `E` and bare names still the candidate's;
//! * below `ess/27` it is refused naming `ess/27`;
//! * the body ties the rows to the candidate with a top-level `b.<f> == <candidate field>`;
//! * the binder names the related row only; `forall` is refused with its `not exists` rewrite; one
//!   level only; `E` is a declared entity;
//! * every other predicate site refuses it by name.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

/// Slots of a group ranked by score, memberships on either side of a slot, and a command pairing
/// on the best slot whose two sides each hold a `Present` membership. `@…@` mark where a test
/// rewrites the model.
const MODEL: &str = "format: @FORMAT@
system: demo
version: v1
domain: demo.slots
types:
  - {name: demo.slots.SlotId, kind: newtype, of: Uuid}
  - {name: demo.slots.MembershipId, kind: newtype, of: Uuid}
  - {name: demo.slots.PairId, kind: newtype, of: Uuid}
  - {name: demo.slots.Side, kind: enum, variants: [Left, Right]}
@TYPES@entities:
  - name: demo.slots.Slot
    identity: {name: slot_id, type: demo.slots.SlotId}
    fields:
      - {name: group_id, type: String}
      - {name: score, type: Integer}
@INVARIANTS@    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
  - name: demo.slots.Membership
    identity: {name: membership_id, type: demo.slots.MembershipId}
    fields:
      - {name: slot_id, type: demo.slots.SlotId}
      - {name: side, type: demo.slots.Side}
      - {name: element_key, type: String}
      - {name: element_score, type: Integer}
    lifecycle:
      initial: Present
      states: [Present, Removed]
      terminal: [Removed]
      transitions:
        - {name: remove, from: [Present], to: Removed}
  - name: demo.slots.Pair
    identity: {name: pair_id, type: demo.slots.PairId}
    fields:
      - {name: group_id, type: String}
    lifecycle:
      initial: Made
      states: [Made, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Made], to: Done}
errors:
  - {name: demo.slots.NoMatchableSlot, summary: No slot of the group can match., fields: []}
  - {name: demo.slots.Refused, summary: Refused., fields: []}
events:
  - name: demo.slots.SlotOpened
    fields:
      - {name: slot_id, type: demo.slots.SlotId}
  - name: demo.slots.SlotClosed
    fields:
      - {name: slot_id, type: demo.slots.SlotId}
  - name: demo.slots.GroupClosed
    fields:
      - {name: group_id, type: String}
      - {name: closed, type: Integer}
  - name: demo.slots.Joined
    fields:
      - {name: membership_id, type: demo.slots.MembershipId}
  - name: demo.slots.Left
    fields:
      - {name: membership_id, type: demo.slots.MembershipId}
  - name: demo.slots.Finished
    fields:
      - {name: pair_id, type: demo.slots.PairId}
  - name: demo.slots.PairMade
    fields:
      - {name: pair_id, type: demo.slots.PairId}
      - {name: slot_id, type: demo.slots.SlotId}
      - {name: left_key, type: String}
commands:
  - name: demo.slots.OpenSlot
    input:
      - {name: slot_id, type: demo.slots.SlotId}
      - {name: group_id, type: String}
      - {name: score, type: Integer}
    outcomes:
      - name: opened
        creates: demo.slots.Slot
        instance: slot_id
        sets: {group_id: input.group_id, score: input.score}
        emits: [demo.slots.SlotOpened]
        payload:
          demo.slots.SlotOpened: {slot_id: input.slot_id}
  - name: demo.slots.CloseSlot
    input:
      - {name: slot_id, type: demo.slots.SlotId}
    outcomes:
@CLOSE@      - name: closed
        moves: demo.slots.Slot.close
        instance: slot_id
        emits: [demo.slots.SlotClosed]
        payload:
          demo.slots.SlotClosed: {slot_id: input.slot_id}
@AFFECTS@  - name: demo.slots.Join
    input:
      - {name: membership_id, type: demo.slots.MembershipId}
      - {name: slot_id, type: demo.slots.SlotId}
      - {name: side, type: demo.slots.Side}
      - {name: element_key, type: String}
      - {name: element_score, type: Integer}
    outcomes:
      - name: joined
        creates: demo.slots.Membership
        instance: membership_id
        sets:
          slot_id: input.slot_id
          side: input.side
          element_key: input.element_key
          element_score: input.element_score
        emits: [demo.slots.Joined]
        payload:
          demo.slots.Joined: {membership_id: input.membership_id}
  - name: demo.slots.Leave
    input:
      - {name: membership_id, type: demo.slots.MembershipId}
      - {name: slot_id, type: demo.slots.SlotId}
    outcomes:
@LEAVE@      - name: left
        moves: demo.slots.Membership.remove
        instance: membership_id
        emits: [demo.slots.Left]
        payload:
          demo.slots.Left: {membership_id: input.membership_id}
  - name: demo.slots.Finish
    input:
      - {name: pair_id, type: demo.slots.PairId}
    outcomes:
      - name: finished
        moves: demo.slots.Pair.finish
        instance: pair_id
        emits: [demo.slots.Finished]
        payload:
          demo.slots.Finished: {pair_id: input.pair_id}
  - name: demo.slots.MatchInGroup
    input:
      - {name: pair_id, type: demo.slots.PairId}
      - {name: group_id, type: String}
    outcomes:
      - name: no-match
        when_related:
          entity: demo.slots.Slot
          where: @GUARD@
          exists: false
        error: demo.slots.NoMatchableSlot
      - name: paired
        creates: demo.slots.Pair
        instance: pair_id
        sets:
          group_id: input.group_id
        emits: [demo.slots.PairMade]
        payload:
          demo.slots.PairMade:
            pair_id: input.pair_id
            slot_id:
              related:
                entity: demo.slots.Slot
                where: @SLOT@
@READ@            left_key:
              related:
                entity: demo.slots.Membership
                where: {all: [slot_id == payload.demo.slots.PairMade.slot_id, side == Left, state == Present]}
                order_by: [element_score desc, membership_id]
                first: element_key
@COMMANDS@@VIEWS@";

const HEAD: &str = "                order_by: [score desc, slot_id]
                first: slot_id
";

const ONE_ROW: &str = "                field: slot_id
";

/// Some membership of the candidate on `side`, `Present`.
fn side(side: &str) -> String {
    format!(
        "{{exists: {{in: {{entity: demo.slots.Membership}}, as: member, that: {{all: \
         [member.slot_id == slot_id, member.side == {side}, member.state == Present]}}}}}}"
    )
}

fn matchable() -> String {
    format!(
        "{{all: [group_id == input.group_id, state == Open, {}, {}]}}",
        side("Left"),
        side("Right")
    )
}

/// What a test changes of [`MODEL`]; every key left out keeps the matching example.
#[derive(Clone)]
struct Edit {
    format: &'static str,
    slot: String,
    guard: String,
    read: &'static str,
    types: String,
    invariants: String,
    close: String,
    affects: String,
    leave: String,
    commands: String,
    views: String,
}

impl Default for Edit {
    fn default() -> Self {
        Self {
            format: "ess/27",
            slot: matchable(),
            guard: matchable(),
            read: HEAD,
            types: String::new(),
            invariants: String::new(),
            close: String::new(),
            affects: String::new(),
            leave: String::new(),
            commands: String::new(),
            views: String::new(),
        }
    }
}

fn model(edit: &Edit) -> String {
    MODEL
        .replace("@FORMAT@", edit.format)
        .replace("@SLOT@", &edit.slot)
        .replace("@GUARD@", &edit.guard)
        .replace("@READ@", edit.read)
        .replace("@TYPES@", &edit.types)
        .replace("@INVARIANTS@", &edit.invariants)
        .replace("@CLOSE@", &edit.close)
        .replace("@AFFECTS@", &edit.affects)
        .replace("@LEAVE@", &edit.leave)
        .replace("@COMMANDS@", &edit.commands)
        .replace("@VIEWS@", &edit.views)
}

/// The model with the slot head's `where:` and the guard's both replaced by `selector`.
fn selecting(selector: &str) -> String {
    model(&Edit {
        slot: selector.to_owned(),
        guard: selector.to_owned(),
        ..Edit::default()
    })
}

/// The selector `{all: [group_id == input.group_id, state == Open, <test>]}`.
fn with_test(test: &str) -> String {
    format!("{{all: [group_id == input.group_id, state == Open, {test}]}}")
}

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the model parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("slots.yaml"), raw)])
}

fn admitted(body: &str) -> Specification {
    assemble(body).unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{body}"))
}

fn refused(body: &str) -> ValidationErrors {
    match assemble(body) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

fn rendered(error: &ValidationError) -> String {
    ValidationErrors::from(error.clone()).to_string()
}

/// Every refusal of `errors` has `code`, and there is one per site in `sites`, located there.
fn at_sites<'e>(
    errors: &'e ValidationErrors,
    code: ValidationCode,
    sites: &[&str],
) -> Vec<&'e ValidationError> {
    let all: Vec<&ValidationError> = errors.as_slice().iter().collect();
    assert_eq!(all.len(), sites.len(), "one refusal per site:\n{errors}");
    for error in &all {
        assert_eq!(error.code, code, "{errors}");
    }
    for site in sites {
        assert!(
            all.iter().any(|error| rendered(error).contains(site)),
            "a refusal at `{site}`:\n{errors}"
        );
    }
    all
}

const GUARD_AT: &str = "command.demo.slots.MatchInGroup.outcomes.no-match.when_related";
const HEAD_AT: &str =
    "command.demo.slots.MatchInGroup.outcomes.paired.payload.demo.slots.PairMade.slot_id";

#[test]
fn a_head_over_matchable_slots_validates_at_ess_27() {
    let spec = admitted(&model(&Edit::default()));
    // The selector keeps its canonical text, the rows collection in it.
    let command = spec
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.slots.MatchInGroup")
        .expect("the command");
    let text = format!("{:?}", command.outcomes[1].payload);
    assert!(
        text.contains("demo.slots.Membership"),
        "the head's selector carries the rows test: {text}"
    );

    // A one-row read admits it too, and so does a test written on the candidate's own entity.
    admitted(&model(&Edit {
        read: ONE_ROW,
        ..Edit::default()
    }));
    let higher = "{exists: {in: {entity: demo.slots.Slot}, as: other, that: {all: \
                  [other.group_id == group_id, other.score > score, other.state == Open]}}}";
    admitted(&selecting(&with_test(higher)));
    // A negated test, a tie on either side, and other conjuncts reading the input.
    let negated = "{not: {exists: {in: {entity: demo.slots.Membership}, as: member, that: \
                   {all: [slot_id == member.slot_id, member.element_key == input.group_id]}}}}";
    admitted(&selecting(&with_test(negated)));
}

#[test]
fn a_row_set_guard_refusing_no_matchable_slot_validates() {
    admitted(&model(&Edit {
        slot: "{all: [group_id == input.group_id, state == Open]}".to_owned(),
        ..Edit::default()
    }));
    // A row-set `forall:` admits it as its `where:` does.
    let forall = model(&Edit {
        slot: "{all: [group_id == input.group_id, state == Open]}".to_owned(),
        guard: "{all: [group_id == input.group_id, state == Open]}".to_owned(),
        ..Edit::default()
    })
    .replace(
        "          exists: false\n        error: demo.slots.NoMatchableSlot",
        &format!(
            "          forall: {}\n        error: demo.slots.NoMatchableSlot",
            side("Left")
        ),
    );
    assert!(forall.contains("forall: {exists:"), "the edit applies");
    admitted(&forall);
}

#[test]
fn the_rows_test_below_ess_27_is_refused_naming_ess_27() {
    let below = model(&Edit {
        format: "ess/26",
        read: ONE_ROW,
        ..Edit::default()
    })
    .replace(
        "                order_by: [element_score desc, membership_id]\n                first: element_key\n",
        "                field: element_key\n",
    )
    // The inner read is not the subject here: give it a selector every format from ess/22 reads.
    .replace(
        "slot_id == payload.demo.slots.PairMade.slot_id",
        "element_key == input.group_id",
    );
    let errors = refused(&below);
    let all = at_sites(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        &[GUARD_AT, HEAD_AT],
    );
    for error in all {
        let text = rendered(error);
        assert!(text.contains("ess/27"), "names ess/27: {text}");
        assert!(
            text.contains("{entity: <Entity>}") || text.contains("rows of"),
            "{text}"
        );
    }
    // At ess/21 a selector is not read at all; the rows test there stays refused, at parse.
    let older = below.replace("format: ess/26", "format: ess/21");
    let read = ess_domain::spec::RawSpecFile::parse(&older)
        .map_err(|error| error.to_string())
        .and_then(|raw| {
            Specification::assemble([(ess_domain::system::Source::new("slots.yaml"), raw)])
                .map_err(|errors| errors.to_string())
        });
    assert!(read.is_err(), "refused at ess/21:\n{older}");
}

#[test]
fn an_untied_rows_test_is_refused() {
    for untied in [
        // No comparison with the candidate at all.
        "{exists: {in: {entity: demo.slots.Membership}, as: member, that: {all: [member.side == \
         Left, member.state == Present]}}}",
        // A comparison with the input is no tie.
        "{exists: {in: {entity: demo.slots.Membership}, as: member, that: member.element_key == \
         input.group_id}}",
        // `state` is no tie.
        "{exists: {in: {entity: demo.slots.Membership}, as: member, that: member.element_key == \
         state}}",
        // An ordering is no tie.
        "{exists: {in: {entity: demo.slots.Membership}, as: member, that: member.element_score > \
         score}}",
    ] {
        let errors = refused(&selecting(&with_test(untied)));
        for error in at_sites(
            &errors,
            ValidationCode::MissingDeclaration,
            &[GUARD_AT, HEAD_AT],
        ) {
            let text = rendered(error);
            assert!(
                text.contains("not tied to the candidate"),
                "{untied}: {text}"
            );
            assert!(
                text.contains("member.<field> =="),
                "{untied}: the hint: {text}"
            );
        }
    }
}

#[test]
fn a_tie_inside_any_is_no_tie() {
    let any = "{exists: {in: {entity: demo.slots.Membership}, as: member, that: {all: [{any: \
               [member.slot_id == slot_id, member.side == Left]}, member.state == Present]}}}";
    let not = "{exists: {in: {entity: demo.slots.Membership}, as: member, that: {all: [{not: \
               member.slot_id == slot_id}, member.state == Present]}}}";
    for test in [any, not] {
        let errors = refused(&selecting(&with_test(test)));
        at_sites(
            &errors,
            ValidationCode::MissingDeclaration,
            &[GUARD_AT, HEAD_AT],
        );
    }
}

#[test]
fn a_binder_naming_a_candidate_field_is_refused() {
    for (binder, shadows) in [
        ("score", "score"),
        ("slot_id", "slot_id"),
        ("state", "state"),
        ("input", "input"),
        ("subject", "subject"),
        ("payload", "payload"),
        ("now", "now"),
    ] {
        let quantifier = format!(
            "{{exists: {{in: {{entity: demo.slots.Membership}}, as: {binder}, that: {{all: \
             [{binder}.slot_id == slot_id, {binder}.side == Left]}}}}}}"
        );
        let errors = refused(&selecting(&with_test(&quantifier)));
        for error in at_sites(
            &errors,
            ValidationCode::ConflictingDeclaration,
            &[GUARD_AT, HEAD_AT],
        ) {
            let text = rendered(error);
            assert!(text.contains(&format!("`{shadows}`")), "{binder}: {text}");
        }
    }
    // A binder of an enclosing quantifier.
    let nested = "{exists: {in: input.tags, as: member, that: {exists: {in: {entity: \
                  demo.slots.Membership}, as: member, that: member.slot_id == slot_id}}}}";
    let errors = refused(&selecting(&with_test(nested)).replace(
        "      - {name: group_id, type: String}\n    outcomes:\n      - name: no-match",
        "      - {name: group_id, type: String}\n      - {name: tags, type: \
                 'List<String>'}\n    outcomes:\n      - name: no-match",
    ));
    at_sites(
        &errors,
        ValidationCode::ConflictingDeclaration,
        &[GUARD_AT, HEAD_AT],
    );
}

#[test]
fn forall_over_rows_is_refused_with_the_not_exists_rewrite() {
    let forall = "{forall: {in: {entity: demo.slots.Membership}, as: member, that: {all: \
                  [member.slot_id == slot_id, member.state == Present]}}}";
    let errors = refused(&selecting(&with_test(forall)));
    for error in at_sites(
        &errors,
        ValidationCode::UnsupportedConstruct,
        &[GUARD_AT, HEAD_AT],
    ) {
        let text = rendered(error);
        assert!(text.contains("forall"), "{text}");
        assert!(text.contains("not: {exists:"), "the rewrite: {text}");
    }
}

#[test]
fn a_nested_rows_test_is_refused() {
    let nested = "{exists: {in: {entity: demo.slots.Membership}, as: member, that: {all: \
                  [member.slot_id == slot_id, {exists: {in: {entity: demo.slots.Pair}, as: pair, \
                  that: pair.group_id == member.element_key}}]}}}";
    let errors = refused(&selecting(&with_test(nested)));
    for error in at_sites(
        &errors,
        ValidationCode::UnsupportedConstruct,
        &[GUARD_AT, HEAD_AT],
    ) {
        assert!(rendered(error).contains("one level"), "{errors}");
    }
    // An ordinary quantifier over a list inside the body stays admitted: nothing here holds a
    // list, so the inner one walks the input's.
}

#[test]
fn an_undeclared_entity_is_refused() {
    let undeclared = "{exists: {in: {entity: demo.slots.Ghost}, as: member, that: \
                      member.slot_id == slot_id}}";
    let errors = refused(&selecting(&with_test(undeclared)));
    for error in at_sites(
        &errors,
        ValidationCode::UndeclaredReference,
        &[GUARD_AT, HEAD_AT],
    ) {
        let text = rendered(error);
        assert!(text.contains("demo.slots.Ghost"), "{text}");
        assert!(
            text.contains("demo.slots.Membership"),
            "lists the declared: {text}"
        );
    }
}

#[test]
fn a_mistyped_row_read_keeps_the_checkers_messages() {
    // A field the related entity does not hold.
    let unknown = "{exists: {in: {entity: demo.slots.Membership}, as: member, that: {all: \
                   [member.slot_id == slot_id, member.colour == Left]}}}";
    let errors = refused(&selecting(&with_test(unknown)));
    assert!(
        errors
            .as_slice()
            .iter()
            .all(|error| rendered(error).contains("member.colour")),
        "{errors}"
    );
    // A tie between types that do not compare.
    let mistyped = "{exists: {in: {entity: demo.slots.Membership}, as: member, that: \
                    member.element_score == slot_id}}";
    let errors = refused(&selecting(&with_test(mistyped)));
    assert!(
        errors
            .as_slice()
            .iter()
            .all(|error| error.code == ValidationCode::TypeMismatch),
        "{errors}"
    );
}

/// A rows test whose rows are tied to `slot_id`, for the sites that refuse it.
const SLOT_TEST: &str = "{exists: {in: {entity: demo.slots.Membership}, as: member, that: \
                         member.slot_id == slot_id}}";

fn refused_by_name(errors: &ValidationErrors, site: &str) -> String {
    let found: Vec<&ValidationError> = errors
        .as_slice()
        .iter()
        .filter(|error| {
            error.code == ValidationCode::UnsupportedConstruct
                && rendered(error).contains("{entity: demo.slots.Membership}")
        })
        .collect();
    assert!(
        !found.is_empty(),
        "{site}: the rows test is refused by name as unsupported_construct:\n{errors}"
    );
    let text = rendered(found[0]);
    assert!(
        text.contains("`when_related:`") && text.contains("`{related: …}`"),
        "{site}: names the two sites that admit it: {text}"
    );
    text
}

#[test]
fn a_rows_test_in_an_affects_filter_is_refused() {
    let affects = format!(
        "        affects:\n          - entity: demo.slots.Slot\n            where: {{all: \
         [group_id == subject.group_id, {SLOT_TEST}]}}\n            moves: \
         demo.slots.Slot.close\n"
    );
    let errors = refused(&model(&Edit {
        affects,
        ..Edit::default()
    }));
    refused_by_name(&errors, "affects[].where");
    let instances = format!(
        "  - name: demo.slots.CloseGroup\n    input:\n      - {{name: group_id, type: \
         String}}\n    outcomes:\n      - name: closed\n        moves: demo.slots.Slot.close\n        \
         instances: {{where: {{all: [group_id == input.group_id, {SLOT_TEST}]}}}}\n        emits: \
         [demo.slots.GroupClosed]\n        payload:\n          demo.slots.GroupClosed: {{group_id: \
         input.group_id, closed: {{count: changed}}}}\n"
    );
    let errors = refused(&model(&Edit {
        commands: instances,
        ..Edit::default()
    }));
    refused_by_name(&errors, "instances:");
}

#[test]
fn a_rows_test_in_when_subject_names_the_when_related_idiom() {
    let close = format!(
        "      - name: refused\n        when_subject:\n          predicate: {SLOT_TEST}\n        \
         error: demo.slots.Refused\n"
    );
    let errors = refused(&model(&Edit {
        close,
        ..Edit::default()
    }));
    let text = refused_by_name(&errors, "when_subject:");
    assert!(
        text.contains("when_related: {entity:"),
        "the idiom for the subject's related rows: {text}"
    );
}

#[test]
fn every_other_predicate_site_refuses_a_rows_test() {
    // An input guard.
    let when = format!(
        "      - name: refused\n        when: {SLOT_TEST}\n        error: demo.slots.Refused\n"
    );
    refused_by_name(
        &refused(&model(&Edit {
            close: when,
            ..Edit::default()
        })),
        "when:",
    );
    // An identity-addressed `when_related:` predicate.
    let related = format!(
        "      - name: no-slot\n        when_related: {{via: input.slot_id, exists: false}}\n        \
         error: demo.slots.Refused\n      - name: refused\n        when_related: {{via: \
         input.slot_id, predicate: {SLOT_TEST}}}\n        error: demo.slots.Refused\n"
    );
    refused_by_name(
        &refused(&model(&Edit {
            leave: related,
            ..Edit::default()
        })),
        "when_related: {via, predicate}",
    );
    // An entity invariant.
    refused_by_name(
        &refused(&model(&Edit {
            invariants: format!("    invariants:\n      - {SLOT_TEST}\n"),
            ..Edit::default()
        })),
        "an entity invariant",
    );
    // A struct invariant.
    refused_by_name(
        &refused(&model(&Edit {
            types: format!(
                "  - name: demo.slots.Probe\n    kind: struct\n    fields:\n      - {{name: \
                 slot_id, type: demo.slots.SlotId}}\n    invariants:\n      - {SLOT_TEST}\n"
            ),
            ..Edit::default()
        })),
        "a struct invariant",
    );
    // A view filter.
    refused_by_name(
        &refused(&model(&Edit {
            views: format!(
                "views:\n  - name: demo.slots.Open\n    source: demo.slots.Slot\n    filter: \
                 {SLOT_TEST}\n    fields:\n      - {{name: slot_id, type: demo.slots.SlotId}}\n"
            ),
            ..Edit::default()
        })),
        "a view filter",
    );
    // An aggregate measure's `where:`.
    refused_by_name(
        &refused(&model(&Edit {
            views: format!(
                "views:\n  - name: demo.slots.PerGroup\n    source: demo.slots.Slot\n    \
                 group_by: [group_id]\n    fields:\n      - {{name: group_id, type: String}}\n      \
                 - {{name: matchable, type: Integer, aggregate: {{count: {{}}, where: \
                 {SLOT_TEST}}}}}\n"
            ),
            ..Edit::default()
        })),
        "an aggregate measure",
    );
}

#[test]
fn a_published_operand_in_the_body_validates() {
    // The inner head's selector over memberships tests the slot each candidate sits in, tied by
    // the candidate's `slot_id`, against the slot the outcome publishes.
    let published = model(&Edit::default()).replace(
        "where: {all: [slot_id == payload.demo.slots.PairMade.slot_id, side == Left, state == \
         Present]}",
        "where: {all: [slot_id == payload.demo.slots.PairMade.slot_id, side == Left, state == \
         Present, {exists: {in: {entity: demo.slots.Slot}, as: seat, that: {all: [seat.slot_id \
         == slot_id, seat.slot_id == payload.demo.slots.PairMade.slot_id, seat.state == \
         Open]}}}]}",
    );
    assert_ne!(published, model(&Edit::default()), "the edit applies");
    admitted(&published);
}
