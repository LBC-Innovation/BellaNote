import type { ReactNode } from "react";
import { FileText, MessageCircle, PanelLeft, Settings2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { isMac, isWindows } from "@/lib/platform";

type Pane = "library" | "transcript" | "chat";

function TitlebarButton({
  label,
  pressed,
  onClick,
  children,
}: {
  label: string;
  pressed?: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      aria-pressed={pressed}
      onClick={onClick}
      className={cn(
        "flex size-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-white/10 hover:text-foreground",
        pressed && "bg-white/10 text-foreground",
      )}
    >
      {children}
    </button>
  );
}

export function Titlebar({
  layoutFocus,
  onShowOnly,
  onOpenSettings,
}: {
  layoutFocus: Pane | null;
  onShowOnly: (pane: Pane) => void;
  onOpenSettings: () => void;
}) {
  return (
    <header
      className={cn("flex h-12 shrink-0 items-stretch", isWindows() && "pr-[140px]")}
    >
      {isMac() ? <div className="w-[72px] shrink-0" /> : <div className="w-3 shrink-0" />}
      <div className="flex min-w-0 flex-1 items-center" data-tauri-drag-region>
        <span className="pointer-events-none select-none text-sm font-semibold tracking-tight">
          BellaNote
        </span>
      </div>
      <nav className="flex shrink-0 items-center gap-1 px-3" aria-label="Window layout">
        <TitlebarButton
          label="Show library only"
          pressed={layoutFocus === "library"}
          onClick={() => onShowOnly("library")}
        >
          <PanelLeft className="size-4" />
        </TitlebarButton>
        <TitlebarButton
          label="Show transcript only"
          pressed={layoutFocus === "transcript"}
          onClick={() => onShowOnly("transcript")}
        >
          <FileText className="size-4" />
        </TitlebarButton>
        <TitlebarButton
          label="Show chat only"
          pressed={layoutFocus === "chat"}
          onClick={() => onShowOnly("chat")}
        >
          <MessageCircle className="size-4" />
        </TitlebarButton>
        <TitlebarButton label="Settings" onClick={onOpenSettings}>
          <Settings2 className="size-4" />
        </TitlebarButton>
      </nav>
    </header>
  );
}
