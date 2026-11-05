//! Bound external diagnostics and the unchanged MIRP runtime in a child process group.
use crate::{Error, Result};
use std::{
    io::{Read, Write},
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};
pub fn capture(command: &mut Command, limit: Duration) -> Result<Output> {
    capture_with_input(command, limit, None)
}
/// As `capture`, optionally streaming `input` to the child's stdin (avoids argv size limits).
pub fn capture_with_input(
    command: &mut Command,
    limit: Duration,
    input: Option<Vec<u8>>,
) -> Result<Output> {
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|x| Error::new(5, x))?;
    let writer = input.map(|bytes| {
        let stdin = child.stdin.take();
        std::thread::spawn(move || {
            if let Some(mut stdin) = stdin {
                let _ = stdin.write_all(&bytes);
            }
        })
    });
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::new(9, "missing stdout pipe"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::new(9, "missing stderr pipe"))?;
    let out = std::thread::spawn(move || {
        let mut b = vec![];
        stdout.read_to_end(&mut b).map(|_| b)
    });
    let err = std::thread::spawn(move || {
        let mut b = vec![];
        stderr.read_to_end(&mut b).map(|_| b)
    });
    let start = Instant::now();
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            // A reaped parent can leave descendants holding the output pipes.
            Ok(Some(s)) if out.is_finished() && err.is_finished() => break s,
            Ok(_) if start.elapsed() < limit => std::thread::sleep(Duration::from_millis(20)),
            result => {
                timed_out = result.is_ok();
                #[cfg(unix)]
                {
                    // The spawned child's PID names its dedicated process group, never the CLI's group.
                    unsafe {
                        libc::kill(-(child.id() as i32), libc::SIGKILL);
                    }
                }
                let _ = child.kill();
                break child.wait().map_err(|x| Error::new(5, x))?;
            }
        }
    };
    if let Some(writer) = writer {
        writer
            .join()
            .map_err(|_| Error::new(9, "stdin writer failed"))?;
    }
    let stdout = out
        .join()
        .map_err(|_| Error::new(9, "stdout reader failed"))?
        .map_err(|x| Error::new(5, x))?;
    let stderr = err
        .join()
        .map_err(|_| Error::new(9, "stderr reader failed"))?
        .map_err(|x| Error::new(5, x))?;
    if timed_out {
        return Err(Error::new(
            6,
            format!(
                "process deadline exceeded; stdout={} stderr={}",
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            ),
        ));
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captures_output_after_successful_exit() {
        let output = capture(
            Command::new("/bin/sh").args(["-c", "printf stdout; printf stderr >&2"]),
            Duration::from_secs(2),
        )
        .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"stdout");
        assert_eq!(output.stderr, b"stderr");
    }

    #[cfg(unix)]
    #[test]
    fn deadline_includes_descendant_output_after_parent_exit() {
        for script in ["sleep 5 >&1 2>/dev/null &", "sleep 5 >/dev/null 2>&2 &"] {
            let start = Instant::now();
            let error = capture(
                Command::new("/bin/sh").args(["-c", script]),
                Duration::from_millis(100),
            )
            .unwrap_err();
            assert_eq!(error.exit, 6);
            assert!(start.elapsed() < Duration::from_secs(2));
        }
    }

    #[test]
    fn deadline_kills_process_group() {
        let start = Instant::now();
        let error = capture(
            Command::new("/bin/sleep").arg("30"),
            Duration::from_millis(40),
        )
        .unwrap_err();
        assert_eq!(error.exit, 6);
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
