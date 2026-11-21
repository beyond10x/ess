//! Adversary pass 2 against the fixes of eb18d0e5fb to `ess-conformance-target/1`
//! (`docs/design/conformance-target-process.md`,
//! story:a-conformance-run-drives-a-target-process-in-any-language).
//!
//! Every case drives the public API: a `ProcessTarget` over an in-memory `Peer` that answers canned
//! lines. No process is started.

use std::collections::{BTreeMap, VecDeque};

use ess_conformance::process::{HandshakeSuite, Peer, PeerFault, ProcessTarget};
use ess_conformance::{
    AdmittedSuite, ConformanceTarget, ScenarioContext, ScenarioId, SemanticCommandRequest,
    TargetError,
};
use ess_primitives::facts::Number;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

const TIMEOUT_MS: u64 = 30_000;

fn billing_suite() -> HandshakeSuite {
    static SUITE: std::sync::OnceLock<HandshakeSuite> = std::sync::OnceLock::new();
    SUITE
        .get_or_init(|| {
            let path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../suites/generated/billing/suite.json"
            );
            let admitted =
                AdmittedSuite::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
            HandshakeSuite::of(&admitted)
        })
        .clone()
}

fn context() -> ScenarioContext {
    scenario("billing.invoice.CreateInvoice/outcome/accepted")
}

fn scenario(id: &str) -> ScenarioContext {
    ScenarioContext::new(
        ScenarioId::parse(id).unwrap(),
        CorrelationId::new("billing-c-000001").unwrap(),
    )
}

fn command_request() -> SemanticCommandRequest {
    serde_json::from_str(
        r#"{"command":"billing.invoice.CreateInvoice","actor":null,"caller":null,"input":{},"correlation":"billing-c-000001"}"#,
    )
    .unwrap()
}

/// A peer of canned answers: the handshake of the `n`th start names `names[n]` (the last one
/// repeats), each `execute_command` takes the next line of `commands` with `{id}` replaced by the
/// request's id (an empty result once they run out), and every other operation answers `null`.
struct Canned {
    names: Vec<String>,
    starts: usize,
    commands: VecDeque<String>,
    answer: Option<Vec<u8>>,
}

impl Canned {
    fn new(names: &[&str], commands: &[&str]) -> Self {
        Self {
            names: names.iter().map(|name| (*name).to_owned()).collect(),
            starts: 0,
            commands: commands.iter().map(|line| (*line).to_owned()).collect(),
            answer: None,
        }
    }
}

impl Peer for Canned {
    fn send(&mut self, line: &str) -> Result<(), PeerFault> {
        let request: serde_json::Value = serde_json::from_str(line).unwrap();
        let id = request["id"].as_u64().unwrap();
        let answer = match request["op"].as_str().unwrap() {
            "handshake" => {
                let name = &self.names[self.starts.min(self.names.len() - 1)];
                serde_json::json!({"id": id, "ok": {"speaks": "ess-conformance-target/1",
                    "implementation": {"name": name, "version": "1"}}})
                .to_string()
            }
            "execute_command" => self.commands.pop_front().map_or_else(
                || format!(r#"{{"id":{id},"ok":{{}}}}"#),
                |line| line.replace("{id}", &id.to_string()),
            ),
            _ => format!(r#"{{"id":{id},"ok":null}}"#),
        };
        self.answer = Some(answer.into_bytes());
        Ok(())
    }

    fn receive(&mut self, _limit_ms: u64) -> Result<Vec<u8>, PeerFault> {
        self.answer.take().ok_or(PeerFault::Timeout)
    }

    fn kill(&mut self) {}

    fn respawn(&mut self) -> Result<(), PeerFault> {
        self.starts += 1;
        Ok(())
    }
}

/// Whether a restart after a fault is admitted when the first start names `first` and the
/// restarted one `second`.
fn restart_admitted(first: &str, second: &str) -> bool {
    let target = ProcessTarget::connect(
        Canned::new(&[first, second], &["garbage"]),
        billing_suite(),
        TIMEOUT_MS,
    )
    .unwrap();
    target.begin_scenario(&context()).unwrap();
    target
        .execute_command(command_request())
        .expect_err("a line that is not JSON is a fault");
    target
        .begin_scenario(&scenario("billing.invoice.CreateInvoice/outcome/rejected"))
        .is_ok()
}

// Claim broken: design `docs/design/conformance-target-process.md` ("one report names one
// implementation") and `HandshakeRefusal::OtherImplementation` (`process.rs:404`, checked at
// `process.rs:626`): a restarted target naming another implementation than the first start did is
// refused. eb18d0e5fb stores the handshake's name escaped (`process/wire.rs` `printable`), and the
// escape is not injective: a backslash is kept as it is (`KEPT` in `process/tests.rs`), so ESC and
// the six characters `\u{1b}` both become `\u{1b}`. The restart compares the escaped names
// (`process.rs:626`), so a restart naming another string than the first start is admitted as the
// same implementation, and the report names one string for two different names.
#[test]
fn a_restart_naming_the_escape_text_of_the_first_starts_control_character_is_another_implementation(
) {
    assert!(
        restart_admitted("billing\u{1b}", "billing\u{1b}"),
        "control: the same name twice is the same implementation"
    );
    assert!(
        !restart_admitted("billing", "billing-2"),
        "control: another name is refused"
    );
    assert!(
        !restart_admitted("billing\u{1b}", "billing\\u{1b}"),
        "a restart naming `billing\\u{{1b}}` (backslash, u, braces) after a first start naming \
         `billing` + ESC is admitted as the same implementation"
    );
}

// ---- the single-pass exact reader against the reader it replaced (item 4) -------------------------

/// The reader before eb18d0e5fb (`process/exact/tests.rs` `reference`), restated against the public
/// `Node`: a value, or a refusal's kind and byte.
mod reference {
    use std::collections::BTreeMap;
    use std::fmt;

    use ess_primitives::facts::Number;
    use ess_primitives::node::Node;
    use serde::de::{MapAccess, Visitor};
    use serde_json::value::RawValue;

    const DEPTH: usize = 128;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Bad {
        NotJson(usize),
        Number(usize),
        Shape(usize),
    }

    fn column(error: &serde_json::Error) -> usize {
        error.column().saturating_sub(1)
    }

    pub fn parse(line: &str) -> Result<Node, Bad> {
        if line.trim().is_empty() {
            return Err(Bad::NotJson(0));
        }
        let raw: &RawValue = serde_json::from_str(line).map_err(|e| Bad::NotJson(column(&e)))?;
        build(line, raw, 0)
    }

    fn offset(line: &str, part: &str) -> usize {
        part.as_ptr().addr().saturating_sub(line.as_ptr().addr())
    }

    fn build(line: &str, raw: &RawValue, depth: usize) -> Result<Node, Bad> {
        let text = raw.get().trim();
        let at = offset(line, text);
        if depth > DEPTH {
            return Err(Bad::Shape(at));
        }
        let again = |e: serde_json::Error| Bad::NotJson(at + column(&e));
        Ok(match text.as_bytes().first() {
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
                Node::Map(map)
            }
            Some(b'[') => {
                let items: Vec<&RawValue> = serde_json::from_str(text).map_err(again)?;
                Node::Seq(
                    items
                        .into_iter()
                        .map(|item| build(line, item, depth + 1))
                        .collect::<Result<_, _>>()?,
                )
            }
            Some(b'"') => Node::Text(serde_json::from_str(text).map_err(again)?),
            Some(b'n') => Node::Null,
            Some(b't') => Node::Bool(true),
            Some(b'f') => Node::Bool(false),
            _ => Node::Number(Number::from_json_token(text).ok_or(Bad::Number(at))?),
        })
    }

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

/// The JSON escape of the UTF-16 unit `hex`: a backslash, `u` and the four digits.
fn u(hex: &str) -> String {
    format!("{}u{hex}", '\\')
}

/// A fixed-seed generator of JSON value texts that reach every branch of both readers.
struct Gen(u64);

impl Gen {
    fn next(&mut self) -> u64 {
        // SplitMix64.
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(n).unwrap()).unwrap()
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }

    fn space(&mut self) -> &'static str {
        ["", "", "", " ", "\t", "\r", "  \t "][self.below(7)]
    }

    fn string(&mut self) -> String {
        let mut pieces: Vec<String> = [
            "a",
            "k",
            "id",
            "ok",
            "\u{e9}",
            "\u{1d11e}",
            "\u{7f}",
            "\u{85}",
            "\u{2028}",
            "x y",
            ",",
            "]",
            "}",
            ":",
            "[",
            "{",
        ]
        .iter()
        .map(|piece| (*piece).to_owned())
        .collect();
        for escape in ["\"", "\\", "/", "b", "f", "n", "r", "t"] {
            pieces.push(format!("\\{escape}"));
        }
        for hex in ["0041", "00e9", "0000", "001b", "2028"] {
            pieces.push(u(hex));
        }
        pieces.push(format!("{}{}", u("d83d"), u("de00")));
        let rare = [u("d800"), u("dc00"), format!("{}x", u("d800")), u("d83d")];
        let mut out = String::from("\"");
        if self.below(200) == 0 {
            out.push_str(&"long".repeat(2_500 + self.below(500)));
        }
        for _ in 0..self.below(5) {
            if self.below(60) == 0 {
                out.push_str(&rare[self.below(rare.len())]);
            } else {
                out.push_str(&pieces[self.below(pieces.len())]);
            }
        }
        out.push('"');
        out
    }

    fn number(&mut self) -> String {
        const TOKENS: &[&str] = &[
            "0",
            "-0",
            "1",
            "-1",
            "0.0",
            "-0.0",
            "1.5",
            "1e2",
            "1E+2",
            "1e-2",
            "2.5e-7",
            "9007199254740993",
            "9223372036854775808",
            "-9223372036854775809",
            "18446744073709551616",
            "123456789012345678901234567890",
            "0.1000000000000000000001",
            "1e308",
            "1.7976931348623157e308",
            "5e-324",
        ];
        const REFUSED: &[&str] = &["1e400", "-1e400", "1e999", "-0.1e-999999"];
        if self.below(40) == 0 {
            self.pick(REFUSED).to_owned()
        } else {
            self.pick(TOKENS).to_owned()
        }
    }

    fn key(&mut self) -> String {
        let keys = [
            "a".to_owned(),
            "b".to_owned(),
            "k".to_owned(),
            u("0061"),
            String::new(),
            "a\\\"b".to_owned(),
            "c\\\\".to_owned(),
            u("00e9"),
            "\u{e9}".to_owned(),
        ];
        let rare = [u("d800"), u("dc00")];
        let name = if self.below(80) == 0 {
            rare[self.below(rare.len())].clone()
        } else {
            keys[self.below(keys.len())].clone()
        };
        format!("\"{name}\"")
    }

    fn value(&mut self, depth: usize) -> String {
        let roll = if depth > 6 {
            self.below(5)
        } else {
            self.below(9)
        };
        match roll {
            0 => self.string(),
            1 => self.number(),
            2 => ["null", "true", "false"][self.below(3)].to_owned(),
            3 | 4 if depth > 6 => self.number(),
            3 | 5 | 6 => {
                let mut out = String::from("[");
                out.push_str(self.space());
                for index in 0..self.below(4) {
                    if index > 0 {
                        out.push(',');
                    }
                    out.push_str(self.space());
                    out.push_str(&self.value(depth + 1));
                    out.push_str(self.space());
                }
                out.push(']');
                out
            }
            _ => {
                let mut out = String::from("{");
                out.push_str(self.space());
                for index in 0..self.below(4) {
                    if index > 0 {
                        out.push(',');
                    }
                    out.push_str(self.space());
                    out.push_str(&self.key());
                    out.push_str(self.space());
                    out.push(':');
                    out.push_str(self.space());
                    out.push_str(&self.value(depth + 1));
                    out.push_str(self.space());
                }
                out.push('}');
                out
            }
        }
    }

    /// Nesting around a generated value, straddling the reader's limit: the answer line puts the
    /// value three levels down (`ok`, `response`, `v`).
    fn deep(&mut self) -> String {
        let levels = 120 + self.below(14);
        let inner = self.value(7);
        if self.below(2) == 0 {
            format!("{}{inner}{}", "[".repeat(levels), "]".repeat(levels))
        } else {
            let key = self.key();
            format!(
                "{}{inner}{}",
                format!("{{{key}:").repeat(levels),
                "}".repeat(levels)
            )
        }
    }

    fn line(&mut self) -> String {
        let value = if self.below(25) == 0 {
            self.deep()
        } else {
            self.value(0)
        };
        let mut line = format!(
            r#"{{"id":2,{}"ok":{{"response":{{"v":{}{value}{}}}}}}}"#,
            self.space(),
            self.space(),
            self.space()
        );
        if self.below(100) == 0 {
            line.push_str(self.pick(&["x", "]", " 1", ",", "}"]));
        }
        line
    }
}

/// What the process target makes of an `execute_command` answer `line`: the response's `v`, or the
/// fault's text.
fn read_through_the_process_target(line: &str) -> Result<Node, String> {
    let target = ProcessTarget::connect(
        Canned::new(&["canned"], &[line]),
        billing_suite(),
        TIMEOUT_MS,
    )
    .unwrap();
    target.begin_scenario(&context()).unwrap();
    match target.execute_command(command_request()) {
        Ok(result) => Ok(result
            .response
            .expect("the answer holds a response")
            .remove("v")
            .expect("the response holds v")),
        Err(TargetError::Unavailable { detail, .. }) => Err(detail),
        Err(other) => panic!("not a value and not a fault: {other:?}"),
    }
}

/// What the reader before eb18d0e5fb made of it, in the same terms.
fn read_by_the_replaced_reader(line: &str) -> Result<Node, String> {
    let fault =
        |what: &str, at: usize| format!("answer to 2 (execute_command) {what} at byte {at}");
    match reference::parse(line) {
        Ok(Node::Map(mut answer)) => {
            let Some(Node::Map(mut ok)) = answer.remove("ok") else {
                panic!("the generated envelope holds ok")
            };
            let Some(Node::Map(mut response)) = ok.remove("response") else {
                panic!("the generated envelope holds a response")
            };
            Ok(response
                .remove("v")
                .expect("the generated response holds v"))
        }
        Ok(other) => panic!("the generated envelope is an object: {other:?}"),
        Err(reference::Bad::NotJson(at)) => Err(fault("is not JSON", at)),
        Err(reference::Bad::Number(at)) => Err(fault("holds a number outside the domain", at)),
        Err(reference::Bad::Shape(at)) => Err(fault("does not match its operation", at)),
    }
}

// Claim under test: eb18d0e5fb's commit message ("It is two linear passes now, compared with the old
// reader on 112 lines") and `process/exact.rs` `Reader`: "It refuses what the reader before it
// refused, at the same byte and in the same order". 3,000 generated lines and a few fixed ones,
// through the public answer path, against the old reader restated here.
#[test]
fn the_single_pass_reader_reads_generated_answers_as_the_reader_it_replaced() {
    let mut generator = Gen(0x5eed_2026_1121);
    let mut lines: Vec<String> = (0..3_000).map(|_| generator.line()).collect();
    for inner in [
        "[".repeat(100_000),
        format!("{}{}", "[".repeat(100_000), "]".repeat(100_000)),
        format!("{}1e400{}", "[".repeat(124), "]".repeat(124)),
        format!("{}1e400{}", "[".repeat(126), "]".repeat(126)),
        format!(r#"[{},"\ud800"]"#, "[".repeat(200) + &"]".repeat(200)),
        format!(
            r#"{{"a":{},"\ud800":1}}"#,
            "[".repeat(200) + &"]".repeat(200)
        ),
        format!(r#"{{"a":{},"a":1}}"#, "[".repeat(200) + &"]".repeat(200)),
    ] {
        lines.push(format!(r#"{{"id":2,"ok":{{"response":{{"v":{inner}}}}}}}"#));
    }
    let mut disagreements = Vec::new();
    let mut kinds = BTreeMap::<&str, usize>::new();
    for line in &lines {
        let replaced = read_by_the_replaced_reader(line);
        let read = read_through_the_process_target(line);
        *kinds
            .entry(match &replaced {
                Ok(_) => "value",
                Err(text) if text.contains("not JSON") => "not-json",
                Err(text) if text.contains("outside the domain") => "number",
                Err(_) => "shape",
            })
            .or_default() += 1;
        if read != replaced {
            let shown: String = line.chars().take(300).collect();
            disagreements.push(format!(
                "line {shown:?}…\n  replaced reader: {replaced:?}\n  process target:  {read:?}"
            ));
        }
    }
    assert!(
        kinds.len() == 4 && kinds.values().all(|count| *count >= 20),
        "the corpus reaches every answer: {kinds:?}"
    );
    assert!(
        disagreements.is_empty(),
        "{} of {} lines read differently ({kinds:?}):\n{}",
        disagreements.len(),
        lines.len(),
        disagreements
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    // Numbers keep their spelling through both readers.
    let line = r#"{"id":2,"ok":{"response":{"v":[1E+2,-0.0,9007199254740993]}}}"#;
    let Ok(Node::Seq(items)) = read_through_the_process_target(line) else {
        panic!("read")
    };
    let spelled: Vec<String> = items
        .iter()
        .map(|item| match item {
            Node::Number(number) => number.exact_text(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        spelled,
        ["1E+2", "-0.0", "9007199254740993"]
            .iter()
            .map(|token| Number::from_json_token(token).unwrap().exact_text())
            .collect::<Vec<_>>()
    );
}
