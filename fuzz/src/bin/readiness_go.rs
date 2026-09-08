//! Stage-one positive Go map capture only. This is not the final replay/fuzz harness.
use ess_compiler::{
    ir::{ResolvedBody, ResolvedTypeRef},
    EssIr,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_synth::{SynthesisPlan, Target};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    f.write_all(bytes).unwrap();
    f.sync_all().unwrap();
}
fn compile(documents: &[(&str, String)]) -> EssIr {
    let parsed: Vec<_> = documents
        .iter()
        .map(|(label, text)| {
            (
                Source::new(*label),
                RawSpecFile::parse(text).expect("positive fixture parses"),
            )
        })
        .collect();
    let spec = Specification::assemble(parsed).expect("positive fixture validates");
    let mut sources = ess_compiler::source::SourceMap::new();
    for (label, text) in documents {
        sources.insert((*label).to_owned(), text.clone());
    }
    ess_compiler::compile(&spec, &sources).expect("positive fixture compiles")
}
fn main() {
    let output = PathBuf::from(std::env::args_os().nth(1).expect("fresh output directory"));
    fs::create_dir(&output).expect("baseline output must be absent");
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("readiness/positive");
    let system = fs::read_to_string(input.join("system.yaml")).unwrap();
    let mut cases = Vec::new();
    for family in ["newtype", "struct", "enum", "union", "empty-domain"] {
        let variants: &[bool] = if family == "empty-domain" {
            &[false]
        } else {
            &[false, true]
        };
        for &referenced in variants {
            let name = if family == "empty-domain" {
                family.to_string()
            } else {
                format!(
                    "{family}-{}",
                    if referenced {
                        "referenced"
                    } else {
                        "unreferenced"
                    }
                )
            };
            let core = fs::read_to_string(input.join(format!("{name}.yaml"))).unwrap();
            let documents = [("system.yaml", system.clone()), ("core.yaml", core)];
            let ir = compile(&documents);
            assert_eq!(ir.domains().len(), 1);
            assert_eq!(
                ir.types().len(),
                if family == "empty-domain" {
                    0
                } else if referenced {
                    2
                } else {
                    1
                }
            );
            let owned = ir
                .domains()
                .values()
                .flat_map(|d| d.types.iter().map(ToString::to_string))
                .collect::<Vec<_>>();
            for name in ir.types().keys() {
                assert!(
                    owned.contains(&name.to_string()),
                    "all positive types are actually roster-owned"
                );
            }
            if family != "empty-domain" {
                let value = ir
                    .types()
                    .values()
                    .find(|t| t.name.to_string() == "probe.core.Value")
                    .unwrap();
                let actual = match &value.body {
                    ResolvedBody::Newtype { .. } => "newtype",
                    ResolvedBody::Struct { .. } => "struct",
                    ResolvedBody::Enum { .. } => "enum",
                    ResolvedBody::Union { .. } => "union",
                };
                assert_eq!(actual, family);
                if referenced {
                    let holder = ir
                        .types()
                        .values()
                        .find(|t| t.name.to_string() == "probe.core.Holder")
                        .unwrap();
                    let ResolvedBody::Struct { fields, .. } = &holder.body else {
                        panic!("holder must be a struct")
                    };
                    assert!(
                        matches!(&fields[0].type_ref, ResolvedTypeRef::Declared{name} if name.to_string()=="probe.core.Value")
                    );
                }
            }
            let plan = SynthesisPlan::of(&ir);
            let synthesis =
                ess_synth::synthesize_for(&ir, Target::Go).expect("owned Go facade succeeds");
            assert_eq!(plan, synthesis.plan);
            let direct =
                ess_synth::go::workspace(&ir, &plan).expect("owned direct Go emitter succeeds");
            let direct_map = direct
                .artifacts
                .iter()
                .map(|a| (a.path.clone(), a.clone()))
                .collect::<BTreeMap<_, _>>();
            assert_eq!(direct_map.len(), direct.artifacts.len());
            for (path, artifact) in &direct_map {
                assert_eq!(
                    synthesis.artifacts.get(path),
                    Some(artifact),
                    "direct/facade output correspondence"
                );
            }
            assert_eq!(synthesis.target.as_ref(), Some(&direct.report));
            ess_gen::artifact::validate_paths(synthesis.artifacts.keys().map(String::as_str))
                .unwrap();
            let base = output.join(&name);
            let sources = documents
                .iter()
                .map(|(label, text)| {
                    write(&base.join("source").join(label), text.as_bytes());
                    json!({"label":label,"bytes":text.len(),"sha256":hash(text.as_bytes())})
                })
                .collect::<Vec<_>>();
            write(&base.join("ir.json"), ir.to_canonical_json().as_bytes());
            write(&base.join("plan.json"), plan.to_canonical_json().as_bytes());
            write(&base.join("PLAN.md"), plan.to_markdown().as_bytes());
            write(
                &base.join("facade-artifact-map.json"),
                &serde_json::to_vec_pretty(&synthesis.artifacts).unwrap(),
            );
            write(
                &base.join("direct-artifact-map.json"),
                &serde_json::to_vec_pretty(&direct_map).unwrap(),
            );
            write(
                &base.join("target-report.json"),
                direct.report.to_canonical_json().as_bytes(),
            );
            let map=synthesis.artifacts.iter().map(|(path,a)| {write(&base.join("artifacts").join(path),a.contents.as_bytes()); json!({"path":path,"bytes":a.contents.len(),"sha256":hash(a.contents.as_bytes())})}).collect::<Vec<_>>();
            println!("case {name}: parsed=2 assembled=1 compiled=1 facade=success direct=success artifacts={} actual_roster_types={}",map.len(),owned.len());
            cases.push(json!({"name":name,"family":family,"referenced":referenced,"sources":sources,"owned":owned,"artifact_files":map,"neutral_plan_sha256":hash(plan.to_canonical_json().as_bytes())}));
        }
    }
    assert_eq!(cases.len(), 9);
    write(
        &output.join("cases.json"),
        &serde_json::to_vec_pretty(&cases).unwrap(),
    );
    println!("executed 9 positive specification cases, 18 documents, 9 compilations, 9 facade and 9 direct Go emissions");
}
