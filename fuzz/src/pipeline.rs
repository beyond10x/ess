//! Actual production calls. Ordinary Results continue; default execution never catches a panic.
use crate::{
    carrier::Bundle,
    observation::{self, Entry, Outcome, Record, Returned, Stage, Target, Writer},
    Result,
};
use ess_compiler::source::SourceMap;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

/// Explicit, narrow verifier injection. Default replay and both real engine adapters use None.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    None,
    SkipDocsIr,
    SkipGo,
    PanicDocsIr,
    SwallowPanic,
    RefuseDocsIr,
    EraseBoundaries,
    DropTerminal,
    DropStage,
}
impl Control {
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SkipDocsIr => "skip-docs-ir",
            Self::SkipGo => "skip-go",
            Self::PanicDocsIr => "panic-docs-ir",
            Self::SwallowPanic => "swallow-panic",
            Self::RefuseDocsIr => "refuse-docs-ir",
            Self::EraseBoundaries => "erase-boundaries",
            Self::DropTerminal => "drop-terminal",
            Self::DropStage => "drop-stage",
        }
    }
    pub fn parse(s: &str) -> Result<Self> {
        [
            Self::None,
            Self::SkipDocsIr,
            Self::SkipGo,
            Self::PanicDocsIr,
            Self::SwallowPanic,
            Self::RefuseDocsIr,
            Self::EraseBoundaries,
            Self::DropTerminal,
            Self::DropStage,
        ]
        .into_iter()
        .find(|c| c.name() == s)
        .ok_or_else(|| format!("unknown explicit verifier control {s}").into())
    }
}
fn start(w: &mut Writer, id: u64, stage: &Stage, control: Control) -> Result<()> {
    if !(control == Control::DropStage && *stage == Stage::DocsIr) {
        w.emit(&Record::StageStart {
            attempt: id,
            identity: stage.clone(),
        })?;
    }
    Ok(())
}
fn returned(
    w: &mut Writer,
    id: u64,
    stage: &Stage,
    result: Returned,
    control: Control,
) -> Result<()> {
    if !(control == Control::DropStage && *stage == Stage::DocsIr) {
        w.emit(&Record::StageResult {
            attempt: id,
            identity: stage.clone(),
            result,
        })?;
    }
    Ok(())
}
fn result<T, E: std::fmt::Display>(
    w: &mut Writer,
    id: u64,
    stage: &Stage,
    value: std::result::Result<T, E>,
    control: Control,
) -> Result<Option<T>> {
    match value {
        Ok(value) => {
            returned(w, id, stage, Returned::Success, control)?;
            Ok(Some(value))
        }
        Err(error) => {
            let detail = w.blob(error.to_string().as_bytes())?;
            returned(w, id, stage, Returned::Refused { detail }, control)?;
            Ok(None)
        }
    }
}
fn finish(
    w: &mut Writer,
    id: u64,
    outcome: Outcome,
    remaining: &[Stage],
    control: Control,
) -> Result<Outcome> {
    if control != Control::DropTerminal {
        w.emit(&Record::Finish {
            attempt: id,
            outcome: outcome.clone(),
            not_reached: remaining.to_vec(),
        })?;
    }
    Ok(outcome)
}
fn stopped(
    w: &mut Writer,
    id: u64,
    outcome: Outcome,
    all: &[Stage],
    at: &Stage,
    control: Control,
) -> Result<Outcome> {
    let pos = all
        .iter()
        .position(|s| s == at)
        .ok_or("internal stage missing")?;
    finish(w, id, outcome, &all[pos + 1..], control)
}
pub fn check_dispatch() -> Result<()> {
    let actual = ess_gen::generators()
        .into_iter()
        .map(|g| g.name().to_owned())
        .collect::<Vec<_>>();
    if actual != observation::GENERATORS {
        return Err(format!("generator registry drift: {actual:?}").into());
    }
    for target in Target::ALL {
        if Target::from_production(target.production()) != target {
            return Err("synthesis dispatch drift".into());
        }
    }
    Ok(())
}
pub fn run(w: &mut Writer, entry: Entry, input: &[u8], control: Control) -> Result<Outcome> {
    check_dispatch()?;
    let id = w.begin(entry, input)?;
    let decode = Stage::Decode;
    start(w, id, &decode, control)?;
    let mut bundle = match entry.decode(input) {
        Ok(bundle) => bundle,
        Err(error) => {
            let detail = w.blob(error.to_string().as_bytes())?;
            returned(w, id, &decode, Returned::Refused { detail }, control)?;
            return stopped(
                w,
                id,
                Outcome::InputRefused,
                &observation::stages(None),
                &decode,
                control,
            );
        }
    };
    if control == Control::EraseBoundaries && bundle.documents.len() > 1 {
        let text = bundle
            .documents
            .iter()
            .map(|d| d.text.as_str())
            .collect::<String>();
        bundle = Bundle {
            documents: vec![crate::carrier::Document {
                label: "merged.yaml".into(),
                text,
            }],
        };
    }
    let bundle_blob = w.blob(&bundle.encode()?)?;
    returned(
        w,
        id,
        &decode,
        Returned::Decoded {
            bundle: bundle_blob,
        },
        control,
    )?;
    let all = observation::stages(Some(&bundle));
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (index, doc) in bundle.documents.iter().enumerate() {
        let stage = Stage::Parse {
            index,
            label: doc.label.clone(),
        };
        start(w, id, &stage, control)?;
        let Some(raw) = result(w, id, &stage, RawSpecFile::parse(&doc.text), control)? else {
            return stopped(w, id, Outcome::ParseRefused, &all, &stage, control);
        };
        sources.insert(doc.label.clone(), doc.text.clone());
        parsed.push((Source::new(doc.label.clone()), raw));
    }
    let assembly = Stage::AssembleValidate;
    start(w, id, &assembly, control)?;
    let specification = match Specification::assemble(parsed) {
        Ok(spec) => spec,
        Err(error) => {
            let detail = w.blob(error.to_string().as_bytes())?;
            returned(w, id, &assembly, Returned::Refused { detail }, control)?;
            return stopped(w, id, Outcome::ValidationRefused, &all, &assembly, control);
        }
    };
    let major = specification.system().format.major();
    returned(w, id, &assembly, Returned::Assembled { major }, control)?;
    if major != 1 {
        return stopped(
            w,
            id,
            Outcome::OutsideDomain { major },
            &all,
            &assembly,
            control,
        );
    }
    let compile = Stage::Compile;
    start(w, id, &compile, control)?;
    let Some(ir) = result(
        w,
        id,
        &compile,
        ess_compiler::compile(&specification, &sources),
        control,
    )?
    else {
        return stopped(w, id, Outcome::CompileRefused, &all, &compile, control);
    };
    for generator in ess_gen::generators() {
        let stage = Stage::Generator {
            name: generator.name().into(),
        };
        start(w, id, &stage, control)?;
        result(
            w,
            id,
            &stage,
            ess_gen::artifact::run(generator.as_ref(), &ir),
            control,
        )?;
    }
    if control != Control::SkipDocsIr {
        let stage = Stage::DocsIr;
        start(w, id, &stage, control)?;
        let value: std::result::Result<(), String> = match control {
            Control::PanicDocsIr => panic!("injected production-call panic"),
            Control::SwallowPanic => {
                // Deliberately broken control: the propagation verifier must reject this.
                let _ = std::panic::catch_unwind(|| panic!("injected production-call panic"));
                Ok(())
            }
            Control::RefuseDocsIr => Err("injected ordinary returned refusal".into()),
            _ => {
                let mint = ess_gen::provenance::ProvenanceMint::new(&ir);
                let document = ess_gen::docs::document(&ir, &mint);
                serde_json::to_string_pretty(&document)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
        };
        result(w, id, &stage, value, control)?;
    }
    for target in Target::ALL {
        if control == Control::SkipGo && target == Target::Go {
            continue;
        }
        let stage = Stage::Synthesis { target };
        start(w, id, &stage, control)?;
        result(
            w,
            id,
            &stage,
            ess_synth::synthesize_for(&ir, target.production()),
            control,
        )?;
    }
    finish(w, id, Outcome::Compiled, &[], control)
}
