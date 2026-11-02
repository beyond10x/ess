//! The boundary between a start this runner never delivered and a `BiDi` defect,
//! held at the `WebSocket` upgrade — the one startup site whose give-up is
//! decided by what the far end of the socket did rather than by a timer.
//!
//! Every way `Browser` can fail to obtain a started Firefox produces a fixture
//! environment refusal carrying the stage, the measured startup time and
//! `firefox.stderr`; `STARTUP_DEADLINE` bounds how long that can take; and the
//! deliberate exception is a malformed upgrade or protocol answer. A boot-time
//! HTTP 404 does not establish readiness, even when it repeats until the deadline.
// The fixture module is shared by five test binaries; this one drives the
// startup path and reaches none of the page-level helpers.
#[allow(dead_code)]
#[path = "support/browser.rs"]
mod browser;
#[allow(dead_code)]
#[path = "support/executable.rs"]
mod executable;

use ess_cli::TemporaryDirectory;
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// A stand-in for Firefox that announces a `BiDi` endpoint on stderr in exactly
/// the shape the fixture's own scraper requires, and then stays alive.
fn announcing_stand_in(dir: &Path, port: u16) -> PathBuf {
    let script = dir.join("stand-in-firefox.sh");
    executable::install_bytes(
        &script,
        format!(
            "#!/bin/sh\n\
             echo \"WebDriver BiDi listening on ws://127.0.0.1:{port}\" >&2\n\
             exec sleep 45\n"
        )
        .as_bytes(),
    )
    .unwrap();
    script
}

/// A private evidence directory, made the way the unit's own cases make theirs.
fn evidence_dir(name: &str) -> TemporaryDirectory {
    let dir =
        TemporaryDirectory::create(&format!("ess-browser-startup-adversary1-{name}")).unwrap();
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
    match startup_outcome(evidence.to_path_buf(), program, Duration::from_secs(1)) {
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
    let outcome = startup_outcome(evidence.to_path_buf(), program, deadline);
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
            // story:a-marked-region-is-not-a-scan-of-what-runs: an upgrade that never answered
            // is lost at the deadline, which is `connect`, not an upgrade error.
            assert!(
                refusal.contains("\n  stage:            connect\n"),
                "{refusal}"
            );
            assert!(refusal.contains("last upgrade error:"), "{refusal}");
        }
    }
}

/// A browser that completes the upgrade and then loses the socket on the first `session.new`
/// round trip has still not been started: the session is the last step of the start. The fixture
/// used to end such a start one call below `launch_program`, in `receive`'s bare `read_exact`
/// unwrap, with no stage and no `firefox.stderr`.
#[test]
fn a_socket_lost_on_the_first_session_round_trip_is_a_lost_start() {
    let evidence = evidence_dir("session-lost");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") && request.len() < 8192 {
            let mut byte = [0];
            if stream.read_exact(&mut byte).is_err() {
                return;
            }
            request.push(byte[0]);
        }
        let accepted = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n\
                        Connection: Upgrade\r\n\
                        Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
        let _ = stream.write_all(accepted.as_bytes());
        // Read the `session.new` frame's header and close without answering it.
        let mut header = [0; 2];
        let _ = stream.read_exact(&mut header);
    });
    let program = announcing_stand_in(&evidence, port);
    match startup_outcome(evidence.to_path_buf(), program, Duration::from_secs(5)) {
        Err(panic) => panic!(
            "story:a-marked-region-is-not-a-scan-of-what-runs: a socket lost on session.new \
             must be a fixture environment refusal carrying its stage and firefox.stderr; the \
             fixture panicked instead: {panic}"
        ),
        Ok(Ok(())) => panic!("a session.new that was never answered admitted a session"),
        Ok(Err(refusal)) => {
            assert!(
                refusal.starts_with("fixture environment refusal:"),
                "{refusal}"
            );
            assert!(
                refusal.contains("\n  stage:            session ("),
                "{refusal}"
            );
            let log = fs::read_to_string(evidence.join("firefox.stderr")).unwrap();
            assert!(refusal.contains(log.trim()), "{refusal}");
        }
    }
}

/// A booting browser can answer HTTP 404 before registering `/session`. Repeated
/// 404s never establish readiness and must retain the measured startup evidence.
#[test]
fn a_handshake_answered_404_forever_refuses_with_the_last_response_and_stderr() {
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
    match startup_outcome(evidence.to_path_buf(), program, Duration::from_secs(2)) {
        Err(panic) => panic!("a pre-upgrade 404 is not readiness: {panic}"),
        Ok(Ok(())) => panic!("a /session endpoint answering 404 admitted a session"),
        Ok(Err(refusal)) => {
            assert!(
                refusal.starts_with("fixture environment refusal:"),
                "{refusal}"
            );
            assert!(refusal.contains("measured startup:"), "{refusal}");
            assert!(refusal.contains("2.000s (expired)"), "{refusal}");
            assert!(refusal.contains("HTTP/1.1 404 Not Found"), "{refusal}");
            let log = fs::read_to_string(evidence.join("firefox.stderr")).unwrap();
            assert!(refusal.contains(log.trim()), "{refusal}");
        }
    }
}

/// A WebSocket close frame (opcode 8, status 1001 going away) on the first `session.new` is the
/// protocol's own way of losing the socket, not a malformed answer, so it is a lost start too.
/// Adversary pass 1 of D357 U2 (F6) found the fixture ending the start with its
/// `unexpected WebSocket frame` defect assert instead.
#[test]
fn a_close_frame_on_the_first_session_round_trip_is_a_lost_start() {
    let evidence = evidence_dir("close-frame");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") && request.len() < 8192 {
            let mut byte = [0];
            if stream.read_exact(&mut byte).is_err() {
                return;
            }
            request.push(byte[0]);
        }
        let accepted = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n\
                        Connection: Upgrade\r\n\
                        Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
        if stream.write_all(accepted.as_bytes()).is_err() {
            return;
        }
        // Read the whole masked `session.new` frame, then go away the way the protocol says to.
        let mut header = [0; 2];
        if stream.read_exact(&mut header).is_err() {
            return;
        }
        let mut rest = vec![0; 4 + usize::from(header[1] & 0x7f)];
        if stream.read_exact(&mut rest).is_err() {
            return;
        }
        let _ = stream.write_all(&[0x88, 0x02, 0x03, 0xe9]);
        std::thread::sleep(Duration::from_secs(2));
    });
    let program = announcing_stand_in(&evidence, port);
    match startup_outcome(evidence.to_path_buf(), program, Duration::from_secs(5)) {
        Err(panic) => panic!(
            "story:a-marked-region-is-not-a-scan-of-what-runs: a browser that closed the \
             WebSocket with a close frame during the first session.new lost the start, and \
             the fixture panicked as a protocol defect instead of refusing with a stage and \
             firefox.stderr: {panic}"
        ),
        Ok(Ok(())) => panic!("a closed session admitted a browser"),
        Ok(Err(refusal)) => {
            assert!(
                refusal.starts_with("fixture environment refusal:"),
                "{refusal}"
            );
            assert!(
                refusal.contains("\n  stage:            session ("),
                "{refusal}"
            );
        }
    }
}

/// An error answer to the first `session.new` is a well-formed answer that the browser will not
/// start a session, so the start is lost and the refusal names the answer (D357 U2, F7).
#[test]
fn an_error_answer_to_the_first_session_new_is_a_lost_start_that_names_it() {
    let evidence = evidence_dir("session-error");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") && request.len() < 8192 {
            let mut byte = [0];
            if stream.read_exact(&mut byte).is_err() {
                return;
            }
            request.push(byte[0]);
        }
        let accepted = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n\
                        Connection: Upgrade\r\n\
                        Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
        if stream.write_all(accepted.as_bytes()).is_err() {
            return;
        }
        let mut header = [0; 2];
        if stream.read_exact(&mut header).is_err() {
            return;
        }
        let mut rest = vec![0; 4 + usize::from(header[1] & 0x7f)];
        if stream.read_exact(&mut rest).is_err() {
            return;
        }
        let body = br#"{"id":1,"type":"error","error":"session not created"}"#;
        let mut frame = vec![0x81, u8::try_from(body.len()).unwrap()];
        frame.extend_from_slice(body);
        let _ = stream.write_all(&frame);
        std::thread::sleep(Duration::from_secs(2));
    });
    let program = announcing_stand_in(&evidence, port);
    match startup_outcome(evidence.to_path_buf(), program, Duration::from_secs(5)) {
        Err(panic) => panic!(
            "story:a-marked-region-is-not-a-scan-of-what-runs: an error answer to session.new \
             is a lost start; the fixture panicked instead: {panic}"
        ),
        Ok(Ok(())) => panic!("an error answer admitted a session"),
        Ok(Err(refusal)) => {
            assert!(
                refusal.starts_with("fixture environment refusal:"),
                "{refusal}"
            );
            assert!(
                refusal.contains("\n  stage:            session (")
                    && refusal.contains("session not created"),
                "{refusal}"
            );
        }
    }
}

/// Completed non-404 responses are protocol evidence, not a missing startup.
#[test]
fn malformed_upgrade_responses_remain_protocol_defects() {
    for (name, response) in [
        ("http-200", "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n"),
        (
            "wrong-accept",
            "HTTP/1.1 101 Switching Protocols\r\nSec-WebSocket-Accept: wrong\r\n\r\n",
        ),
    ] {
        let evidence = evidence_dir(name);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                stream.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            stream.write_all(response.as_bytes()).unwrap();
        });
        let program = announcing_stand_in(&evidence, port);
        let outcome = startup_outcome(evidence.to_path_buf(), program, Duration::from_secs(2));
        server.join().unwrap();
        let Err(panic) = outcome else {
            panic!("a malformed upgrade was not a protocol defect: {outcome:?}")
        };
        assert!(panic.contains(response.trim()), "{panic}");
        assert!(!panic.contains("fixture environment refusal:"), "{panic}");
    }
}

/// Receiving partial bytes must not restart the absolute upgrade deadline.
#[test]
fn a_trickling_upgrade_header_cannot_extend_the_startup_deadline() {
    let evidence = evidence_dir("trickling-header");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (began, accepted_at) = std::sync::mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        began.send(Instant::now()).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        for _ in 0..8 {
            if stream.write_all(b"H").is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(250));
        }
    });
    let program = announcing_stand_in(&evidence, port);
    let deadline = Duration::from_millis(500);
    let outcome = startup_outcome(evidence.to_path_buf(), program, deadline);
    // Waiting for another test's startup lock is outside this start's deadline.
    let elapsed = accepted_at.recv().unwrap().elapsed();
    server.join().unwrap();
    let Ok(Err(refusal)) = outcome else {
        panic!("an incomplete header did not refuse: {outcome:?}")
    };
    assert!(refusal.contains("0.500s (expired)"), "{refusal}");
    assert!(
        elapsed < deadline + Duration::from_secs(1),
        "partial bytes extended a 0.500s startup to {elapsed:?}: {refusal}"
    );
}
