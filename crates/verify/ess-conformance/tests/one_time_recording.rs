//! Legacy histories cannot carry the observer's private capture lifetime or disclosure proof.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{
    record::{self, Call, Interleaved, Subject, Workload},
    recorded, sessions,
    target::*,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use std::{cell::Cell, collections::BTreeMap};

fn model(marked: bool) -> EssIr {
    let source = include_str!("../../../../docs/design/one-time-response-values.example.yaml");
    let text = if marked {
        source.to_owned()
    } else {
        source.replace("        one_time_response: [secret]\n", "")
    };
    let spec = Specification::assemble([(
        Source::new("recording.yaml"),
        RawSpecFile::parse(&text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

#[derive(Default)]
struct Counter(Cell<usize>);
impl Interleaved for Counter {
    type Pending = ();
    fn invoke(&self, _: SemanticCommandRequest) {
        self.0.set(self.0.get() + 1);
    }
    fn complete(&self, _: ()) -> Result<SemanticCommandResult, TargetError> {
        Err(TargetError::unavailable(
            "fixture",
            "sensitive target detail",
        ))
    }
}

fn call() -> Call {
    Call::new("credentials.api.Issue", BTreeMap::new(), Subject::Creates)
}

#[test]
fn command_recorder_refuses_before_any_target_callback() {
    let target = Counter::default();
    let result = record::record(
        &model(true),
        &target,
        &Workload {
            prefix: vec![call()],
            clients: vec![],
        },
        0,
    );
    let refusal = result.expect_err("one-time models require upfront history refusal");
    assert_eq!(target.0.get(), 0);
    assert!(refusal.to_string().contains("one-time"));
    assert!(!refusal.to_string().contains("sensitive target detail"));
}

#[test]
fn session_recorder_refuses_before_any_target_callback() {
    let target = Counter::default();
    let result = sessions::record_with(
        &model(true),
        &target,
        &ess_conformance::reference::Billing::new(),
        &sessions::Workload {
            prefix: vec![call()],
            clients: vec![],
        },
        0,
        sessions::FaultInjection::Declared,
    );
    let refusal = result.expect_err("one-time sessions require upfront history refusal");
    assert_eq!(target.0.get(), 0);
    assert!(refusal.to_string().contains("one-time"));
}

#[test]
fn model_aware_import_refuses_before_reading_sensitive_log_bytes() {
    let adapter = recorded::adapter(include_str!("fixtures/recorded/adapter.yaml")).unwrap();
    let refusal =
        recorded::import_for(b"sensitive target detail", &adapter, &model(true)).unwrap_err();
    assert_eq!(refusal.code(), "import.one-time-disclosure-unsupported");
    assert!(!refusal.to_string().contains("sensitive target detail"));
}

#[test]
fn unmarked_models_keep_the_existing_recording_path() {
    let target = Counter::default();
    let history = record::record(
        &model(false),
        &target,
        &Workload {
            prefix: vec![call()],
            clients: vec![],
        },
        0,
    )
    .unwrap();
    assert_eq!(target.0.get(), 1);
    assert_eq!(history.operations.len(), 1);
}
