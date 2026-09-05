import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import * as api from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import type { Artifact } from "@/lib/types";

type ArtifactState = {
  groupId: string | null;
  artifacts: Artifact[];
  activeId: string | null;
  loading: boolean;
  error: string | null;
  load: (groupId: string | null) => Promise<void>;
  setActive: (id: string | null) => void;
  refresh: () => Promise<void>;
};

export const useArtifactStore = create<ArtifactState>((set, get) => ({
  groupId: null,
  artifacts: [],
  activeId: null,
  loading: false,
  error: null,
  load: async (groupId) => {
    if (!groupId) {
      set({ groupId: null, artifacts: [], activeId: null, loading: false, error: null });
      return;
    }
    set({ groupId, loading: true, error: null });
    try {
      const artifacts = await api.listArtifacts(groupId);
      const activeId = artifacts.some((item) => item.id === get().activeId)
        ? get().activeId
        : (artifacts[0]?.id ?? null);
      set({ artifacts, activeId, loading: false });
    } catch (err) {
      set({ loading: false, error: errorMessage(err) });
    }
  },
  setActive: (id) => set({ activeId: id }),
  refresh: async () => {
    const groupId = get().groupId;
    if (groupId) await get().load(groupId);
  },
}));

let listening = false;
export async function installArtifactListeners() {
  if (listening) return;
  listening = true;
  await listen<string>("artifact-updated", () => {
    void useArtifactStore.getState().refresh();
  });
}
