//! Original byte/source/lineage admission must precede construction of any implementation.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    web_execution::{
        self as web,
        bundle::{self, Blob, Execution, Loaded, SourceDocument},
        Installation, Installed, Product, RunContext,
    },
    AdmittedSuite, AdvancingClock, ConformanceSuite, SuiteProvenance,
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
fn bundle(coverage: bool) -> (String, Vec<Blob>) {
    let spec = Specification::assemble([(
        Source::new("source.yaml"),
        RawSpecFile::parse(SOURCE).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let execution = if coverage {
        Execution::Coverage(
            ess_conformance::coverage_build::build(
                &ir,
                &[],
                ess_conformance::coverage::Scope::System,
                ess_conformance::coverage::Origins::Generated,
            )
            .unwrap(),
        )
    } else {
        let mut suite = ConformanceSuite::new(SuiteProvenance::of(&ir));
        suite.select_fresh_format_for(&ir);
        Execution::Ordinary(AdmittedSuite::from_suite(&suite).unwrap())
    };
    bundle::create(
        &[SourceDocument {
            path: "sources/0000.yaml".into(),
            text: SOURCE.into(),
        }],
        &execution,
    )
    .unwrap()
}
fn repin(manifest: &str, blob: &Blob) -> String {
    let mut value: serde_json::Value = serde_json::from_str(manifest).unwrap();
    let reference = if blob.path.starts_with("sources/") {
        &mut value["sources"][0]
    } else {
        &mut value["execution"]["file"]
    };
    reference["sha256"] = bundle::hash(&blob.bytes).into();
    reference["byte_length"] = blob.bytes.len().into();
    serde_json::to_string(&value).unwrap()
}
#[test]
fn original_sources_and_inputs_are_checked_before_factory_even_after_success() {
    FACTORIES.with(|n| n.set(0));
    for coverage in [false, true] {
        let (manifest, blobs) = bundle(coverage);
        let mut product = Product::<Guard>::new();
        let handle = product.load(&manifest, blobs.clone()).unwrap();
        let original = String::from_utf8(blobs[1].bytes.clone()).unwrap();
        assert_eq!(product.loaded(handle).unwrap().original_input(), original);
        let mut changed = blobs.clone();
        changed[1].bytes.push(b' ');
        assert_eq!(
            product.load(&manifest, changed).unwrap_err(),
            web::Error::InvalidBundle
        );
        assert!(product.loaded(handle).is_err());
        let mut changed = blobs.clone();
        changed[0].bytes = b"format: ess/999\n".to_vec();
        let repinned = repin(&manifest, &changed[0]);
        assert_eq!(
            product.load(&repinned, changed).unwrap_err(),
            web::Error::AdmissionRefused
        );
        let mut changed = blobs.clone();
        changed[1].bytes = vec![0xff];
        let repinned = repin(&manifest, &changed[1]);
        assert_eq!(
            product.load(&repinned, changed).unwrap_err(),
            web::Error::InvalidBundle
        );
    }
    assert_eq!(FACTORIES.with(Cell::get), 0);
}
#[test]
fn manifest_is_closed_and_rejects_duplicates_paths_and_wrong_abi() {
    let (manifest, blobs) = bundle(false);
    for changed in [
        manifest.replacen('{', "{\"format\":\"ess-conformance-browser/1\",", 1),
        manifest.replacen('{', "{\"trusted\":true,", 1),
        manifest.replace(
            "ess-conformance-browser-abi/1",
            "ess-conformance-browser-abi/2",
        ),
        manifest.replace("sources/0000.yaml", "../source.yaml"),
    ] {
        assert!(Loaded::admit(&changed, blobs.clone()).is_err());
    }
    let mut changed = blobs.clone();
    changed.last_mut().unwrap().bytes = b"{}".to_vec();
    assert!(Loaded::admit(&manifest, changed).is_err());
}
#[test]
fn no_ordinary_coverage_fabrication_and_handles_do_not_rebind() {
    let (manifest, blobs) = bundle(false);
    let mut product = Product::<Guard>::new();
    let first = product.load(&manifest, blobs.clone()).unwrap();
    assert_eq!(
        product.select(first, &[]).unwrap_err(),
        web::Error::SelectionNotAvailable
    );
    product.release(first).unwrap();
    assert!(product.loaded(first).is_err());
    let second = product.load(&manifest, blobs).unwrap();
    assert_ne!(first, second);
    assert!(product.loaded(first).is_err());
}
#[test]
fn nonces_are_consumed_on_factory_failure_and_capacity_never_evicts() {
    FACTORIES.with(|n| n.set(0));
    let (manifest, blobs) = bundle(false);
    let mut product = Product::<Guard>::new();
    let handle = product.load(&manifest, blobs).unwrap();
    for n in 1_u32..=65_536 {
        let mut nonce = [0; 16];
        nonce[..4].copy_from_slice(&n.to_le_bytes());
        assert_eq!(
            product.run(handle, nonce, 0).err(),
            Some(web::Error::InstallationRequired)
        );
    }
    let mut duplicate = [0; 16];
    duplicate[0] = 1;
    assert_eq!(
        product.run(handle, duplicate, 0).err(),
        Some(web::Error::InvalidFrame)
    );
    let mut fresh = [0; 16];
    fresh[..4].copy_from_slice(&65_537_u32.to_le_bytes());
    assert_eq!(
        product.run(handle, fresh, 0).err(),
        Some(web::Error::ResourceLimit)
    );
    assert_eq!(FACTORIES.with(Cell::get), 65_536);
}
#[test]
fn bounded_abi_admits_original_load_and_rejects_buffer_misuse() {
    FACTORIES.with(|n| n.set(0));
    let (manifest, blobs) = bundle(false);
    let frame = web::abi::load_request(1, &manifest, &blobs).unwrap();
    let mut bridge = web::abi::Bridge::<Guard>::new();
    let len = u32::try_from(frame.len()).unwrap();
    bridge.reserve(len).unwrap().copy_from_slice(&frame);
    let response = bridge.dispatch(len);
    assert_eq!(&response[..4], b"ESBW");
    assert_eq!(u32::from_le_bytes(response[12..16].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(response[20..24].try_into().unwrap()), 0);
    let response = bridge.dispatch(len);
    assert_eq!(u32::from_le_bytes(response[12..16].try_into().unwrap()), 5);
    assert!(bridge
        .reserve(u32::try_from(web::MAX_FRAME + 1).unwrap())
        .is_err());
    assert_eq!(FACTORIES.with(Cell::get), 0);
}

#[test]
fn full_coverage_lineage_survives_selection_and_missing_parent_refuses_before_installation() {
    FACTORIES.with(|n| n.set(0));
    let (manifest, blobs) = bundle(true);
    let mut product = Product::<Guard>::new();
    let mut handle = product.load(&manifest, blobs.clone()).unwrap();
    for count in 1..=3 {
        handle = product.select(handle, &[]).unwrap();
        let original = product.loaded(handle).unwrap().original_input();
        let input = ess_conformance::coverage::AdmittedInput::from_json(original).unwrap();
        assert_eq!(input.parents().len(), count);
        assert_eq!(
            input.parents().last().unwrap().original_json(),
            ess_conformance::coverage::AdmittedInput::from_json(
                std::str::from_utf8(&blobs[1].bytes).unwrap()
            )
            .unwrap()
            .selected()
            .original_json()
        );
    }
    let mut input: serde_json::Value =
        serde_json::from_str(product.loaded(handle).unwrap().original_input()).unwrap();
    input["parent_suites"].as_array_mut().unwrap().pop();
    let mut changed = blobs;
    changed[1].bytes = input.to_string().into_bytes();
    let repinned = repin(&manifest, &changed[1]);
    assert_eq!(
        product.load(&repinned, changed).err(),
        Some(web::Error::AdmissionRefused)
    );
    assert!(product.loaded(handle).is_err());
    assert_eq!(FACTORIES.with(Cell::get), 0);
}

#[test]
fn original_suite_unknown_fields_duplicate_keys_and_bad_provenance_are_rejected() {
    FACTORIES.with(|n| n.set(0));
    let (manifest, blobs) = bundle(false);
    let original = std::str::from_utf8(&blobs[1].bytes).unwrap();
    let variants = [
        original.replacen('{', "{\"trusted\":true,", 1),
        original.replacen('{', "{\"scenarios\":{},", 1),
        original.replace("\"system\": \"browser\"", "\"system\": \"forged\""),
    ];
    for variant in variants {
        assert_ne!(variant, original);
        let mut changed = blobs.clone();
        changed[1].bytes = variant.into_bytes();
        let repinned = repin(&manifest, &changed[1]);
        let mut product = Product::<Guard>::new();
        assert!(product.load(&repinned, changed).is_err());
    }
    assert_eq!(FACTORIES.with(Cell::get), 0);
}
