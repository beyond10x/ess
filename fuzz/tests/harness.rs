use ess_specification_fuzz::{
    carrier::{self, Bundle, Document},
    observation::{self, Entry, Record, Stage, Writer},
    pipeline::{self, Control},
    regressions,
    replay::{self, Disposition},
    structured,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};
fn fresh(name: &str) -> PathBuf {
    replay::fresh_default().join(name)
}
fn bundle(label: &str, text: &str) -> Bundle {
    Bundle {
        documents: vec![Document {
            label: label.into(),
            text: text.into(),
        }],
    }
}
fn produce(root: &Path, control: Control) {
    let mut w = Writer::create(root).unwrap();
    pipeline::run(
        &mut w,
        Entry::ByteCarrier,
        regressions::encoded("system-types"),
        control,
    )
    .unwrap();
}
fn records(root: &Path) -> Vec<Record> {
    let mut f = fs::File::open(root.join("observations.frames")).unwrap();
    let mut out = vec![];
    while let Some(r) = observation::read_frame(&mut f).unwrap() {
        out.push(r)
    }
    out
}
fn replace(root: &Path, records: &[Record]) {
    let mut f = fs::File::create(root.join("observations.frames")).unwrap();
    for r in records {
        observation::write_frame(&mut f, r).unwrap()
    }
}
#[test]
fn closed_carrier_rejects_duplicate_unknown_nested_and_wrong_scalar_fields() {
    for input in [
        r#"{"documents":[],"documents":[]}"#,
        r#"{"documents":[],"extra":0}"#,
        r#"{"documents":[{"label":"a","text":"","text":"x"}]}"#,
        r#"{"documents":[{"label":"a","text":"","extra":0}]}"#,
        r#"{"documents":[{"label":[],"text":""}]}"#,
        r#"{"documents":[{"label":"a","text":{}}]}"#,
        r#"{"documents":{}}"#,
    ] {
        assert!(carrier::decode(input.as_bytes()).is_err(), "{input}");
    }
    assert!(carrier::decode(&[b' '; carrier::MAX_ENCODED + 1])
        .unwrap_err()
        .to_string()
        .contains("encoded"));
}
#[test]
fn carrier_enforces_exact_utf8_and_aggregate_bounds_without_reordering() {
    let mut b = bundle(&"é".repeat(32), &"x".repeat(carrier::MAX_TEXT));
    assert!(b.validate().is_ok());
    b.documents[0].label.push('a');
    assert!(b.validate().is_err());
    b.documents[0].label = "z/../../opaque".into();
    b.documents[0].text.push('a');
    assert!(b.validate().is_err());
    b.documents[0].text.pop();
    b.documents.push(Document {
        label: "a".into(),
        text: "x".repeat(carrier::MAX_TEXT),
    });
    assert!(b.validate().is_ok());
    b.documents.push(Document {
        label: "b".into(),
        text: "x".into(),
    });
    assert!(b.validate().is_err());
    b.documents.pop();
    assert_eq!(carrier::decode(&b.encode().unwrap()).unwrap(), b);
    assert_eq!(b.documents[0].label, "z/../../opaque");
    b.documents[1].label = b.documents[0].label.clone();
    assert!(b.validate().is_err());
    for n in [0, 9] {
        assert!(Bundle {
            documents: (0..n)
                .map(|i| Document {
                    label: i.to_string(),
                    text: String::new()
                })
                .collect()
        }
        .validate()
        .is_err());
    }
    let eight = Bundle {
        documents: (0..8)
            .map(|i| Document {
                label: i.to_string(),
                text: String::new(),
            })
            .collect(),
    };
    assert!(eight.validate().is_ok());
    assert!(bundle("", "").validate().is_err());
    let escaped = bundle("a", &"\u{0}".repeat(carrier::MAX_TEXT));
    assert!(escaped.validate().is_ok());
    assert!(escaped.encode().is_err());
}
#[test]
fn original_multi_document_seed_bytes_and_boundaries_are_pinned() {
    regressions::verify().unwrap();
    let b = carrier::decode(regressions::encoded("system-types")).unwrap();
    assert_eq!(b.documents.len(), 2);
    assert_eq!(b.documents[0].label, "system.yaml");
    assert!(b.documents[0].text.contains("name: demo.Code"));
    assert!(!b.documents[0].text.contains("domain: demo.core"));
    assert_eq!(b.documents[1].label, "core.yaml");
}
#[test]
fn sixteen_distinct_structured_sources_compile_through_every_production_dispatch() {
    let root = fresh("structured");
    let mut writer = Writer::create(&root).unwrap();
    let mut identities = BTreeSet::new();
    let mut families = BTreeSet::new();
    let mut owners = BTreeSet::new();
    for input in structured::vectors() {
        let b = structured::render(&input).unwrap();
        identities.insert(ess_specification_fuzz::digest(&b.encode().unwrap()));
        families.insert(input[2] % 4);
        owners.insert(input[1] & 1);
        assert_eq!(
            pipeline::run(&mut writer, Entry::Structured, &input, Control::None).unwrap(),
            observation::Outcome::Compiled
        );
    }
    assert_eq!(identities.len(), 16);
    assert_eq!(families.len(), 4);
    assert_eq!(owners.len(), 2);
    let s = observation::admit(&root).unwrap();
    assert_eq!(s.compiled, 16);
    assert_eq!(s.compiled_source_identities, identities);
    for stage in observation::downstream() {
        let c = &s.stages[&stage.key()];
        assert_eq!((c.started, c.success + c.refused), (16, 16));
    }
}
#[test]
fn structured_maximum_shape_stays_bounded_and_deterministic() {
    let input = vec![255; 65536];
    let b = structured::render(&input).unwrap();
    assert_eq!(b, structured::render(&input).unwrap());
    assert!(b.encode().unwrap().len() <= carrier::MAX_ENCODED);
    assert!(structured::render(&vec![0; 65537]).is_err());
    // Eight declarations plus a holder; eight union alternatives each; four constructors.
    let mut v = vec![255; 26];
    v[4] = 4;
    let b = structured::render(&v).unwrap();
    assert!(
        b.documents
            .iter()
            .map(|d| d.text.matches("  - name:").count())
            .sum::<usize>()
            <= 32
    );
}
#[test]
fn mandatory_seeds_compile_and_refusals_do_not_suppress_later_targets() {
    let root = fresh("mandatory");
    let mut w = Writer::create(&root).unwrap();
    for name in regressions::NAMES {
        assert_eq!(
            pipeline::run(
                &mut w,
                Entry::ByteCarrier,
                regressions::encoded(name),
                Control::None
            )
            .unwrap(),
            observation::Outcome::Compiled
        );
    }
    let s = observation::admit(&root).unwrap();
    assert_eq!((s.attempts, s.compiled), (3, 3));
    for stage in observation::downstream() {
        let c = &s.stages[&stage.key()];
        assert_eq!((c.started, c.success + c.refused), (3, 3));
    }
    assert!(s.stages["synthesis/go"].refused > 0);
}
#[test]
fn ordinary_docs_ir_refusal_continues_all_four_synthesis_targets() {
    let root = fresh("ordinary");
    produce(&root, Control::RefuseDocsIr);
    let s = observation::admit(&root).unwrap();
    assert_eq!(s.stages["docs-ir"].refused, 1);
    for target in observation::Target::ALL {
        assert_eq!(s.stages[&format!("synthesis/{}", target.name())].started, 1);
    }
}
#[test]
fn omission_translation_and_exit_zero_mutations_fail_independent_admission() {
    for control in [
        Control::SkipDocsIr,
        Control::SkipGo,
        Control::EraseBoundaries,
        Control::DropTerminal,
        Control::DropStage,
    ] {
        let root = fresh(control.name());
        let e = replay::child(
            Path::new(env!("CARGO_BIN_EXE_replay")),
            Entry::ByteCarrier,
            regressions::encoded("system-types"),
            &root,
            control,
            replay::DEADLINE,
        )
        .unwrap();
        assert_eq!(e.status, Some(0), "{control:?}: {}", e.detail);
        assert_eq!(
            e.disposition,
            Disposition::ObservationFailure,
            "{control:?}"
        );
        assert!(e.reaped);
    }
}
#[test]
fn panic_is_a_crash_with_exact_unfinished_stage_and_swallowing_fails_verifier() {
    let root = fresh("panic");
    let e = replay::child(
        Path::new(env!("CARGO_BIN_EXE_replay")),
        Entry::ByteCarrier,
        regressions::encoded("system-types"),
        &root,
        Control::PanicDocsIr,
        replay::DEADLINE,
    )
    .unwrap();
    assert_eq!(e.disposition, Disposition::Crash);
    assert_eq!(e.status, Some(101));
    assert!(e.reaped);
    assert!(matches!(
        records(&root.join("observations")).last(),
        Some(Record::StageStart {
            identity: Stage::DocsIr,
            ..
        })
    ));
    let control = fresh("swallowed");
    let o = Command::new(env!("CARGO_BIN_EXE_replay"))
        .args(["--control", "swallow-panic"])
        .arg(&control)
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(String::from_utf8(o.stderr)
        .unwrap()
        .contains("expected Crash, actual Completed"));
    let good = fresh("propagated");
    assert!(Command::new(env!("CARGO_BIN_EXE_replay"))
        .args(["--control", "panic-docs-ir"])
        .arg(good)
        .status()
        .unwrap()
        .success());
}
#[test]
fn duplicate_missing_out_of_order_overlapping_and_false_terminal_records_refuse() {
    let root = fresh("sequences");
    produce(&root, Control::None);
    let original = records(&root);
    assert!(observation::admit(&root).is_ok());
    let mut variants = Vec::new();
    let mut v = original.clone();
    v.insert(0, v[0].clone());
    variants.push(v);
    let mut v = original.clone();
    v.remove(1);
    variants.push(v);
    let mut v = original.clone();
    v.swap(1, 2);
    variants.push(v);
    let mut v = original.clone();
    v.insert(2, v[1].clone());
    variants.push(v);
    let mut v = original.clone();
    v.push(v.last().unwrap().clone());
    variants.push(v);
    let mut v = original.clone();
    if let Record::Finish { not_reached, .. } = v.last_mut().unwrap() {
        not_reached.push(Stage::DocsIr)
    }
    variants.push(v);
    let mut v = original.clone();
    v.pop();
    variants.push(v);
    for v in variants {
        replace(&root, &v);
        assert!(observation::admit(&root).is_err());
    }
    replace(&root, &original);
    assert!(observation::admit(&root).is_ok());
    let bytes = fs::read(root.join("observations.frames")).unwrap();
    for cut in [1, 3, bytes.len() - 1] {
        fs::write(root.join("observations.frames"), &bytes[..cut]).unwrap();
        assert!(observation::admit(&root).is_err());
    }
}
#[test]
fn frames_reject_oversize_malformed_unknown_fields_and_wrong_blob_bytes() {
    for bytes in [
        ((observation::MAX_RECORD + 1) as u32)
            .to_be_bytes()
            .to_vec(),
        vec![0, 0, 0, 0],
        vec![0, 0, 0, 1, b'{'],
    ] {
        assert!(observation::read_frame(&mut &bytes[..]).is_err());
    }
    let raw=br#"{"record":"finish","attempt":1,"outcome":{"outcome":"compiled"},"not_reached":[],"extra":0}"#;
    let mut bytes = (raw.len() as u32).to_be_bytes().to_vec();
    bytes.extend(raw);
    assert!(observation::read_frame(&mut &bytes[..]).is_err());
    let root = fresh("blob");
    produce(&root, Control::None);
    let first = records(&root).remove(0);
    let Record::Start { input, .. } = first else {
        panic!()
    };
    fs::write(root.join("blobs").join(&input.sha256), b"changed").unwrap();
    assert!(observation::admit(&root)
        .unwrap_err()
        .to_string()
        .contains("identity mismatch"));
    let invalid = observation::Blob {
        sha256: "../forbidden".into(),
        bytes: 1,
    };
    assert!(observation::read_blob(&root, &invalid)
        .unwrap_err()
        .to_string()
        .contains("invalid observation blob reference"));
}
struct Failing {
    flush: bool,
}
impl Write for Failing {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        if self.flush {
            Ok(b.len())
        } else {
            Err(io::Error::other("write failure"))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("flush failure"))
    }
}
#[test]
fn observation_write_and_flush_failures_are_errors_and_empty_stream_never_qualifies() {
    let record = Record::StageStart {
        attempt: 1,
        identity: Stage::Decode,
    };
    for flush in [false, true] {
        let e = observation::write_frame(&mut Failing { flush }, &record).unwrap_err();
        assert!(e.to_string().contains(if flush {
            "flush failure"
        } else {
            "write failure"
        }));
    }
    let root = fresh("empty");
    let w = Writer::create(&root).unwrap();
    assert!(w.blob(&vec![0; observation::MAX_BLOB + 1]).is_err());
    let s = observation::admit(&root).unwrap();
    assert_eq!(s.attempts, 0);
    assert!(observation::qualify_live(&s, Entry::ByteCarrier).is_err());
}
#[test]
fn watchdog_uses_ten_second_deadline_kills_and_reaps_the_actual_child() {
    let root = fresh("watchdog");
    let e = replay::supervised(
        Path::new(env!("CARGO_BIN_EXE_replay")),
        &["--stall"],
        regressions::encoded("system-types"),
        &root,
        "none",
        replay::DEADLINE,
    )
    .unwrap();
    assert_eq!(e.disposition, Disposition::Timeout);
    assert!(e.reaped);
    assert!(e.elapsed_millis >= 10_000);
    assert!(e.elapsed_millis < 15_000);
    #[cfg(target_os = "linux")]
    assert!(!Path::new("/proc").join(e.pid.unwrap().to_string()).exists());
    assert!(matches!(
        records(&root.join("observations")).last(),
        Some(Record::StageStart {
            identity: Stage::Decode,
            ..
        })
    ));
    assert!(observation::admit(&root.join("observations")).is_err());
}
#[test]
fn launch_failure_is_distinct_from_observation_failure() {
    let e = replay::child(
        Path::new("/nonexistent/ess-fuzz-replay"),
        Entry::ByteCarrier,
        b"",
        &fresh("launch"),
        Control::None,
        replay::DEADLINE,
    )
    .unwrap();
    assert_eq!(e.disposition, Disposition::LaunchFailure);
    assert_eq!(e.pid, None);
    assert!(!e.reaped);
    let root = fresh("io");
    let e = replay::supervised(
        Path::new(env!("CARGO_BIN_EXE_replay")),
        &["--child", "unknown-entry"],
        b"",
        &root,
        "none",
        replay::DEADLINE,
    )
    .unwrap();
    assert_eq!(e.status, Some(74));
    assert_eq!(e.disposition, Disposition::ObservationFailure);
}
fn file_map(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, at: &Path, m: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in fs::read_dir(at).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(root, &p, m)
            } else {
                m.insert(
                    p.strip_prefix(root).unwrap().to_owned(),
                    fs::read(&p).unwrap(),
                );
            }
        }
    }
    let mut map = BTreeMap::new();
    walk(root, root, &mut map);
    map
}
#[test]
fn all_nine_go_positive_artifact_maps_and_plans_match_the_pre_change_bytes() {
    let output = fresh("go-positive");
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_readiness_go"))
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let expected = file_map(&Path::new(env!("CARGO_MANIFEST_DIR")).join("readiness/expected"));
    assert_eq!(expected.len(), 136);
    assert_eq!(file_map(&output), expected);
}
#[test]
fn structured_rendered_sources_match_reviewed_exact_identities() {
    structured::verify_vectors().unwrap();
}
#[test]
fn live_callback_identity_cannot_be_replaced_by_replay_or_no_work() {
    let real = fresh("live-real");
    let mut session =
        ess_specification_fuzz::live::Session::create(&real, Entry::ByteCarrier, false).unwrap();
    session
        .callback(regressions::encoded("system-types"))
        .unwrap();
    drop(session);
    assert_eq!(
        ess_specification_fuzz::live::admit(&real, Entry::ByteCarrier)
            .unwrap()
            .compiled,
        1
    );
    let no_work = fresh("live-no-work");
    let mut session =
        ess_specification_fuzz::live::Session::create(&no_work, Entry::ByteCarrier, true).unwrap();
    session
        .callback(regressions::encoded("system-types"))
        .unwrap();
    drop(session);
    assert_eq!(observation::admit(&no_work).unwrap().attempts, 0);
    assert!(
        ess_specification_fuzz::live::admit(&no_work, Entry::ByteCarrier)
            .unwrap_err()
            .to_string()
            .contains("lacks its exact shared pipeline attempt")
    );
    let path = real.join("callbacks.frames");
    let original = fs::read(&path).unwrap();
    fs::write(&path, [original.clone(), original.clone()].concat()).unwrap();
    assert!(ess_specification_fuzz::live::admit(&real, Entry::ByteCarrier).is_err());
    fs::write(&path, &original[..original.len() - 1]).unwrap();
    assert!(ess_specification_fuzz::live::admit(&real, Entry::ByteCarrier).is_err());
    fs::write(&path, []).unwrap();
    assert!(ess_specification_fuzz::live::admit(&real, Entry::ByteCarrier).is_err());
}
#[test]
fn typed_compile_refusal_preserves_remaining_stages_without_claiming_a_source_failure() {
    let root = fresh("typed-compile-refusal");
    produce(&root, Control::None);
    let mut rs = records(&root);
    let result = rs
        .iter()
        .position(|r| {
            matches!(
                r,
                Record::StageResult {
                    identity: Stage::Compile,
                    ..
                }
            )
        })
        .unwrap();
    let Record::Start { input, .. } = &rs[0] else {
        panic!()
    };
    let detail = input.clone();
    rs[result] = Record::StageResult {
        attempt: 1,
        identity: Stage::Compile,
        result: observation::Returned::Refused { detail },
    };
    rs.truncate(result + 1);
    rs.push(Record::Finish {
        attempt: 1,
        outcome: observation::Outcome::CompileRefused,
        not_reached: observation::downstream(),
    });
    replace(&root, &rs);
    let summary = observation::admit(&root).unwrap();
    assert_eq!(summary.outcomes["compile-refused"], 1);
    assert_eq!(summary.compiled, 0);
    assert_eq!(summary.stages["compile"].refused, 1);
}
