use std::collections::BTreeSet;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::schema::ModelTypes;
use schema_contract::{
    bundle::{import, Dialect},
    realize::normalize::{Plan, Root},
};
use serde_json::{json, Value};

pub const PATTERN: &str = "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$";

pub fn fixture() -> (Plan, Value) {
    let source = "format: ess/1\nsystem: sample\nversion: v1\ndomains: [sample.patterns]\ndomain: sample.patterns\ntypes:\n  - name: sample.patterns.Blob\n    kind: newtype\n    of: Bytes\n  - name: sample.patterns.Lookup\n    kind: newtype\n    of: Map<Bytes, Bytes>\n";
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, source.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(source).unwrap())])
            .unwrap();
    let ir = compile(&specification, &sources).unwrap();
    let model = ModelTypes::select(
        &ir,
        &BTreeSet::from([
            "sample.patterns.Blob".to_owned(),
            "sample.patterns.Lookup".to_owned(),
        ]),
    )
    .unwrap();
    assert_eq!(
        model.definitions()["sample.patterns.Blob"]["pattern"],
        PATTERN
    );
    assert_eq!(
        model.definitions()["sample.patterns.Lookup"]["propertyNames"]["pattern"],
        PATTERN
    );
    let source = json!({"components":{"schemas":{"Encoded":{"type":"string","contentEncoding":"base64","pattern":PATTERN}}}});
    let bundle = import(
        &source.to_string(),
        &BTreeSet::from(["Encoded".to_owned()]),
        Dialect::Draft202012,
    )
    .unwrap();
    let stage = |root: Root| json!({"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}});
    let recipe = json!({"format":"ess-normalization/3","branches":{
        "bundle":[stage(Root::pin(&bundle,"Encoded").unwrap())],
        "model":[stage(Root::pin_model(&model,"sample.patterns.Blob").unwrap())],
        "model_keys":[stage(Root::pin_model(&model,"sample.patterns.Lookup").unwrap())]
    }});
    let plan =
        Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[bundle], &[model])
            .unwrap();
    let mut cases = Vec::new();
    for (text, accepted) in corpus() {
        for branch in ["bundle", "model", "model_keys"] {
            let value = if branch == "model_keys" {
                json!({text.as_str(): "AA=="})
            } else {
                json!(text)
            };
            let input = value.to_string();
            let actual = plan.run_json(branch, &input);
            if accepted {
                assert_eq!(actual.unwrap(), value, "{branch}: {text:?}");
                cases.push(json!({"branch":branch,"input":input,"value":value}));
            } else {
                let errors = actual.unwrap_err().0;
                assert!(errors.iter().all(|error| error.rule == "schema_validation"));
                cases.push(json!({"branch":branch,"input":input,"errors":errors}));
            }
        }
    }
    eprintln!(
        "base64 reference corpus: executed {} strings through bundle values, model values and model keys, {} cases",
        cases.len() / 3,
        cases.len()
    );
    (plan, json!(cases))
}

fn corpus() -> Vec<(String, bool)> {
    let mut values = vec![
        ("", true),
        ("A", false),
        ("AA", false),
        ("AAA", false),
        ("AAAA", true),
        ("AA==", true),
        ("AB==", true),
        ("AAA=", true),
        ("AAB=", true),
        ("AAAAAA==", true),
        ("AAAAAAA=", true),
        ("AAAAAAAA", true),
        ("AAAAA===", false),
        ("=AAA", false),
        ("A=AA", false),
        ("AA=A", false),
        ("AA===", false),
        ("AAA==", false),
        ("AAAA=", false),
        ("AAAA====", false),
        ("====", false),
        ("-___", false),
        ("+///", true),
        ("AZ==", true),
        ("AA/=", true),
    ]
    .into_iter()
    .map(|(text, valid)| (text.to_owned(), valid))
    .collect::<Vec<_>>();
    for byte in 0_u8..=127 {
        let alphabet = byte.is_ascii_alphanumeric() || b"+/".contains(&byte);
        values.push((char::from(byte).to_string().repeat(4), alphabet));
        for position in 0..4 {
            let mut text = vec![b'A'; 4];
            text[position] = byte;
            values.push((
                String::from_utf8(text).unwrap(),
                alphabet || (byte == b'=' && position == 3),
            ));
        }
    }
    for scalar in [
        '\n',
        '\r',
        '\t',
        '\0',
        '\u{85}',
        '\u{a0}',
        '\u{2028}',
        '\u{2029}',
        '\u{200d}',
        '\u{e9}',
        '\u{ff21}',
        '\u{1f642}',
    ] {
        for valid in ["", "AAAA", "AA==", "AAA="] {
            values.push((format!("{scalar}{valid}"), false));
            values.push((format!("{valid}{scalar}"), false));
            values.push((format!("AA{scalar}AA"), false));
        }
    }
    values.push(("AAAA\r\n".to_owned(), false));
    for count in [4, 64, 1024, 65536] {
        let text = "A".repeat(count);
        values.push((text.clone(), true));
        values.push((format!("{text}AA=="), true));
        values.push((format!("{text}AAA="), true));
        values.push((format!("{text}!"), false));
        values.push((format!("{text}="), false));
    }
    values
}
