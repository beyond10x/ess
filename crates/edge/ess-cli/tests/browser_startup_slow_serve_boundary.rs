//! The third startup state: Firefox is listening on the endpoint it announced
//! and is too slow to *serve* `/session` before the deadline expires.
//!
//! The unit's boundary has two states in it — nothing listening at all (a
//! fixture environment refusal) and a browser answering HTTP forever (kept as a
//! `BiDi` defect signal). This file drives the state between them, which is the
//! one the story was filed about: `browser.rs` says in its own words that
//! "Firefox can listen before registering /session", so a healthy Firefox on a
//! starved runner answers `404` on `/session` while it boots and only then
//! becomes ready. A start that misses the deadline in that state is a slow start
//! on a shared runner, which the story's acceptance requires to be reported as a
//! fixture environment refusal carrying the measured startup time and
//! `firefox.stderr`.
// The fixture module is shared by six test binaries; this one drives the
// startup path and reaches none of the page-level helpers.
#[allow(dead_code)]
#[path = "support/browser.rs"]
mod browser;

use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
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
        "ess-browser-slow-serve-{name}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Read one HTTP request off an accepted socket, under a timeout of its own so
/// that a stand-in server never outlives the case that built it.
fn read_request(stream: &mut TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut request = Vec::new();
    while !request.ends_with(b"\r\n\r\n") && request.len() < 8192 {
        let mut byte = [0];
        if stream.read_exact(&mut byte).is_err() {
            return;
        }
        request.push(byte[0]);
    }
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

/// A listener that answers `404` on the first upgrade attempt — the state
/// `browser.rs:421` documents a booting Firefox as being in — and then accepts
/// every later connection and never answers it, which is what a process starved
/// of CPU looks like from the other end of the socket. Returns the port and the
/// count of accepted connections.
fn booting_then_starved(port_sink: &Arc<AtomicUsize>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let accepted = Arc::clone(port_sink);
    std::thread::spawn(move || {
        let mut held = Vec::new();
        while let Ok((mut stream, _)) = listener.accept() {
            read_request(&mut stream);
            if accepted.fetch_add(1, Ordering::SeqCst) == 0 {
                // Firefox is listening and has not registered /session yet.
                let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
            } else {
                // The runner never gave it the CPU to answer the next one.
                held.push(stream);
            }
        }
    });
    port
}

/// The story: a slow Firefox start on a shared runner is reported as a fixture
/// environment refusal. This is that start — the browser is up, it answered
/// while it was still registering `/session`, and the deadline then expired
/// mid-request because the runner starved it.
#[test]
#[ignore = "story:a-browser-that-answered-http-once-is-still-a-slow-start — one transient 404 sets a sticky discriminator, so a starved start panics with the story's own headline instead of refusing"]
fn a_browser_too_slow_to_serve_session_before_the_deadline_is_the_slow_start_the_story_is_about() {
    let evidence = evidence_dir("boot-then-starve");
    let accepted = Arc::new(AtomicUsize::new(0));
    let port = booting_then_starved(&accepted);
    let program = announcing_stand_in(&evidence, port);
    match startup_outcome(evidence, program, Duration::from_secs(3)) {
        Err(panic) => panic!(
            "a Firefox that was listening, answered 404 while it was still registering \
             /session and was then too slow to serve it before the 3.000s deadline is a slow \
             start on a shared runner, which the story requires to be a fixture environment \
             refusal; the fixture blamed BiDi instead, with the same headline the story was \
             filed to remove: {panic}"
        ),
        Ok(Ok(())) => panic!("a /session that was never served admitted a session"),
        Ok(Err(refusal)) => assert!(
            refusal.starts_with("fixture environment refusal:"),
            "{refusal}"
        ),
    }
}

/// The give-up the unit kept a panic for says the reader can decide between a
/// defective `/session` and a starved runner "by the response and the log
/// below". The response is below it. The log has to be too, or the sentence is
/// an instruction to read something that is not there — which is the position
/// the base message (`read firefox.stderr`) already left the reader in.
#[test]
#[ignore = "story:a-browser-that-answered-http-once-is-still-a-slow-start — the give-up tells the reader to decide by a log it does not print"]
fn the_kept_panic_carries_the_log_it_tells_the_reader_to_decide_by() {
    let evidence = evidence_dir("promised-log");
    let accepted = Arc::new(AtomicUsize::new(0));
    let port = booting_then_starved(&accepted);
    let program = announcing_stand_in(&evidence, port);
    let outcome = startup_outcome(evidence.clone(), program, Duration::from_secs(2));
    let log = fs::read_to_string(evidence.join("firefox.stderr")).unwrap();
    assert!(!log.is_empty(), "the stand-in wrote nothing to stderr");
    match outcome {
        Err(panic) => assert!(
            panic.contains(log.trim()),
            "the give-up says the reader decides by \"the response and the log below\" and \
             prints no firefox.stderr at all; the {} bytes the browser actually wrote were \
             {:?} and the message below carries none of them:\n{panic}",
            log.len(),
            log.trim()
        ),
        Ok(Ok(())) => panic!("a /session that was never served admitted a session"),
        Ok(Err(refusal)) => assert!(
            refusal.contains(log.trim()),
            "the refusal does not carry the stderr it says it carries:\n{refusal}"
        ),
    }
}

/// The startup loop retries a connection that is refused, and retries a `404`,
/// because both are states a browser passes through on its way up. A socket that
/// is accepted and closed is the same kind of state — the child is still alive
/// and the deadline still has its whole budget left — and it ends the start on
/// the first attempt instead.
#[test]
#[ignore = "review-result:adversary-wave24-unit1-pass-2 G3 — a refused connect and a 404 retry, an upgrade io error does not; pre-existing, reproduces at base, no story filed"]
fn a_socket_lost_while_the_child_is_alive_is_retried_like_every_other_state_on_the_way_up() {
    let evidence = evidence_dir("closed-once");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let accepted = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&accepted);
    std::thread::spawn(move || {
        while let Ok((stream, _)) = listener.accept() {
            counter.fetch_add(1, Ordering::SeqCst);
            drop(stream);
        }
    });
    let program = announcing_stand_in(&evidence, port);
    let deadline = Duration::from_secs(2);
    let _ = startup_outcome(evidence, program, deadline);
    assert!(
        accepted.load(Ordering::SeqCst) >= 2,
        "the fixture gave up on the first lost socket and made {} connection attempt(s) \
         against a {:.3}s deadline, while the child was still alive: a connection that is \
         refused and a response of 404 are both retried, and this state is not",
        accepted.load(Ordering::SeqCst),
        deadline.as_secs_f64()
    );
}

/// The upgrade socket's read and write timeouts are now the *remaining* startup
/// budget, and that socket is the one the `Browser` keeps for the rest of its
/// life. A browser that took most of its deadline to become ready therefore
/// carries whatever little was left of the deadline as the timeout on every
/// `BiDi` call it will ever make — starting with `session.new`, which
/// `launch_program` issues before it returns. On the slow runner the story is
/// about, that is the difference between a start that is reported and a start
/// that ends in a bare `WouldBlock` at `support/browser.rs:610`, reached from
/// `support/browser.rs:374`, a line inside the marked startup region.
///
/// No longer ignored. `reach_bidi` restores the socket's read and write timeouts to
/// `SESSION_TIMEOUT` on the successful return, so what this case measures is now a property of the
/// session rather than a leftover of the start. It is the case that fails if that restore is
/// removed: the browser here registers with 0.3s of a 3.0s budget left and then takes 0.8s to
/// answer, which no leftover budget can serve.
#[test]
fn a_browser_that_became_ready_late_does_not_inherit_the_leftover_deadline_as_its_call_timeout() {
    let evidence = evidence_dir("late-ready");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let deadline = Duration::from_secs(3);
    // Registered with 0.3s of the budget left: the browser is up, just slow,
    // which is the whole subject of the story.
    let registers_at = Duration::from_millis(2_700);
    let started = std::time::Instant::now();
    std::thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            read_request(&mut stream);
            if started.elapsed() < registers_at {
                let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
                continue;
            }
            let accepted = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n\
                            Connection: Upgrade\r\n\
                            Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
            if stream.write_all(accepted.as_bytes()).is_err() {
                continue;
            }
            // One masked client frame: session.new.
            let mut header = [0; 2];
            if stream.read_exact(&mut header).is_err() {
                continue;
            }
            let length = usize::from(header[1] & 0x7f);
            let mut mask = [0; 4];
            let mut payload = vec![0; length];
            if stream.read_exact(&mut mask).is_err() || stream.read_exact(&mut payload).is_err() {
                continue;
            }
            // A browser this slow to start is a browser this slow to answer.
            std::thread::sleep(Duration::from_millis(800));
            let body = br#"{"id":1,"type":"success","result":{}}"#;
            let mut frame = vec![0x81, u8::try_from(body.len()).unwrap()];
            frame.extend_from_slice(body);
            let _ = stream.write_all(&frame);
            return;
        }
    });
    let program = announcing_stand_in(&evidence, port);
    match startup_outcome(evidence, program, deadline) {
        Ok(Ok(())) => (),
        Ok(Err(refusal)) => assert!(
            refusal.starts_with("fixture environment refusal:"),
            "{refusal}"
        ),
        Err(panic) => panic!(
            "a browser that reached the WebSocket upgrade 2.700s into a 3.000s deadline kept \
             the 0.300s that were left as the read timeout on the session it then had to \
             create, and the start ended in a bare io panic with no stage, no measured \
             startup and no firefox.stderr, one call below a line inside the marked startup \
             region: {panic}"
        ),
    }
}

/// The control for the case above, and the whole of the difference between
/// them: the same stand-in, the same 0.800s answer to `session.new`, the same
/// 3.000s deadline — registered at once instead of late. This one is expected to
/// pass, and a pass here is what says the case above failed because of the
/// budget the socket inherited and not because the stand-in was slow.
#[test]
fn the_same_browser_answering_the_same_call_at_the_same_speed_succeeds_when_it_is_ready_early() {
    let evidence = evidence_dir("early-ready");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            read_request(&mut stream);
            let accepted = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n\
                            Connection: Upgrade\r\n\
                            Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
            if stream.write_all(accepted.as_bytes()).is_err() {
                continue;
            }
            let mut header = [0; 2];
            if stream.read_exact(&mut header).is_err() {
                continue;
            }
            let length = usize::from(header[1] & 0x7f);
            let mut mask = [0; 4];
            let mut payload = vec![0; length];
            if stream.read_exact(&mut mask).is_err() || stream.read_exact(&mut payload).is_err() {
                continue;
            }
            std::thread::sleep(Duration::from_millis(800));
            let body = br#"{"id":1,"type":"success","result":{}}"#;
            let mut frame = vec![0x81, u8::try_from(body.len()).unwrap()];
            frame.extend_from_slice(body);
            let _ = stream.write_all(&frame);
            return;
        }
    });
    let program = announcing_stand_in(&evidence, port);
    match startup_outcome(evidence, program, Duration::from_secs(3)) {
        Ok(Ok(())) => (),
        Ok(Err(refusal)) => panic!("the control start was refused:\n{refusal}"),
        Err(panic) => panic!("the control start panicked: {panic}"),
    }
}

/// The other half of the story's acceptance: "the three tests do not compete for
/// CPU during startup". `launch_program` releases the startup lock at
/// `support/browser.rs:338` and then issues `session.new` at
/// `support/browser.rs:346` — the first `BiDi` round trip of a cold start, and
/// the one that actually creates the session the caller asked for — outside it.
/// `peak_concurrent_startups()` cannot see that, because `InStartup` is scoped to
/// `reach_bidi` alone; this counts the overlap from the browser's own side
/// instead, which is where competing for CPU is visible.
#[test]
#[ignore = "story:the-startup-lock-does-not-cover-the-first-round-trip — the lock is released eight lines before session.new"]
fn three_fixtures_do_not_ask_three_browsers_to_create_a_session_at_the_same_time() {
    let root = evidence_dir("session-new-overlap");
    let in_flight = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let work = Duration::from_millis(500);
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let mut threads = Vec::new();
    for index in 0..3 {
        let evidence = root.join(index.to_string());
        fs::create_dir_all(&evidence).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let live = Arc::clone(&in_flight);
        let most = Arc::clone(&peak);
        std::thread::spawn(move || {
            while let Ok((mut stream, _)) = listener.accept() {
                read_request(&mut stream);
                let accepted = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n\
                                Connection: Upgrade\r\n\
                                Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
                if stream.write_all(accepted.as_bytes()).is_err() {
                    continue;
                }
                let mut header = [0; 2];
                if stream.read_exact(&mut header).is_err() {
                    continue;
                }
                let length = usize::from(header[1] & 0x7f);
                let mut mask = [0; 4];
                let mut payload = vec![0; length];
                if stream.read_exact(&mut mask).is_err() || stream.read_exact(&mut payload).is_err()
                {
                    continue;
                }
                // The browser is now doing the work session.new asks for.
                let live_now = live.fetch_add(1, Ordering::SeqCst) + 1;
                most.fetch_max(live_now, Ordering::SeqCst);
                std::thread::sleep(work);
                live.fetch_sub(1, Ordering::SeqCst);
                let body = br#"{"id":1,"type":"success","result":{}}"#;
                let mut frame = vec![0x81, u8::try_from(body.len()).unwrap()];
                frame.extend_from_slice(body);
                let _ = stream.write_all(&frame);
                return;
            }
        });
        let program = announcing_stand_in(&evidence, port);
        let barrier = Arc::clone(&barrier);
        threads.push(std::thread::spawn(move || {
            barrier.wait();
            browser::Browser::launch_program(
                &evidence,
                program.as_os_str(),
                Duration::from_secs(20),
            )
            .map(|_| ())
        }));
    }
    let started = std::time::Instant::now();
    let outcomes: Vec<_> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();
    let elapsed = started.elapsed();
    for outcome in &outcomes {
        assert!(
            outcome.is_ok(),
            "a stand-in browser was refused: {outcome:?}"
        );
    }
    assert_eq!(
        peak.load(Ordering::SeqCst),
        1,
        "{} of the three fixtures were creating their BiDi session at the same instant, so \
         three cold Firefoxes competed for CPU during startup after all; the startup lock is \
         released at support/browser.rs:338 and session.new is issued at :346",
        peak.load(Ordering::SeqCst)
    );
    assert!(
        elapsed >= (work * 3).saturating_sub(Duration::from_millis(200)),
        "three {:.3}s session.new calls finished in {:.3}s, so the fixture overlapped them",
        work.as_secs_f64(),
        elapsed.as_secs_f64()
    );
}
