//! Adversary pass 1 for beyond10x/ess#389 slice 2: the receipt against design section 8, and the
//! source-count bound shared by the Rust host and the JavaScript player against section 4.
#[allow(dead_code, unused_imports, clippy::all, clippy::pedantic)]
#[path = "../../../edge/ess-cli/tests/support/browser.rs"]
mod browser;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    web_execution::{
        self as web,
        bundle::{self, Blob, Execution, Loaded, SourceDocument},
        Installation, Installed, Product, RunContext,
    },
    AdmittedSuite, AdvancingClock, ConformanceSuite, RunnerConfig, SuiteProvenance,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use std::{fs, path::PathBuf};

const SOURCE: &str = "format: ess/1\nsystem: browser\nversion: v1\ndomain: browser.test\n";

struct Reference;
impl Installation for Reference {
    type Target = ess_conformance::reference::Billing;
    type Clock = AdvancingClock;
    fn create(_: &RunContext) -> web::Result<Installed<Self::Target, Self::Clock>> {
        Ok(Installed {
            target: ess_conformance::reference::Billing::default(),
            clock: AdvancingClock::default(),
            config: RunnerConfig::default(),
        })
    }
}

fn sources() -> Vec<SourceDocument> {
    vec![SourceDocument {
        path: "sources/0000.yaml".into(),
        text: SOURCE.into(),
    }]
}
fn ordinary() -> Execution {
    let spec = Specification::assemble([(
        Source::new("sources/0000.yaml"),
        RawSpecFile::parse(SOURCE).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(&ir));
    suite.select_fresh_format_for(&ir);
    Execution::Ordinary(AdmittedSuite::from_suite(&suite).unwrap())
}

/// Design section 8: "The product display/receipt context binds source/input/selected digests,
/// namespace, installation/build identity, generation and complete/aborted state". The receipt
/// this unit wraps a completed run in names the selected and parent digests only; the original
/// source documents the run's model was compiled from are not bound to it.
#[test]
fn adversary_completed_receipt_binds_every_original_source_digest() {
    let (manifest, blobs) = bundle::create(&sources(), &ordinary()).unwrap();
    let declared: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    let source_digests: Vec<String> = declared["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|source| source["sha256"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(source_digests.len(), 1);
    let mut product = Product::<Reference>::new();
    let handle = product.load(&manifest, blobs).unwrap();
    let completed = product.run(handle, [7; 16], 3).unwrap();
    let display: serde_json::Value = serde_json::from_str(&completed.display).unwrap();
    let receipt = display["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pair| pair[0] == "receipt")
        .expect("a completed display carries its receipt")[1]
        .to_string();
    for digest in &source_digests {
        assert!(
            receipt.contains(digest.as_str()),
            "the receipt does not bind original source {digest}: {receipt}"
        );
    }
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("adv389s2-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// Design section 4 and the emitted README: the source count bound is 1,024 and "anything over is
/// a `resource_limit`". Rust admission of a manifest declaring 1,025 sources says `resource_limit`;
/// the JavaScript player refuses the identical manifest bytes as `invalid_bundle`, so the two
/// halves of the product name the same over-budget input differently.
#[test]
fn adversary_player_and_rust_agree_a_1025_source_manifest_is_a_resource_limit() {
    let root = scratch("sources");
    let site = root.join("site");
    let files = ess_conformance::web::emit_product(&sources(), &ordinary()).unwrap();
    for (path, artifact) in files {
        let output = site.join(path);
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::write(output, artifact.contents).unwrap();
    }
    let original = fs::read_to_string(site.join("browser.json")).unwrap();
    let mut manifest: serde_json::Value = serde_json::from_str(&original).unwrap();
    let first = manifest["sources"][0].clone();
    let list = manifest["sources"].as_array_mut().unwrap();
    for index in 1..=web::MAX_SOURCES {
        let mut extra = first.clone();
        extra["path"] = format!("sources/extra-{index:04}.yaml").into();
        list.push(extra);
    }
    assert_eq!(list.len(), web::MAX_SOURCES + 1);
    let over = serde_json::to_string(&manifest).unwrap();
    assert!(over.len() <= web::MAX_MANIFEST, "{}", over.len());

    // Rust half: the same manifest bytes are a resource limit.
    let blobs: Vec<Blob> = Vec::new();
    assert_eq!(
        Loaded::admit(&over, blobs).err(),
        Some(web::Error::ResourceLimit),
        "Rust admission of 1,025 declared sources"
    );

    let over_site = root.join("over");
    for (path, artifact) in ess_conformance::web::emit_product(&sources(), &ordinary()).unwrap() {
        let output = over_site.join(path);
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::write(output, artifact.contents).unwrap();
    }
    fs::write(over_site.join("browser.json"), &over).unwrap();

    let evidence = root.join("firefox");
    fs::create_dir_all(&evidence).unwrap();
    let mut firefox = browser::Browser::new(&evidence);
    let mut observed = serde_json::Map::new();
    for (name, directory) in [("exact", &site), ("over", &over_site)] {
        let server = browser::Server::new(directory);
        let context = firefox.open(&format!("{}/index.html", server.url));
        let page = firefox.evaluate(
            &context,
            r"(async()=>{
            const until=performance.now()+10000;
            while(!document.body.dataset.ready&&!document.body.dataset.error&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
            return JSON.stringify({ready:document.body.dataset.ready??null,error:document.getElementById('error').textContent});
        })()",
        );
        observed.insert(name.into(), page);
    }
    drop(firefox);
    let observed = serde_json::Value::Object(observed);
    fs::write(root.join("observed.json"), observed.to_string()).unwrap();
    assert_eq!(observed["exact"]["ready"], "true", "{observed}");
    assert_eq!(
        observed["over"]["error"], "resource_limit",
        "the player must name 1,025 sources as Rust does: {observed}"
    );
}
