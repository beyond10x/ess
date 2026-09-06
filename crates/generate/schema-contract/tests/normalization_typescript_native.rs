//! Explicit external-tool lane. Default package tests never require Node/TypeScript.
#[path = "fixtures/normalization_typescript.rs"]
mod fixture;
#[path = "support/normalization_typescript.rs"]
mod support;

use serde_json::{json, Value};
use std::fmt::Write as _;

#[test]
fn native_typescript_executes_every_shared_text_and_retained_document_case() {
    support::compiler();
    let mut count = 0;
    for (name, plan, cases) in fixture::corpora() {
        count += support::run_cases(name, &plan, &cases);
    }
    assert_eq!(count, 3091);
    eprintln!("TypeScript shared runtime corpus: executed {count}; failed 0; skipped 0 (13 named value-only entrypoints are inapplicable)");
}

#[test]
fn native_schema_numeric_refinements_and_diagnostic_multiplicity() {
    let (plan, cases) = fixture::schema_controls();
    assert_eq!(support::run_cases("schema-controls", &plan, &cases), 31);
}

#[test]
fn native_unicode_prototypes_lexical_precedence_and_private_arity_defense() {
    assert_eq!(
        support::run_cases("typescript-boundaries", &fixture::boundary_plan(), &[]),
        0
    );
}

#[test]
fn native_equality_preserves_the_frozen_integer_float_limitation() {
    for ((plan, cases), version) in fixture::integer_equality_controls()
        .into_iter()
        .zip([1, 5, 6])
    {
        assert_eq!(
            support::run_cases(&format!("equality-v{version}"), &plan, &cases),
            7
        );
    }
    let cases = [
        json!({"branch":"equal","input":"0","value":true}),
        json!({"branch":"equal","input":"-0","value":true}),
        json!({"branch":"equal","input":"1","value":false}),
        json!({"branch":"equal","input":"5e-324","value":false}),
    ];
    assert_eq!(
        support::run_cases(
            "equality-checked-binary64",
            &fixture::binary64::plan(),
            &cases
        ),
        4
    );
}

#[test]
fn native_private_validator_conjunctions_match_the_pinned_schema_engine() {
    let root = support::write_package("private-validator", &fixture::boundary_plan());
    let mut source = "import {validate, type Schema} from './schema.js';\nimport {parseInput, EMPTY_POLICIES} from './input.js';\nimport {Refusal} from './value.js';\nconst definitions = new Map<string,Schema>([['bound',{types:['number'],minimum:9007199254740993n}]]);\nlet count=0;\n".to_owned();
    // These keyword conjunctions are currently refused by Plan's pre-existing
    // intersection-shape admission. Qualify the private validator independently;
    // this does not claim public target admission or bypass any source contract.
    for (schema, lowered, input, expected) in [
        (
            json!({"allOf":[{"type":"number","minimum":1},{"type":"number","maximum":-1}]}),
            "{allOf:[{types:['number'],minimum:1n},{types:['number'],maximum:-1n}]}",
            "0",
            vec!["", ""],
        ),
        (
            json!({"allOf":[{"type":"number"},{"type":"number"}]}),
            "{allOf:[{types:['number']},{types:['number']}]}",
            "null",
            vec!["", ""],
        ),
        (
            json!({"type":"string","const":1}),
            "{types:['string'],constant:1n}",
            "null",
            vec!["", ""],
        ),
        (
            json!({"type":"string","enum":[1]}),
            "{types:['string'],enumeration:[1n]}",
            "null",
            vec!["", ""],
        ),
        (
            json!({"type":"object","propertyNames":{"type":"string","minLength":3,"enum":["zzz"]}}),
            "{types:['object'],names:{types:['string'],minLength:3n,enumeration:['zzz']}}",
            r#"{"a":0,"b":1}"#,
            vec!["", "", "", ""],
        ),
        (
            json!({"$defs":{"bound":{"type":"number","minimum":9_007_199_254_740_993_u64}},"$ref":"#/$defs/bound","maximum":9_007_199_254_740_994_u64}),
            "{ref:'bound',maximum:9007199254740994n}",
            "9007199254740992.0",
            vec![""],
        ),
        (
            json!({"$defs":{"bound":{"type":"number","minimum":9_007_199_254_740_993_u64}},"$ref":"#/$defs/bound","maximum":9_007_199_254_740_994_u64}),
            "{ref:'bound',maximum:9007199254740994n}",
            "9007199254740995",
            vec![""],
        ),
    ] {
        let input_value: Value = serde_json::from_str(input).unwrap();
        let validator = jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .should_validate_formats(false)
            .build(&schema)
            .unwrap();
        let mut actual = validator
            .iter_errors(&input_value)
            .map(|error| error.instance_path().to_string())
            .collect::<Vec<_>>();
        actual.sort();
        assert_eq!(actual, expected, "{schema} {input}");
        writeln!(source, "{{let got:string[]=[];try{{validate({{root:{lowered},definitions}},parseInput({},EMPTY_POLICIES),'/check')}}catch(e){{if(!(e instanceof Refusal))throw e;got=e.findings.map(f=>f.pointer)}}if(JSON.stringify(got)!==JSON.stringify({}))throw new Error('private schema '+count+': '+JSON.stringify(got));count++;}}",
            serde_json::to_string(input).unwrap(), json!(expected.iter().map(|p| format!("/check{p}")).collect::<Vec<_>>())).unwrap();
    }
    source.push_str("console.log('private schema conjunctions: executed '+count+'; failed 0');\n");
    std::fs::write(root.join("src/qualification.ts"), source).unwrap();
    support::compile(&root);
    support::run(&root);
}

#[test]
fn native_numeric_bits_shortest_text_and_exact_midpoints() {
    use std::collections::BTreeSet;
    use std::fs;
    use std::process::Command;
    support::compiler();
    let plan = fixture::boundary_plan();
    let root = support::write_package("numeric-qualification", &plan);
    let mut patterns = BTreeSet::new();
    let mut add = |bits| {
        if f64::from_bits(bits).is_finite() {
            patterns.insert(bits);
        }
    };
    for bits in 0..1000 {
        add(bits);
        add(bits | (1 << 63));
    }
    for exponent in (1_u64..2047).step_by(7) {
        for bits in ((exponent << 52) - 2)..=((exponent << 52) + 2) {
            add(bits);
        }
    }
    let mut state = 0xd1b5_4a32_d192_ed03_u64;
    for _ in 0..10_000 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        add(state);
    }
    assert_eq!(patterns.len(), 13_455);
    let tokens = patterns
        .iter()
        .map(|bits| serde_json::to_string(&f64::from_bits(*bits)).unwrap())
        .collect::<Vec<_>>();
    fs::write(
        root.join("reference-number-tokens.json"),
        serde_json::to_string(&tokens).unwrap(),
    )
    .unwrap();
    let output = Command::new("node").args(["-e", "const fs=require('node:fs'); const tokens=JSON.parse(fs.readFileSync('reference-number-tokens.json','utf8')); console.log(JSON.stringify(tokens.map(t=>{const n=Number(t);return Object.is(n,-0)?'-0.0':n.toString()})))"])
        .current_dir(&root).output().unwrap();
    fs::write(root.join("shortest-tokens.stdout.log"), &output.stdout).unwrap();
    fs::write(root.join("shortest-tokens.stderr.log"), &output.stderr).unwrap();
    assert!(output.status.success());
    let shortest: Vec<String> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(shortest.len(), patterns.len());
    let mut cases = Vec::new();
    for ((bits, token), native) in patterns.iter().zip(&tokens).zip(&shortest) {
        let value = Value::from(f64::from_bits(*bits));
        cases.push(
            json!({"branch":"rounded","input":native,"value":value,"bits":format!("{bits:016x}")}),
        );
        // The native shortest input may take the reference's integer fast path.
        let exact = plan.run_json("plain", native).unwrap();
        cases.push(json!({"branch":"plain","input":native,"value":exact}));
        assert_eq!(&value.to_string(), token);
    }
    let midpoints = midpoint_tokens();
    assert_eq!(midpoints.len(), 28);
    for token in midpoints {
        for branch in ["rounded", "plain"] {
            let case = match plan.run_json(branch, &token) {
                Ok(value) => json!({"branch":branch,"input":token,"value":value}),
                Err(error) => json!({"branch":branch,"input":token,"errors":error.0}),
            };
            cases.push(case);
        }
    }
    assert_eq!(
        support::run_cases("numeric-qualification", &plan, &cases),
        26_966
    );
    eprintln!("numeric qualification: 13455 independent finite bit patterns, native-shortest text through rounded and exact entry; 28 exact rational midpoint/huge-exponent tokens through both entries");
}

fn midpoint_tokens() -> Vec<String> {
    // Test-owned decimal arithmetic constructs rational halfway values without
    // a float conversion or a new bigint dependency supplying the expected input.
    fn multiply(digits: &mut Vec<u8>, factor: u8) {
        let mut carry = 0;
        for digit in digits.iter_mut() {
            let value = *digit * factor + carry;
            *digit = value % 10;
            carry = value / 10;
        }
        while carry != 0 {
            digits.push(carry % 10);
            carry /= 10;
        }
    }
    fn adjacent(mut digits: Vec<u8>, delta: i8) -> String {
        if delta == 1 {
            let mut index = 0;
            loop {
                if index == digits.len() {
                    digits.push(0);
                }
                if digits[index] < 9 {
                    digits[index] += 1;
                    break;
                }
                digits[index] = 0;
                index += 1;
            }
        } else if delta == -1 {
            let mut index = 0;
            while digits[index] == 0 {
                digits[index] = 9;
                index += 1;
            }
            digits[index] -= 1;
        }
        digits
            .iter()
            .rev()
            .map(|digit| char::from(b'0' + digit))
            .collect()
    }
    let mut tokens = Vec::new();
    for (initial, factor, power, exponent) in [
        ("9007199254740993", 5, 53, -53),
        ("1", 5, 1075, -1075),
        ("9007199254740991", 5, 1075, -1075),
        ("18014398509481983", 2, 970, 0),
    ] {
        let mut coefficient = initial
            .bytes()
            .rev()
            .map(|digit| digit - b'0')
            .collect::<Vec<_>>();
        for _ in 0..power {
            multiply(&mut coefficient, factor);
        }
        for delta in [-1, 0, 1] {
            for sign in ["", "-"] {
                let digits = adjacent(coefficient.clone(), delta);
                tokens.push(if exponent == 0 {
                    format!("{sign}{digits}")
                } else {
                    format!("{sign}{digits}e{exponent}")
                });
            }
        }
    }
    for (coefficient, sign) in [("0", ""), ("-0", ""), ("1", "-"), ("-1", "-")] {
        tokens.push(format!("{coefficient}e{sign}{}", "9".repeat(1000)));
    }
    tokens
}
