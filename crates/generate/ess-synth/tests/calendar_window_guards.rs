//! Generated Rust and Go behaviour and a calendar-window guard (beyond10x/ess#244 part b,
//! `docs/design/calendar-window-guards.md`).
//!
//! No generated behaviour evaluates a window, over an input, a stored instant or `now`: each command
//! deciding by one stays an owed method whose reason names the window, in both targets, rather than
//! a behaviour generated without it. A command of the same model without a window is still
//! generated.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const RELEASES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/calendar-windows.yaml");

const COMPONENTS: &str = "components:
  - component: release-service
    owns: {domains: [demo.releases]}
    accepts: {commands: [demo.releases.Schedule, demo.releases.Maintain, demo.releases.Prepare, demo.releases.Promote, demo.releases.Deploy]}
    publishes: {events: [demo.releases.Scheduled, demo.releases.MaintenanceAllowed, demo.releases.Prepared, demo.releases.Promoted, demo.releases.Deployed]}
";

fn ir() -> EssIr {
    let text = format!("{RELEASES}{COMPONENTS}");
    let spec = Specification::assemble([(
        Source::new("releases.yaml"),
        RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn window_commands_stay_owed_naming_the_window_in_both_targets() {
    let ir = ir();
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target)
            .unwrap_or_else(|error| panic!("{target:?} synthesizes: {error:?}"));
        for (command, at) in [
            ("demo.releases.Schedule", "at starts_at"),
            ("demo.releases.Maintain", "at requested_at"),
            ("demo.releases.Promote", "at ready_at"),
            ("demo.releases.Deploy", "at now"),
        ] {
            match synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, command)
            {
                Some(SynthesisDisposition::Obligation(obligation)) => {
                    let reason = serde_json::to_string(&obligation.reason).unwrap();
                    assert!(
                        reason.contains("a calendar window no generated guard evaluates")
                            && reason.contains(at),
                        "{target:?} {command}: {reason}"
                    );
                }
                other => panic!("{target:?} {command} is owed, not {other:?}"),
            }
        }
        assert!(
            synthesis
                .plan
                .is_generated(CapabilityKind::CommandBehavior, "demo.releases.Prepare"),
            "{target:?}: a command without a window is still generated"
        );
    }
}
