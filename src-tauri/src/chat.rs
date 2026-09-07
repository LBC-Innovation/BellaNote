use crate::db::Artifact;
use crate::error::{AppError, AppResult};
use crate::llm::{self, ChatTurn};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const CHAR_BUDGET: usize = 110_000;

pub const SYSTEM_PROMPT: &str = "You are Bella, a meeting analysis assistant. Be clear, direct, and concise. Avoid fluffy language.

Use only the provided transcripts as the source of truth. Never invent people, dates, or decisions.

When quoting, use a markdown blockquote.
When pointing at a moment, cite a discrete timestamp like [00:17] or [1:02:03] and the artifact title.
Never output time ranges.

If the transcripts do not contain the answer, say so and suggest a wider or narrower scope.
Do not say “based on the transcript”. Just answer.

If someone asks about your father, this is the one exception to using only the transcripts: answer gently and in one short sentence that he is Zach, a software engineer and technology executive who lives in North Georgia. Do not mention him unless asked.";

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScopeFile {
    pub id: String,
    pub title: String,
    pub meeting_group_id: String,
    pub included: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScopePreview {
    pub ready_count: i64,
    pub total_count: i64,
    pub used_count: i64,
    pub omitted_count: i64,
    pub files: Vec<ScopeFile>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatThreadView {
    pub messages: Vec<ChatTurn>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeArgs {
    pub scope_type: String,
    pub scope_id: String,
}

fn pack_files(files: &[Artifact]) -> (Vec<Artifact>, Vec<Artifact>) {
    let mut used = Vec::new();
    let mut chars = 0usize;
    for file in files {
        let next = file.transcript.len() + file.title.len() + 40;
        if !used.is_empty() && chars + next > CHAR_BUDGET {
            break;
        }
        chars += next;
        used.push(file.clone());
    }
    let omitted = files.iter().skip(used.len()).cloned().collect();
    (used, omitted)
}

fn to_scope_file(file: &Artifact, included: bool) -> ScopeFile {
    ScopeFile {
        id: file.id.clone(),
        title: file.title.clone(),
        meeting_group_id: file.meeting_group_id.clone(),
        included,
    }
}

pub fn preview(state: &Arc<AppState>, scope_type: &str, scope_id: &str) -> AppResult<ScopePreview> {
    let ready = state.db.ready_artifacts_for_scope(scope_type, scope_id)?;
    let (total_ready, total) = state.db.count_artifacts_in_scope(scope_type, scope_id)?;
    let (used, omitted) = pack_files(&ready);
    let mut files: Vec<ScopeFile> = used.iter().map(|f| to_scope_file(f, true)).collect();
    files.extend(omitted.iter().map(|f| to_scope_file(f, false)));
    Ok(ScopePreview {
        ready_count: total_ready,
        total_count: total,
        used_count: used.len() as i64,
        omitted_count: omitted.len() as i64,
        files,
    })
}

fn context_block(files: &[Artifact]) -> String {
    let mut out = String::from("## Transcripts in scope\n");
    for file in files {
        out.push_str(&format!(
            "\n### {} (id: {})\n{}\n",
            file.title, file.id, file.transcript
        ));
    }
    out
}

pub async fn ask(
    state: &Arc<AppState>,
    scope_type: &str,
    scope_id: &str,
    question: &str,
) -> AppResult<ChatThreadView> {
    let question = question.trim();
    if question.is_empty() {
        return Err(AppError::Message("Ask a question first.".into()));
    }
    let ready = state.db.ready_artifacts_for_scope(scope_type, scope_id)?;
    if ready.is_empty() {
        return Err(AppError::Message(
            "There is no ready transcript in this scope yet.".into(),
        ));
    }
    let (used, omitted) = pack_files(&ready);
    let mut history = state.db.get_chat_messages(scope_type, scope_id)?;
    let mut user_content = format!("{}\n\n{}", context_block(&used), question);
    if !omitted.is_empty() {
        user_content.push_str(&format!(
            "\n\n(Note: {} older ready transcripts were omitted to fit the context window.)",
            omitted.len()
        ));
    }
    history.push(ChatTurn::new("user", question));
    let mut model_messages = history
        .iter()
        .cloned()
        .take(history.len().saturating_sub(1))
        .collect::<Vec<_>>();
    model_messages.push(ChatTurn::new("user", user_content));
    let answer = llm::chat_completion(SYSTEM_PROMPT, &model_messages).await?;
    history.push(ChatTurn::new("assistant", answer));
    state
        .db
        .set_chat_messages(scope_type, scope_id, &history)?;
    Ok(ChatThreadView { messages: history })
}

pub fn get_thread(state: &Arc<AppState>, scope_type: &str, scope_id: &str) -> AppResult<ChatThreadView> {
    Ok(ChatThreadView {
        messages: state.db.get_chat_messages(scope_type, scope_id)?,
    })
}

pub fn new_thread(state: &Arc<AppState>, scope_type: &str, scope_id: &str) -> AppResult<ChatThreadView> {
    state.db.set_chat_messages(scope_type, scope_id, &[])?;
    Ok(ChatThreadView { messages: vec![] })
}
