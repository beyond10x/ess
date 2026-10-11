//! Go synthesis of a model whose bounded contexts reference each other in a loop.
//!
//! The Go target emits one package per bounded context under `types/`, and Go refuses an import
//! cycle where Rust admits a module cycle. An adopter's model on ess 0.58.0 had a command outcome
//! in `publish` refusing with an error declared in `session`, `session` referencing
//! `subscription`, and `subscription` referencing `publish.Channel`: `ess specify validate`
//! accepted it, and `go build` of the synthesized module failed with `import cycle not allowed`.
//! Story `go-synthesis-breaks-import-cycles-between-domains`.

use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Synthesis, Target};

const SYSTEM: &str = "format: ess/4
system: relay
version: v1
domains:
  - relay.audit
  - relay.publish
  - relay.session
  - relay.subscription
components:
  - component: relay-service
    owns:
      domains: [relay.audit, relay.publish, relay.session, relay.subscription]
    accepts:
      commands: [relay.publish.Publish, relay.session.Open]
    publishes:
      events: [relay.publish.Published]
    reached_by: network
";

/// `publish` declares the channel and the command; `error` is the qualified error its
/// `unavailable` outcome refuses with — declared in `session` this closes the loop, declared in
/// `publish` it does not.
fn publish(error: &str) -> String {
    let own = if error == "relay.publish.StoreUnavailable" {
        "errors:\n  - name: relay.publish.StoreUnavailable\n    summary: The session store did \
         not answer.\n"
    } else {
        ""
    };
    format!(
        "domain: relay.publish
types:
  - name: relay.publish.Channel
    kind: newtype
    of: String
events:
  - name: relay.publish.Published
    fields:
      - {{name: channel, type: relay.publish.Channel}}
commands:
  - name: relay.publish.Publish
    input:
      - {{name: channel, type: relay.publish.Channel}}
    outcomes:
      - name: published
        emits: [relay.publish.Published]
        payload:
          relay.publish.Published: {{channel: input.channel}}
      - name: unavailable
        external: the session store did not answer
        error: {error}
{own}"
    )
}

const SESSION: &str = "domain: relay.session
types:
  - name: relay.session.Session
    kind: struct
    fields:
      - {name: subscription, type: relay.subscription.Subscription}
errors:
  - name: relay.session.StoreUnavailable
    summary: The session store did not answer.
events:
  - name: relay.session.Opened
    fields:
      - {name: subscription, type: relay.subscription.Subscription}
commands:
  # Owed: no emitter generates a typed response, so the bounded context's stub carries a method.
  - name: relay.session.Open
    input:
      - {name: subscription, type: relay.subscription.Subscription}
    response:
      - {name: session, type: relay.session.Session}
    outcomes:
      - name: opened
        emits: [relay.session.Opened]
        payload:
          relay.session.Opened: {subscription: input.subscription}
      - name: refused
        external: the session store did not answer
        error: relay.session.StoreUnavailable
";

const SUBSCRIPTION: &str = "domain: relay.subscription
types:
  - name: relay.subscription.Subscription
    kind: struct
    fields:
      - {name: channel, type: relay.publish.Channel}
";

/// Outside every loop: it references the loop, nothing references it.
const AUDIT: &str = "domain: relay.audit
types:
  - name: relay.audit.Entry
    kind: struct
    fields:
      - {name: channel, type: relay.publish.Channel}
      - {name: session, type: relay.session.Session}
";

fn model(error: &str) -> EssIr {
    model_with(error, SUBSCRIPTION)
}

/// The model, with the bounded context `relay.subscription` given by the document `subscription`.
fn model_with(error: &str, subscription: &str) -> EssIr {
    let publish = publish(error);
    let documents = [
        ("system.yaml", SYSTEM),
        ("audit.yaml", AUDIT),
        ("publish.yaml", publish.as_str()),
        ("session.yaml", SESSION),
        ("subscription.yaml", subscription),
    ];
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (label, text) in documents {
        let raw = RawSpecFile::parse(text)
            .unwrap_or_else(|error| panic!("the fixture `{label}` is well formed: {error}"));
        sources.insert(label.to_owned(), text.to_owned());
        labels.push(label.to_owned());
        parsed.push((Source::new(label.to_owned()), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile_locating(&specification, &sources, &labels)
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn go_found() -> bool {
    let found = Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success());
    if !found {
        println!("skipped: no `go` on PATH, so the generated module was not built");
    }
    found
}

fn written(name: &str, synthesis: &Synthesis) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("go-domain-loops-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, artifact) in &synthesis.artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().expect("a file has a parent"))
            .expect("the scratch tree is writable");
        std::fs::write(path, &artifact.contents).expect("the scratch tree is writable");
    }
    directory
}

/// `go build ./...` and `go vet ./...` over the written module; the combined log on failure.
fn builds(directory: &Path) -> Result<(), String> {
    for verb in ["build", "vet"] {
        let output = Command::new("go")
            .args([verb, "./..."])
            .current_dir(directory)
            .env("GOFLAGS", "-mod=mod")
            .env("GOPROXY", "off")
            .env("GOWORK", "off")
            .output()
            .expect("go runs");
        if !output.status.success() {
            return Err(format!(
                "go {verb}: exit {:?}\n{}{}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    Ok(())
}

fn go_files_under<'a>(synthesis: &'a Synthesis, dir: &str) -> Vec<&'a str> {
    synthesis
        .artifacts
        .keys()
        .map(String::as_str)
        .filter(|path| {
            path.strip_prefix(dir)
                .and_then(|rest| rest.strip_prefix('/'))
                .is_some_and(|rest| {
                    !rest.contains('/')
                        && Path::new(rest).extension().is_some_and(|ext| ext == "go")
                })
        })
        .collect()
}

#[test]
fn a_loop_of_domain_references_synthesizes_a_go_module_that_builds() {
    let synthesis = synthesize_for(&model("relay.session.StoreUnavailable"), Target::Go)
        .expect("the looping model has a realizable Go target");
    // The domain outside the loop keeps a package of its own, exactly as today.
    assert_eq!(
        go_files_under(&synthesis, "types/audit"),
        ["types/audit/audit.go"],
        "a bounded context outside every loop keeps its own package"
    );
    // The three of the loop share one package, one file each, beside the package's own file.
    let shared = "types/publishsessionsubscription";
    assert_eq!(
        go_files_under(&synthesis, shared),
        [
            format!("{shared}/publish.go"),
            format!("{shared}/publishsessionsubscription.go"),
            format!("{shared}/session.go"),
            format!("{shared}/subscription.go"),
        ],
        "the bounded contexts of one loop share one package"
    );
    for domain in ["publish", "session", "subscription"] {
        assert!(
            go_files_under(&synthesis, &format!("types/{domain}")).is_empty(),
            "`{domain}` has no package of its own"
        );
    }
    let package = &synthesis.artifacts[&format!("{shared}/publishsessionsubscription.go")].contents;
    assert!(
        package.contains(
            "// Package publishsessionsubscription holds the bounded contexts `relay.publish`, \
             `relay.session`, `relay.subscription`, one file each."
        ),
        "the shared package names its loop:\n{package}"
    );
    // `session`'s owed behaviour is a method of the one stub the shared package declares.
    assert_eq!(
        package.matches("type Unimplemented struct{}").count(),
        1,
        "the shared package declares its stub once:\n{package}"
    );
    for domain in ["publish", "session", "subscription"] {
        let file = &synthesis.artifacts[&format!("{shared}/{domain}.go")].contents;
        assert!(
            !file.contains("type Unimplemented struct{}"),
            "`{domain}`'s file does not redeclare the stub:\n{file}"
        );
    }
    let session = &synthesis.artifacts[&format!("{shared}/session.go")].contents;
    assert!(
        session.contains("func (Unimplemented) Open("),
        "`session`'s file carries its owed method on the shared stub:\n{session}"
    );
    let again = synthesize_for(&model("relay.session.StoreUnavailable"), Target::Go)
        .expect("the looping model has a realizable Go target");
    assert!(
        synthesis.artifacts == again.artifacts,
        "emitting a looping model twice is byte-identical"
    );
    if !go_found() {
        return;
    }
    let directory = written("loop", &synthesis);
    let built = builds(&directory);
    let _ = std::fs::remove_dir_all(&directory);
    if let Err(log) = built {
        panic!("the generated Go module of a looping model does not build:\n{log}");
    }
}

#[test]
fn the_same_model_without_the_loop_keeps_one_package_per_domain_and_builds() {
    let synthesis = synthesize_for(&model("relay.publish.StoreUnavailable"), Target::Go)
        .expect("the loop-free model has a realizable Go target");
    for domain in ["audit", "publish", "session", "subscription"] {
        let dir = format!("types/{domain}");
        assert_eq!(
            go_files_under(&synthesis, &dir),
            [format!("{dir}/{domain}.go")],
            "one package per bounded context, as today"
        );
        let file = &synthesis.artifacts[&format!("{dir}/{domain}.go")].contents;
        assert!(
            file.contains(&format!("\npackage {domain}\n")),
            "`{dir}` is package `{domain}`:\n{file}"
        );
    }
    if !go_found() {
        return;
    }
    let directory = written("no-loop", &synthesis);
    let built = builds(&directory);
    let _ = std::fs::remove_dir_all(&directory);
    if let Err(log) = built {
        panic!("the generated Go module of a loop-free model does not build:\n{log}");
    }
}

/// Two bounded contexts of one loop declaring the same local name share a package, so one of the
/// two Go names is repaired, exactly as two names within one bounded context are.
const SUBSCRIPTION_WITH_CHANNEL: &str = "domain: relay.subscription
types:
  - name: relay.subscription.Channel
    kind: newtype
    of: String
  - name: relay.subscription.Subscription
    kind: struct
    fields:
      - {name: channel, type: relay.publish.Channel}
      - {name: feed, type: relay.subscription.Channel}
";

#[test]
fn a_name_two_bounded_contexts_of_one_loop_declare_is_repaired_and_the_module_builds() {
    let synthesis = synthesize_for(
        &model_with("relay.session.StoreUnavailable", SUBSCRIPTION_WITH_CHANNEL),
        Target::Go,
    )
    .expect("the looping model has a realizable Go target");
    let shared = "types/publishsessionsubscription";
    let publish = &synthesis.artifacts[&format!("{shared}/publish.go")].contents;
    let subscription = &synthesis.artifacts[&format!("{shared}/subscription.go")].contents;
    assert!(
        publish.contains("\ntype Channel struct {") && subscription.contains("\ntype Channel_ struct {"),
        "the first in name order keeps the name, the second is repaired:\n{publish}\n{subscription}"
    );
    if !go_found() {
        return;
    }
    let directory = written("repaired", &synthesis);
    let built = builds(&directory);
    let _ = std::fs::remove_dir_all(&directory);
    if let Err(log) = built {
        panic!("the generated Go module with a repaired name does not build:\n{log}");
    }
}
