//! PTY integration tests for terminal lifecycle (IO-06).
//!
//! These tests spawn the real `gitmedit` binary inside a pseudo-terminal,
//! read the emitted byte stream, and assert that alternate screen escape
//! sequences are absent. Requires PTY allocation — marked `#[ignore]` for
//! CI environments that cannot allocate a PTY.
//!
//! Run explicitly: `cargo test -- --ignored`

#[test]
#[ignore] // Requires PTY allocation; run with: cargo test -- --ignored
fn no_alternate_screen_escape_sequences() {
    use portable_pty::{native_pty_system, CommandBuilder, PtySize};
    use std::io::{Read, Write};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tempfile::NamedTempFile;

    // Create a temp file for gitmedit to open
    let mut tmp = NamedTempFile::new().expect("failed to create temp file");
    writeln!(tmp, "test commit message").expect("failed to write temp file");
    tmp.flush().expect("failed to flush temp file");

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("failed to open PTY");

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_gitmedit"));
    cmd.arg(tmp.path());

    let mut child = pair.slave.spawn_command(cmd).expect("failed to spawn gitmedit in PTY");

    // Get writer and reader handles before dropping slave
    let mut writer = pair.master.take_writer().expect("failed to take PTY writer");
    let mut reader = pair.master.try_clone_reader().expect("failed to clone PTY reader");
    drop(pair.slave); // Close slave so reader will see EOF after child exits

    // Collect output bytes in a background thread. Use a timeout backstop so
    // the reader thread does not hang if the child never exits (macOS PTY
    // read_to_end blocks until master fd is closed).
    let collected: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let collected_clone = Arc::clone(&collected);
    let reader_thread = std::thread::spawn(move || {
        let mut buf = [0u8; 512];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    collected_clone.lock().unwrap().extend_from_slice(&buf[..n]);
                }
            }
        }
    });

    // Wait for the TUI to initialize (debug binary may take up to 2 seconds),
    // then send Escape to cancel and exit.
    std::thread::sleep(Duration::from_millis(1000));
    let _ = writer.write_all(b"\x1b");
    writer.flush().ok();

    // Give the app time to process Escape and call process::exit.
    std::thread::sleep(Duration::from_millis(500));

    // Kill the child process as a backstop in case Escape was not processed.
    // This ensures the test never hangs regardless of PTY timing.
    if let Err(e) = child.kill() {
        // kill() fails if the process already exited — that is the happy path.
        let _ = e;
    }
    let _ = child.wait();

    // Close writer so the reader thread sees EOF and exits.
    drop(writer);
    let _ = reader_thread.join();

    let output = collected.lock().unwrap().clone();
    let raw = String::from_utf8_lossy(&output);

    // Assert no EnterAlternateScreen (\x1b[?1049h) — IO-06
    assert!(
        !raw.contains("\x1b[?1049h"),
        "EnterAlternateScreen escape sequence found in output — IO-06 violated.\n\
         The binary must not emit \\x1b[?1049h. Check src/terminal.rs TerminalGuard::new()."
    );

    // Assert no LeaveAlternateScreen (\x1b[?1049l) — IO-06
    assert!(
        !raw.contains("\x1b[?1049l"),
        "LeaveAlternateScreen escape sequence found in output — IO-06 violated.\n\
         The binary must not emit \\x1b[?1049l. Check src/terminal.rs TerminalGuard::Drop."
    );
}
