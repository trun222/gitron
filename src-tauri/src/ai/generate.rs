use git2::Repository;
use serde::Deserialize;

use super::credential;
use super::error::{AIError, AIResult};
use super::providers;
use super::types::GenerateResult;
use crate::git::diff;

const MAX_DIFF_CHARS: usize = 8000;

const SYSTEM_PROMPT: &str = "\
You are a commit message generator. Given a git diff, write a concise conventional commit message.

Format:
- First line: type(scope): description (max 72 chars)
- Types: feat, fix, refactor, docs, chore, test, style
- Blank line, then a short body explaining what changed and why (2-4 lines max)

Rules:
- Be specific about what changed
- Focus on the \"why\" not just the \"what\"
- Use imperative mood (\"Add feature\" not \"Added feature\")
- If the diff is too large to summarize, focus on the most significant changes
- Return ONLY the commit message, no markdown formatting or code blocks";

/// Generate a commit message from staged diffs.
pub async fn generate_commit_message(
    path: &str,
    provider_id: &str,
    model_id: &str,
    base_url: Option<&str>,
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

    let api_key = credential::get_key(provider_id)
        .ok_or_else(|| AIError::NoApiKey(provider_id.to_string()))?;

    let default_url = providers::default_base_url(provider_id)
        .unwrap_or("https://api.openai.com/v1");
    let effective_base_url = base_url.unwrap_or(default_url);

    let response_text = match provider_id {
        "anthropic" => call_anthropic(effective_base_url, &api_key, model_id, &user_prompt).await?,
        "gemini" => call_gemini(effective_base_url, &api_key, model_id, &user_prompt).await?,
        "openai" => call_openai(effective_base_url, &api_key, model_id, &user_prompt).await?,
        _ => call_openai_compatible(effective_base_url, &api_key, model_id, &user_prompt).await?,
    };

    parse_commit_message(&response_text)
}

/// Call OpenAI API (uses max_completion_tokens).
async fn call_openai(
    base_url: &str,
    api_key: &str,
    model: &str,
    user_prompt: &str,
) -> AIResult<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": user_prompt }
        ],
        "temperature": 0.3,
        "max_completion_tokens": 300,
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
    user_prompt: &str,
) -> AIResult<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": user_prompt }
        ],
        "temperature": 0.3,
        "max_tokens": 300,
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
    user_prompt: &str,
) -> AIResult<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/messages", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 300,
        "system": SYSTEM_PROMPT,
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
    user_prompt: &str,
) -> AIResult<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/models/{}:generateContent?key={}",
        base_url.trim_end_matches('/'),
        model,
        api_key
    );

    let full_prompt = format!("{}\n\n{}", SYSTEM_PROMPT, user_prompt);

    let body = serde_json::json!({
        "contents": [{
            "parts": [{ "text": full_prompt }]
        }],
        "generationConfig": {
            "temperature": 0.3,
            "maxOutputTokens": 300,
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

/// Parse AI response into title + body.
fn parse_commit_message(text: &str) -> AIResult<GenerateResult> {
    let trimmed = text.trim();
    // Strip markdown code block wrappers if present
    let cleaned = if trimmed.starts_with("```") {
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
    };

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
