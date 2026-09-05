import { useEffect } from "react";
import { Settings2 } from "lucide-react";
import { ChatPanel } from "@/components/chat-panel";
import { LibraryPanel } from "@/components/library-panel";
import { WorkspacePanel } from "@/components/workspace-panel";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useLibraryStore } from "@/store/useLibraryStore";

export default function App() {
  const load = useLibraryStore((s) => s.load);
  const error = useLibraryStore((s) => s.error);

  useEffect(() => {
    void load();
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
        <Button variant="ghost" size="icon-sm" aria-label="Settings" disabled>
          <Settings2 />
        </Button>
      </header>

      {error ? (
        <p className="px-6 pb-2 text-sm text-destructive">{error}</p>
      ) : null}

      <main className="grid min-h-0 flex-1 grid-cols-[300px_minmax(0,1fr)_340px] gap-3 px-3 pb-3">
        <LibraryPanel />
        <WorkspacePanel />
        <ChatPanel />
      </main>
    </div>
  );
}
