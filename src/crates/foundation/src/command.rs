//! One bounded runner for external commands.
//!
//! Standard input is written to completion before the wait. A timeout terminates
//! through `process_control` (`waitid` on macOS and Linux). Output past 16 MiB
//! is counted with the crate's filters and rejected. `memory_limit` is not used:
//! that method is absent on macOS.

use std::io::{self, Write};
use std::process::{Command, Output, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use process_control::{ChildExt, Control};

/// Per-stream ceiling already used by provider commands.
pub const OUTPUT_LIMIT_BYTES: usize = 16 * 1024 * 1024;

/// What a bounded command did.
#[derive(Debug)]
#[must_use]
pub enum CommandEnd {
    /// The process exited. Bytes over the limit never reach this variant.
    Finished(Output),
    /// The time limit elapsed and the process was terminated.
    TimedOut,
    /// Standard output or standard error grew past [`OUTPUT_LIMIT_BYTES`].
    OutputLimit,
}

/// Runs `command` with a time limit.
///
/// When `input` is set, those bytes are written and the pipe is closed before
/// waiting. Either stream over [`OUTPUT_LIMIT_BYTES`] yields [`CommandEnd::OutputLimit`].
///
/// # Errors
///
/// Returns an error when the process cannot be spawned, standard input cannot
/// be written, or the limiter itself fails. A timeout is [`CommandEnd::TimedOut`],
/// not an error.
pub fn run_bounded(
    command: &mut Command,
    input: Option<&[u8]>,
    time_limit: Duration,
) -> io::Result<CommandEnd> {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn()?;
    if let Some(bytes) = input {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("child standard input is missing"))?;
        if let Err(error) = stdin.write_all(bytes) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    }
    drop(child.stdin.take());

    let stdout_bytes = Arc::new(AtomicUsize::new(0));
    let stderr_bytes = Arc::new(AtomicUsize::new(0));
    let stdout_seen = Arc::clone(&stdout_bytes);
    let stderr_seen = Arc::clone(&stderr_bytes);
    let output = child
        .controlled_with_output()
        .time_limit(time_limit)
        .terminate_for_timeout()
        .stdout_filter(move |chunk| Ok(count_within_limit(&stdout_seen, chunk)))
        .stderr_filter(move |chunk| Ok(count_within_limit(&stderr_seen, chunk)))
        .wait()?;
    if stdout_bytes.load(Ordering::Relaxed) > OUTPUT_LIMIT_BYTES
        || stderr_bytes.load(Ordering::Relaxed) > OUTPUT_LIMIT_BYTES
    {
        return Ok(CommandEnd::OutputLimit);
    }
    Ok(match output {
        Some(output) => CommandEnd::Finished(output.into_std_lossy()),
        None => CommandEnd::TimedOut,
    })
}

fn count_within_limit(counter: &AtomicUsize, chunk: &[u8]) -> bool {
    let total = counter.fetch_add(chunk.len(), Ordering::Relaxed) + chunk.len();
    total <= OUTPUT_LIMIT_BYTES
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use super::{CommandEnd, run_bounded};

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock follows the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "hctl2-command-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("temp dir");
        path
    }

    fn script(dir: &Path, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("cmd.sh");
        fs::write(&path, body).expect("script");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("mode");
        path
    }

    fn process_gone(pid: u32) -> bool {
        let end = run_bounded(
            Command::new("ps").args(["-p", &pid.to_string(), "-o", "pid="]),
            None,
            Duration::from_secs(5),
        )
        .expect("ps");
        let CommandEnd::Finished(output) = end else {
            return false;
        };
        output.stdout.iter().all(u8::is_ascii_whitespace)
    }

    #[test]
    fn a_stuck_command_ends_as_timed_out_and_leaves_no_process() {
        let dir = temp_dir("stuck");
        let pidfile = dir.join("pid");
        let program = script(&dir, "#!/bin/sh\necho $$ > \"$1\"\nexec sleep 30\n");
        let started = Instant::now();
        let end = run_bounded(
            Command::new(&program).arg(&pidfile),
            None,
            Duration::from_secs(1),
        )
        .expect("spawn");
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "stuck command ran for {:?}",
            started.elapsed()
        );
        assert!(matches!(end, CommandEnd::TimedOut), "{end:?}");
        let pid: u32 = fs::read_to_string(&pidfile)
            .expect("pid file")
            .trim()
            .parse()
            .expect("pid");
        assert!(process_gone(pid), "pid {pid} still present");
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn output_past_16_mib_is_rejected() {
        let started = Instant::now();
        let end = run_bounded(
            Command::new("dd").args(["if=/dev/zero", "bs=1048576", "count=17"]),
            None,
            Duration::from_secs(10),
        )
        .expect("spawn dd");
        assert!(started.elapsed() < Duration::from_secs(10));
        assert!(matches!(end, CommandEnd::OutputLimit), "{end:?}");
    }

    #[test]
    fn stdin_is_fully_written_before_the_child_is_waited_on() {
        let dir = temp_dir("stdin");
        let copy = dir.join("body");
        let program = script(&dir, "#!/bin/sh\ncat > \"$1\"\ncat \"$1\"\n");
        let payload = vec![0xA5_u8; 256 * 1024];
        let started = Instant::now();
        let end = run_bounded(
            Command::new(&program).arg(&copy),
            Some(&payload),
            Duration::from_secs(10),
        )
        .expect("spawn sink");
        assert!(started.elapsed() < Duration::from_secs(10));
        match end {
            CommandEnd::Finished(output) => {
                assert!(
                    output.status.success(),
                    "sink failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert_eq!(output.stdout, payload);
            }
            other => panic!("stdin was not echoed: {other:?}"),
        }
        fs::remove_dir_all(dir).expect("cleanup");
    }
}
