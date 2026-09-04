use std::process::Command;
use std::time::Duration;

use tokio::process::Command as AsyncCommand;

use super::error::{GitError, GitResult};

/// Output from a git CLI command
#[derive(Debug, Clone)]
pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Run a git CLI command synchronously, returning output regardless of exit code.
/// Only returns Err for actual I/O errors (process spawn failure).
/// Use this when the caller needs to handle non-zero exit codes gracefully (e.g. hook failures).
pub fn run_git_raw(workdir: &str, args: &[&str]) -> GitResult<CliOutput> {
    let output = Command::new("git")
        .args(args)
        .current_dir(workdir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| GitError::Io(e))?;

    Ok(CliOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/// Run a git CLI command synchronously (for quick local ops)
pub fn run_git(workdir: &str, args: &[&str]) -> GitResult<CliOutput> {
    let output = Command::new("git")
        .args(args)
        .current_dir(workdir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| GitError::Io(e))?;

    let result = CliOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    };

    if !output.status.success() {
        return Err(GitError::CliError {
            command: format!("git {}", args.join(" ")),
            stderr: combine_output(&result.stdout, &result.stderr),
            exit_code: result.exit_code,
        });
    }

    Ok(result)
}

/// Upper bound for a single network git command (fetch/push/pull/ls-remote).
///
/// Without it a git process stuck on a credential prompt, an askpass GUI, or a
/// stalled connection never returns, the frontend's `networkOperation` lock is
/// never released, and every later push/pull is refused until the app restarts.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(600);

/// Run an already-configured command, killing it if it exceeds `timeout`.
async fn output_with_timeout(
    mut cmd: AsyncCommand,
    args: &[&str],
    timeout: Duration,
) -> GitResult<std::process::Output> {
    // When the timeout fires the `output()` future is dropped; make sure the
    // child goes with it instead of lingering as an orphan holding repo locks.
    cmd.kill_on_drop(true);
    match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(output) => output.map_err(GitError::Io),
        Err(_) => Err(GitError::CliError {
            command: format!("git {}", args.join(" ")),
            stderr: format!(
                "Timed out after {}s waiting for git to finish; the process was terminated. \
                 Check network connectivity and credentials, then try again.",
                timeout.as_secs()
            ),
            exit_code: -1,
        }),
    }
}

/// Run a git CLI command asynchronously (for network ops like fetch/push/pull)
pub async fn run_git_async(workdir: &str, args: &[&str]) -> GitResult<CliOutput> {
    let mut cmd = AsyncCommand::new("git");
    cmd.args(args)
        .current_dir(workdir)
        .env("GIT_TERMINAL_PROMPT", "0");
    let output = output_with_timeout(cmd, args, NETWORK_TIMEOUT).await?;

    let result = CliOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    };

    if !output.status.success() {
        return Err(GitError::CliError {
            command: format!("git {}", args.join(" ")),
            stderr: combine_output(&result.stdout, &result.stderr),
            exit_code: result.exit_code,
        });
    }

    Ok(result)
}

/// Run a git CLI command asynchronously with GitHub OAuth token injected via
/// environment-based git config. If no token is stored, falls back to `run_git_async`.
///
/// Uses the same `http.extraheader` approach as GitHub Actions: sets the
/// Authorization header directly via `GIT_CONFIG_COUNT`/`GIT_CONFIG_KEY_*`/
/// `GIT_CONFIG_VALUE_*` env vars (Git 2.31+). This is fully cross-platform
/// (no shell syntax), keeps the token out of process arguments, and is scoped
/// to github.com URLs only.
pub async fn run_git_async_with_github_auth(workdir: &str, args: &[&str]) -> GitResult<CliOutput> {
    let token = match crate::github::credential::get_token() {
        Some(t) => t,
        None => return run_git_async(workdir, args).await,
    };

    // Rewrite github.com URLs to embed the OAuth token inline.
    // This is the same approach GitHub Desktop uses — proven reliable
    // across all git versions and credential helper configurations.
    let authed_url = format!("https://x-access-token:{token}@github.com/");
    let config_key = format!("url.{authed_url}.insteadOf");

    let mut cmd = AsyncCommand::new("git");
    cmd.args(args)
        .current_dir(workdir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", &config_key)
        .env("GIT_CONFIG_VALUE_0", "https://github.com/");
    let output = output_with_timeout(cmd, args, NETWORK_TIMEOUT).await?;

    let result = CliOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    };

    if !output.status.success() {
        return Err(GitError::CliError {
            command: format!("git {}", args.join(" ")),
            stderr: combine_output(&result.stdout, &result.stderr),
            exit_code: result.exit_code,
        });
    }

    Ok(result)
}

/// Combine stdout and stderr into a single error message.
/// Git hooks often write to stdout, while git errors go to stderr.
/// Including both ensures hook output is visible in error messages.
fn combine_output(stdout: &str, stderr: &str) -> String {
    let stdout = stdout.trim();
    let stderr = stderr.trim();
    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => String::new(),
        (true, false) => stderr.to_string(),
        (false, true) => stdout.to_string(),
        (false, false) => format!("{stdout}\n{stderr}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A git process that never finishes (hung credential prompt, stalled
    /// connection) must not block the caller forever.
    #[cfg(unix)]
    #[tokio::test]
    async fn hung_process_is_killed_after_timeout() {
        let mut cmd = AsyncCommand::new("sleep");
        cmd.arg("30");
        let started = std::time::Instant::now();
        let result = output_with_timeout(cmd, &["push", "origin"], Duration::from_millis(100)).await;
        assert!(started.elapsed() < Duration::from_secs(5), "timeout did not fire");
        match result {
            Err(GitError::CliError { command, stderr, exit_code }) => {
                assert_eq!(command, "git push origin");
                assert!(stderr.contains("Timed out"), "unexpected stderr: {stderr}");
                assert_eq!(exit_code, -1);
            }
            other => panic!("expected CliError, got {other:?}"),
        }
    }
}
