use ess_specification_fuzz::{
    observation::{self, Entry, Record, Stage, Writer},
    pipeline::{self, Control},
    replay::{self, Disposition},
    Result,
};
use std::{fs, path::Path, time::Duration};
fn entry(s: &str) -> Result<Entry> {
    match s {
        "byte-carrier" => Ok(Entry::ByteCarrier),
        "structured" => Ok(Entry::Structured),
        _ => Err(format!("unknown entry {s}").into()),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(74)
    }
}
fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [mode,kind,input,out,control] if mode=="--child"=>{
            let input=fs::read(input)?;let mut writer=Writer::create(Path::new(out))?;
            pipeline::run(&mut writer,entry(kind)?,&input,Control::parse(control)?)?;Ok(())
        }
        [mode,input,out,_] if mode=="--stall"=>{
            let input=fs::read(input)?;let mut writer=Writer::create(Path::new(out))?;
            let id=writer.begin(Entry::ByteCarrier,&input)?;
            writer.emit(&Record::StageStart{attempt:id,identity:Stage::Decode})?;
            loop{std::thread::sleep(Duration::from_secs(1));}
        }
        [mode] if mode=="--regressions"=>regressions(&replay::fresh_default()),
        [mode,flag,path] if mode=="--regressions" && flag=="--out"=>regressions(Path::new(path)),
        [mode,kind,path] if mode=="--admit-live"=>{
            let summary=ess_specification_fuzz::live::admit(Path::new(path),entry(kind)?)?;
            println!("{}",serde_json::to_string_pretty(&summary)?);
            observation::qualify_live(&summary,entry(kind)?)?;Ok(())
        }
        [mode,name,path] if mode=="--control"=>{
            let control=Control::parse(name)?;
            let execution=replay::child(&std::env::current_exe()?,Entry::ByteCarrier,
                ess_specification_fuzz::regressions::encoded("system-types"),Path::new(path),control,replay::DEADLINE)?;
            println!("{}",serde_json::to_string_pretty(&execution)?);
            let expected=if matches!(control,Control::PanicDocsIr|Control::SwallowPanic){Disposition::Crash}else{Disposition::Completed};
            if execution.disposition!=expected{return Err(format!("verifier rejected {name}: expected {expected:?}, actual {:?}: {}",execution.disposition,execution.detail).into())}
            Ok(())
        }
        [mode,path] if mode=="--starting-inputs"=>starting_inputs(Path::new(path)),
        _=>Err("usage: replay --regressions [--out DIR] | --admit-live ENTRY DIR | --control CONTROL DIR | --starting-inputs DIR".into()),
    }
}
fn regressions(root: &Path) -> Result<()> {
    let executions = replay::run(&std::env::current_exe()?, root)?;
    let compiled = executions
        .iter()
        .filter_map(|e| e.summary.as_ref())
        .map(|s| s.compiled)
        .sum::<u64>();
    let attempts = executions
        .iter()
        .filter_map(|e| e.summary.as_ref())
        .map(|s| s.attempts)
        .sum::<u64>();
    let mut stages = std::collections::BTreeMap::<String, observation::Counts>::new();
    for summary in executions.iter().filter_map(|e| e.summary.as_ref()) {
        for (key, count) in &summary.stages {
            let total = stages.entry(key.clone()).or_default();
            total.started += count.started;
            total.success += count.success;
            total.refused += count.refused;
        }
    }
    let summary = serde_json::json!({"cases":executions.len(),"attempts":attempts,"compiled":compiled,"stages":stages,"output":root});
    fs::write(
        root.join("totals.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
fn starting_inputs(root: &Path) -> Result<()> {
    fs::create_dir(root)?;
    fs::create_dir(root.join("byte-carrier"))?;
    fs::create_dir(root.join("structured"))?;
    fs::create_dir(root.join("rendered"))?;
    ess_specification_fuzz::regressions::verify()?;
    for name in ess_specification_fuzz::regressions::NAMES {
        fs::write(
            root.join("byte-carrier").join(format!("{name}.json")),
            ess_specification_fuzz::regressions::encoded(name),
        )?;
    }
    let mut identities = Vec::new();
    for (i, input) in ess_specification_fuzz::structured::vectors()
        .into_iter()
        .enumerate()
    {
        fs::write(root.join("structured").join(format!("{i:02}")), &input)?;
        let encoded = ess_specification_fuzz::structured::render(&input)?.encode()?;
        fs::write(root.join("rendered").join(format!("{i:02}.json")), &encoded)?;
        identities.push(ess_specification_fuzz::digest(&encoded));
    }
    fs::write(
        root.join("structured-source-sha256.json"),
        serde_json::to_vec_pretty(&identities)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&identities)?);
    Ok(())
}
