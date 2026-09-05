use crate::error::{AppError, AppResult};

const KEYRING_SERVICE: &str = "com.bellanote.app";
const KEYRING_USER: &str = "openai_api_key";
pub const OPENAI_MODEL: &str = "gpt-4o";
const OPENAI_URL: &str = "https://api.openai.com/v1/chat/completions";

fn entry() -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
}

pub fn set_api_key(key: &str) -> AppResult<()> {
    let key = key.trim();
    if key.is_empty() {
        return clear_api_key();
    }
    entry()
        .map_err(|e| AppError::Message(e.to_string()))?
        .set_password(key)
        .map_err(|e| AppError::Message(e.to_string()))
}

pub fn clear_api_key() -> AppResult<()> {
    match entry().map_err(|e| AppError::Message(e.to_string()))?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Message(e.to_string())),
    }
}

pub fn api_key_configured() -> bool {
    get_api_key().is_some()
}

pub fn get_api_key() -> Option<String> {
    entry()
        .ok()?
        .get_password()
        .ok()
        .filter(|s| !s.trim().is_empty())
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ChatTurn {
    pub role: String,
    pub content: String,
}

#[derive(serde::Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(serde::Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(serde::Deserialize)]
struct OpenAiMessage {
    content: Option<String>,
}

pub async fn chat_completion(system: &str, messages: &[ChatTurn]) -> AppResult<String> {
    let api_key = get_api_key().ok_or_else(|| {
        AppError::Message("Add an OpenAI API token in Settings before chatting.".into())
    })?;
    let mut payload_messages = vec![serde_json::json!({
        "role": "system",
        "content": system
    })];
    for turn in messages {
        payload_messages.push(serde_json::json!({
            "role": turn.role,
            "content": turn.content
        }));
    }
    let client = reqwest::Client::new();
    let response = client
        .post(OPENAI_URL)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&serde_json::json!({
            "model": OPENAI_MODEL,
            "temperature": 0.2,
            "messages": payload_messages
        }))
        .send()
        .await
        .map_err(|e| AppError::Message(format!("Could not reach OpenAI: {e}")))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if status.as_u16() == 401 {
            return Err(AppError::Message(
                "OpenAI rejected the API token. Check Settings.".into(),
            ));
        }
        if status.as_u16() == 429 {
            return Err(AppError::Message(
                "OpenAI rate-limited the request. Wait a moment and try again.".into(),
            ));
        }
        return Err(AppError::Message(format!("OpenAI error ({status}): {body}")));
    }
    let parsed: OpenAiResponse = response
        .json()
        .await
        .map_err(|e| AppError::Message(format!("Could not read OpenAI response: {e}")))?;
    parsed
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AppError::Message("OpenAI returned an empty answer.".into()))
}
