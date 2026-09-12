use crate::artifacts;
use crate::audio_peaks::{self, AudioPeaksResult};
use crate::chat::{self, ChatThreadView, ScopePreview};
use crate::db::{
    Artifact, ArtifactComment, LibraryOrganization, MeetingGroup, Organization, Topic,
};
use crate::error::AppResult;
use crate::llm;
use crate::recording::{self, RecordingCapabilities, RecordingStatus, StartRecordingResult};
use crate::state::AppState;
use serde::Deserialize;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameArgs {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameArgs {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdArgs {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteArtifactArgs {
    pub id: String,
    /// When true, also remove BellaNote’s library audio. Never deletes the user’s original import path.
    #[serde(default)]
    pub delete_audio: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTopicArgs {
    pub organization_id: String,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupArgs {
    pub topic_id: String,
    pub name: String,
    pub occurred_at: Option<String>,
}

#[tauri::command]
pub fn get_library(state: State<'_, Arc<AppState>>) -> AppResult<Vec<LibraryOrganization>> {
    state.db.get_library()
}

#[tauri::command]
pub fn create_organization(
    state: State<'_, Arc<AppState>>,
    args: NameArgs,
) -> AppResult<Organization> {
    state.db.create_organization(&args.name)
}

#[tauri::command]
pub fn rename_organization(
    state: State<'_, Arc<AppState>>,
    args: RenameArgs,
) -> AppResult<Organization> {
    state.db.rename_organization(&args.id, &args.name)
}

#[tauri::command]
pub fn delete_organization(state: State<'_, Arc<AppState>>, args: IdArgs) -> AppResult<()> {
    state.db.delete_organization(&args.id)
}

#[tauri::command]
pub fn create_topic(state: State<'_, Arc<AppState>>, args: CreateTopicArgs) -> AppResult<Topic> {
    state.db.create_topic(&args.organization_id, &args.name)
}

#[tauri::command]
pub fn rename_topic(state: State<'_, Arc<AppState>>, args: RenameArgs) -> AppResult<Topic> {
    state.db.rename_topic(&args.id, &args.name)
}

#[tauri::command]
pub fn delete_topic(state: State<'_, Arc<AppState>>, args: IdArgs) -> AppResult<()> {
    state.db.delete_topic(&args.id)
}

#[tauri::command]
pub fn create_meeting_group(
    state: State<'_, Arc<AppState>>,
    args: CreateGroupArgs,
) -> AppResult<MeetingGroup> {
    state.db.create_meeting_group(&args.topic_id, &args.name, args.occurred_at)
}

#[tauri::command]
pub fn rename_meeting_group(
    state: State<'_, Arc<AppState>>,
    args: RenameArgs,
) -> AppResult<MeetingGroup> {
    state.db.rename_meeting_group(&args.id, &args.name)
}

#[tauri::command]
pub async fn delete_meeting_group(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    args: IdArgs,
) -> AppResult<()> {
    let state = state.inner().clone();
    recording::abort_if_group(&app, &state, &args.id).await;
    state.db.delete_meeting_group(&args.id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupIdArgs {
    pub meeting_group_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFileArgs {
    pub meeting_group_id: String,
    pub path: String,
}

#[tauri::command]
pub fn list_artifacts(
    state: State<'_, Arc<AppState>>,
    args: GroupIdArgs,
) -> AppResult<Vec<Artifact>> {
    state.db.list_artifacts(&args.meeting_group_id)
}

#[tauri::command]
pub fn get_artifact(state: State<'_, Arc<AppState>>, args: IdArgs) -> AppResult<Artifact> {
    state.db.get_artifact(&args.id)
}

#[tauri::command]
pub fn import_audio(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    args: ImportFileArgs,
) -> AppResult<Artifact> {
    artifacts::import_audio(&app, &state, &args.meeting_group_id, &args.path)
}

#[tauri::command]
pub fn import_video(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    args: ImportFileArgs,
) -> AppResult<Artifact> {
    artifacts::import_video(&app, &state, &args.meeting_group_id, &args.path)
}

#[tauri::command]
pub fn import_transcript(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    args: ImportFileArgs,
) -> AppResult<Artifact> {
    artifacts::import_transcript_file(&app, &state, &args.meeting_group_id, &args.path)
}

#[tauri::command]
pub fn rename_artifact(state: State<'_, Arc<AppState>>, args: RenameArgs) -> AppResult<Artifact> {
    state.db.rename_artifact(&args.id, &args.name)
}

#[tauri::command]
pub async fn delete_artifact(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    args: DeleteArtifactArgs,
) -> AppResult<()> {
    let state = state.inner().clone();
    let artifact = state.db.get_artifact(&args.id)?;
    recording::abort_if_artifact(&app, &state, &args.id).await;
    state.db.delete_artifact(&args.id)?;
    let is_recording = artifact.source_type == "voice" || artifact.source_type == "system";
    // Imports always drop BellaNote’s library copy. Recordings only when asked.
    // Never delete the user’s original import path (Downloads, etc.).
    if args.delete_audio || !is_recording {
        artifacts::delete_artifact_files(&app, &args.id);
    }
    audio_peaks::invalidate_artifact(&args.id);
    Ok(())
}

#[tauri::command]
pub fn get_artifact_audio_path(app: AppHandle, args: IdArgs) -> AppResult<Option<String>> {
    Ok(artifacts::find_audio_path(&app, &args.id).map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
pub fn get_artifact_video_path(app: AppHandle, args: IdArgs) -> AppResult<Option<String>> {
    Ok(artifacts::find_video_path(&app, &args.id).map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
pub async fn get_artifact_audio_peaks(app: AppHandle, args: IdArgs) -> AppResult<AudioPeaksResult> {
    let path = artifacts::find_audio_path(&app, &args.id).ok_or_else(|| {
        crate::error::AppError::Message("The original audio is missing.".into())
    })?;
    let cache_key = audio_peaks::cache_key(&args.id, &path)?;
    if let Some(cached) = audio_peaks::get_cached(&cache_key) {
        return Ok(cached);
    }
    let result = tokio::task::spawn_blocking(move || {
        audio_peaks::compute_audio_peaks(&path, audio_peaks::PEAK_BUCKETS)
    })
    .await
    .map_err(|e| crate::error::AppError::Message(e.to_string()))?
    .map_err(crate::error::AppError::from)?;
    audio_peaks::put_cached(cache_key, result.clone());
    Ok(result)
}

#[tauri::command]
pub fn retry_artifact(app: AppHandle, state: State<'_, Arc<AppState>>, args: IdArgs) -> AppResult<Artifact> {
    artifacts::retry_artifact(&app, &state, &args.id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportArtifactAudioArgs {
    pub id: String,
    pub dest_path: String,
}

#[tauri::command]
pub fn export_artifact_audio(app: AppHandle, args: ExportArtifactAudioArgs) -> AppResult<()> {
    artifacts::export_artifact_audio(&app, &args.id, &args.dest_path)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportArtifactTranscriptArgs {
    pub id: String,
    pub dest_path: String,
}

#[tauri::command]
pub fn export_artifact_transcript(
    state: State<'_, Arc<AppState>>,
    args: ExportArtifactTranscriptArgs,
) -> AppResult<()> {
    artifacts::export_artifact_transcript(&state, &args.id, &args.dest_path)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactIdArgs {
    pub artifact_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentArgs {
    pub artifact_id: String,
    pub time_ms: i64,
    pub body: String,
}

#[tauri::command]
pub fn list_artifact_comments(
    state: State<'_, Arc<AppState>>,
    args: ArtifactIdArgs,
) -> AppResult<Vec<ArtifactComment>> {
    state.db.list_artifact_comments(&args.artifact_id)
}

#[tauri::command]
pub fn create_artifact_comment(
    state: State<'_, Arc<AppState>>,
    args: CreateCommentArgs,
) -> AppResult<ArtifactComment> {
    state
        .db
        .create_artifact_comment(&args.artifact_id, args.time_ms, &args.body)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommentArgs {
    pub id: String,
    pub body: String,
}

#[tauri::command]
pub fn update_artifact_comment(
    state: State<'_, Arc<AppState>>,
    args: UpdateCommentArgs,
) -> AppResult<ArtifactComment> {
    state.db.update_artifact_comment(&args.id, &args.body)
}

#[tauri::command]
pub fn delete_artifact_comment(state: State<'_, Arc<AppState>>, args: IdArgs) -> AppResult<()> {
    state.db.delete_artifact_comment(&args.id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyArgs {
    pub api_key: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAskArgs {
    pub scope_type: String,
    pub scope_id: String,
    pub question: String,
}

#[tauri::command]
pub fn set_openai_api_key(args: ApiKeyArgs) -> AppResult<()> {
    llm::set_api_key(&args.api_key)
}

#[tauri::command]
pub fn clear_openai_api_key() -> AppResult<()> {
    llm::clear_api_key()
}

#[tauri::command]
pub fn openai_api_key_configured() -> bool {
    llm::api_key_configured()
}

#[tauri::command]
pub fn chat_scope_preview(
    state: State<'_, Arc<AppState>>,
    args: chat::ScopeArgs,
) -> AppResult<ScopePreview> {
    chat::preview(&state, &args.scope_type, &args.scope_id)
}

#[tauri::command]
pub fn get_chat_thread(
    state: State<'_, Arc<AppState>>,
    args: chat::ScopeArgs,
) -> AppResult<ChatThreadView> {
    chat::get_thread(&state, &args.scope_type, &args.scope_id)
}

#[tauri::command]
pub fn new_chat_thread(
    state: State<'_, Arc<AppState>>,
    args: chat::ScopeArgs,
) -> AppResult<ChatThreadView> {
    chat::new_thread(&state, &args.scope_type, &args.scope_id)
}

#[tauri::command]
pub async fn ask_chat(
    state: State<'_, Arc<AppState>>,
    args: ChatAskArgs,
) -> AppResult<ChatThreadView> {
    let state = state.inner().clone();
    chat::ask(&state, &args.scope_type, &args.scope_id, &args.question).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRecordingArgs {
    pub meeting_group_id: String,
    pub source: String,
}

#[tauri::command]
pub async fn start_recording(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    args: StartRecordingArgs,
) -> AppResult<StartRecordingResult> {
    recording::start_recording(
        app,
        state.inner().clone(),
        args.meeting_group_id,
        args.source,
    )
    .await
}

#[tauri::command]
pub async fn stop_recording(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> AppResult<Artifact> {
    recording::stop_recording(app, state.inner().clone()).await
}

#[tauri::command]
pub fn recording_status(state: State<'_, Arc<AppState>>) -> RecordingStatus {
    recording::status(&state)
}

#[tauri::command]
pub fn recording_capabilities() -> RecordingCapabilities {
    recording::capabilities()
}

#[tauri::command]
pub fn get_rms(state: State<'_, Arc<AppState>>) -> f32 {
    recording::rms(&state)
}

#[tauri::command]
pub fn get_spectrum(state: State<'_, Arc<AppState>>) -> Vec<f32> {
    recording::spectrum(&state)
}
