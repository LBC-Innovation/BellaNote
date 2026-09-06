import { create } from "zustand";
import * as api from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { useArtifactStore } from "@/store/useArtifactStore";
import type { RecordingSource } from "@/lib/api";

type RecordingState = {
  active: boolean;
  pending: boolean;
  artifactId: string | null;
  groupId: string | null;
  source: RecordingSource | null;
  startedAtUnixMs: number | null;
  inputLabel: string | null;
  error: string | null;
  systemAudioAvailable: boolean;
  hydrate: () => Promise<void>;
  start: (meetingGroupId: string, source: RecordingSource) => Promise<void>;
  stop: () => Promise<void>;
};

export const useRecordingStore = create<RecordingState>((set, get) => ({
  active: false,
  pending: false,
  artifactId: null,
  groupId: null,
  source: null,
  startedAtUnixMs: null,
  inputLabel: null,
  error: null,
  systemAudioAvailable: true,
  hydrate: async () => {
    try {
      const [status, caps] = await Promise.all([
        api.recordingStatus(),
        api.recordingCapabilities(),
      ]);
      set({
        active: status.active,
        artifactId: status.artifactId,
        groupId: status.meetingGroupId,
        source: status.source === "system" ? "system" : status.source === "voice" ? "voice" : null,
        startedAtUnixMs: status.startedAtUnixMs,
        inputLabel: status.inputLabel,
        systemAudioAvailable: caps.systemAudio,
      });
    } catch {
      /* native commands unavailable in browser preview */
    }
  },
  start: async (meetingGroupId, source) => {
    if (get().active || get().pending) return;
    set({ pending: true, error: null });
    try {
      const result = await api.startRecording(meetingGroupId, source);
      set({
        active: true,
        pending: false,
        artifactId: result.artifact.id,
        groupId: meetingGroupId,
        source,
        startedAtUnixMs: Date.now(),
        inputLabel: result.inputLabel,
      });
      await useArtifactStore.getState().load(meetingGroupId);
      useArtifactStore.getState().setActive(result.artifact.id);
    } catch (err) {
      set({ pending: false, error: errorMessage(err) });
      throw err;
    }
  },
  stop: async () => {
    if (!get().active || get().pending) return;
    set({ pending: true });
    const groupId = get().groupId;
    try {
      await api.stopRecording();
      set({
        active: false,
        pending: false,
        artifactId: null,
        groupId: null,
        source: null,
        startedAtUnixMs: null,
        inputLabel: null,
      });
      if (groupId) await useArtifactStore.getState().load(groupId);
    } catch (err) {
      set({ pending: false, error: errorMessage(err) });
      throw err;
    }
  },
}));
