use git2::Repository;
use serde::Deserialize;

use super::credential;
use super::error::{AIError, AIResult};
use super::providers;
use super::types::{GenerateResult, ReleaseNotesResult};
use crate::git::{diff, range};

const MAX_DIFF_CHARS: usize = 8000;
/// Upper bound on the commit-list portion of a release-notes prompt.
const MAX_RANGE_CHARS: usize = 24000;
/// Max characters of a single commit body included in a release-notes prompt.
const MAX_BODY_CHARS: usize = 400;
/// Max changed-file paths listed in a release-notes prompt.
const MAX_FILES_LISTED: usize = 80;

const COMMIT_SYSTEM_PROMPT: &str = "\
You are a commit message generator. Given a git diff, write a conventional commit message.

You MUST return exactly two parts separated by a blank line:
1. A title line: type(scope): description (max 72 chars)
2. A body: 2-4 lines explaining what changed and why

Types: feat, fix, refactor, docs, chore, test, style

Example output:
feat(auth): Add OAuth2 login flow

Add Google and GitHub OAuth2 providers with token refresh.
Update the login page to show provider buttons and handle
the redirect callback.

Rules:
- Be specific about what changed
- Focus on the \"why\" not just the \"what\"
- Use imperative mood (\"Add feature\" not \"Added feature\")
- Always include a body, never return only a title
- Return ONLY the commit message, no markdown formatting or code blocks";

const RELEASE_NOTES_SYSTEM_PROMPT: &str = "\
You are a release notes writer for a software project. You will be given the git commits \
between two revisions (newest first) plus a summary of the files that changed. Write release \
notes in Markdown for end users and developers.

Structure:
- Start with a short overview paragraph (1-3 sentences) describing the theme of the release.
- Then group changes under these headings, omitting any that would be empty:
  ### Features
  ### Bug Fixes
  ### Improvements
  ### Breaking Changes
  ### Other
- Each item is one concise bullet describing the user-visible change, not the implementation.
- Merge related commits into a single bullet when they describe one change.
- Preserve issue and pull request references such as #123 when they appear in a commit.
- Do not list commit SHAs or author names.
- Do not invent changes that are not supported by the commits.

Rules:
- Return ONLY the Markdown release notes.
- Do not wrap the output in a code block.
- Do not add a top-level title or version heading; the caller adds that.";

/// Generate a commit message from staged diffs.
pub async fn generate_commit_message(
    path: &str,
    provider_id: &str,
    model_id: &str,
    base_url: Option<&str>,
    max_tokens: u32,
) -> AIResult<GenerateResult> {
    let repo = Repository::open(path)
        .map_err(|e| AIError::ApiError(format!("Failed to open repo: {}", e)))?;

    let staged = diff::diff_staged(&repo)
        .map_err(|e| AIError::ApiError(format!("Failed to get staged diff: {}", e)))?;

    if staged.is_empty() {
        return Err(AIError::NoStagedFiles);
    }

    // Build diff text, truncating to stay within limits
    let mut diff_text = String::new();
    for file in &staged {
        let header = format!("--- {}\n", file.path);
        if diff_text.len() + header.len() > MAX_DIFF_CHARS {
            diff_text.push_str("\n... (diff truncated)\n");
            break;
        }
        diff_text.push_str(&header);

        for hunk in &file.hunks {
            for line in &hunk.lines {
                let prefix = match line.origin {
                    crate::git::types::DiffLineType::Addition => "+",
                    crate::git::types::DiffLineType::Deletion => "-",
                    crate::git::types::DiffLineType::Context => " ",
                    _ => " ",
                };
                let formatted = format!("{}{}", prefix, line.content);
                if diff_text.len() + formatted.len() > MAX_DIFF_CHARS {
                    diff_text.push_str("\n... (diff truncated)\n");
                    break;
                }
                diff_text.push_str(&formatted);
            }
        }
    }

    let user_prompt = format!("Generate a commit message for this diff:\n\n{}", diff_text);

    let response_text = call_provider(
        provider_id,
        base_url,
        model_id,
        COMMIT_SYSTEM_PROMPT,
        &user_prompt,
        max_tokens,
    )
    .await?;

    parse_commit_message(&response_text)
}

/// Generate Markdown release notes for the commits in `from..to`.
///
/// `from` is exclusive and `to` is inclusive. Both accept any revision spec
/// (tag, branch, SHA, `HEAD~3`).
pub async fn generate_release_notes(
    path: &str,
    from: &str,
    to: &str,
    provider_id: &str,
    model_id: &str,
    base_url: Option<&str>,
    max_tokens: u32,
) -> AIResult<ReleaseNotesResult> {
    let repo = Repository::open(path)
        .map_err(|e| AIError::ApiError(format!("Failed to open repo: {}", e)))?;

    let summary = range::summarize_range(&repo, from, to)?;
    if summary.commits.is_empty() {
        return Err(AIError::NoCommitsInRange(from.to_string(), to.to_string()));
    }

    let user_prompt = build_release_notes_prompt(from, to, &summary);

    let response_text = call_provider(
        provider_id,
        base_url,
        model_id,
        RELEASE_NOTES_SYSTEM_PROMPT,
        &user_prompt,
        max_tokens,
    )
    .await?;

    let markdown = strip_code_fence(&response_text).to_string();
    if markdown.is_empty() {
        return Err(AIError::InvalidResponse("Empty response from AI".into()));
    }

    Ok(ReleaseNotesResult { markdown, range: summary })
}

/// Render a commit range as prompt text, truncating so large ranges stay within limits.
fn build_release_notes_prompt(from: &str, to: &str, summary: &crate::git::types::CommitRangeSummary) -> String {
    let mut text = format!(
        "Write release notes for the changes between `{}` and `{}`.\n\n\
         {} commits, {} files changed, +{} / -{} lines.\n\nCommits (newest first):\n",
        from,
        to,
        summary.commits.len(),
        summary.files_changed,
        summary.insertions,
        summary.deletions,
    );

    let mut truncated = false;
    for commit in &summary.commits {
        let mut entry = format!("- {}", commit.summary);
        if commit.is_merge {
            entry.push_str(" [merge]");
        }
        if !commit.body.is_empty() {
            let body: String = commit.body.chars().take(MAX_BODY_CHARS).collect();
            for line in body.lines() {
                entry.push_str("\n    ");
                entry.push_str(line);
            }
            if commit.body.chars().count() > MAX_BODY_CHARS {
                entry.push_str("\n    ...");
            }
        }
        entry.push('\n');
        if text.len() + entry.len() > MAX_RANGE_CHARS {
            truncated = true;
            break;
        }
        text.push_str(&entry);
    }
    if truncated {
        text.push_str("... (older commits omitted)\n");
    }

    if !summary.files.is_empty() {
        text.push_str("\nChanged files:\n");
        for file in summary.files.iter().take(MAX_FILES_LISTED) {
            text.push_str("- ");
            text.push_str(file);
            text.push('\n');
        }
        if summary.files.len() > MAX_FILES_LISTED {
            text.push_str(&format!("... and {} more\n", summary.files.len() - MAX_FILES_LISTED));
        }
    }

    text
}

/// Look up the provider's key and dispatch to the matching API client.
async fn call_provider(
    provider_id: &str,
    base_url: Option<&str>,
    model_id: &str,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: u32,
) -> AIResult<String> {
    let api_key = credential::get_key(provider_id)
        .ok_or_else(|| AIError::NoApiKey(provider_id.to_string()))?;

    let default_url = providers::default_base_url(provider_id)
        .unwrap_or("https://api.openai.com/v1");
    let effective_base_url = base_url.unwrap_or(default_url);

    match provider_id {
        "anthropic" => call_anthropic(effective_base_url, &api_key, model_id, system_prompt, user_prompt, max_tokens).await,
        "gemini" => call_gemini(effective_base_url, &api_key, model_id, system_prompt, user_prompt, max_tokens).await,
        "openai" => call_openai(effective_base_url, &api_key, model_id, system_prompt, user_prompt, max_tokens).await,
        _ => call_openai_compatible(effective_base_url, &api_key, model_id, system_prompt, user_prompt, max_tokens).await,
    }
}

/// Call OpenAI API (uses max_completion_tokens).
async fn call_openai(
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: u32,
) -> AIResult<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ],
        "temperature": 0.3,
        "max_completion_tokens": max_tokens,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AIError::ApiError(format!("{}: {}", status, text)));
    }

    #[derive(Deserialize)]
    struct Choice {
        message: ChoiceMessage,
    }
    #[derive(Deserialize)]
    struct ChoiceMessage {
        content: String,
    }
    #[derive(Deserialize)]
    struct OpenAIResponse {
        choices: Vec<Choice>,
    }

    let data: OpenAIResponse = resp.json().await?;
    data.choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| AIError::InvalidResponse("No choices in response".into()))
}

/// Call OpenAI-compatible API (OpenRouter, custom endpoints — uses max_tokens).
async fn call_openai_compatible(
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: u32,
) -> AIResult<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ],
        "temperature": 0.3,
        "max_tokens": max_tokens,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AIError::ApiError(format!("{}: {}", status, text)));
    }

    #[derive(Deserialize)]
    struct Choice {
        message: ChoiceMessage,
    }
    #[derive(Deserialize)]
    struct ChoiceMessage {
        content: String,
    }
    #[derive(Deserialize)]
    struct OpenAIResponse {
        choices: Vec<Choice>,
    }

    let data: OpenAIResponse = resp.json().await?;
    data.choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| AIError::InvalidResponse("No choices in response".into()))
}

/// Call Anthropic Messages API.
async fn call_anthropic(
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: u32,
) -> AIResult<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/messages", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "max_tokens": max_tokens,
        "system": system_prompt,
        "messages": [
            { "role": "user", "content": user_prompt }
        ],
    });

    let resp = client
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AIError::ApiError(format!("{}: {}", status, text)));
    }

    #[derive(Deserialize)]
    struct ContentBlock {
        text: Option<String>,
    }
    #[derive(Deserialize)]
    struct AnthropicResponse {
        content: Vec<ContentBlock>,
    }

    let data: AnthropicResponse = resp.json().await?;
    data.content
        .into_iter()
        .find_map(|b| b.text)
        .ok_or_else(|| AIError::InvalidResponse("No text in response".into()))
}

/// Call Gemini generateContent API.
async fn call_gemini(
    base_url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: u32,
) -> AIResult<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/models/{}:generateContent?key={}",
        base_url.trim_end_matches('/'),
        model,
        api_key
    );

    let body = serde_json::json!({
        "systemInstruction": {
            "parts": [{ "text": system_prompt }]
        },
        "contents": [{
            "parts": [{ "text": user_prompt }]
        }],
        "generationConfig": {
            "temperature": 0.4,
            "maxOutputTokens": max_tokens,
        }
    });

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AIError::ApiError(format!("{}: {}", status, text)));
    }

    #[derive(Deserialize)]
    struct Part {
        text: Option<String>,
    }
    #[derive(Deserialize)]
    struct Content {
        parts: Vec<Part>,
    }
    #[derive(Deserialize)]
    struct Candidate {
        content: Content,
    }
    #[derive(Deserialize)]
    struct GeminiResponse {
        candidates: Vec<Candidate>,
    }

    let data: GeminiResponse = resp.json().await?;
    data.candidates
        .into_iter()
        .next()
        .and_then(|c| c.content.parts.into_iter().find_map(|p| p.text))
        .ok_or_else(|| AIError::InvalidResponse("No text in response".into()))
}

/// Strip a surrounding markdown code fence, if the model added one.
fn strip_code_fence(text: &str) -> &str {
    let trimmed = text.trim();
    if trimmed.starts_with("```") {
        let without_opener = trimmed
            .strip_prefix("```")
            .unwrap_or(trimmed)
            .trim_start_matches(|c: char| c != '\n')
            .trim_start_matches('\n');
        without_opener
            .strip_suffix("```")
            .unwrap_or(without_opener)
            .trim()
    } else {
        trimmed
    }
}

/// Parse AI response into title + body.
fn parse_commit_message(text: &str) -> AIResult<GenerateResult> {
    let cleaned = strip_code_fence(text);

    if cleaned.is_empty() {
        return Err(AIError::InvalidResponse("Empty response from AI".into()));
    }

    // Split on first blank line: title is first line, body is the rest
    let mut lines = cleaned.lines();
    let title = lines.next().unwrap_or("").to_string();
    let mut body = String::new();

    // Skip blank lines after title
    let mut found_content = false;
    for line in lines {
        if !found_content && line.trim().is_empty() {
            continue;
        }
        found_content = true;
        if !body.is_empty() {
            body.push('\n');
        }
        body.push_str(line);
    }

    Ok(GenerateResult { title, body })
}
