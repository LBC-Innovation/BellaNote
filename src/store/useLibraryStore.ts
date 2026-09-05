import { create } from "zustand";
import * as api from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import type { LibraryOrganization, Selection } from "@/lib/types";

type LibraryState = {
  orgs: LibraryOrganization[];
  selection: Selection;
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
  select: (selection: Selection) => void;
};

function stillValid(orgs: LibraryOrganization[], selection: Selection): Selection {
  if (selection.kind === "none") return selection;
  const org = orgs.find((o) => o.id === selection.organizationId);
  if (!org) return { kind: "none" };
  if (selection.kind === "organization") return selection;
  const topic = org.topics.find((t) => t.id === selection.topicId);
  if (!topic) return { kind: "organization", organizationId: org.id };
  if (selection.kind === "topic") return selection;
  if (!topic.groups.some((g) => g.id === selection.groupId)) {
    return { kind: "topic", organizationId: org.id, topicId: topic.id };
  }
  return selection;
}

export const useLibraryStore = create<LibraryState>((set, get) => ({
  orgs: [],
  selection: { kind: "none" },
  loading: false,
  error: null,
  load: async () => {
    set({ loading: true, error: null });
    try {
      const orgs = await api.getLibrary();
      set({ orgs, selection: stillValid(orgs, get().selection), loading: false });
    } catch (err) {
      set({ loading: false, error: errorMessage(err) });
    }
  },
  select: (selection) => set({ selection }),
}));
