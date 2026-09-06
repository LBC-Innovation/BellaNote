import { invoke } from "@tauri-apps/api/core";
import type {
  Artifact,
  ArtifactComment,
  LibraryOrganization,
  MeetingGroup,
  Organization,
  Topic,
} from "./types";

export function getLibrary() {
  return invoke<LibraryOrganization[]>("get_library");
}

export function createOrganization(name: string) {
  return invoke<Organization>("create_organization", { args: { name } });
}

export function renameOrganization(id: string, name: string) {
  return invoke<Organization>("rename_organization", { args: { id, name } });
}

export function deleteOrganization(id: string) {
  return invoke<void>("delete_organization", { args: { id } });
}

export function createTopic(organizationId: string, name: string) {
  return invoke<Topic>("create_topic", { args: { organizationId, name } });
}

export function renameTopic(id: string, name: string) {
  return invoke<Topic>("rename_topic", { args: { id, name } });
}

export function deleteTopic(id: string) {
  return invoke<void>("delete_topic", { args: { id } });
}

export function createMeetingGroup(topicId: string, name: string, occurredAt?: string) {
  return invoke<MeetingGroup>("create_meeting_group", {
    args: { topicId, name, occurredAt: occurredAt ?? null },
  });
}

export function renameMeetingGroup(id: string, name: string) {
  return invoke<MeetingGroup>("rename_meeting_group", { args: { id, name } });
}

export function deleteMeetingGroup(id: string) {
  return invoke<void>("delete_meeting_group", { args: { id } });
}

export function listArtifacts(meetingGroupId: string) {
  return invoke<Artifact[]>("list_artifacts", { args: { meetingGroupId } });
}

export function getArtifact(id: string) {
  return invoke<Artifact>("get_artifact", { args: { id } });
}

export function importAudio(meetingGroupId: string, path: string) {
  return invoke<Artifact>("import_audio", { args: { meetingGroupId, path } });
}

export function importTranscript(meetingGroupId: string, path: string) {
  return invoke<Artifact>("import_transcript", { args: { meetingGroupId, path } });
}

export function renameArtifact(id: string, name: string) {
  return invoke<Artifact>("rename_artifact", { args: { id, name } });
}

export function deleteArtifact(id: string) {
  return invoke<void>("delete_artifact", { args: { id } });
}

export function retryArtifact(id: string) {
  return invoke<Artifact>("retry_artifact", { args: { id } });
}

export function getArtifactAudioPath(id: string) {
  return invoke<string | null>("get_artifact_audio_path", { args: { id } });
}

export type AudioPeaks = {
  peaks: number[];
  durationSecs: number;
};

export function getArtifactAudioPeaks(id: string) {
  return invoke<AudioPeaks>("get_artifact_audio_peaks", { args: { id } });
}

export function listArtifactComments(artifactId: string) {
  return invoke<ArtifactComment[]>("list_artifact_comments", { args: { artifactId } });
}

export function createArtifactComment(artifactId: string, timeMs: number, body: string) {
  return invoke<ArtifactComment>("create_artifact_comment", {
    args: { artifactId, timeMs: Math.round(timeMs), body },
  });
}

export function updateArtifactComment(id: string, body: string) {
  return invoke<ArtifactComment>("update_artifact_comment", { args: { id, body } });
}

export function deleteArtifactComment(id: string) {
  return invoke<void>("delete_artifact_comment", { args: { id } });
}

export type ChatTurn = { role: string; content: string; createdAt?: string };
export type ScopeFile = {
  id: string;
  title: string;
  meetingGroupId: string;
  included: boolean;
};
export type ScopePreview = {
  readyCount: number;
  totalCount: number;
  usedCount: number;
  omittedCount: number;
  files: ScopeFile[];
};
export type ChatScopeType = "organization" | "topic" | "meeting_group" | "artifact";

export function setOpenAiKey(apiKey: string) {
  return invoke<void>("set_openai_api_key", { args: { apiKey } });
}

export function clearOpenAiKey() {
  return invoke<void>("clear_openai_api_key");
}

export function openAiKeyConfigured() {
  return invoke<boolean>("openai_api_key_configured");
}

export function chatScopePreview(scopeType: ChatScopeType, scopeId: string) {
  return invoke<ScopePreview>("chat_scope_preview", { args: { scopeType, scopeId } });
}

export function getChatThread(scopeType: ChatScopeType, scopeId: string) {
  return invoke<{ messages: ChatTurn[] }>("get_chat_thread", { args: { scopeType, scopeId } });
}

export function newChatThread(scopeType: ChatScopeType, scopeId: string) {
  return invoke<{ messages: ChatTurn[] }>("new_chat_thread", { args: { scopeType, scopeId } });
}

export function askChat(scopeType: ChatScopeType, scopeId: string, question: string) {
  return invoke<{ messages: ChatTurn[] }>("ask_chat", { args: { scopeType, scopeId, question } });
}
