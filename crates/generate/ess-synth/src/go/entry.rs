//! Executable wiring for components explicitly reached by network.
use super::{
    behaviour::{self, Seams},
    context,
    layout::{Layout, Package},
    name, store, Emit,
};
use crate::{plan::SynthesisPlan, served::Reachable};
use ess_compiler::ir::{EssIr, ResolvedComponent};
use ess_gen::Artifact;
use std::fmt::Write as _;

pub(super) fn artifacts(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    seams: &Seams,
) -> Vec<Artifact> {
    let components = crate::rust::http::served(ir);
    if components.is_empty() {
        return Vec::new();
    }
    let mut artifacts = Vec::new();
    // A collector has its own imports: discarded renderings never pollute the emitted file.
    let collector = Emit::new(ir, layout, layout.behaviour(), None);
    let uses = behaviour::requirements(&collector, seams, None);
    if !seams.generated.is_empty() {
        let emit = Emit::new(ir, layout, layout.server(), None);
        let body = store::implementation(&emit, &uses, seams);
        artifacts.push(emit.file_at(
            format!("{}/ess_memory.go", layout.server().dir),
            &plan.provenance,
            "// Ephemeral generated storage and context.\n",
            &body,
        ));
    }
    let emit = Emit::new(ir, layout, layout.server(), None);
    for import in ["net/http", "os", "path/filepath", "strings"] {
        emit.import(import);
    }
    artifacts.push(emit.file_at(
        format!("{}/ess_static.go", layout.server().dir),
        &plan.provenance,
        "// Static fallback, confined to the selected root.\n",
        STATIC,
    ));
    for component in components {
        artifacts.push(binary(ir, plan, layout, seams, component));
    }
    artifacts
}

fn binary(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    seams: &Seams,
    component: &ResolvedComponent,
) -> Artifact {
    let collector = Emit::new(ir, layout, layout.behaviour(), None);
    let reachable = Reachable::of(ir, component);
    let mut unmet = reachable.obligations(plan);
    unmet.extend(context::unmet(
        ir,
        &behaviour::requirements(&collector, seams, Some(&reachable)),
    ));
    unmet.extend(
        seams
            .weakened
            .iter()
            .filter(|capability| {
                reachable
                    .commands
                    .iter()
                    .chain(&reachable.views)
                    .any(|name| name.to_string() == capability.source)
            })
            .map(|capability| format!("target cannot generate: {}", capability.source)),
    );
    let package = Package {
        name: "main".into(),
        dir: format!("cmd/{}-server", component.name),
        import: format!("{}/cmd/{}-server", layout.module(), component.name),
    };
    let emit = Emit::new(ir, layout, &package, None);
    for import in ["flag", "fmt", "os"] {
        emit.import(import);
    }
    let mut body = String::from(MAIN);
    if unmet.is_empty() {
        let server = layout.server();
        let memory = emit.qualify(server, "NewMemoryPorts");
        let all = behaviour::requirements(&collector, seams, None);
        let context =
            !all.callers.is_empty() || !all.generates.is_empty() || all.external || all.clock;
        let generated = emit.qualify(
            layout.behaviour(),
            if context { "NewWithContext" } else { "New" },
        );
        let context_argument = if context {
            format!(", &{}{{}}", emit.qualify(server, "MemoryContext"))
        } else {
            String::new()
        };

        let mut arguments = Vec::new();
        for component in ir.components().keys() {
            let constructor = emit.qualify(layout.component(component), layout.port_new(component));
            arguments.push(format!(
                "{constructor}({generated}(ports{context_argument}))"
            ));
        }
        if crate::rust::system::has_obligations(ir, plan) {
            arguments.push(format!(
                "{}{{}}",
                emit.qualify(layout.system(), layout.unimplemented(layout.system()))
            ));
        }
        let constructor = emit.qualify(layout.system(), layout.system_name("NewSystem"));
        let _ = writeln!(body, "\tports := {memory}()\n\tsystem := {constructor}({})\n\tif *callers == \"actor-header\" {{\n\t\tfmt.Fprintln(os.Stderr, `{{\"format\":\"ess/1\",\"callers\":\"actor-header\",\"demonstration\":true}}`)\n\t}} else {{\n\t\tfmt.Fprintln(os.Stderr, `{{\"format\":\"ess/1\",\"callers\":\"none\"}}`)\n\t}}", arguments.join(", "));
        let auth = if ess_gen::http::checks_grants(ir) {
            body.push_str(&authentication(&emit));
            ", authenticate"
        } else {
            ""
        };
        let serve = emit.qualify(
            server,
            &format!(
                "Serve{}WithStatic",
                name::exported(&component.name.to_string())
            ),
        );
        let _ = writeln!(body, "\tif err := {serve}(system, *listen{auth}, *staticRoot); err != nil {{\n\t\tfmt.Fprintln(os.Stderr, err)\n\t\tos.Exit(1)\n\t}}\n}}");
    } else {
        let message = format!(
            "unmet startup obligations:\n{}",
            unmet.into_iter().collect::<Vec<_>>().join("\n")
        );
        let _ = writeln!(
                body,
                "\t_ = listen\n\t_ = staticRoot\n\tfmt.Fprintln(os.Stderr, {message:?})\n\tos.Exit(1)\n}}"
            );
    }
    emit.file_at(
        format!("{}/main.go", package.dir),
        &plan.provenance,
        "// Generated ephemeral server; no production authentication or durable storage.\n",
        &body,
    )
}

fn authentication(emit: &Emit<'_>) -> String {
    emit.import("net/http");
    emit.import("strings");
    let caller = emit.qualify(emit.layout.server(), "Caller");
    let actor = emit.qualify(emit.layout.server(), "Actor");
    let mut counts = std::collections::BTreeMap::new();
    for actor in emit.ir.actors().keys() {
        *counts
            .entry(actor.to_string().rsplit('.').next().unwrap().to_owned())
            .or_insert(0_usize) += 1;
    }
    let mut cases = String::new();
    for actor in emit.ir.actors().keys() {
        let full = actor.to_string();
        let short = full.rsplit('.').next().unwrap();
        if counts[short] == 1 {
            let _ = writeln!(cases, "\t\tcase {short:?}:\n\t\t\tname = {full:?}");
        }
    }
    format!(
        r#"	authenticate := func(request *http.Request) *{caller} {{
		if *callers != "actor-header" {{
			return nil
		}}
		// One credential: a request carrying two Authorization headers is authenticated as nobody.
		values := request.Header.Values("Authorization")
		if len(values) != 1 {{
			return nil
		}}
		header := values[0]
		if !strings.HasPrefix(header, "Actor ") {{
			return nil
		}}
		name := strings.TrimPrefix(header, "Actor ")
		switch name {{
{cases}		}}
		return &{caller}{{Actor: {actor}(name)}}
	}}
"#
    )
}

const MAIN: &str = r#"
func main() {
	listen := flag.String("listen", "127.0.0.1:8080", "HTTP listen address")
	callers := flag.String("callers", "none", "none or demonstration actor-header")
	staticRoot := flag.String("static", "", "same-origin static directory")
	flag.Parse()
	if *callers != "none" && *callers != "actor-header" {
		fmt.Fprintln(os.Stderr, "unknown caller mode")
		os.Exit(2)
	}
"#;

const STATIC: &str = r#"
func memoryStaticRoot(root string) (string, error) {
	if root == "" {
		return "", nil
	}
	resolved, err := filepath.EvalSymlinks(root)
	if err != nil {
		return "", err
	}
	resolved, err = filepath.Abs(resolved)
	if err != nil {
		return "", err
	}
	info, err := os.Stat(resolved)
	if err != nil {
		return "", err
	}
	if !info.IsDir() {
		return "", &os.PathError{Op: "static", Path: root, Err: os.ErrInvalid}
	}
	return resolved, nil
}

func memoryStatic(writer http.ResponseWriter, request *http.Request, root string) {
	if request.Method != "GET" && request.Method != "HEAD" {
		writer.WriteHeader(405)
		return
	}
	decoded := request.URL.Path
	if !strings.HasPrefix(decoded, "/") || strings.ContainsAny(decoded, "\\\x00") {
		http.NotFound(writer, request)
		return
	}
	for _, segment := range strings.Split(decoded, "/") {
		if segment == ".." {
			http.NotFound(writer, request)
			return
		}
	}
	path, err := filepath.EvalSymlinks(filepath.Join(root, strings.TrimPrefix(decoded, "/")))
	if err != nil {
		http.NotFound(writer, request)
		return
	}
	info, err := os.Stat(path)
	if err != nil {
		http.NotFound(writer, request)
		return
	}
	if info.IsDir() {
		path, err = filepath.EvalSymlinks(filepath.Join(path, "index.html"))
		if err != nil {
			http.NotFound(writer, request)
			return
		}
	}
	relative, err := filepath.Rel(root, path)
	if err != nil || relative == ".." || strings.HasPrefix(relative, ".."+string(os.PathSeparator)) {
		http.NotFound(writer, request)
		return
	}
	file, err := os.Open(path)
	if err != nil {
		http.NotFound(writer, request)
		return
	}
	defer file.Close()
	info, err = file.Stat()
	if err != nil || !info.Mode().IsRegular() {
		http.NotFound(writer, request)
		return
	}
	http.ServeContent(writer, request, info.Name(), info.ModTime(), file)
}
"#;
