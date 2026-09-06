import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { ChatPanel } from "@/components/chat-panel";
import { LibraryPanel } from "@/components/library-panel";
import { SettingsDialog } from "@/components/settings-dialog";
import { Titlebar } from "@/components/titlebar";
import { WorkspacePanel } from "@/components/workspace-panel";
import * as api from "@/lib/api";
import { installArtifactListeners } from "@/store/useArtifactStore";
import { useLibraryStore } from "@/store/useLibraryStore";

const LIBRARY_COLLAPSED_KEY = "bellanote.libraryCollapsed";
const TRANSCRIPT_COLLAPSED_KEY = "bellanote.transcriptCollapsed";
const CHAT_COLLAPSED_KEY = "bellanote.chatCollapsed";
const PANE_MS = 420;
const RAIL = 56;
const LIBRARY_OPEN = 300;
const GAP = 12;

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

function useContainerWidth() {
  const ref = useRef<HTMLElement>(null);
  const [width, setWidth] = useState(0);

  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const read = () => {
      const style = getComputedStyle(el);
      return el.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
    };
    setWidth(read());
    const observer = new ResizeObserver(() => setWidth(read()));
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  return [ref, width] as const;
}

function paneWidths(
  containerWidth: number,
  libraryCollapsed: boolean,
  transcriptCollapsed: boolean,
  chatCollapsed: boolean,
): [number, number, number] {
  const available = Math.max(0, containerWidth - GAP * 2);
  const widths: [number, number, number] = [0, 0, 0];
  const flex: number[] = [];

  if (libraryCollapsed) {
    widths[0] = RAIL;
  } else if (!(transcriptCollapsed && chatCollapsed)) {
    widths[0] = LIBRARY_OPEN;
  } else {
    flex.push(0);
  }

  if (transcriptCollapsed) widths[1] = RAIL;
  else flex.push(1);

  if (chatCollapsed) widths[2] = RAIL;
  else flex.push(2);

  const reserved = widths.reduce((sum, value) => sum + value, 0);
  const share = flex.length ? Math.max(0, available - reserved) / flex.length : 0;
  for (const index of flex) widths[index] = share;
  return widths;
}

function useDeferredCollapsed(collapsed: boolean) {
  const [visual, setVisual] = useState(collapsed);

  useEffect(() => {
    if (!collapsed) {
      setVisual(false);
      return;
    }
    const id = window.setTimeout(() => setVisual(true), PANE_MS - 90);
    return () => window.clearTimeout(id);
  }, [collapsed]);

  return visual;
}

function PaneShell({
  collapsed,
  width,
  animate,
  children,
}: {
  collapsed: boolean;
  width: number;
  animate: boolean;
  children: (visualCollapsed: boolean) => ReactNode;
}) {
  const visualCollapsed = useDeferredCollapsed(collapsed);

  return (
    <div
      className="flex min-h-0 min-w-0 shrink-0 flex-col overflow-hidden"
      style={{
        width,
        minWidth: RAIL,
        transition: animate ? `width ${PANE_MS}ms ease-in-out` : undefined,
      }}
    >
      <div className="flex h-full min-h-0 w-full min-w-0 flex-col">{children(visualCollapsed)}</div>
    </div>
  );
}

export default function App() {
  const load = useLibraryStore((s) => s.load);
  const error = useLibraryStore((s) => s.error);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [keyConfigured, setKeyConfigured] = useState(false);
  const [libraryCollapsed, setLibraryCollapsed] = usePersistedFlag(LIBRARY_COLLAPSED_KEY);
  const [transcriptCollapsed, setTranscriptCollapsed] = usePersistedFlag(TRANSCRIPT_COLLAPSED_KEY);
  const [chatCollapsed, setChatCollapsed] = usePersistedFlag(CHAT_COLLAPSED_KEY);
  const [mainRef, mainWidth] = useContainerWidth();
  const [animatePanes, setAnimatePanes] = useState(false);

  const [libraryWidth, transcriptWidth, chatWidth] = paneWidths(
    mainWidth,
    libraryCollapsed,
    transcriptCollapsed,
    chatCollapsed,
  );

  const layoutFocus =
    !libraryCollapsed && transcriptCollapsed && chatCollapsed
      ? "library"
      : libraryCollapsed && !transcriptCollapsed && chatCollapsed
        ? "transcript"
        : libraryCollapsed && transcriptCollapsed && !chatCollapsed
          ? "chat"
          : null;

  function showOnly(pane: "library" | "transcript" | "chat") {
    setLibraryCollapsed(pane !== "library");
    setTranscriptCollapsed(pane !== "transcript");
    setChatCollapsed(pane !== "chat");
  }

  useLayoutEffect(() => {
    if (mainWidth <= 0 || animatePanes) return;
    const id = requestAnimationFrame(() => setAnimatePanes(true));
    return () => cancelAnimationFrame(id);
  }, [mainWidth, animatePanes]);

  useEffect(() => {
    void load();
    void installArtifactListeners();
    void api.openAiKeyConfigured().then(setKeyConfigured);
  }, [load]);

  return (
    <div className="flex h-full flex-col text-foreground">
      <Titlebar
        layoutFocus={layoutFocus}
        onShowOnly={showOnly}
        onOpenSettings={() => setSettingsOpen(true)}
      />

      {error ? <p className="px-6 pb-2 text-sm text-destructive">{error}</p> : null}

      <main ref={mainRef} className="flex min-h-0 flex-1 gap-3 px-3 pb-3">
        <PaneShell collapsed={libraryCollapsed} width={libraryWidth} animate={animatePanes}>
          {(collapsed) => (
            <LibraryPanel collapsed={collapsed} onCollapsedChange={setLibraryCollapsed} />
          )}
        </PaneShell>
        <PaneShell collapsed={transcriptCollapsed} width={transcriptWidth} animate={animatePanes}>
          {(collapsed) => (
            <WorkspacePanel collapsed={collapsed} onCollapsedChange={setTranscriptCollapsed} />
          )}
        </PaneShell>
        <PaneShell collapsed={chatCollapsed} width={chatWidth} animate={animatePanes}>
          {(collapsed) => (
            <ChatPanel
              collapsed={collapsed}
              onCollapsedChange={setChatCollapsed}
              keyConfigured={keyConfigured}
              onNeedKey={() => setSettingsOpen(true)}
            />
          )}
        </PaneShell>
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
