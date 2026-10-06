//! Adversary pass 1 on bulk removal and deleting `affects:` entries (beyond10x/ess#452).
//!
//! * Below ess/23 a document that never writes a deletion keeps the refusal text it always had.
//! * Below ess/23 the pre-pass that takes off `instances:` beside `deletes:` does not pre-empt the
//!   refusal the conversion always gave first.
//! * From ess/23 the pre-pass that takes off a misnamed deleting entry does not move the position
//!   every later entry's refusal names.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/set-effects.yaml");
const DELETES: &str = include_str!("../../ess-compiler/tests/fixtures/set-deletes.yaml");

const DELETING_ENTRY: &str = "        affects:
          - entity: demo.auth.Token
            where: user_id == subject.user_id
            deletes: demo.auth.Token
";

fn refused(body: &str) -> ValidationErrors {
    let raw = ess_domain::spec::RawSpecFile::parse(body).expect("the document parses");
    match Specification::assemble([(ess_domain::system::Source::new("doc.yaml"), raw)]) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the fixture holds:\n{from}");
    text.replacen(from, to, 1)
}

/// The deletion fixture under `header`, with its deleting `affects:` entry setting a field instead,
/// so a reader without the entry's `deletes:` key (0.53.0) parses it.
fn below_23_readable_by_base(header: &str) -> String {
    replaced(
        &DELETES.replacen("format: ess/23", header, 1),
        "            deletes: demo.auth.Token\n",
        "            sets: {scope: gone}\n",
    )
}

fn under<'e>(errors: &'e ValidationErrors, needle: &str) -> Vec<&'e ValidationError> {
    errors
        .as_slice()
        .iter()
        .filter(|error| error.location.contains(needle))
        .collect()
}

/// ess/16 never admitted `deletes:` beside `instances:`; its refusal of `preserves:` beside
/// `instances:` says what the base said, and does not offer `deletes:` as a remedy the format
/// refuses.
#[test]
fn ess16_preserves_beside_instances_keeps_its_refusal_text() {
    let body = replaced(
        MODEL,
        "        updates: demo.desk.Session\n        instances: {where: team == input.team}\n",
        "        preserves: demo.desk.Session\n        instances: {where: team == input.team}\n",
    );
    let errors = refused(&body);
    let found = under(&errors, "outcomes.noted.instances");
    let [refusal] = found.as_slice() else {
        panic!("one refusal at `noted.instances`:\n{errors}")
    };
    assert_eq!(
        refusal.code,
        ValidationCode::UnsupportedConstruct,
        "{refusal}"
    );
    assert_eq!(
        refusal.message,
        "outcome `noted` preserves an entity and declares `instances:`; a set subject is admitted \
         beside `moves:` and `updates:` only",
        "the ess/16 refusal text is the base's"
    );
    assert_eq!(
        refusal.hint.as_deref(),
        Some("name one row with `instance:`, or change the rows with `moves:` or `updates:`"),
        "the ess/16 hint offers no verb ess/16 refuses"
    );
}

/// Below ess/23 `instance:` and `instances:` on one `deletes:` branch were refused
/// `conflicting_declaration`, before the verb was looked at; the format pre-pass must not
/// replace that with a format refusal whose hint asks for the `instance:` already written.
#[test]
fn ess22_deletes_with_instance_and_instances_keeps_its_conflicting_declaration() {
    let body = replaced(
        &below_23_readable_by_base("format: ess/22"),
        "        deletes: demo.auth.Token\n        instances:",
        "        deletes: demo.auth.Token\n        instance: user_id\n        instances:",
    );
    let errors = refused(&body);
    let found = under(&errors, "RevokeTokens");
    assert!(
        found
            .iter()
            .any(|error| error.code == ValidationCode::ConflictingDeclaration
                && error.message.contains("both `instance:` and `instances:`")),
        "the base's conflicting_declaration at `revoked.instances`:\n{errors}"
    );
}

/// A misnamed deleting entry taken off before conversion leaves every later entry's refusal at
/// the position the author wrote.
#[test]
fn ess23_misnamed_deleting_entry_keeps_later_entry_positions() {
    let body = replaced(
        DELETES,
        DELETING_ENTRY,
        "        affects:
          - entity: demo.auth.Token
            where: user_id == subject.user_id
            deletes: demo.auth.Session
          - entity: demo.auth.Nowhere
            where: user_id == subject.user_id
            sets: {scope: gone}
",
    );
    let errors = refused(&body);
    let found = under(&errors, "DeleteUser");
    assert!(
        found
            .iter()
            .any(|error| error.code == ValidationCode::ConflictingDeclaration
                && error.location.ends_with("outcomes.deleted.affects[0]")),
        "the misnamed entry is refused at affects[0]:\n{errors}"
    );
    assert!(
        found
            .iter()
            .any(|error| error.location.contains("outcomes.deleted.affects[1]")),
        "the undeclared entity is refused at the entry written second, affects[1]:\n{errors}"
    );
    assert!(
        !found
            .iter()
            .any(|error| error.code != ValidationCode::ConflictingDeclaration
                && error.location.contains("outcomes.deleted.affects[0]")),
        "nothing about the second entry is sited at the first:\n{errors}"
    );
}

/// The same shift through the deletion-shape rule: a deleting entry and a setting entry over one
/// entity, written second and third, are refused naming `affects[1]` and `affects[2]`.
#[test]
fn ess23_misnamed_deleting_entry_keeps_the_conflict_pair_positions() {
    let body = replaced(
        DELETES,
        DELETING_ENTRY,
        "        affects:
          - entity: demo.auth.Token
            where: user_id == subject.user_id
            deletes: demo.auth.Session
          - entity: demo.auth.Token
            where: user_id == subject.user_id
            deletes: demo.auth.Token
          - entity: demo.auth.Token
            where: user_id == subject.user_id
            sets: {scope: gone}
",
    );
    let errors = refused(&body);
    let found = under(&errors, "DeleteUser");
    assert!(
        found
            .iter()
            .any(|error| error.code == ValidationCode::ConflictingDeclaration
                && error.location.ends_with("outcomes.deleted.affects[2]")
                && error.message.contains("`affects[1]` and `affects[2]`")),
        "the pair is refused at the third entry, naming the second and third:\n{errors}"
    );
}

/// Below ess/16 a bulk deletion is refused too, and the acceptance's "one refusal, no
/// ESS-COMMAND-007" holds there as it does at ess/22: the branch whose `instances:` was refused is
/// not then also reported as a `deletes:` naming no `instance`, nor its command as declaring none.
#[test]
fn ess15_bulk_delete_is_refused_without_cascade() {
    let errors = refused(&below_23_readable_by_base("format: ess/15"));
    let found = under(&errors, "RevokeTokens");
    assert!(
        found
            .iter()
            .all(|error| error.code != ValidationCode::EmptyDeclaration
                && error.code != ValidationCode::MissingDeclaration),
        "no cascade after the refusal of `instances:`:\n{errors}"
    );
}

/// Below ess/23 a bulk deletion whose filter does not parse kept its `unparsable_predicate`: the
/// filter is wrong under every format, and the format pre-pass drops it unread.
#[test]
fn ess22_bulk_delete_with_unparsable_filter_keeps_its_parse_refusal() {
    let body = replaced(
        &below_23_readable_by_base("format: ess/22"),
        "        instances: {where: {all: [user_id == input.user_id, scope == input.scope]}}\n",
        "        instances: {where: {all: [user_id ==, scope]}}\n",
    );
    let errors = refused(&body);
    assert!(
        under(&errors, "RevokeTokens")
            .iter()
            .any(|error| error.code == ValidationCode::UnparsablePredicate),
        "the base's unparsable_predicate at `revoked.instances.where`:\n{errors}"
    );
}

/// A typo in the entry's `entity:` is the mistake, and `deletes:` names the entity meant. The
/// misnamed-deletion refusal alone tells the author to write `deletes: <the typo>`; the undeclared
/// entity is reported as well, as it is beside a misnamed move (`refuse_affect_moves` keeps the
/// entry and takes off only the move).
#[test]
fn ess23_deleting_entry_with_a_misspelt_entity_reports_the_undeclared_entity() {
    let body = replaced(
        DELETES,
        DELETING_ENTRY,
        "        affects:
          - entity: demo.auth.Tokn
            where: user_id == subject.user_id
            deletes: demo.auth.Token
",
    );
    let errors = refused(&body);
    assert!(
        under(&errors, "DeleteUser").iter().any(|error| error.code
            == ValidationCode::UndeclaredReference
            && error
                .location
                .ends_with("outcomes.deleted.affects[0].entity")),
        "the misspelt entity is reported at affects[0].entity:\n{errors}"
    );
}

/// `selection-effects.md` ("Removal in other domains is one binding per domain"): "`affects:` stays
/// inside the domain of its outcome", and the story declines cross-domain cascade. A deleting entry
/// over another domain's entity is therefore refused, not admitted as the declined cascade.
#[test]
fn ess23_deleting_entry_over_another_domains_entity_is_refused() {
    let auth = replaced(
        &replaced(
            DELETES,
            "version: v1\n",
            "version: v1\ndomains: [demo.auth, demo.mail]\n",
        ),
        DELETING_ENTRY,
        &format!(
            "{DELETING_ENTRY}          - entity: demo.mail.Mailbox
            where: owner == subject.user_id
            deletes: demo.mail.Mailbox
"
        ),
    );
    let mail = "domain: demo.mail
types:
  - {name: demo.mail.BoxId, kind: newtype, of: String}
entities:
  - name: demo.mail.Mailbox
    identity: {name: box_id, type: demo.mail.BoxId}
    fields: [{name: owner, type: demo.auth.UserId}]
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
";
    let files = [("auth.yaml", auth.as_str()), ("mail.yaml", mail)].map(|(name, body)| {
        (
            ess_domain::system::Source::new(name),
            ess_domain::spec::RawSpecFile::parse(body).expect("the document parses"),
        )
    });
    match Specification::assemble(files) {
        Ok(_) => panic!(
            "a deleting `affects:` entry over `demo.mail.Mailbox`, beside a `demo.auth` subject, \
             is admitted: the cross-domain cascade the page says `affects:` does not reach"
        ),
        Err(errors) => assert!(
            under(&errors, "DeleteUser")
                .iter()
                .any(|error| error.location.contains("outcomes.deleted.affects[1]")),
            "refused at the cross-domain entry:\n{errors}"
        ),
    }
}
