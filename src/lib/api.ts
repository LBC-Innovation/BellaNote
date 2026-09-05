import { invoke } from "@tauri-apps/api/core";
import type { Artifact, LibraryOrganization, MeetingGroup, Organization, Topic } from "./types";

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

export function getArtifactAudioPath(id: string) {
  return invoke<string | null>("get_artifact_audio_path", { args: { id } });
}
