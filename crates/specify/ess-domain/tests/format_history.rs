//! `FORMAT_HISTORY`, the one catalogue of what each `ess/N` admits (beyond10x/ess#460).
//!
//! `ess specify formats`, the `ess/` table of the version history page and the docs lane's release
//! of each `ess/N` all read it. A `const` assertion beside it makes its majors equal
//! `SUPPORTED_FORMATS` at compile time; this file says the same thing where a reader looks for it.
use ess_domain::system::{FORMAT_HISTORY, SUPPORTED_FORMATS};

#[test]
fn format_history_majors_equal_supported_formats() {
    let majors: Vec<u32> = FORMAT_HISTORY.iter().map(|entry| entry.major).collect();
    assert_eq!(
        majors, SUPPORTED_FORMATS,
        "one row per supported major, in order"
    );
}

#[test]
fn every_format_says_what_it_added_and_names_a_release_or_none() {
    for entry in FORMAT_HISTORY {
        assert!(
            !entry.added.is_empty(),
            "ess/{} says nothing it added",
            entry.major
        );
        for item in entry.added.iter().chain(entry.stricter) {
            assert!(
                !item.trim().is_empty() && item.trim() == *item,
                "ess/{}: `{item}` is empty or padded",
                entry.major
            );
            assert!(
                !item.contains('\n'),
                "ess/{}: `{item}` spans lines",
                entry.major
            );
        }
        if let Some(release) = entry.release {
            let parts: Vec<&str> = release.split('.').collect();
            assert!(
                parts.len() == 3
                    && parts
                        .iter()
                        .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit())),
                "ess/{} names `{release}`, not a release",
                entry.major
            );
        }
    }
}

#[test]
fn only_a_suffix_of_the_history_is_unreleased() {
    let first_unreleased = FORMAT_HISTORY
        .iter()
        .position(|entry| entry.release.is_none())
        .unwrap_or(FORMAT_HISTORY.len());
    assert!(
        FORMAT_HISTORY[first_unreleased..]
            .iter()
            .all(|entry| entry.release.is_none()),
        "a released format follows an unreleased one"
    );
}

#[test]
fn ess_38_is_the_newest_and_unreleased_and_ess_37_keeps_its_released_row() {
    assert_eq!(SUPPORTED_FORMATS.last(), Some(&38));
    let last = FORMAT_HISTORY.last().unwrap();
    assert_eq!((last.major, last.release), (38, None));
    let released = FORMAT_HISTORY
        .iter()
        .find(|entry| entry.major == 37)
        .unwrap();
    assert_eq!(released.release, Some("0.93.0"));
    assert!(released.stricter.is_empty());
    assert_eq!(
        released.added,
        ["The next format, open with no construct of its own: a specification declaring it reads and compiles exactly as at the format before it, and the constructs of one release are added to it, each refused as `unsupported_format_version` naming `ess/37` below it."]
    );
}

/// Addressing a row through a member of a struct input, `ess/38`, beyond10x/ess#640: the one gate
/// every member address is put behind.
#[test]
fn the_member_address_gate_opens_at_ess_38() {
    use ess_domain::command::input_path::{addresses_members, below_ess_38};
    use ess_domain::FormatVersion;
    use ess_primitives::error::{ConstructKind, ConstructRef, ValidationCode};

    assert!(!addresses_members(Some(FormatVersion::new(37).unwrap())));
    assert!(!addresses_members(Some(FormatVersion::V22)));
    assert!(addresses_members(Some(FormatVersion::V38)));
    assert!(addresses_members(None));
    let refusal = below_ess_38(
        &ConstructRef::new(ConstructKind::Command, "shop.route.Send")
            .key("outcomes")
            .named("unknown-depot"),
        "`via: input.settings.depots`",
    );
    assert_eq!(refusal.code, ValidationCode::UnsupportedFormatVersion);
    assert_eq!(
        refusal.message,
        "`via: input.settings.depots` names a member of a struct input, which requires specification format ess/38"
    );
    assert_eq!(
        refusal.hint.as_deref(),
        Some("write `format: ess/38` on the source that declares the system")
    );
    assert_eq!(
        refusal.location,
        "command.shop.route.Send.outcomes.unknown-depot"
    );
}

/// The format that carries the one precedence rule (G4, `docs/design/generic-language-features.md`
/// § G4) shipped in 0.63.0 and its row names the rule.
#[test]
fn format_a_shipped_in_0_63_0_and_names_the_rule() {
    let rule = ess_domain::command::precedence::ONE_RULE;
    let entry = FORMAT_HISTORY
        .iter()
        .find(|entry| entry.major == rule.major())
        .expect("the rule's format is supported");
    assert_eq!(
        entry.release,
        Some("0.63.0"),
        "ess/{} shipped in 0.63.0",
        rule.major()
    );
    assert!(
        entry.added.iter().any(|item| item.starts_with(
            "One precedence rule: a branch answers in the order of what it must read"
        )),
        "ess/{} names the rule: {:#?}",
        rule.major(),
        entry.added
    );
}
