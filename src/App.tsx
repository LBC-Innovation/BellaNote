import { useEffect, useState } from "react";
import { Settings2 } from "lucide-react";
import { ChatPanel } from "@/components/chat-panel";
import { LibraryPanel } from "@/components/library-panel";
import { SettingsDialog } from "@/components/settings-dialog";
import { WorkspacePanel } from "@/components/workspace-panel";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import * as api from "@/lib/api";
import { installArtifactListeners } from "@/store/useArtifactStore";
import { useLibraryStore } from "@/store/useLibraryStore";

const LIBRARY_COLLAPSED_KEY = "bellanote.libraryCollapsed";
const TRANSCRIPT_COLLAPSED_KEY = "bellanote.transcriptCollapsed";

function usePersistedFlag(key: string) {
  const [value, setValue] = useState(() => {
    try {
      return localStorage.getItem(key) === "1";
    } catch {
      return false;
    }
  });

  function change(next: boolean) {
    setValue(next);
    try {
      localStorage.setItem(key, next ? "1" : "0");
    } catch {
      /* ignore quota / private mode */
    }
  }

  return [value, change] as const;
}

function mainGridCols(libraryCollapsed: boolean, transcriptCollapsed: boolean) {
  const library = libraryCollapsed ? "56px" : "300px";
  const transcript = transcriptCollapsed ? "56px" : "minmax(0,1fr)";
  const chat = transcriptCollapsed ? "minmax(0,1fr)" : "340px";
  return `${library} ${transcript} ${chat}`;
}

export default function App() {
  const load = useLibraryStore((s) => s.load);
  const error = useLibraryStore((s) => s.error);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [keyConfigured, setKeyConfigured] = useState(false);
  const [libraryCollapsed, setLibraryCollapsed] = usePersistedFlag(LIBRARY_COLLAPSED_KEY);
  const [transcriptCollapsed, setTranscriptCollapsed] = usePersistedFlag(TRANSCRIPT_COLLAPSED_KEY);

  useEffect(() => {
    void load();
    void installArtifactListeners();
    void api.openAiKeyConfigured().then(setKeyConfigured);
  }, [load]);

  return (
    <div className="flex h-full flex-col text-foreground">
      <header
        className="flex h-12 shrink-0 items-center justify-between px-5"
        data-tauri-drag-region
      >
        <div className="flex items-center gap-2.5 pl-16">
          <span className="text-sm font-semibold tracking-tight">BellaNote</span>
          <Badge variant="secondary" className="font-normal">
            Beautiful Note
          </Badge>
        </div>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Settings"
          onClick={() => setSettingsOpen(true)}
        >
          <Settings2 />
        </Button>
      </header>

      {error ? <p className="px-6 pb-2 text-sm text-destructive">{error}</p> : null}

      <main
        className="grid min-h-0 flex-1 gap-3 px-3 pb-3 transition-[grid-template-columns] duration-300 ease-out"
        style={{ gridTemplateColumns: mainGridCols(libraryCollapsed, transcriptCollapsed) }}
      >
        <LibraryPanel collapsed={libraryCollapsed} onCollapsedChange={setLibraryCollapsed} />
        <WorkspacePanel collapsed={transcriptCollapsed} onCollapsedChange={setTranscriptCollapsed} />
        <ChatPanel keyConfigured={keyConfigured} onNeedKey={() => setSettingsOpen(true)} />
      </main>

      <SettingsDialog
        open={settingsOpen}
        configured={keyConfigured}
        onOpenChange={setSettingsOpen}
        onChanged={() => {
          void api.openAiKeyConfigured().then(setKeyConfigured);
        }}
      />
    </div>
  );
}
