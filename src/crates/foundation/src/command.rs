//! One bounded runner for external commands.
//!
//! Standard input is written on its own thread; a child that closes it early
//! (`BrokenPipe`) is not a failure to start. Standard output and standard
//! error are read on their own threads, which count the 16 MiB ceiling.
//! `process_control` waits and, on timeout, terminates through `waitid` on
//! macOS and Linux. After the child exits, a pipe that is still open is waited
//! on only until the original deadline. `memory_limit` is not used: that
//! method is absent on macOS.

use std::io::{self, Read, Write};
use std::process::{Child, ChildStdin, Command, Output, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use process_control::{ChildExt, Control};

/// Per-stream ceiling already used by provider commands.
pub const OUTPUT_LIMIT_BYTES: usize = 16 * 1024 * 1024;

/// How long to let reader threads observe a closed pipe after the group is signaled.
const PIPE_REAP_GRACE: Duration = Duration::from_millis(500);

/// What a bounded command did.
#[derive(Debug)]
#[must_use]
pub enum CommandEnd {
    /// The process exited and both pipes closed within the limit.
    Finished(Output),
    /// The time limit elapsed. The process was terminated, or a pipe stayed open.
    TimedOut,
    /// Standard output or standard error grew past [`OUTPUT_LIMIT_BYTES`].
    OutputLimit,
}

/// Runs `command` with a time limit.
///
/// When `input` is set, those bytes are written on a separate thread while the
/// output pipes are read. Either stream over [`OUTPUT_LIMIT_BYTES`] yields
/// [`CommandEnd::OutputLimit`]. A pipe that is still open when the deadline
/// arrives yields [`CommandEnd::TimedOut`].
///
/// # Errors
///
/// Returns an error when the process cannot be spawned, a pipe thread cannot
/// be started, standard input cannot be written for a reason other than the
/// child closing it, or the limiter itself fails. A timeout is
/// [`CommandEnd::TimedOut`], not an error.
pub fn run_bounded(
    command: &mut Command,
    input: Option<&[u8]>,
    time_limit: Duration,
) -> io::Result<CommandEnd> {
    // The child and anything it does not move into a new group share one group.
    // A direct-child timeout still ends only through `terminate_for_timeout`.
    // The group signal below runs only after that child has already exited and
    // a pipe is still open, so a grandchild holding the pipe does not outlive
    // the deadline.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn()?;
    #[cfg(unix)]
    let group = rustix::process::Pid::from_child(&child);

    let stdout = match child.stdout.take() {
        Some(pipe) => pipe,
        None => {
            abandon(&mut child);
            return Err(io::Error::other("child standard output is missing"));
        }
    };
    let stderr = match child.stderr.take() {
        Some(pipe) => pipe,
        None => {
            abandon(&mut child);
            return Err(io::Error::other("child standard error is missing"));
        }
    };
    let started = Instant::now();
    let stdout_count = Arc::new(AtomicUsize::new(0));
    let stderr_count = Arc::new(AtomicUsize::new(0));
    let stdout_rx = match spawn_reader(stdout, Arc::clone(&stdout_count)) {
        Ok(receiver) => receiver,
        Err(error) => {
            abandon(&mut child);
            return Err(error);
        }
    };
    let stderr_rx = match spawn_reader(stderr, Arc::clone(&stderr_count)) {
        Ok(receiver) => receiver,
        Err(error) => {
            abandon(&mut child);
            return Err(error);
        }
    };
    let stdin_rx = match input {
        Some(bytes) => {
            let Some(stdin) = child.stdin.take() else {
                abandon(&mut child);
                return Err(io::Error::other("child standard input is missing"));
            };
            match spawn_writer(stdin, bytes) {
                Ok(receiver) => Some(receiver),
                Err(error) => {
                    abandon(&mut child);
                    return Err(error);
                }
            }
        }
        None => {
            drop(child.stdin.take());
            None
        }
    };

    let status = match child
        .controlled()
        .time_limit(time_limit)
        .terminate_for_timeout()
        .wait()
    {
        Ok(status) => status,
        Err(error) => {
            abandon(&mut child);
            return Err(error);
        }
    };

    let deadline = started + time_limit;
    let mut stdout_end = recv_until(&stdout_rx, deadline);
    let mut stderr_end = recv_until(&stderr_rx, deadline);
    let mut stdin_end = match &stdin_rx {
        Some(receiver) => recv_until(receiver, deadline),
        None => Some(Ok(())),
    };
    let late = stdout_end.is_none() || stderr_end.is_none() || stdin_end.is_none();
    // A pipe that is still open at the deadline has a live holder in the
    // child's group: a grandchild after the child exited, or one the child
    // left behind when the deadline terminated it. Signal the group, then
    // give the readers one grace period so they can leave. The result stays a
    // timeout: the pipes missed the deadline.
    if late {
        #[cfg(unix)]
        {
            let _ = rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
        }
        let grace = Instant::now() + PIPE_REAP_GRACE;
        if stdout_end.is_none() {
            stdout_end = recv_until(&stdout_rx, grace);
        }
        if stderr_end.is_none() {
            stderr_end = recv_until(&stderr_rx, grace);
        }
        if stdin_end.is_none()
            && let Some(receiver) = &stdin_rx
        {
            stdin_end = recv_until(receiver, grace);
        }
    }

    if stdout_count.load(Ordering::Relaxed) > OUTPUT_LIMIT_BYTES
        || stderr_count.load(Ordering::Relaxed) > OUTPUT_LIMIT_BYTES
    {
        return Ok(CommandEnd::OutputLimit);
    }
    if status.is_none() || late {
        return Ok(CommandEnd::TimedOut);
    }

    // A child that exits without reading its standard input closes the pipe
    // under the writer thread. That is the child's answer, not a failure to
    // start it: the status and output read below are still the real result, so
    // a `BrokenPipe` from the writer is ignored, as the unbounded runner did.
    if let Some(Err(error)) = stdin_end
        && error.kind() != io::ErrorKind::BrokenPipe
    {
        return Err(error);
    }
    let stdout = stdout_end.unwrap_or(Err(io::Error::other(
        "standard output closed without a result",
    )))?;
    let stderr = stderr_end.unwrap_or(Err(io::Error::other(
        "standard error closed without a result",
    )))?;
    let status = status.expect("exited child").into_std_lossy();
    Ok(CommandEnd::Finished(Output {
        status,
        stdout,
        stderr,
    }))
}

fn abandon(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn read_capped(mut pipe: impl Read, counter: &AtomicUsize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buf = [0_u8; 8 * 1024];
    loop {
        let read = match pipe.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        let previous = counter.fetch_add(read, Ordering::Relaxed);
        if previous >= OUTPUT_LIMIT_BYTES {
            continue;
        }
        let room = OUTPUT_LIMIT_BYTES - previous;
        let keep = read.min(room);
        bytes.extend_from_slice(&buf[..keep]);
    }
    Ok(bytes)
}

fn spawn_reader(
    pipe: impl Read + Send + 'static,
    counter: Arc<AtomicUsize>,
) -> io::Result<Receiver<io::Result<Vec<u8>>>> {
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("hctl2-command-read".into())
        .spawn(move || {
            let _ = sender.send(read_capped(pipe, &counter));
        })?;
    Ok(receiver)
}

fn spawn_writer(mut stdin: ChildStdin, bytes: &[u8]) -> io::Result<Receiver<io::Result<()>>> {
    let bytes = bytes.to_vec();
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("hctl2-command-stdin".into())
        .spawn(move || {
            let result = stdin.write_all(&bytes);
            drop(stdin);
            let _ = sender.send(result);
        })?;
    Ok(receiver)
}

fn recv_until<T>(receiver: &Receiver<T>, deadline: Instant) -> Option<T> {
    let now = Instant::now();
    if now >= deadline {
        return receiver.try_recv().ok();
    }
    receiver
        .recv_timeout(deadline.saturating_duration_since(now))
        .ok()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::panic::{self, AssertUnwindSafe};
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use super::{CommandEnd, run_bounded};

    /// Slack past the limit. The command must return inside this window.
    const MARGIN: Duration = Duration::from_secs(4);

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

    fn assert_gone(pid: u32) {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(2) {
            if process_gone(pid) {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("pid {pid} still present");
    }

    /// Fails while any live process carries `marker` on its command line.
    ///
    /// A pid file needs the stand-in to survive until its first write, which a
    /// process killed at the deadline may not. A marker in the command line is
    /// true whether or not the process ever got that far.
    fn assert_no_process_with(marker: &str) {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(2) {
            let listed = run_bounded(
                Command::new("pgrep").args(["-f", marker]),
                None,
                Duration::from_secs(5),
            )
            .expect("pgrep");
            let CommandEnd::Finished(output) = listed else {
                panic!("pgrep did not finish");
            };
            if output.stdout.iter().all(u8::is_ascii_whitespace) {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("a process matching {marker} is still present");
    }

    fn pid_from(path: &Path) -> u32 {
        fs::read_to_string(path)
            .expect("pid file")
            .trim()
            .parse()
            .expect("pid")
    }

    /// Runs `body` on another thread so a hung `run_bounded` fails inside the margin.
    fn call_within<T>(limit: Duration, body: impl FnOnce() -> T + Send + 'static) -> T
    where
        T: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let result = panic::catch_unwind(AssertUnwindSafe(body));
            let _ = sender.send(result);
        });
        match receiver.recv_timeout(limit + MARGIN) {
            Ok(Ok(value)) => value,
            Ok(Err(payload)) => panic::resume_unwind(payload),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                eprintln!("run_bounded did not return within the limit plus margin");
                std::process::exit(2);
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("runner thread ended without a result");
            }
        }
    }

    #[test]
    fn a_stuck_command_ends_as_timed_out_and_leaves_no_process() {
        let dir = temp_dir("stuck");
        let program = script(&dir, "#!/bin/sh\nexec sleep 31.4159\n");
        let limit = Duration::from_secs(3);
        let started = Instant::now();
        let end = run_bounded(&mut Command::new(&program), None, limit).expect("spawn");
        assert!(
            started.elapsed() < limit + MARGIN,
            "stuck command ran for {:?}",
            started.elapsed()
        );
        assert!(matches!(end, CommandEnd::TimedOut), "{end:?}");
        assert_no_process_with("31.4159");
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

    #[test]
    fn exited_parent_with_stdout_holder_times_out_and_leaves_no_process() {
        let dir = temp_dir("hold-stdout");
        let program = script(&dir, "#!/bin/sh\nsleep 31.4160 &\nexit 0\n");
        let limit = Duration::from_secs(2);
        let end = call_within(limit, move || {
            run_bounded(&mut Command::new(&program), None, limit).expect("spawn")
        });
        assert!(matches!(end, CommandEnd::TimedOut), "{end:?}");
        assert_no_process_with("31.4160");
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn unread_stdin_over_the_pipe_buffer_times_out_and_leaves_no_process() {
        let dir = temp_dir("unread-stdin");
        let program = script(&dir, "#!/bin/sh\nexec sleep 31.4161\n");
        let limit = Duration::from_secs(3);
        let payload = vec![0_u8; 1024 * 1024];
        let end = call_within(limit, move || {
            run_bounded(&mut Command::new(&program), Some(&payload), limit).expect("spawn")
        });
        assert!(matches!(end, CommandEnd::TimedOut), "{end:?}");
        assert_no_process_with("31.4161");
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn cat_of_one_mebibyte_finishes_within_the_limit_and_leaves_no_process() {
        let dir = temp_dir("cat-stdin");
        let pidfile = dir.join("pid");
        let program = script(&dir, "#!/bin/sh\necho $$ > \"$1\"\nexec cat\n");
        let limit = Duration::from_secs(5);
        let payload = vec![0x5A_u8; 1024 * 1024];
        let input = payload.clone();
        let pidfile_for_call = pidfile.clone();
        let end = call_within(limit, move || {
            run_bounded(
                Command::new(&program).arg(&pidfile_for_call),
                Some(&input),
                limit,
            )
            .expect("spawn")
        });
        match end {
            CommandEnd::Finished(output) => {
                assert!(
                    output.status.success(),
                    "cat failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert_eq!(output.stdout, payload);
            }
            other => panic!("cat did not finish: {other:?}"),
        }
        assert_gone(pid_from(&pidfile));
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn a_stuck_command_that_left_a_pipe_holder_leaves_no_process() {
        let dir = temp_dir("stuck-holder");
        let program = script(&dir, "#!/bin/sh\nsleep 31.4162 &\nexec sleep 31.4163\n");
        let limit = Duration::from_secs(3);
        let end = call_within(limit, move || {
            run_bounded(&mut Command::new(&program), None, limit).expect("spawn")
        });
        assert!(matches!(end, CommandEnd::TimedOut), "{end:?}");
        assert_no_process_with("31.4162");
        assert_no_process_with("31.4163");
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn unread_stdin_with_an_immediate_exit_keeps_the_child_answer() {
        let dir = temp_dir("instant-refusal");
        let program = script(&dir, "#!/bin/sh\necho refused >&2\nexit 3\n");
        let payload = vec![0_u8; 1024 * 1024];
        let started = Instant::now();
        let end = run_bounded(
            &mut Command::new(&program),
            Some(&payload),
            Duration::from_secs(10),
        )
        .expect("spawn");
        assert!(started.elapsed() < Duration::from_secs(10));
        match end {
            CommandEnd::Finished(output) => {
                assert_eq!(
                    output.status.code(),
                    Some(3),
                    "the child's own status must survive a closed stdin"
                );
                assert_eq!(
                    String::from_utf8_lossy(&output.stderr).trim(),
                    "refused",
                    "the child's stderr must survive a closed stdin"
                );
            }
            other => panic!("the child's answer was lost: {other:?}"),
        }
        fs::remove_dir_all(dir).expect("cleanup");
    }
}
