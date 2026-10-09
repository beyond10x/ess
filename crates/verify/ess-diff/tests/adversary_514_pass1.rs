//! Adversarial cases for <https://github.com/beyond10x/ess/issues/514>: a change rated breaking
//! for callers only where an input a caller of the earlier revision could send, and that the
//! earlier revision did not already refuse, is refused by the later one.
//!
//! The unit's own case `a_widened_guard_whose_added_inputs_an_earlier_outcome_already_refused_stays_unknown`
//! states the rule: an input the earlier revision already refused does not move. These cases hold
//! the implementation to that rule where the earlier refusal is declared after the outcome, where
//! an added refusal refines or renames one that already refused the same inputs, and where the
//! witness is a value the earlier revision's type does not admit.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, Compatibility, EssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const V1: &str = include_str!("fixtures/narrowed/v1.yaml");
const WIDENED_GUARD: &str = include_str!("fixtures/narrowed/widened-guard.yaml");

const GUARD: &str = "command/demo.tok.Present/outcome-condition-changed/empty-scope";

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn edited(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "`{from}` in\n{text}");
    text.replacen(from, to, 1)
}

fn callers(delta: &EssDelta, id: &str) -> Compatibility {
    let change = delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == id)
        .unwrap_or_else(|| panic!("`{id}` in {}", delta.to_canonical_json()));
    delta
        .compatibility_of(&change.id())
        .expect("classified")
        .callers()
}

const BEFORE_EMPTY: &str = "      - name: empty-scope\n";
const BEFORE_ACCEPTED: &str = "      - name: accepted\n";

/// `scope == "none"` was refused in the earlier revision by `none-scope`, declared after
/// `empty-scope`, with the same error. Widening `empty-scope` to take it moves no caller from an
/// answer to a refusal: the same input is refused with the same error.
#[test]
fn a_widened_guard_whose_added_inputs_a_later_refusal_already_refused_is_not_breaking() {
    let later = "      - name: none-scope\n        when: scope == \"none\"\n        \
                 error: demo.tok.Rejected\n      - name: accepted\n";
    let before = edited(V1, BEFORE_ACCEPTED, later);
    let after = edited(WIDENED_GUARD, BEFORE_ACCEPTED, later);
    let delta = classified(&ir(&before), &ir(&after)).unwrap();
    assert_ne!(
        callers(&delta, GUARD),
        Compatibility::Breaking,
        "every input the widened guard now takes was refused before\n{}",
        delta.to_canonical_json()
    );
}

/// The earlier revision refused `""` and `"none"` with `demo.tok.Rejected`; the later one splits
/// out `none-scope` for `"none"` with the same error. No input a caller could have had answered is
/// refused now.
#[test]
fn a_refusal_added_inside_an_existing_refusal_is_not_breaking_for_callers() {
    let refined = "      - name: none-scope\n        when: scope == \"none\"\n        \
                   error: demo.tok.Rejected\n      - name: empty-scope\n";
    let after = edited(WIDENED_GUARD, BEFORE_EMPTY, refined);
    let delta = classified(&ir(WIDENED_GUARD), &ir(&after)).unwrap();
    assert_ne!(
        callers(&delta, "command/demo.tok.Present/outcome-added/none-scope"),
        Compatibility::Breaking,
        "`none-scope` refuses only inputs `empty-scope` refused before\n{}",
        delta.to_canonical_json()
    );
}

/// A refusal renamed, its guard and error unchanged: the diff sees it removed and added. The added
/// one refuses exactly the inputs the removed one refused.
#[test]
fn a_renamed_refusal_is_not_breaking_for_callers_as_an_added_refusal() {
    let after = edited(V1, BEFORE_EMPTY, "      - name: blank-scope\n");
    let delta = classified(&ir(V1), &ir(&after)).unwrap();
    assert_ne!(
        callers(&delta, "command/demo.tok.Present/outcome-added/blank-scope"),
        Compatibility::Breaking,
        "`blank-scope` refuses only inputs `empty-scope` refused before\n{}",
        delta.to_canonical_json()
    );
}

/// The later revision adds the variant `Admin` to the input enum `Mode` and refuses it. No caller
/// of the earlier revision can send `Admin`: its decoder refused it. The witness `satisfiable`
/// finds is the guard's own literal, which is not a value of the earlier revision's type.
#[test]
fn a_refusal_of_a_variant_the_earlier_revision_lacks_is_not_breaking_for_callers() {
    let with_mode = |variants: &str| {
        edited(
            &edited(
                V1,
                "  - {name: demo.tok.TokenId, kind: newtype, of: String}\n",
                &format!(
                    "  - {{name: demo.tok.TokenId, kind: newtype, of: String}}\n  \
                     - {{name: demo.tok.Mode, kind: enum, variants: [{variants}]}}\n"
                ),
            ),
            "      - {name: scope, type: String}\n    outcomes:",
            "      - {name: scope, type: String}\n      - {name: mode, type: demo.tok.Mode}\n    outcomes:",
        )
    };
    let before = with_mode("Read, Write");
    let after = edited(
        &with_mode("Read, Write, Admin"),
        BEFORE_ACCEPTED,
        "      - name: admin-mode\n        when: mode == Admin\n        \
         error: demo.tok.Rejected\n      - name: accepted\n",
    );
    let delta = classified(&ir(&before), &ir(&after)).unwrap();
    assert_ne!(
        callers(&delta, "command/demo.tok.Present/outcome-added/admin-mode"),
        Compatibility::Breaking,
        "no caller of the earlier revision can send `Admin`\n{}",
        delta.to_canonical_json()
    );
}

// ---- probes that held -----------------------------------------------------------------------

/// A refusal guarded by an input the later revision adds as `Optional`: no caller of the earlier
/// revision sends it, so the refusal is not decided against them.
#[test]
fn held_a_refusal_on_an_input_added_as_optional_is_not_breaking() {
    let after = edited(
        &edited(
            V1,
            "      - {name: scope, type: String}\n    outcomes:",
            "      - {name: scope, type: String}\n      - {name: coupon, type: Optional<String>}\n    outcomes:",
        ),
        BEFORE_ACCEPTED,
        "      - name: bad-coupon\n        when: coupon == \"x\"\n        \
         error: demo.tok.Rejected\n      - name: accepted\n",
    );
    let delta = classified(&ir(V1), &ir(&after)).unwrap();
    assert_ne!(
        callers(&delta, "command/demo.tok.Present/outcome-added/bad-coupon"),
        Compatibility::Breaking,
        "{}",
        delta.to_canonical_json()
    );
}

/// A refusal of an earlier `Optional` input's absence refuses a caller that left it out.
#[test]
fn held_a_refusal_of_an_absent_optional_input_is_breaking() {
    let with_nick = edited(
        V1,
        "      - {name: scope, type: String}\n    outcomes:",
        "      - {name: scope, type: String}\n      - {name: nick, type: Optional<String>}\n    outcomes:",
    );
    let after = edited(
        &with_nick,
        BEFORE_ACCEPTED,
        "      - name: no-nick\n        when: not defined(nick)\n        \
         error: demo.tok.Rejected\n      - name: accepted\n",
    );
    let delta = classified(&ir(&with_nick), &ir(&after)).unwrap();
    assert_eq!(
        callers(&delta, "command/demo.tok.Present/outcome-added/no-nick"),
        Compatibility::Breaking,
        "{}",
        delta.to_canonical_json()
    );
}

/// A refusal added after an accepting outcome that takes every input it would refuse is shadowed.
#[test]
fn held_a_refusal_shadowed_by_an_earlier_accepting_outcome_is_not_breaking() {
    let guest = "      - name: guest\n        when: scope == \"guest\"\n        creates: demo.tok.Token\n        \
                 instance: token_id\n        sets: {scope: input.scope}\n        emits: [demo.tok.Presented]\n        \
                 payload:\n          demo.tok.Presented: {token_id: input.token_id}\n";
    let before = edited(V1, BEFORE_ACCEPTED, &format!("{guest}{BEFORE_ACCEPTED}"));
    let after = edited(
        V1,
        BEFORE_ACCEPTED,
        &format!(
            "{guest}      - name: guest-again\n        when: scope == \"guest\"\n        \
             error: demo.tok.Rejected\n{BEFORE_ACCEPTED}"
        ),
    );
    let delta = classified(&ir(&before), &ir(&after)).unwrap();
    assert_ne!(
        callers(&delta, "command/demo.tok.Present/outcome-added/guest-again"),
        Compatibility::Breaking,
        "{}",
        delta.to_canonical_json()
    );
}

/// A required input whose type is a newtype of `Optional` may be left out.
#[test]
fn held_an_added_input_of_a_newtype_of_optional_is_not_breaking() {
    let after = edited(
        &edited(
            V1,
            "  - {name: demo.tok.TokenId, kind: newtype, of: String}\n",
            "  - {name: demo.tok.TokenId, kind: newtype, of: String}\n  \
             - {name: demo.tok.Maybe, kind: newtype, of: Optional<String>}\n",
        ),
        "      - {name: scope, type: String}\n    outcomes:",
        "      - {name: scope, type: String}\n      - {name: audience, type: demo.tok.Maybe}\n    outcomes:",
    );
    let delta = classified(&ir(V1), &ir(&after)).unwrap();
    assert_ne!(
        callers(&delta, "command/demo.tok.Present/input-added/audience"),
        Compatibility::Breaking,
        "{}",
        delta.to_canonical_json()
    );
}
