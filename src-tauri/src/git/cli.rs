use std::process::Command;

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
            stderr: result.stderr,
            exit_code: result.exit_code,
        });
    }

    Ok(result)
}

/// Run a git CLI command asynchronously (for network ops like fetch/push/pull)
pub async fn run_git_async(workdir: &str, args: &[&str]) -> GitResult<CliOutput> {
    let output = AsyncCommand::new("git")
        .args(args)
        .current_dir(workdir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .await
        .map_err(|e| GitError::Io(e))?;

    let result = CliOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    };

    if !output.status.success() {
        return Err(GitError::CliError {
            command: format!("git {}", args.join(" ")),
            stderr: result.stderr,
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

    let header_value = format!("AUTHORIZATION: Bearer {token}");

    let output = AsyncCommand::new("git")
        .args(args)
        .current_dir(workdir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "http.https://github.com/.extraheader")
        .env("GIT_CONFIG_VALUE_0", &header_value)
        .output()
        .await
        .map_err(|e| GitError::Io(e))?;

    let result = CliOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    };

    if !output.status.success() {
        return Err(GitError::CliError {
            command: format!("git {}", args.join(" ")),
            stderr: result.stderr,
            exit_code: result.exit_code,
        });
    }

    Ok(result)
}
