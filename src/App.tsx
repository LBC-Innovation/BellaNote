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

export default function App() {
  const load = useLibraryStore((s) => s.load);
  const error = useLibraryStore((s) => s.error);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [keyConfigured, setKeyConfigured] = useState(false);

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

      <main className="grid min-h-0 flex-1 grid-cols-[300px_minmax(0,1fr)_340px] gap-3 px-3 pb-3">
        <LibraryPanel />
        <WorkspacePanel />
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
