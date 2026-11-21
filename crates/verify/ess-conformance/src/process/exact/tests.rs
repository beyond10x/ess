//! The exact reader reads a line in one pass, and reads it exactly as the reader it replaced did.
//!
//! [`reference`] is that reader, kept verbatim: it re-parsed the text of every object and array at
//! every level below it, so a line nested `d` deep was gone over about `d` times. Every line of
//! [`corpus`] is read by both, and each must answer the same value or the same refusal at the same
//! byte.

use std::fmt;

use serde::de::{MapAccess, Visitor};

use super::*;

/// [`parse`], and the bytes it went over.
fn counted(line: &str) -> (Result<Value, Bad>, usize) {
    VISITED.with(|visited| visited.set(0));
    let read = parse(line);
    (read, VISITED.with(std::cell::Cell::get))
}

/// The reader before the single pass, verbatim but for its name.
mod reference {
    use super::*;

    pub fn parse(line: &str) -> Result<Value, Bad> {
        if line.trim().is_empty() {
            return Err(Bad::NotJson(0));
        }
        let raw: &RawValue = serde_json::from_str(line).map_err(|e| Bad::NotJson(column(&e)))?;
        build(line, raw, 0)
    }

    fn offset(line: &str, part: &str) -> usize {
        part.as_ptr().addr().saturating_sub(line.as_ptr().addr())
    }

    fn build(line: &str, raw: &RawValue, depth: usize) -> Result<Value, Bad> {
        let text = raw.get().trim();
        let at = offset(line, text);
        if depth > DEPTH {
            return Err(Bad::Shape(at));
        }
        let again = |e: serde_json::Error| Bad::NotJson(at + column(&e));
        let kind = match text.as_bytes().first() {
            Some(b'{') => {
                let Entries(entries) = serde_json::from_str(text).map_err(again)?;
                let mut map = BTreeMap::new();
                for (name, item) in entries {
                    if map.contains_key(&name) {
                        return Err(Bad::Shape(offset(line, item.get())));
                    }
                    let item = build(line, item, depth + 1)?;
                    map.insert(name, item);
                }
                Kind::Map(map)
            }
            Some(b'[') => {
                let items: Vec<&RawValue> = serde_json::from_str(text).map_err(again)?;
                Kind::Seq(
                    items
                        .into_iter()
                        .map(|item| build(line, item, depth + 1))
                        .collect::<Result<_, _>>()?,
                )
            }
            Some(b'"') => Kind::Text(serde_json::from_str(text).map_err(again)?),
            Some(b'n') => Kind::Null,
            Some(b't') => Kind::Bool(true),
            Some(b'f') => Kind::Bool(false),
            _ => Kind::Number(Number::from_json_token(text).ok_or(Bad::Number(at))?),
        };
        Ok(Value { at, kind })
    }

    /// An object's members in the order the line writes them, repeated names included.
    struct Entries<'a>(Vec<(String, &'a RawValue)>);

    impl<'de: 'a, 'a> serde::Deserialize<'de> for Entries<'a> {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct EntryVisitor<'a>(std::marker::PhantomData<&'a ()>);
            impl<'de: 'a, 'a> Visitor<'de> for EntryVisitor<'a> {
                type Value = Entries<'a>;
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("a JSON object")
                }
                fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                    let mut entries = Vec::new();
                    while let Some(entry) = map.next_entry::<String, &'a RawValue>()? {
                        entries.push(entry);
                    }
                    Ok(Entries(entries))
                }
            }
            deserializer.deserialize_map(EntryVisitor(std::marker::PhantomData))
        }
    }
}

/// `levels` arrays around `inner`.
fn nested(levels: usize, inner: &str) -> String {
    format!("{}{inner}{}", "[".repeat(levels), "]".repeat(levels))
}

/// `levels` objects of one member `k` around `inner`.
fn nested_objects(levels: usize, inner: &str) -> String {
    format!("{}{inner}{}", r#"{"k":"#.repeat(levels), "}".repeat(levels))
}

/// Lines that reach every branch of the reader: each kind of value, whitespace everywhere it may
/// stand, escapes, every refusal, and refusals racing each other for which one is reported.
fn corpus() -> Vec<String> {
    let mut lines: Vec<String> = [
        // Not JSON, or blank: refused before the tree is read.
        "",
        "   ",
        "{",
        "{\"a\":1",
        "{\"a\":1}x",
        "[1,]",
        "{\"a\" 1}",
        "nul",
        "\"\\x\"",
        "\"\u{1}\"",
        // Scalars at the top, with whitespace on both sides.
        "0",
        " -0 ",
        "\t1e2\r",
        "1E+2",
        "2.5e-7",
        "9007199254740993",
        "0.1000000000000000000001",
        "123456789012345678901234567890",
        "true",
        "false",
        "null",
        "\"\"",
        r#""a\"b\\c\/d\b\f\n\r\t\u0041\ud83d\ude00é""#,
        r#""\\""#,
        // Numbers the domain refuses, at the top and nested.
        "1e400",
        "-1e400",
        "[1,2,1e400]",
        r#"{"a":{"b":[0,{"c":1e999}]}}"#,
        // Containers, empty and not, with whitespace inside.
        "{}",
        "[]",
        " { } ",
        "[ [ ] , { } ]",
        r#"{ "a" : [ 1 , 2.5 , null , true , false , "x" ] , "b" : { "c" : { } } }"#,
        "{\n\"a\"\t:\r1 }",
        r#"{"a\"b":1,"c\\":[{"\u0041":"\u00e9"}]}"#,
        r#"{"":0}"#,
        // Repeated names, at every depth, and where the repeat is.
        r#"{"id":2,"ok":{"rows":[],"rows":[]}}"#,
        r#"{"a":1,"a":2}"#,
        r#"{"a":1, "a" :  2}"#,
        r#"[{"x":1},{"y":{"z":1,"z":1}}]"#,
        r#"{"a":1,"\u0061":2}"#,
        // Lone surrogates: valid JSON, refused only when the string is decoded.
        r#""\ud800""#,
        r#"{"a":"\udc00"}"#,
        r#"{"\ud800":1}"#,
        r#"[{"x":1},{"\udc00":2}]"#,
        r#"{"a":1,"b":{"c":"\ud800x"}}"#,
        // Which refusal wins inside one object: every key of an object is decoded before any of
        // its members is read, then members in order, a repeat before its value.
        r#"{"a":[1e400],"\ud800":1}"#,
        r#"{"a":[1e400],"a":1}"#,
        r#"{"a":1,"a":[1e400]}"#,
        r#"{"a":"\ud800","\udc00":1}"#,
        r#"{"a":"\ud800","a":1}"#,
        r#"{"a":1,"a":1,"\ud800":1}"#,
        r#"{"a":{"\ud800":1},"\udc00":2}"#,
        r#"{"a":{"b":1,"b":2},"\udc00":2}"#,
        r#"{"a":{"b":1e400},"a":2}"#,
        r#"[[1e400],{"\ud800":1}]"#,
        r#"[{"a":1,"a":2},1e400]"#,
        r#"{"a":[{"b":[1,{"c":1,"c":2}]}],"d":"\ud800"}"#,
        r#"{"a":[{"b":[1,{"c":1,"c":2}]}],"d":1,"d":2}"#,
        r#"{"x":"\\","y":"\"","x":0}"#,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    // The nesting limit, by arrays and by objects, at it and past it, and past it with a refusal
    // that comes earlier or later.
    for levels in [127, 128, 129, 130, 200] {
        lines.push(nested(levels, ""));
        lines.push(nested(levels, "1"));
        lines.push(nested(levels, "1e400"));
        lines.push(nested_objects(levels, "1"));
        lines.push(nested_objects(levels, r#"{"a":1,"a":2}"#));
        lines.push(format!(r#"{{"k":{},"\ud800":1}}"#, nested(levels, "1")));
        lines.push(format!(r#"{{"k":{},"k":1}}"#, nested(levels, "1")));
        lines.push(format!("[{},1e400]", nested(levels, "1")));
        lines.push(format!("[1e400,{}]", nested(levels, "1")));
    }
    lines
}

#[test]
fn every_line_reads_as_the_reader_it_replaced_read_it() {
    let lines = corpus();
    for line in &lines {
        assert_eq!(parse(line), reference::parse(line), "line: {line}");
    }
    let refused = lines.iter().filter(|line| parse(line).is_err()).count();
    assert!(
        refused > 0 && refused < lines.len(),
        "the corpus holds both answers"
    );
}

#[test]
fn a_line_is_gone_over_once_however_deep_it_nests() {
    // At the deepest a line may nest, around a long string, a long run of numbers and a repeat.
    let numbers = vec!["1.5"; 20_000].join(",");
    let inner = format!(
        r#"{{"k":"{}","n":[{numbers}],"m":{{"x":null,"y":true}}}}"#,
        "x".repeat(100_000)
    );
    for line in [
        nested(DEPTH, &inner),
        nested_objects(DEPTH, &inner),
        // Past the limit, and a megabyte deep: refused, still in one pass.
        nested(1 << 19, ""),
        format!(
            r#"{{"k":{},"z":"{}"}}"#,
            nested(1 << 19, ""),
            "z".repeat(1000)
        ),
        // A refusal deep down, and a long tail after it that is still gone over to its end.
        format!(
            r#"[{},"{}"]"#,
            nested(DEPTH - 2, "1e400"),
            "t".repeat(100_000)
        ),
    ] {
        let (read, visited) = counted(&line);
        assert_eq!(read, reference::parse(&line));
        assert!(
            visited <= 2 * line.len(),
            "a line of {} bytes nested deep was gone over for {visited} bytes",
            line.len()
        );
    }
}

#[test]
fn the_nesting_limit_is_128_levels_below_the_line() {
    assert_eq!(DEPTH, 128);
    assert!(parse(&nested(DEPTH + 1, "")).is_ok());
    assert!(parse(&nested(DEPTH, "1")).is_ok());
    // The value at depth 129 is refused at its own first byte.
    assert_eq!(parse(&nested(DEPTH + 1, "1")), Err(Bad::Shape(DEPTH + 1)));
    assert_eq!(parse(&nested(DEPTH + 2, "")), Err(Bad::Shape(DEPTH + 1)));
}
