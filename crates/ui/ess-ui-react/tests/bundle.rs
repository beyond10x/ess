//! The generated project builds with one esbuild step and serves with esbuild: the `build`
//! script type-checks and bundles `src/main.tsx` into `www/assets/`, which `www/index.html`
//! loads, and the `dev` server answers a page path with `index.html` so a reload or a
//! `page.goto` of any route reaches the app.
//!
//! The cases run the scripts `package.json` declares, against stand-in `react` and `react-dom`
//! packages written into the scratch project (with the project's own offline declarations as
//! their types), so no registry is needed. `esbuild` must be on `PATH`.

use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-bundle")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory is created");
    std::fs::write(path, text).expect("the file is written");
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

const MARKER: &str = "ess-ui-react stand-in react";

const REACT: &str = r#""use strict";
exports.marker = "ess-ui-react stand-in react";
exports.useState = (initial) => [typeof initial === "function" ? initial() : initial, () => undefined];
exports.useEffect = () => undefined;
exports.useLayoutEffect = () => undefined;
exports.useMemo = (factory) => factory();
exports.useCallback = (callback) => callback;
exports.useRef = (initial) => ({ current: initial });
exports.useContext = (context) => context._default;
exports.createContext = (value) => ({ _default: value, Provider: (props) => props.children });
exports.Fragment = (props) => props.children;
exports.StrictMode = (props) => props.children;
"#;

const JSX: &str = r#""use strict";
const React = require("./index.js");
const jsx = (type, props, key) => ({ type, props: props || {}, key: key === undefined ? null : key });
module.exports = { jsx, jsxs: jsx, Fragment: React.Fragment };
"#;

const DOM: &str = r#""use strict";
exports.createRoot = () => ({ render() {}, unmount() {} });
"#;

/// The example project with stand-in `react` and `react-dom` packages under `node_modules/`.
fn project(name: &str) -> PathBuf {
    let out = scratch(name);
    ess_ui_react::generate_path(&root().join("examples/partner-portal/ui.yaml"), &out)
        .unwrap_or_else(|error| panic!("{error}"));
    let modules = out.join("node_modules");
    let types = |file: &str| read(&out.join("types").join(file));
    write(
        &modules.join("react/package.json"),
        r#"{"name":"react","main":"index.js","types":"index.d.ts"}"#,
    );
    write(&modules.join("react/index.js"), REACT);
    write(&modules.join("react/index.d.ts"), &types("react.d.ts"));
    write(&modules.join("react/jsx-runtime.js"), JSX);
    write(
        &modules.join("react/jsx-runtime.d.ts"),
        &types("react-jsx-runtime.d.ts"),
    );
    write(
        &modules.join("react-dom/package.json"),
        r#"{"name":"react-dom","main":"client.js"}"#,
    );
    write(&modules.join("react-dom/client.js"), DOM);
    write(
        &modules.join("react-dom/client.d.ts"),
        &types("react-dom-client.d.ts"),
    );
    out
}

fn script(project: &Path, name: &str) -> String {
    let package: serde_json::Value =
        serde_json::from_str(&read(&project.join("package.json"))).expect("package.json parses");
    package["scripts"][name]
        .as_str()
        .unwrap_or_else(|| panic!("package.json has no `{name}` script"))
        .to_owned()
}

#[test]
fn the_build_script_bundles_with_esbuild_against_stub_react() {
    let project = project("build");
    let build = script(&project, "build");
    let output = Command::new("sh")
        .args(["-c", &build])
        .current_dir(&project)
        .output()
        .expect("sh runs");
    assert!(
        output.status.success(),
        "`{build}` failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let html = read(&project.join("www/index.html"));
    for (reference, file) in [
        ("src=\"/assets/main.js\"", "www/assets/main.js"),
        ("href=\"/assets/main.css\"", "www/assets/main.css"),
    ] {
        assert!(
            html.contains(reference),
            "index.html lacks {reference}:\n{html}"
        );
        assert!(project.join(file).is_file(), "the build wrote no {file}");
    }
    let js = read(&project.join("www/assets/main.js"));
    assert!(
        js.contains(MARKER),
        "the bundle does not include the react it was built against"
    );
    assert!(
        js.contains("data-ui-path"),
        "the bundle does not include the app"
    );
    assert!(
        !js.contains("react-router"),
        "the bundle mentions react-router"
    );
    let css = read(&project.join("www/assets/main.css"));
    assert!(
        css.contains(".ui-shell"),
        "the stylesheet main.tsx imports is not bundled"
    );
}

/// Kills the server when the case ends, pass or fail.
struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn get(port: u16, path: &str) -> (String, String) {
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut stream = loop {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => break stream,
            Err(error) if Instant::now() < deadline => {
                let _ = error;
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(error) => panic!("the dev server never listened on {port}: {error}"),
        }
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .expect("a read timeout");
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )
    .expect("the request is sent");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("the response is read");
    let response = String::from_utf8_lossy(&response).into_owned();
    let (head, body) = response
        .split_once("\r\n\r\n")
        .unwrap_or_else(|| panic!("no HTTP response: {response}"));
    let status = head.lines().next().unwrap_or_default().to_owned();
    (status, body.to_owned())
}

#[test]
fn the_dev_server_answers_a_page_path_with_index_html() {
    let project = project("dev");
    let dev = script(&project, "dev");
    let port = TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .expect("a free port")
        .port();
    let mut words = dev.split_whitespace();
    assert_eq!(words.next(), Some("esbuild"), "`dev` runs esbuild: {dev}");
    assert!(dev.contains("--serve=5173"), "`dev` serves on 5173: {dev}");
    let args: Vec<String> = words
        .map(|word| {
            if word.starts_with("--serve=") {
                format!("--serve=127.0.0.1:{port}")
            } else {
                word.to_owned()
            }
        })
        .collect();
    let _server = Server(
        Command::new("esbuild")
            .args(&args)
            .current_dir(&project)
            // esbuild stops serving when its stdin closes; the pipe stays open while the
            // child is held.
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("esbuild is on PATH"),
    );
    let index = read(&project.join("www/index.html"));
    let (status, body) = get(port, "/partners/detail/p-1");
    assert!(status.contains(" 200"), "a page path: {status}");
    assert_eq!(body, index, "a page path is answered with www/index.html");
    let (status, body) = get(port, "/assets/main.js");
    assert!(status.contains(" 200"), "the bundle: {status}");
    assert!(body.contains(MARKER), "the dev server bundles on request");
}
