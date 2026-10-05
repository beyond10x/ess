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
use std::fmt::Write as _;
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
    bundle::create(
        &[SourceDocument {
            path: "sources/0000.yaml".into(),
            text: SOURCE.into(),
        }],
        &execution(coverage),
    )
    .unwrap()
}
fn execution(coverage: bool) -> Execution {
    let spec = Specification::assemble([(
        Source::new("source.yaml"),
        RawSpecFile::parse(SOURCE).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    if coverage {
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
    }
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

/// A specification split over `count` original documents: one system document naming
/// `count - 1` domains and one document per domain, with the ordinary suite it obliges.
fn split_specification(count: usize) -> (Vec<SourceDocument>, Execution) {
    let mut system = String::from("format: ess/1\nsystem: browser\nversion: v1\ndomains:\n");
    let mut texts = Vec::new();
    for index in 1..count {
        writeln!(system, "  - browser.d{index:04}").unwrap();
        texts.push(format!("domain: browser.d{index:04}\n"));
    }
    texts.insert(0, system);
    let sources: Vec<SourceDocument> = texts
        .iter()
        .enumerate()
        .map(|(index, text)| SourceDocument {
            path: format!("sources/{index:04}.yaml"),
            text: text.clone(),
        })
        .collect();
    let borrowed: Vec<&str> = texts.iter().map(String::as_str).collect();
    let spec = Specification::assemble(
        RawSpecFile::parse_all(&borrowed)
            .into_iter()
            .zip(&sources)
            .map(|(raw, source)| (Source::new(source.path.clone()), raw.unwrap())),
    )
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(&ir));
    suite.select_fresh_format_for(&ir);
    (
        sources,
        Execution::Ordinary(AdmittedSuite::from_suite(&suite).unwrap()),
    )
}

fn dispatch(bridge: &mut web::abi::Bridge<Guard>, frame: &[u8]) -> (u32, u32, Vec<u8>) {
    let len = u32::try_from(frame.len()).unwrap();
    bridge.reserve(len).unwrap().copy_from_slice(frame);
    let response = bridge.dispatch(len).to_vec();
    (
        u32::from_le_bytes(response[12..16].try_into().unwrap()),
        u32::from_le_bytes(response[20..24].try_into().unwrap()),
        response,
    )
}

/// Section 4's finite profile, bound by bound: each exact value is admitted (or reaches the next
/// check), and one byte or one item more is a `resource_limit` before any installation callback.
#[allow(clippy::too_many_lines)]
#[test]
fn resource_profile_admits_each_exact_bound_and_refuses_one_over() {
    use web::{MAX_FRAME, MAX_MANIFEST, MAX_PATH, MAX_SAFE_ERROR, MAX_SCENARIO_ID, MAX_SOURCES};
    FACTORIES.with(|n| n.set(0));
    let limit = web::Error::ResourceLimit;

    // Manifest: insignificant whitespace before the final LF reaches the exact bound.
    let (manifest, blobs) = bundle(false);
    let padded = |extra: usize| {
        let mut text = manifest.trim_end_matches('\n').to_owned();
        text.push_str(&" ".repeat(MAX_MANIFEST + extra - manifest.len()));
        text.push('\n');
        text
    };
    let (exact, over) = (padded(0), padded(1));
    assert_eq!(exact.len(), MAX_MANIFEST);
    Loaded::admit(&exact, blobs.clone()).expect("an exactly 256 KiB manifest is admitted");
    assert_eq!(Loaded::admit(&over, blobs.clone()).err(), Some(limit));
    let mut bridge = web::abi::Bridge::<Guard>::new();
    let (tag, status, _) = dispatch(
        &mut bridge,
        &web::abi::load_request(1, &exact, &blobs).unwrap(),
    );
    assert_eq!((tag, status), (1, 0), "exact manifest through the ABI");
    let (tag, status, _) = dispatch(
        &mut bridge,
        &web::abi::load_request(2, &over, &blobs).unwrap(),
    );
    assert_eq!(
        (tag, status),
        (5, limit as u32),
        "one byte over through the ABI"
    );

    // Path label: 1,024 UTF-8 bytes is admitted, 1,025 is a resource limit.
    let label = |length: usize| format!("sources/{}.yaml", "a".repeat(length - 13));
    assert_eq!(label(MAX_PATH).len(), MAX_PATH);
    for (length, admitted) in [(MAX_PATH, true), (MAX_PATH + 1, false)] {
        let created = bundle::create(
            &[SourceDocument {
                path: label(length),
                text: SOURCE.into(),
            }],
            &execution(false),
        );
        if admitted {
            let (manifest, blobs) = created.expect("an exactly 1,024-byte label is admitted");
            Loaded::admit(&manifest, blobs).unwrap();
        } else {
            assert_eq!(created.err(), Some(limit), "a 1,025-byte label");
        }
    }
    assert_eq!(bundle::validate_path(&label(MAX_PATH + 1)), Err(limit));

    // Original source documents: 1,024 are admitted, 1,025 are a resource limit.
    let (sources, ordinary) = split_specification(MAX_SOURCES);
    let (manifest, blobs) = bundle::create(&sources, &ordinary).expect("1,024 sources");
    Loaded::admit(&manifest, blobs).unwrap();
    let (sources, ordinary) = split_specification(MAX_SOURCES + 1);
    assert_eq!(bundle::create(&sources, &ordinary).err(), Some(limit));

    // Scenario-id control string: 4,096 bytes reaches id parsing, 4,097 is a resource limit.
    let (manifest, blobs) = bundle(true);
    let mut bridge = web::abi::Bridge::<Guard>::new();
    let (tag, status, _) = dispatch(
        &mut bridge,
        &web::abi::load_request(1, &manifest, &blobs).unwrap(),
    );
    assert_eq!((tag, status), (1, 0));
    let select = |id: u32, handle: u32, scenario: &str| {
        let mut payload = Vec::new();
        payload.extend_from_slice(&handle.to_le_bytes());
        payload.extend_from_slice(&1_u32.to_le_bytes());
        payload.extend_from_slice(&u32::try_from(scenario.len()).unwrap().to_le_bytes());
        payload.extend_from_slice(scenario.as_bytes());
        let mut frame = b"ESBW".to_vec();
        for word in [1, 0, 2, id, u32::try_from(payload.len()).unwrap()] {
            frame.extend_from_slice(&word.to_le_bytes());
        }
        frame.extend_from_slice(&payload);
        frame
    };
    let (tag, status, _) = dispatch(&mut bridge, &select(2, 1, &"a".repeat(MAX_SCENARIO_ID)));
    assert_eq!(tag, 5);
    assert_ne!(
        status, limit as u32,
        "an exactly 4,096-byte id reaches id parsing"
    );
    let (tag, status, _) = dispatch(&mut bridge, &select(3, 1, &"a".repeat(MAX_SCENARIO_ID + 1)));
    assert_eq!((tag, status), (5, limit as u32));

    // Whole frame: exactly 64 MiB is reserved, one byte more is not.
    let mut bridge = web::abi::Bridge::<Guard>::new();
    assert_eq!(
        bridge
            .reserve(u32::try_from(MAX_FRAME).unwrap())
            .unwrap()
            .len(),
        MAX_FRAME
    );
    assert_eq!(
        bridge.reserve(u32::try_from(MAX_FRAME + 1).unwrap()).err(),
        Some(limit)
    );

    // Safe error text: every closed category with its location fits the bound, and an actual
    // error response carries nothing beyond it.
    for error in [
        web::Error::InvalidFrame,
        web::Error::IncompatibleAbi,
        web::Error::InvalidBundle,
        web::Error::AdmissionRefused,
        web::Error::InvalidHandle,
        web::Error::SelectionNotAvailable,
        web::Error::InstallationRequired,
        web::Error::ResourceLimit,
        web::Error::ExecutionError,
        web::Error::InternalFailure,
    ] {
        assert!(error.code().len() + "$browser".len() <= MAX_SAFE_ERROR);
    }
    let mut bridge = web::abi::Bridge::<Guard>::new();
    let (tag, _, response) = dispatch(&mut bridge, &[0; 24]);
    assert_eq!(tag, 5);
    assert!(response.len() - 28 <= MAX_SAFE_ERROR + 8);
    assert_eq!(FACTORIES.with(Cell::get), 0);
}
