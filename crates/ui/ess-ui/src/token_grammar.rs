//! The grammars a token's name and value are written in, shared by `ess ui check` (which files a
//! finding at the key path) and the React generator (which refuses the token, since `generate`
//! does not run the check): the color, length and weight grammars of `Tokens.grammar` in the
//! schema, and what a name or a value may hold to be written into one CSS custom property.

/// Why `value` is not a color of `Tokens.grammar.color`, or `None` when it is one.
pub fn color_problem(text: &str) -> Option<String> {
    if let Some(digits) = text.strip_prefix('#') {
        let hex = digits.chars().all(|c| c.is_ascii_hexdigit());
        return (!(hex && [3, 6, 8].contains(&digits.len()))).then(|| {
            format!("`{text}` is not a color: a hex color is `#rgb`, `#rrggbb` or `#rrggbbaa`")
        });
    }
    if let Some(inner) = text
        .strip_prefix("rgb(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split_whitespace().collect();
        let channel =
            |part: &str| part.parse::<u8>().is_ok() && part.chars().all(|c| c.is_ascii_digit());
        let alpha = |part: &str| {
            is_decimal(part)
                && part
                    .parse::<f64>()
                    .is_ok_and(|alpha| (0.0..=1.0).contains(&alpha))
        };
        let fits = matches!(parts.as_slice(), [r, g, b, "/", a]
            if channel(r) && channel(g) && channel(b) && alpha(a));
        return (!fits).then(|| {
            format!(
                "`{text}` is not a color: `rgb(r g b / a)` takes r, g and b from 0 to 255 and a \
                 from 0 to 1"
            )
        });
    }
    let shown = if text.is_empty() {
        "an empty value (an unquoted `#` starts a YAML comment; quote the color)".to_owned()
    } else {
        format!("`{text}`")
    };
    Some(format!(
        "{shown} is not a color: a color is `#rgb`, `#rrggbb`, `#rrggbbaa` or `rgb(r g b / a)`, \
         never `hsl()` or a color name"
    ))
}

/// A decimal: digits, optionally with a fraction (`1`, `0.25`, `.5`).
pub fn is_decimal(text: &str) -> bool {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    let digits = |part: &str| part.chars().all(|c| c.is_ascii_digit());
    let has_fraction = text.contains('.');
    digits(whole)
        && digits(fraction)
        && (!whole.is_empty() || !fraction.is_empty())
        && (!has_fraction || !fraction.is_empty())
}

/// Why `value` is not a length of `Tokens.grammar.length`, or `None` when it is one.
pub fn length_problem(text: &str) -> Option<String> {
    if text == "0" {
        return None;
    }
    let fits = ["rem", "px", "em"]
        .iter()
        .any(|unit| text.strip_suffix(unit).is_some_and(is_decimal));
    (!fits).then(|| {
        format!("`{text}` is not a length: a length is `0`, or a decimal with `px`, `rem` or `em`")
    })
}

/// Why `value` is not a weight of `Tokens.grammar.weight`, or `None` when it is one.
pub fn weight_problem(text: &str) -> Option<String> {
    let fits = text.chars().all(|c| c.is_ascii_digit())
        && text
            .parse::<u32>()
            .is_ok_and(|weight| (100..=900).contains(&weight) && weight % 100 == 0);
    (!fits).then(|| format!("`{text}` is not a weight: a weight is 100 to 900 in steps of 100"))
}

/// Why `name` cannot be a token name, or `None` when it can: a name is written into a CSS custom
/// property name (`--ui-color-<name>`), so it is `^[A-Za-z0-9_-]+$` (`token_name` in the schema).
pub fn name_problem(name: &str) -> Option<String> {
    let fits = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    (!fits).then(|| {
        format!(
            "`{}` is not a token name: a name holds only letters, digits, `_` and `-`",
            name.escape_debug()
        )
    })
}

/// Why `text` cannot be written into a CSS declaration, or `None` when it can: it holds none of
/// `; { } < > \`, no comment and no control character, and every quote and bracket it opens it
/// closes, since an open one carries the declaration past its own `;` into the next ones.
pub fn css_unsafe(text: &str) -> Option<String> {
    let bad = text
        .chars()
        .any(|c| c.is_control() || [';', '{', '}', '<', '>', '\\'].contains(&c))
        || text.contains("/*")
        || text.contains("*/");
    if bad {
        return Some(format!(
            "`{}` is not a value: it holds a character that would end or open a CSS rule (`; {{ }} < > \\`, a comment or a control character)",
            text.escape_debug()
        ));
    }
    unbalanced(text).then(|| {
        format!(
            "`{}` is not a value: it opens a quote or a bracket it does not close",
            text.escape_debug()
        )
    })
}

/// Whether `text` leaves a quote or a bracket open, or closes one it did not open.
fn unbalanced(text: &str) -> bool {
    let mut quote: Option<char> = None;
    let mut open: Vec<char> = Vec::new();
    for c in text.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '(' | '[') => open.push(c),
            (None, ')') => {
                if open.pop() != Some('(') {
                    return true;
                }
            }
            (None, ']') => {
                if open.pop() != Some('[') {
                    return true;
                }
            }
            _ => {}
        }
    }
    quote.is_some() || !open.is_empty()
}

/// Why `text` cannot be written as the value of a color token: the color grammar, then
/// [`css_unsafe`].
pub fn color_value_problem(text: &str) -> Option<String> {
    color_problem(text).or_else(|| css_unsafe(text))
}

/// Why `text` cannot be written as the value of a space or radius token.
pub fn length_value_problem(text: &str) -> Option<String> {
    length_problem(text).or_else(|| css_unsafe(text))
}

/// Why `text` cannot be written as a type size.
pub fn size_value_problem(text: &str) -> Option<String> {
    length_value_problem(text)
}

/// Why `text` cannot be written as a type weight.
pub fn weight_value_problem(text: &str) -> Option<String> {
    weight_problem(text).or_else(|| css_unsafe(text))
}
