//! `prefix:` on a `String` newtype (beyond10x/ess#146, ess/15).
//!
//! A literal prefix, not a pattern: every value starts with it. It composes with `alphabet:` (its
//! characters must be in the alphabet) and with the prefixes of the newtypes it wraps (one has to
//! extend the other). A literal written for such a field that does not start with it is refused.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::types::{NamedType, TypeBody, TypeRef};

fn single(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("msgs.yaml"), raw)]).map_err(|errors| errors.to_string())
}

const HEADER: &str = "format: ess/15\nsystem: demo\nversion: v1\ndomain: demo.msgs\n";

fn channel(extra: &str) -> String {
    format!(
        "{HEADER}types:\n  - name: demo.msgs.Channel\n    kind: newtype\n    of: String\n    \
         prefix: \"/\"\n{extra}"
    )
}

fn declared<'a>(spec: &'a Specification, name: &str) -> &'a NamedType {
    spec.system()
        .types
        .get(&name.parse().unwrap())
        .expect("declared")
}

#[test]
fn issue_146_a_string_newtype_declares_a_prefix() {
    let spec = single(&channel("")).expect("#146's newtype validates");
    let TypeBody::Newtype { prefix, .. } = &declared(&spec, "demo.msgs.Channel").body else {
        panic!("a newtype");
    };
    assert_eq!(prefix.as_deref(), Some("/"));
    assert_eq!(
        spec.system()
            .types
            .effective_prefix(&TypeRef::parse("demo.msgs.Channel").unwrap())
            .as_deref(),
        Some("/")
    );
}

#[test]
fn a_prefix_is_written_back_and_absence_keeps_the_bytes() {
    let spec = single(&channel("")).expect("validates");
    let written = serde_yaml::to_string(declared(&spec, "demo.msgs.Channel")).expect("writes");
    assert!(written.contains("prefix: /"), "{written}");
    let plain = single(&format!(
        "{HEADER}types:\n  - {{name: demo.msgs.Plain, kind: newtype, of: String}}\n"
    ))
    .expect("validates");
    let written = serde_yaml::to_string(declared(&plain, "demo.msgs.Plain")).expect("writes");
    assert!(!written.contains("prefix"), "{written}");
}

#[test]
fn a_prefix_below_ess_15_is_refused_with_the_format_it_needs() {
    let error = single(&channel("").replace("format: ess/15", "format: ess/14"))
        .expect_err("an older reader drops the prefix");
    assert!(error.contains("ess/15"), "{error}");
    assert!(error.contains("prefix"), "{error}");
}

#[test]
fn a_prefix_on_what_is_not_text_is_refused() {
    let error = single(&channel("").replace("of: String", "of: Integer")).expect_err("refused");
    assert!(error.contains("prefix"), "{error}");
    let error =
        single(&channel("").replace("of: String", "of: Optional<String>")).expect_err("refused");
    assert!(error.contains("prefix"), "{error}");
}

#[test]
fn an_empty_prefix_is_refused() {
    let error = single(&channel("").replace("prefix: \"/\"", "prefix: \"\"")).expect_err("refused");
    assert!(error.contains("prefix"), "{error}");
}

#[test]
fn a_prefix_character_outside_the_alphabet_is_refused() {
    let text = channel("").replace("prefix: \"/\"", "prefix: \"/x\"\n    alphabet: \"/abc\"");
    let error = single(&text).expect_err("no value could start with it");
    assert!(error.contains("prefix"), "{error}");
    assert!(error.contains("'x'"), "{error}");
    single(&text.replace("prefix: \"/x\"", "prefix: \"/a\"")).expect("inside the alphabet");
}

#[test]
fn a_prefix_character_outside_a_wrapped_alphabet_is_refused() {
    let text = format!(
        "{HEADER}types:\n  - {{name: demo.msgs.Path, kind: newtype, of: String, alphabet: \"/abc\"}}\n  \
         - {{name: demo.msgs.Channel, kind: newtype, of: demo.msgs.Path, prefix: \"/z\"}}\n"
    );
    let error = single(&text).expect_err("refused");
    assert!(error.contains("'z'"), "{error}");
}

#[test]
fn nested_prefixes_must_extend_one_another() {
    let nested = |inner: &str, outer: &str| {
        format!(
            "{HEADER}types:\n  - {{name: demo.msgs.Path, kind: newtype, of: String, prefix: \"{inner}\"}}\n  \
             - {{name: demo.msgs.Channel, kind: newtype, of: demo.msgs.Path, prefix: \"{outer}\"}}\n"
        )
    };
    let spec = single(&nested("/", "/chat")).expect("the outer prefix extends the inner one");
    assert_eq!(
        spec.system()
            .types
            .effective_prefix(&TypeRef::parse("demo.msgs.Channel").unwrap())
            .as_deref(),
        Some("/chat")
    );
    let spec = single(&nested("/chat", "/")).expect("the inner prefix extends the outer one");
    assert_eq!(
        spec.system()
            .types
            .effective_prefix(&TypeRef::parse("demo.msgs.Channel").unwrap())
            .as_deref(),
        Some("/chat")
    );
    let error = single(&nested("/a", "/b")).expect_err("no value starts with both");
    assert!(error.contains("prefix"), "{error}");
}

#[test]
fn a_prefix_is_not_a_key_of_a_struct_enum_or_union() {
    let error = single(&format!(
        "{HEADER}types:\n  - name: demo.msgs.S\n    kind: struct\n    prefix: \"/\"\n    fields: \
         [{{name: a, type: String}}]\n"
    ))
    .expect_err("refused");
    assert!(error.contains("prefix"), "{error}");
}

const COMMAND: &str = "events:
  - name: demo.msgs.Posted
    fields:
      - {name: channel, type: demo.msgs.Channel}
actors:
  - {name: demo.msgs.Poster, may: [demo.msgs.Post]}
commands:
  - name: demo.msgs.Post
    input:
      - {name: channel, type: demo.msgs.Channel, example: \"/general\"}
    outcomes:
      - name: posted
        emits: [demo.msgs.Posted]
        payload:
          demo.msgs.Posted: {channel: input.channel}
";

#[test]
fn an_example_that_does_not_start_with_the_prefix_is_refused() {
    single(&channel(COMMAND)).expect("`/general` starts with `/`");
    let error = single(&channel(COMMAND).replace("example: \"/general\"", "example: general"))
        .expect_err("refused");
    assert!(error.contains("general"), "{error}");
    assert!(error.contains("prefix"), "{error}");
}

#[test]
fn a_payload_literal_that_does_not_start_with_the_prefix_is_refused() {
    let literal = |value: &str| {
        channel(COMMAND).replace(
            "demo.msgs.Posted: {channel: input.channel}",
            &format!("demo.msgs.Posted: {{channel: \"{value}\"}}"),
        )
    };
    single(&literal("/general")).expect("starts with the prefix");
    let error = single(&literal("channel")).expect_err("no value of the type is `channel`");
    assert!(error.contains("channel"), "{error}");
    assert!(error.contains("prefix"), "{error}");
}
