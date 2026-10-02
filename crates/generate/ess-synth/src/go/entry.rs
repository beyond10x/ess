//! The opt-in demonstration realization and its executable.
use super::{
    behaviour::{self, Seams, Uses},
    context, http,
    layout::Layout,
    name,
    refusal::TargetRefusals,
    Emit,
};
use crate::plan::{SynthesisPlan, REGENERATE};
use ess_compiler::ir::EssIr;
use ess_gen::Artifact;
use std::collections::BTreeSet;
use std::fmt::Write as _;

pub(super) fn binaries(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    target_refusals: &TargetRefusals,
    uses: &Uses,
    seams: &Seams,
) -> Vec<Artifact> {
    let probe = Emit::new(ir, layout, layout.behaviour(), None);
    let mut refusals = context::refusals(&probe, uses);
    refusals.extend(
        plan.obligations()
            .map(|(capability, _)| format!("{:?} `{}`", capability.kind, capability.source)),
    );
    refusals.extend(
        seams
            .colliding
            .iter()
            .chain(&seams.weakened)
            .map(|capability| format!("{:?} `{}`", capability.kind, capability.source)),
    );
    http::served(ir, target_refusals)
        .into_iter()
        .map(|component| {
            let mut out = plan.provenance.commented_for("//", REGENERATE);
            out.push_str("\npackage main\n\nimport (\n");
            let runnable = refusals.is_empty();
            let grants = ess_gen::http::checks_grants(ir);
            let mut imports: BTreeSet<String> = ["flag", "fmt", "os"]
                .into_iter()
                .map(str::to_owned)
                .collect();
            if runnable {
                imports.extend(
                    [layout.system(), layout.server()].map(|package| package.import.clone()),
                );
                if !seams.generated.is_empty() {
                    imports.insert(layout.behaviour().import.clone());
                }
                imports.extend(
                    ir.components()
                        .keys()
                        .map(|component| layout.component(component).import.clone()),
                );
                if grants {
                    imports.extend(["net/http".into(), "strings".into()]);
                }
            }
            for import in imports {
                let _ = writeln!(out, "\t{import:?}");
            }
            out.push_str(")\n");
            out.push_str(FLAGS);
            if runnable {
                runnable_body(
                    &mut out,
                    ir,
                    layout,
                    uses,
                    seams,
                    &component.name.to_string(),
                );
            } else {
                let _ = writeln!(
                    out,
                    "\t_ = listen\n\t_ = static\n\tfmt.Fprintln(os.Stderr, {:?})\n\tos.Exit(1)\n}}",
                    format!(
                        "generated entry point requires a realization: {}",
                        refusals.join("; ")
                    )
                );
            }
            Artifact::new(format!("cmd/{}-server/main.go", component.name), out)
        })
        .collect()
}

fn runnable_body(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    uses: &Uses,
    seams: &Seams,
    component: &str,
) {
    if seams.generated.is_empty() {
        out.push_str("\tgenerated := struct{}{}\n");
    } else {
        let storages = behaviour::storage_names(ir, layout);
        // Individual assignments avoid alignment varying with the model's field names.
        out.push_str("\tports := behaviour.Ports{}\n");
        for entity in &uses.storages {
            let _ = writeln!(
                out,
                "\tports.{} = &behaviour.InMemory{}{{}}",
                storages[entity], storages[entity]
            );
        }
        if !uses.callers.is_empty() || !uses.generates.is_empty() || uses.external {
            out.push_str("\tports.Context = &behaviour.InMemoryContext{}\n");
        }
        out.push_str("\tgenerated := behaviour.New(ports)\n");
    }
    let ports: Vec<_> = ir
        .components()
        .keys()
        .map(|component| {
            format!(
                "{}.{}(generated)",
                layout.component(component).name,
                layout.port_new(component)
            )
        })
        .collect();
    let _ = writeln!(out, "\tassembled := system.NewSystem({})", ports.join(", "));
    let grants = ess_gen::http::checks_grants(ir);
    if grants {
        out.push_str(AUTHENTICATE);
        for actor in ir.actors().keys() {
            let full = actor.to_string();
            let short = full.rsplit('.').next().unwrap();
            let _ = writeln!(out, "\t\tif value == {full:?} || value == {short:?} {{\n\t\t\tif found != nil {{\n\t\t\t\treturn nil\n\t\t\t}}\n\t\t\tfound = &server.Caller{{Actor: server.Actor({full:?})}}\n\t\t}}");
        }
        out.push_str("\t\treturn found\n\t}\n");
    }
    out.push_str("\tmode := *callers\n\tif mode == \"actor-header\" {\n\t\tmode += \" (demonstration mode; not authentication)\"\n\t}\n\tfmt.Fprintln(os.Stderr, \"generated entry: ephemeral storage; callers=\"+mode)\n");
    let _ = writeln!(out, "\tif err := server.Serve{}WithStatic(assembled, *listen{}, *static); err != nil {{\n\t\tfmt.Fprintln(os.Stderr, err)\n\t\tos.Exit(1)\n\t}}\n}}", name::exported(component), if grants { ", authenticate" } else { "" });
}

const FLAGS: &str = r#"
func main() {
	listen := flag.String("listen", "127.0.0.1:8080", "listen address")
	callers := flag.String("callers", "none", "none or actor-header (demonstration only)")
	static := flag.String("static", "", "same-origin static directory")
	flag.Parse()
	if flag.NArg() != 0 || (*callers != "none" && *callers != "actor-header") {
		fmt.Fprintln(os.Stderr, "expected --callers none or actor-header and no positional arguments")
		os.Exit(2)
	}
"#;
const AUTHENTICATE: &str = r#"	authenticate := func(request *http.Request) *server.Caller {
		if *callers != "actor-header" {
			return nil
		}
		value, ok := strings.CutPrefix(request.Header.Get("Authorization"), "Actor ")
		if !ok {
			return nil
		}
		var found *server.Caller
"#;
