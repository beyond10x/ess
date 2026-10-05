//! Consumer-owned Cargo installation of the exact emitted Rust module.
/// Exact generated module: concrete installation, no hidden target or static callback.
pub const MODULE: &str = r#"// Include this emitted module unchanged in your consumer cdylib.
// The installation macro binds a concrete Target/Clock; it does not create either at startup.
pub use ess_conformance::web_execution::{Installation, Installed, RunContext, Error};
#[macro_export]
macro_rules! install_browser_target {
    ($installation:ty) => {
        #[cfg(target_arch = "wasm32")]
        mod ess_browser_exports {
            use std::cell::RefCell;
            use ess_conformance::web_execution::{abi::Bridge, ABI_VERSION};
            std::thread_local! { static HOST: RefCell<Bridge<$installation>> = RefCell::new(Bridge::new()); }
            #[unsafe(no_mangle)]
            pub extern "C" fn ess_browser_abi_version() -> u32 { ABI_VERSION }
            #[unsafe(no_mangle)]
            pub extern "C" fn ess_browser_reserve(len: u32) -> u32 {
                HOST.with(|host| host.borrow_mut().reserve(len)
                    .map_or(0, |bytes| bytes.as_mut_ptr() as u32))
            }
            #[unsafe(no_mangle)]
            pub extern "C" fn ess_browser_dispatch(len: u32) -> u32 {
                HOST.with(|host| host.borrow_mut().dispatch(len).as_ptr() as u32)
            }
            #[unsafe(no_mangle)]
            pub extern "C" fn ess_browser_response_len() -> u32 {
                HOST.with(|host| host.borrow().response().len() as u32)
            }
        }
    };
}
"#;
/// Example manifest; consumer supplies actual complete runtime and installation checkouts.
pub const MANIFEST: &str = r#"[package]
name = "consumer-conformance-browser"
version = "0.1.0"
edition = "2024"
publish = false
[lib]
crate-type = ["cdylib"]
[workspace]
[dependencies]
ess-conformance = { path = "REPLACE_WITH_RUNTIME_WORKSPACE/crates/verify/ess-conformance" }
consumer-installation = { path = "REPLACE_WITH_YOUR_INSTALLATION_CRATE" }
[profile.release]
debug = 0
incremental = false
"#;
/// Consumer's source example, outside generated ownership when copied into their own crate.
pub const LIBRARY: &str = r#"// Point to the exact emitted module; keep your installation outside generated owned output.
include!("REPLACE_WITH_EMITTED_DIRECTORY/rust/browser_host.rs");
install_browser_target!(consumer_installation::BrowserInstallation);
"#;
/// Honest static-navigation/build/run instructions, with no implicit dependency download.
pub const README: &str = r"# Browser conformance product

Serve this directory with your existing static HTTP server and open index.html. Exact declaration
navigation is available immediately. It is admitted at emission, not executed. Run shows
build_required until you supply runner.wasm with your independent concrete Rust target and clock.

Create a consumer-owned cdylib outside this generated directory. Adapt rust/Cargo.toml.example
and rust/lib.rs.example with actual paths to your complete local ESS runtime workspace and your
installation crate. Include rust/browser_host.rs unchanged. There is no crates.io or git-HEAD
assumption and no default reference target. Installation::create receives only a fresh run
namespace; it must supply its independent target, matching wall/budget clock and RunnerConfig.
No installation callback is permitted during module initialization or Load.

Install the wasm32-unknown-unknown toolchain and required dependencies explicitly beforehand.
Generate/review/retain your consumer Cargo.lock once with:

    cargo generate-lockfile --manifest-path <consumer>/Cargo.toml --offline

Build the exact emitted module with the bounded memory profile:

    CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUSTFLAGS='-C link-arg=--max-memory=536870912' cargo build --manifest-path <consumer>/Cargo.toml --locked --offline --target wasm32-unknown-unknown --release

Copy the resulting consumer .wasm artifact to runner.wasm beside index.html. Record rustc/cargo
versions, command exit, Cargo.lock hash, emitted module hash, exact runtime and installation
revision/content identity and WASM hash with your build evidence. Offline missing dependencies
or lock drift is a build failure; do not replace the module with a stub. The frame (64 MiB),
manifest (256 KiB), source count (1,024), path label (1,024 bytes), scenario id (4,096 bytes)
and linear memory (512 MiB) bounds hold at their exact values; anything over is a
resource_limit, never truncated. A run the 300-second watchdog ends is aborted with cleanup
unconfirmed and produces no report.

Connect runtime passes the original byte bundle and full lineage through Rust admission before
any factory/target callback. Ordinary Run executes the whole suite; scenario navigation never
creates subset coverage. Coverage selection is explicit and retains full original parents.
Only actual Runner ExecutedRun produces CountReport/CountRun downloads. JavaScript never parses
suite payloads or simulates source behavior. Legacy replay/1 remains a separate reading product.

Target and Clock must share time authority. Each Run receives a fresh namespace used for fixtures,
setup and scenario isolation. The synchronous interface is not an asynchronous network adapter.
Reset/navigation cannot roll back target effects. Abort/watchdog termination reports cleanup
unconfirmed, with no fabricated partial result. Your installation owns external cleanup leases.
One-time values stay within Rust target/runner execution; do not log them from your installation.
";
