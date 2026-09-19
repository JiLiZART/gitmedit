//! PTY integration test for terminal lifecycle.
//!
//! Spawns the real `gitmedit` binary in a pseudo-terminal and asserts that it enters the
//! alternate screen with mouse capture and restores both on exit. Needs PTY allocation, so it
//! is `#[ignore]`d; run it with `cargo test -- --ignored`.

#[test]
#[ignore] // Requires PTY allocation; run with: cargo test -- --ignored
fn enters_alternate_screen_and_restores_terminal_on_exit() {
    use portable_pty::{CommandBuilder, PtySize, native_pty_system};
    use std::io::{Read, Write};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tempfile::NamedTempFile;

    let mut tmp = NamedTempFile::new().expect("failed to create temp file");
    writeln!(tmp, "test commit message").expect("failed to write temp file");
    tmp.flush().expect("failed to flush temp file");

    let pair = native_pty_system()
        .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
        .expect("failed to open PTY");

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_gitmedit"));
    cmd.arg(tmp.path());
    let mut child = pair.slave.spawn_command(cmd).expect("failed to spawn gitmedit in PTY");

    let mut writer = pair.master.take_writer().expect("failed to take PTY writer");
    let mut reader = pair.master.try_clone_reader().expect("failed to clone PTY reader");
    drop(pair.slave);

    // Collect output in a background thread; macOS PTY reads block until the master closes.
    let collected: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let collected_clone = Arc::clone(&collected);
    let reader_thread = std::thread::spawn(move || {
        let mut buf = [0u8; 512];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => collected_clone.lock().unwrap().extend_from_slice(&buf[..n]),
            }
        }
    });

    // Let the TUI start, then press Esc to cancel.
    std::thread::sleep(Duration::from_millis(1000));
    let _ = writer.write_all(b"\x1b");
    writer.flush().ok();
    std::thread::sleep(Duration::from_millis(500));

    // Backstop so the test never hangs; kill fails harmlessly if the child already exited.
    let _ = child.kill();
    let _ = child.wait();
    drop(writer);
    let _ = reader_thread.join();

    let output = collected.lock().unwrap().clone();
    let raw = String::from_utf8_lossy(&output);
    assert!(raw.contains("\x1b[?1049h"), "alternate screen was not entered");
    assert!(raw.contains("\x1b[?1049l"), "alternate screen was not left on exit");
    assert!(raw.contains("\x1b[?1000l"), "mouse capture was not disabled on exit");
}
