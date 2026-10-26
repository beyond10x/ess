//! A lifecycle state's wire name (`ess/32`, beyond10x/ess#639) through the served Rust surface: a
//! view answers the state by its wire name, and an input of the state type decodes the wire name
//! and refuses the declared one.
//!
//! Read off the generated codecs rather than by building and running the served workspace: a test
//! that spawns Cargo turns its lane into a multi-hour one (`ess-xtask`'s `no_test_builds`). The
//! state is spelled in one place, the encoder and decoder of `<Entity>.State`, so the assertions are
//! that place and every route that reaches the state through it.

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_laid_out, OutputLayout, Target};

use super::artifact;

const SOURCE: &str = include_str!("../../../ess-gen/tests/fixtures/state-wire.yaml");

/// The body of the generated function whose signature opens with `signature`.
fn body<'a>(wire: &'a str, signature: &str) -> &'a str {
    let start = wire
        .find(signature)
        .unwrap_or_else(|| panic!("`{signature}` is generated:\n{wire}"));
    let rest = &wire[start..];
    let end = rest.find("\n}\n").expect("a generated function closes");
    &rest[..end]
}

#[test]
fn a_served_view_answers_a_state_by_its_wire_name() {
    let spec = Specification::assemble([(
        Source::new("jobs.yaml"),
        RawSpecFile::parse(SOURCE).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("jobs.yaml", SOURCE);
    let ir = compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let synthesis = synthesize_laid_out(&ir, Target::Rust, OutputLayout::Workspace)
        .unwrap_or_else(|error| panic!("synthesizes: {error:?}"));
    let wire = artifact(&synthesis, "jobs-server/src/wire.rs");

    let encode = body(wire, "pub fn encode_jobs_queue_job_state(");
    assert!(
        encode.contains("JobState::Indexing => json::push_text(out, \"indexing\"),"),
        "the state is written by its wire name:\n{encode}"
    );
    let decode = body(wire, "pub fn decode_jobs_queue_job_state(");
    assert!(
        decode.contains("\"indexing\" => jobs_types::queue::JobState::Indexing,"),
        "the wire name decodes:\n{decode}"
    );
    assert!(
        !decode.contains("\"Indexing\""),
        "the declared name is no wire value:\n{decode}"
    );

    let view = body(wire, "pub fn encode_view_jobs_queue_jobs(");
    assert!(
        view.contains("encode_jobs_queue_job_state(&value.state, out);"),
        "the view answers the state through its codec:\n{view}"
    );
    let input = body(wire, "pub fn decode_command_jobs_queue_finish(");
    assert!(
        input.contains("decode_jobs_queue_job_state("),
        "an input of the state type is read through its codec:\n{input}"
    );
}
