//! The boundary between a start this runner never delivered and a `BiDi` defect,
//! held at the `WebSocket` upgrade — the one startup site whose give-up is
//! decided by what the far end of the socket did rather than by a timer.
//!
//! Every way `Browser` can fail to obtain a started Firefox produces a fixture
//! environment refusal carrying the stage, the measured startup time and
//! `firefox.stderr`; `STARTUP_DEADLINE` bounds how long that can take; and the
//! deliberate exception is a browser that answered, which stays a defect signal
//! because a browser that answered is a browser that started. These cases arrived
//! from the adversary that falsified all three.
// The fixture module is shared by five test binaries; this one drives the
// startup path and reaches none of the page-level helpers.
#[allow(dead_code)]
#[path = "support/browser.rs"]
mod browser;

use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// A stand-in for Firefox that announces a `BiDi` endpoint on stderr in exactly
/// the shape the fixture's own scraper requires, and then stays alive.
fn announcing_stand_in(dir: &Path, port: u16) -> PathBuf {
    let script = dir.join("stand-in-firefox.sh");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\n\
             echo \"WebDriver BiDi listening on ws://127.0.0.1:{port}\" >&2\n\
             exec sleep 45\n"
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    script
}

/// A private evidence directory, made the way the unit's own cases make theirs.
fn evidence_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ess-browser-startup-adversary1-{name}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Run one startup on its own thread, so that a panic raised inside the fixture
/// is observable as data rather than ending the case with the fixture's message.
/// `Err` is a panic and its payload; `Ok` is the fixture's own outcome.
fn startup_outcome(
    evidence: PathBuf,
    program: PathBuf,
    deadline: Duration,
) -> Result<Result<(), String>, String> {
    std::thread::spawn(move || {
        browser::Browser::launch_program(&evidence, program.as_os_str(), deadline).map(|_| ())
    })
    .join()
    .map_err(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                payload
                    .downcast_ref::<&str>()
                    .map(|text| (*text).to_owned())
            })
            .unwrap_or_else(|| "<non-string panic payload>".to_owned())
    })
}

/// A Firefox killed after it announced `BiDi` — the runner ran out of memory, the
/// job hit its own timeout, the supervisor reaped it — leaves an accepted socket
/// that closes without answering the upgrade. The unit's table calls this "child
/// exited while connecting" and promises `stage: exited`.
#[test]
fn a_browser_that_closes_the_socket_mid_handshake_refuses_like_every_other_lost_start() {
    let evidence = evidence_dir("closed-socket");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        while let Ok((stream, _)) = listener.accept() {
            drop(stream);
        }
    });
    let program = announcing_stand_in(&evidence, port);
    match startup_outcome(evidence, program, browser::STARTUP_DEADLINE) {
        Err(panic) => panic!(
            "a start lost at the WebSocket upgrade must be a fixture environment refusal \
             carrying a stage, the measured startup time and firefox.stderr; the fixture \
             panicked instead, with none of them: {panic}"
        ),
        Ok(Ok(())) => panic!("a socket that never completed the upgrade admitted a session"),
        Ok(Err(refusal)) => assert!(
            refusal.starts_with("fixture environment refusal:"),
            "{refusal}"
        ),
    }
}

/// The story's acceptance is about a runner too slow to deliver a start. A
/// runner slow enough to accept the connection and not answer it is the same
/// class, and `STARTUP_DEADLINE` is supposed to be the bound on it.
#[test]
fn a_handshake_this_runner_never_answers_is_refused_at_the_deadline_it_was_given() {
    let evidence = evidence_dir("silent-socket");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let mut held = Vec::new();
        while let Ok((stream, _)) = listener.accept() {
            held.push(stream);
        }
    });
    let program = announcing_stand_in(&evidence, port);
    let deadline = Duration::from_secs(1);
    let started = Instant::now();
    let outcome = startup_outcome(evidence, program, deadline);
    let elapsed = started.elapsed();
    match outcome {
        Err(panic) => panic!(
            "a startup this runner never delivered must be a fixture environment refusal \
             carrying the measured startup time; after {:.3}s against a 1.000s deadline the \
             fixture panicked instead: {panic}",
            elapsed.as_secs_f64()
        ),
        Ok(Ok(())) => panic!("a socket that never answered admitted a session"),
        Ok(Err(refusal)) => {
            assert!(
                refusal.starts_with("fixture environment refusal:"),
                "{refusal}"
            );
            assert!(
                elapsed < deadline + Duration::from_secs(5),
                "the 1.000s deadline was overrun by {:.3}s before the refusal arrived:\n{refusal}",
                elapsed.saturating_sub(deadline).as_secs_f64()
            );
        }
    }
}

/// A browser that is running, listening and answering HTTP on the port it
/// announced has started. If it then answers the `BiDi` upgrade with anything
/// other than `101` forever, that is the `BiDi` defect the unit says it kept a
/// panic for — not a statement about the machine.
#[test]
fn a_handshake_answered_404_forever_stays_a_bidi_defect_and_is_not_blamed_on_the_runner() {
    let evidence = evidence_dir("permanent-404");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") && request.len() < 8192 {
                let mut byte = [0];
                if stream.read_exact(&mut byte).is_err() {
                    break;
                }
                request.push(byte[0]);
            }
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        }
    });
    let program = announcing_stand_in(&evidence, port);
    match startup_outcome(evidence, program, Duration::from_secs(2)) {
        // A panic is the signal the unit says it deliberately kept for this class.
        Err(_) => (),
        Ok(Ok(())) => panic!("a /session endpoint answering 404 admitted a session"),
        Ok(Err(refusal)) => panic!(
            "a browser that started, listened and answered HTTP 404 on /session was reported \
             as the runner's fault, in a message that denies being a BiDi defect:\n{refusal}"
        ),
    }
}
