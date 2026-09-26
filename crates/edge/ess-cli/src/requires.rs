//! `requires:` in `ess-inputs/2`: the `ess` release a specification is maintained with (ess#106).
//!
//! `docs/design/specification-requires-release.md` is the binding table. Older than required
//! refuses; newer warns once per process and continues unless `--strict-requires`.

use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Once,
    },
};

use anyhow::{bail, Result};

/// This `ess`.
const THIS: &str = env!("CARGO_PKG_VERSION");

static STRICT: AtomicBool = AtomicBool::new(false);
static WARNED: Once = Once::new();

/// `--strict-requires`, set once from the parsed command line.
pub(crate) fn set_strict(strict: bool) {
    STRICT.store(strict, Ordering::Relaxed);
}

/// What a `requires` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Required {
    /// `ess X.Y.Z`: exactly that release.
    Exact(u64, u64, u64),
    /// `ess X.Y`: any `X.Y.*`.
    Line(u64, u64),
}

/// One decimal component: digits only, no sign, no leading zero.
fn component(text: &str) -> Option<u64> {
    if text.is_empty()
        || !text.bytes().all(|b| b.is_ascii_digit())
        || (text.len() > 1 && text.starts_with('0'))
    {
        return None;
    }
    text.parse().ok()
}

fn parse(text: &str) -> Option<Required> {
    let version = text.strip_prefix("ess ")?;
    let parts = version
        .split('.')
        .map(component)
        .collect::<Option<Vec<_>>>()?;
    match parts[..] {
        [major, minor] => Some(Required::Line(major, minor)),
        [major, minor, patch] => Some(Required::Exact(major, minor, patch)),
        _ => None,
    }
}

/// This release as (major, minor, patch); a pre-release suffix is not part of the comparison.
fn this() -> (u64, u64, u64) {
    let mut parts = THIS
        .split(['.', '-', '+'])
        .map(|part| part.parse::<u64>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

/// Decide `requires` from `manifest` against this release.
pub(crate) fn check(manifest: &Path, requires: &str) -> Result<()> {
    let Some(pin) = parse(requires) else {
        bail!(
            "{}: `requires: {requires}` is not `ess X.Y.Z` (an exact release) or `ess X.Y` (a \
             minor line)",
            manifest.display()
        );
    };
    let (major, minor, patch) = this();
    let ordering = match pin {
        Required::Exact(x, y, z) => (major, minor, patch).cmp(&(x, y, z)),
        Required::Line(x, y) => (major, minor).cmp(&(x, y)),
    };
    let stated = format!(
        "{} requires {requires} and this is ess {THIS}",
        manifest.display()
    );
    match ordering {
        std::cmp::Ordering::Equal => Ok(()),
        std::cmp::Ordering::Less => bail!(
            "{stated}, which is older; install the required release with `b10x upgrade`, or \
             `/ess:upgrade` in an agent session"
        ),
        std::cmp::Ordering::Greater if STRICT.load(Ordering::Relaxed) => {
            bail!("{stated}, which is newer, and --strict-requires refuses a newer release")
        }
        std::cmp::Ordering::Greater => {
            WARNED.call_once(|| {
                eprintln!(
                    "warning: {stated}, which is newer; continuing (--strict-requires refuses)"
                );
            });
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_forms_parse_and_everything_else_does_not() {
        assert_eq!(parse("ess 0.32.1"), Some(Required::Exact(0, 32, 1)));
        assert_eq!(parse("ess 0.32"), Some(Required::Line(0, 32)));
        for malformed in [
            "ess 0",
            "ess 0.32.1.4",
            "ess 00.32",
            "ess 0.32.1-rc1",
            "ess +0.32",
            "ess  0.32",
            "ess latest",
            "aep 0.32",
            "0.32",
            "ess 0..1",
        ] {
            assert_eq!(parse(malformed), None, "{malformed}");
        }
    }
}
