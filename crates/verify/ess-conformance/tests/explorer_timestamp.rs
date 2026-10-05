//! Seeded Timestamp inputs are real planner/draw behavior, shared by both emitted explorers.
use std::{fs, path::Path, process::Command};

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const MODEL: &str = r"format: ess/16
system: clocks
version: v1
domain: clocks.api
types:
  - {name: clocks.api.Instant, kind: newtype, of: Timestamp}
  - {name: clocks.api.Wrapped, kind: newtype, of: clocks.api.Instant}
events:
  - {name: clocks.api.Stamped, fields: []}
commands:
  - name: clocks.api.Direct
    input: [{name: at, type: Timestamp}]
    outcomes: [{name: accepted, emits: [clocks.api.Stamped]}]
  - name: clocks.api.Newtype
    input: [{name: at, type: clocks.api.Wrapped}]
    outcomes: [{name: accepted, emits: [clocks.api.Stamped]}]
  - name: clocks.api.Unsupported
    input: [{name: duration, type: Duration}]
    outcomes: [{name: accepted, emits: [clocks.api.Stamped]}]
";

fn run(command: &mut Command) {
    let output = command
        .output()
        .expect("required runtime tool is installed");
    assert!(
        output.status.success(),
        "{command:?}\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write(root: &Path, path: &str, bytes: impl AsRef<[u8]>) {
    let file = root.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, bytes).unwrap();
}

const GO: &str = r#"package essconform
import ("encoding/json"; "os"; "testing")
func TestTimestampInputs(t *testing.T) {
  ir, err := exploreLoad(exploreIR, exploreSuite); if err != nil { t.Fatal(err) }
  plan := explorePlanAs(ir, true)
  commands := []string{}; for _, command := range plan.commands { commands = append(commands, command.name) }
  refs := []map[string]any{{"kind":"primitive","name":"timestamp"},{"kind":"declared","name":"clocks.api.Wrapped"},{"kind":"primitive","name":"duration"}}
  results := []any{}
  for _, ref := range refs {
    kind := exploreResolveAs(ir, ref, 0, true)
    serial := exploreResolveAs(ir, ref, 0, false)
    seeds := []any{}
    for _, seed := range []uint32{1,7,4294967295} {
      rng := NewMulberry32(seed); values := []any{}
      for i:=0;i<16;i++ { held := []exploreRef{}; value,err:=exploreDraw(kind,rng,&exploreCommand{},&exploreModel{},"at",false,&held); if err!=nil {t.Fatal(err)}; values=append(values,value) }
      seeds=append(seeds,map[string]any{"values":values,"next":rng.NextUint32()})
    }
    results=append(results,map[string]any{"kind":kind.kind,"serial":serial.kind,"seeds":seeds})
  }
  bytes,err:=json.Marshal(map[string]any{"commands":commands,"results":results});if err!=nil {t.Fatal(err)}
  if err=os.WriteFile("timestamp-result.json",bytes,0600);err!=nil {t.Fatal(err)}
}
"#;

// This test-only export invokes the private production planner and drawer unchanged.
const TS: &str = r"
export function timestampProbe(ir: any): any {
  const refs = [{kind:'primitive',name:'timestamp'},{kind:'declared',name:'clocks.api.Wrapped'},{kind:'primitive',name:'duration'}];
  return {commands:plan(ir,true).commands.map(c=>c.name),results:refs.map(ref=> {
    const kind=resolveKind(ir,ref,0,true);
    return {kind:kind.kind,serial:resolveKind(ir,ref,0,false).kind,seeds:[1,7,4294967295].map(seed=> {
      const rng=new Mulberry32(seed);const values=[];
      for(let i=0;i<16;i++) values.push(drawValue(kind,rng,{} as Command,new Model(),'at',false,[]));
      return {values,next:rng.nextUint32()};
    })};
  })};
}
";

#[test]
fn both_emitted_planners_draw_exact_seeded_timestamps_through_newtypes() {
    let specification = Specification::assemble([(
        Source::new("timestamp.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    let ir = compile(&specification, &SourceMap::new()).unwrap();
    let suite = ess_conformance::synthesize(&ir).suite;
    let root = std::env::temp_dir().join(format!("ess-explorer-timestamps-{}", std::process::id()));
    let go = root.join("go");
    let ts = root.join("ts");
    for artifact in ess_conformance::go::emit_with_model(&suite, &ir).unwrap() {
        write(&go, &artifact.path, artifact.contents);
    }
    for artifact in ess_conformance::ts::emit_with_model(&suite, &ir).unwrap() {
        write(&ts, &artifact.path, artifact.contents);
    }
    write(
        &go,
        "go.mod",
        "module example.invalid/timestamps\n\ngo 1.24\n",
    );
    write(&go, "essconform/timestamp_test.go", GO);
    run(Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "^TestTimestampInputs$",
            "-count=1",
        ])
        .env("GOWORK", "off")
        .current_dir(&go));
    let package = ts.join("essconform");
    let source = package.join("src/explore.ts");
    fs::write(
        &source,
        format!("{}{TS}", fs::read_to_string(&source).unwrap()),
    )
    .unwrap();
    write(
        &package,
        "timestamp.tsconfig.json",
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    );
    run(Command::new("tsc")
        .args(["--project", "timestamp.tsconfig.json"])
        .current_dir(&package));
    run(Command::new("node").args(["--input-type=module", "-e", "import {timestampProbe} from './dist/explore.js';import fs from 'node:fs';fs.writeFileSync('timestamp-result.json',JSON.stringify(timestampProbe(JSON.parse(fs.readFileSync('ir.json','utf8')))));"]).current_dir(&package));
    let go: Value =
        serde_json::from_slice(&fs::read(go.join("essconform/timestamp-result.json")).unwrap())
            .unwrap();
    let ts: Value =
        serde_json::from_slice(&fs::read(package.join("timestamp-result.json")).unwrap()).unwrap();
    assert_eq!(go, ts, "exact planner and draw parity");
    assert_eq!(
        go["commands"],
        serde_json::json!(["clocks.api.Direct", "clocks.api.Newtype"])
    );
    for kind in &go["results"].as_array().unwrap()[..2] {
        assert_eq!(kind["kind"], "timestamp");
        assert_eq!(
            kind["serial"], "unsupported",
            "serial temporal predicate semantics remain bounded"
        );
        for (case, seed) in kind["seeds"]
            .as_array()
            .unwrap()
            .iter()
            .zip([1_u32, 7, u32::MAX])
        {
            let mut state = seed;
            for value in case["values"].as_array().unwrap() {
                let second = u64::from(next(&mut state)) * 86_400 / (1_u64 << 32);
                let expected = format!(
                    "2020-01-01T{:02}:{:02}:{:02}Z",
                    second / 3600,
                    second / 60 % 60,
                    second % 60
                );
                assert_eq!(value, &Value::String(expected));
                assert!(ess_primitives::time::Rfc3339Instant::parse_rfc3339(
                    value.as_str().unwrap()
                )
                .is_some());
            }
            assert_eq!(
                case["next"].as_u64(),
                Some(u64::from(next(&mut state))),
                "one RNG draw per timestamp"
            );
        }
    }
    assert_eq!(
        go["results"][2]["kind"], "unsupported",
        "no general primitive-to-text fallback"
    );
}

fn next(state: &mut u32) -> u32 {
    *state = state.wrapping_add(0x6d2b_79f5);
    let mut value = *state;
    value = (value ^ (value >> 15)).wrapping_mul(value | 1);
    value ^= value.wrapping_add((value ^ (value >> 7)).wrapping_mul(value | 0x3d));
    value ^ (value >> 14)
}
