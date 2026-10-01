//! Adversary pass 1 on the plain React output: the serve scripts as they are actually started
//! (a background job, a CI step, a container without `-i`: stdin is `/dev/null`), the address
//! they listen on, and a link to the page the browser is already on.

mod support;

use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn generated(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-plain")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    ess_ui_react::generate_path(&root().join("examples/partner-portal/ui.yaml"), &dir)
        .unwrap_or_else(|error| panic!("{error}"));
    dir
}

fn script(project: &Path, name: &str) -> String {
    let text = std::fs::read_to_string(project.join("package.json")).expect("package.json");
    let package: serde_json::Value = serde_json::from_str(&text).expect("package.json parses");
    package["scripts"][name]
        .as_str()
        .unwrap_or_else(|| panic!("package.json has no `{name}` script"))
        .to_owned()
}

/// The script with its port swapped for a free one on loopback; nothing else changes.
fn on_free_port(script: &str) -> (String, u16) {
    let port = TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .expect("a free port")
        .port();
    let words: Vec<String> = script
        .split_whitespace()
        .map(|word| {
            if word.starts_with("--serve=") {
                format!("--serve=127.0.0.1:{port}")
            } else {
                word.to_owned()
            }
        })
        .collect();
    (words.join(" "), port)
}

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn get(port: u16, path: &str) -> Option<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .ok()?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )
    .ok()?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).ok()?;
    let response = String::from_utf8_lossy(&response).into_owned();
    Some(response.lines().next().unwrap_or_default().to_owned())
}

/// Starts `name` the way `npm run <name> &` in a non-interactive shell starts it — stdin is
/// `/dev/null` — and asks for a page path for up to five seconds.
fn serves_without_stdin(name: &str) {
    let project = generated(name);
    let (command, port) = on_free_port(&script(&project, name));
    // `exec`, so the child is esbuild itself and the drop's kill leaves no orphan behind.
    let command = format!("exec {command}");
    let mut server = Server(
        Command::new("sh")
            .args(["-c", &command])
            .current_dir(&project)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("sh runs"),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        // Any HTTP answer shows the server is up: this project has no React installed, so the
        // dev script's on-request bundle answers 503 there; `bundle.rs` checks the 200.
        if let Some(status) = get(port, "/partners/detail/p-1") {
            assert!(status.starts_with("HTTP/1.1 "), "`{name}`: {status}");
            std::thread::sleep(Duration::from_secs(1));
            assert!(
                server.0.try_wait().expect("the server's state").is_none(),
                "`{name}` answered once, then exited without stdin"
            );
            assert!(
                get(port, "/partners/detail/p-1").is_some(),
                "`{name}` stopped answering a second after starting"
            );
            return;
        }
        if let Some(exit) = server.0.try_wait().expect("the server's state") {
            let mut stderr = String::new();
            let _ = server
                .0
                .stderr
                .take()
                .expect("stderr is piped")
                .read_to_string(&mut stderr);
            panic!(
                "`npm run {name}` started without stdin exits ({exit}) instead of serving:\n{stderr}"
            );
        }
        assert!(
            Instant::now() < deadline,
            "`{name}` never listened on {port}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn the_dev_script_keeps_serving_when_started_without_stdin() {
    serves_without_stdin("dev");
}

#[test]
fn the_preview_script_keeps_serving_when_started_without_stdin() {
    serves_without_stdin("preview");
}

/// Vite, which these scripts replace, listened on `localhost` unless told otherwise; esbuild's
/// `--serve=<port>` listens on every interface, so the app and its fixtures are on the LAN.
#[test]
fn the_serve_scripts_listen_on_loopback_only() {
    let project = generated("loopback");
    for name in ["dev", "preview"] {
        let text = script(&project, name);
        let serve = text
            .split_whitespace()
            .find_map(|word| word.strip_prefix("--serve="))
            .unwrap_or_else(|| panic!("`{name}` has no --serve: {text}"));
        assert!(
            serve.starts_with("127.0.0.1:") || serve.starts_with("localhost:"),
            "`{name}` listens on every interface (`--serve={serve}`): {text}"
        );
    }
}

/// react-router's `Link`, which this one replaces, replaces the entry when it points at the
/// location the browser is already on; pushing a duplicate makes Back a no-op for one press —
/// reached by clicking the navigation entry of the current page.
#[test]
fn a_link_to_the_current_location_replaces_instead_of_pushing() {
    let project = generated("same-link");
    support::compile(&project, &["runtime/router.tsx"]);
    support::node(
        &project,
        "same-link",
        r#"
const b = browser("/overview");
const { Router, Link } = out("runtime/router");
const root = React.__root(jsx(Router, { children: jsx(Link, { to: "/overview", children: "Overview" }) }));
root.settle();
const event = click(root.find((n) => n.tag === "a")[0]);
assert.strictEqual(event.defaultPrevented, true);
assert.deepStrictEqual(b.calls, [["replace", "/overview"]], "a link to the current location adds no history entry");
"#,
    );
}
