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
