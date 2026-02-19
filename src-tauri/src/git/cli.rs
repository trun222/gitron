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
