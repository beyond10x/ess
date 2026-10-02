//! Numbers in expected text, compared by value rather than by spelling.
//!
//! The terminal prints an integer as its digits (`1840`); the generated React project groups
//! them by the browser's locale (`1,840`, `1 840`). A `text` or `not_text` check therefore reads a
//! number of four or more digits with or without a grouping separator between its groups of
//! three: a comma, a no-break space or a narrow no-break space. A full stop is not one, because it
//! is also a decimal point.

/// The separators a number's digit groups may be joined by.
const SEPARATORS: [char; 3] = [',', '\u{a0}', '\u{202f}'];

/// The same separators as a regular-expression class, spelled with `\u` escapes.
const CLASS: &str = "[,\\u00a0\\u202f]";

/// `text` with every grouping separator between digit groups removed: `1,840 EUR` → `1840 EUR`.
pub(crate) fn ungrouped(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (at, character) in chars.iter().enumerate() {
        let grouping = SEPARATORS.contains(character)
            && at > 0
            && chars[at - 1].is_ascii_digit()
            && chars.len() > at + 3
            && chars[at + 1..=at + 3].iter().all(char::is_ascii_digit)
            && chars.get(at + 4).is_none_or(|next| !next.is_ascii_digit());
        if !grouping {
            out.push(*character);
        }
    }
    out
}

/// A regular expression matching `text` with each of its numbers of four or more digits spelled
/// with or without grouping, or `None` when it holds no such number.
pub(crate) fn pattern(text: &str) -> Option<String> {
    let text = ungrouped(text);
    let mut out = String::new();
    let mut grouped = false;
    let mut digits = String::new();
    let flush = |digits: &mut String, out: &mut String, grouped: &mut bool| {
        if digits.len() >= 4 {
            *grouped = true;
            let head = digits.len() % 3;
            let mut groups: Vec<&str> = Vec::new();
            if head > 0 {
                groups.push(&digits[..head]);
            }
            let mut at = head;
            while at < digits.len() {
                groups.push(&digits[at..at + 3]);
                at += 3;
            }
            out.push_str(&groups.join(&format!("{CLASS}?")));
        } else {
            out.push_str(digits);
        }
        digits.clear();
    };
    for character in text.chars() {
        if character.is_ascii_digit() {
            digits.push(character);
            continue;
        }
        flush(&mut digits, &mut out, &mut grouped);
        if "\\^$.|?*+()[]{}/".contains(character) {
            out.push('\\');
        }
        out.push(character);
    }
    flush(&mut digits, &mut out, &mut grouped);
    grouped.then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouping_is_removed_only_between_digit_groups() {
        assert_eq!(ungrouped("1,840 EUR"), "1840 EUR");
        assert_eq!(ungrouped("1,234,567"), "1234567");
        assert_eq!(ungrouped("1\u{a0}840"), "1840");
        assert_eq!(ungrouped("a, b, 1,84"), "a, b, 1,84");
        assert_eq!(ungrouped("1,8400"), "1,8400");
        assert_eq!(ungrouped("1.840"), "1.840");
    }

    #[test]
    fn a_pattern_is_made_only_for_long_numbers() {
        assert_eq!(pattern("Cedar 12"), None);
        assert_eq!(
            pattern("1234567 (x)").as_deref(),
            Some("1[,\\u00a0\\u202f]?234[,\\u00a0\\u202f]?567 \\(x\\)")
        );
    }
}
