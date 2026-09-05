use crate::db::{LibraryOrganization, MeetingGroup, Organization, Topic};
use crate::error::AppResult;
use crate::state::AppState;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

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
pub fn delete_meeting_group(state: State<'_, Arc<AppState>>, args: IdArgs) -> AppResult<()> {
    state.db.delete_meeting_group(&args.id)
}
