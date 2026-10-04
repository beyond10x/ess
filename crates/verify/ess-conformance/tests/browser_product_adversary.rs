//! Adversary cases: forged execution kinds and forged lineage must refuse before any factory call.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    coverage::AdmittedInput,
    web_execution::{
        self as web,
        bundle::{self, Blob, Execution, Loaded, SourceDocument},
        Installation, Installed, Product, RunContext,
    },
    AdvancingClock,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use std::cell::Cell;

const SOURCE: &str = "format: ess/1\nsystem: browser\nversion: v1\ndomain: browser.test\n";
thread_local! {static FACTORIES:Cell<usize>=const{Cell::new(0)};}
struct Guard;
impl Installation for Guard {
    type Target = ess_conformance::reference::Billing;
    type Clock = AdvancingClock;
    fn create(_: &RunContext) -> web::Result<Installed<Self::Target, Self::Clock>> {
        FACTORIES.with(|n| n.set(n.get() + 1));
        Err(web::Error::InstallationRequired)
    }
}
fn sources() -> Vec<SourceDocument> {
    vec![SourceDocument {
        path: "sources/0000.yaml".into(),
        text: SOURCE.into(),
    }]
}
fn coverage() -> AdmittedInput {
    let spec = Specification::assemble([(
        Source::new("source.yaml"),
        RawSpecFile::parse(SOURCE).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    ess_conformance::coverage_build::build(
        &ir,
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap()
}
fn manifest_with(manifest: &str, kind: &str, path: &str, bytes: &[u8]) -> String {
    let mut value: serde_json::Value = serde_json::from_str(manifest).unwrap();
    value["execution"] = serde_json::json!({
        "kind": kind,
        "file": {"path": path, "sha256": bundle::hash(bytes), "byte_length": bytes.len()},
    });
    format!("{}\n", serde_json::to_string_pretty(&value).unwrap())
}

/// An ordinary-kind manifest naming coverage suite bytes, with the exact presentation Rust derives
/// for those bytes, is refused. Only the ordinary-kind coverage guard decides this case: every
/// other byte (sources, presentation, hashes, lengths, paths) is the emitted, admissible one.
#[test]
fn ordinary_kind_never_admits_coverage_suite_bytes_without_their_carrier() {
    FACTORIES.with(|n| n.set(0));
    let unfiltered = coverage();
    let filtered = unfiltered.select(&[]).unwrap();
    for input in [unfiltered, filtered] {
        let (manifest, blobs) =
            bundle::create(&sources(), &Execution::Coverage(input.clone())).unwrap();
        let suite = input.selected().original_json().as_bytes().to_vec();
        let forged = manifest_with(&manifest, "ordinary_suite", "suite.json", &suite);
        let changed = vec![
            blobs[0].clone(),
            Blob {
                path: "suite.json".into(),
                bytes: suite,
            },
            blobs[2].clone(),
        ];
        let mut product = Product::<Guard>::new();
        let refused = product.load(&forged, changed.clone());
        assert!(
            refused.is_err(),
            "coverage suite admitted as ordinary (parents {})",
            input.parents().len()
        );
        assert!(Loaded::admit(&forged, changed).is_err());
    }
    assert_eq!(FACTORIES.with(Cell::get), 0);
}

/// A coverage-kind manifest naming ordinary suite bytes is refused before any factory call.
#[test]
fn coverage_kind_never_wraps_an_ordinary_suite() {
    FACTORIES.with(|n| n.set(0));
    let spec = Specification::assemble([(
        Source::new("source.yaml"),
        RawSpecFile::parse(SOURCE).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let mut suite =
        ess_conformance::ConformanceSuite::new(ess_conformance::SuiteProvenance::of(&ir));
    suite.select_fresh_format_for(&ir);
    let ordinary = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
    let (manifest, blobs) =
        bundle::create(&sources(), &Execution::Ordinary(ordinary.clone())).unwrap();
    let bytes = ordinary.original_json().as_bytes().to_vec();
    let forged = manifest_with(&manifest, "coverage_input", "input.json", &bytes);
    let changed = vec![
        blobs[0].clone(),
        Blob {
            path: "input.json".into(),
            bytes,
        },
        blobs[2].clone(),
    ];
    let mut product = Product::<Guard>::new();
    assert_eq!(
        product.load(&forged, changed).err(),
        Some(web::Error::AdmissionRefused)
    );
    assert_eq!(FACTORIES.with(Cell::get), 0);
}

/// A three-level lineage whose parents are reordered, duplicated or replaced is refused before any
/// factory call, also after a prior Load and Run attempt from the same worker.
#[test]
fn forged_multi_level_lineage_refuses_before_factory_after_a_prior_run() {
    FACTORIES.with(|n| n.set(0));
    let mut input = coverage();
    for _ in 0..3 {
        input = input.select(&[]).unwrap();
    }
    assert_eq!(input.parents().len(), 3);
    let (manifest, blobs) =
        bundle::create(&sources(), &Execution::Coverage(input.clone())).unwrap();
    let mut product = Product::<Guard>::new();
    let handle = product.load(&manifest, blobs.clone()).unwrap();
    assert_eq!(
        product.run(handle, [7; 16], 0).err(),
        Some(web::Error::InstallationRequired)
    );
    assert_eq!(FACTORIES.with(Cell::get), 1);
    let original: serde_json::Value = serde_json::from_slice(&blobs[1].bytes).unwrap();
    let parents = original["parent_suites"].as_array().unwrap().clone();
    let forgeries: Vec<(&str, Vec<serde_json::Value>)> = vec![
        (
            "swapped",
            vec![parents[1].clone(), parents[0].clone(), parents[2].clone()],
        ),
        (
            "duplicated",
            vec![
                parents[0].clone(),
                parents[0].clone(),
                parents[1].clone(),
                parents[2].clone(),
            ],
        ),
        (
            "middle-dropped",
            vec![parents[0].clone(), parents[2].clone()],
        ),
        (
            "middle-replaced",
            vec![parents[0].clone(), parents[0].clone(), parents[2].clone()],
        ),
    ];
    for (label, forged_parents) in forgeries {
        let mut value = original.clone();
        value["parent_suites"] = serde_json::Value::Array(forged_parents);
        let bytes = serde_json::to_vec(&value).unwrap();
        let forged = manifest_with(&manifest, "coverage_input", "input.json", &bytes);
        let mut changed = blobs.clone();
        changed[1].bytes = bytes;
        assert!(
            product.load(&forged, changed).is_err(),
            "{label} lineage admitted"
        );
        assert!(
            product.loaded(handle).is_err(),
            "{label} kept prior authority"
        );
        assert_eq!(FACTORIES.with(Cell::get), 1, "{label} reached the factory");
    }
}

/// A run nonce stays consumed across a fresh Load in the same worker, through the byte ABI.
#[test]
fn abi_nonce_reuse_is_refused_after_a_fresh_load_without_a_factory_call() {
    FACTORIES.with(|n| n.set(0));
    let (manifest, blobs) = bundle::create(&sources(), &Execution::Coverage(coverage())).unwrap();
    let mut bridge = web::abi::Bridge::<Guard>::new();
    let dispatch = |bridge: &mut web::abi::Bridge<Guard>, frame: &[u8]| {
        let len = u32::try_from(frame.len()).unwrap();
        bridge.reserve(len).unwrap().copy_from_slice(frame);
        let response = bridge.dispatch(len).to_vec();
        (
            u32::from_le_bytes(response[12..16].try_into().unwrap()),
            u32::from_le_bytes(response[20..24].try_into().unwrap()),
            response,
        )
    };
    let run = |id: u32, handle: u32| {
        let mut frame = b"ESBW".to_vec();
        for word in [1_u32, 0, 3, id, 24, handle] {
            frame.extend_from_slice(&word.to_le_bytes());
        }
        frame.extend_from_slice(&[9; 16]);
        frame.extend_from_slice(&0_u32.to_le_bytes());
        frame
    };
    let (tag, status, _) = dispatch(
        &mut bridge,
        &web::abi::load_request(1, &manifest, &blobs).unwrap(),
    );
    assert_eq!((tag, status), (1, 0));
    let (tag, _, _) = dispatch(&mut bridge, &run(2, 1));
    assert_eq!(tag, 5);
    assert_eq!(FACTORIES.with(Cell::get), 1);
    let (tag, status, _) = dispatch(
        &mut bridge,
        &web::abi::load_request(3, &manifest, &blobs).unwrap(),
    );
    assert_eq!((tag, status), (1, 0));
    let (tag, status, _) = dispatch(&mut bridge, &run(4, 2));
    assert_eq!((tag, status), (5, web::Error::InvalidFrame as u32));
    assert_eq!(FACTORIES.with(Cell::get), 1);
}
