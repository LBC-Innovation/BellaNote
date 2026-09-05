import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { FileAudio, FileText, Loader2 } from "lucide-react";
import { toast } from "sonner";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import * as api from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { cn } from "@/lib/utils";

const AUDIO_EXTS = ["wav", "mp3", "m4a", "aac", "ogg", "flac"];
const TRANSCRIPT_EXTS = ["vtt", "srt", "txt"];

type ImportKind = "audio" | "transcript";

function extensionOf(path: string) {
  const name = path.split("/").pop() ?? path;
  const dot = name.lastIndexOf(".");
  return dot >= 0 ? name.slice(dot + 1).toLowerCase() : "";
}

function pathsFromDataTransfer(transfer: DataTransfer | null) {
  if (!transfer) return [];
  return Array.from(transfer.files)
    .map((file) => (file as File & { path?: string }).path)
    .filter((path): path is string => Boolean(path));
}

export function ImportDropDialog({
  open,
  kind,
  groupId,
  onOpenChange,
  onImported,
}: {
  open: boolean;
  kind: ImportKind | null;
  groupId: string;
  onOpenChange: (open: boolean) => void;
  onImported: () => Promise<void>;
}) {
  const canvasRef = useRef<HTMLButtonElement | null>(null);
  const busyRef = useRef(false);
  const tauriDropRef = useRef(false);
  const kindRef = useRef(kind);
  const groupIdRef = useRef(groupId);
  const onImportedRef = useRef(onImported);
  const onOpenChangeRef = useRef(onOpenChange);
  const [hovering, setHovering] = useState(false);
  const [busy, setBusy] = useState(false);

  kindRef.current = kind;
  groupIdRef.current = groupId;
  onImportedRef.current = onImported;
  onOpenChangeRef.current = onOpenChange;

  async function importPaths(paths: string[]) {
    const currentKind = kindRef.current;
    if (!currentKind || busyRef.current || paths.length === 0) return;
    busyRef.current = true;
    const allowed = currentKind === "audio" ? AUDIO_EXTS : TRANSCRIPT_EXTS;
    const accepted = paths.filter((path) => allowed.includes(extensionOf(path)));
    if (accepted.length === 0) {
      busyRef.current = false;
      toast.error(
        currentKind === "audio"
          ? "BellaNote needs audio files (wav, mp3, m4a, aac, ogg, or flac)."
          : "Import a .vtt, .srt, or .txt transcript.",
      );
      return;
    }
    if (accepted.length < paths.length) {
      toast.error(
        currentKind === "audio"
          ? "Some files were skipped. Use wav, mp3, m4a, aac, ogg, or flac."
          : "Some files were skipped. Use .vtt, .srt, or .txt.",
      );
    }

    setBusy(true);
    let added = 0;
    let lastError: string | null = null;
    try {
      for (const path of accepted) {
        try {
          if (currentKind === "audio") await api.importAudio(groupIdRef.current, path);
          else await api.importTranscript(groupIdRef.current, path);
          added += 1;
        } catch (err) {
          lastError = errorMessage(err);
        }
      }
      if (added > 0) {
        await onImportedRef.current();
        onOpenChangeRef.current(false);
      }
      if (lastError) toast.error(lastError);
    } finally {
      busyRef.current = false;
      setBusy(false);
      setHovering(false);
    }
  }

  useEffect(() => {
    if (!open) {
      setHovering(false);
      setBusy(false);
      busyRef.current = false;
      return;
    }

    let disposed = false;
    const unlistens: Array<() => void> = [];

    function onDragDrop(event: { payload: { type: string; paths?: string[] } }) {
      if (event.payload.type === "enter" || event.payload.type === "over") {
        setHovering(true);
        void getCurrentWindow().setFocus().catch(() => undefined);
      } else if (event.payload.type === "leave") {
        setHovering(false);
      } else if (event.payload.type === "drop") {
        setHovering(false);
        tauriDropRef.current = true;
        void importPaths(event.payload.paths ?? []);
      }
    }

    const hosts = [getCurrentWebviewWindow(), getCurrentWebview(), getCurrentWindow()];
    for (const host of hosts) {
      void host
        .onDragDropEvent(onDragDrop)
        .then((fn) => {
          if (disposed) fn();
          else unlistens.push(fn);
        })
        .catch(() => undefined);
    }

    return () => {
      disposed = true;
      for (const fn of unlistens) fn();
    };
  }, [open]);

  async function pickFiles() {
    if (!kind || busyRef.current) return;
    try {
      const selected = await openFileDialog({
        multiple: true,
        filters:
          kind === "audio"
            ? [{ name: "Audio", extensions: AUDIO_EXTS }]
            : [{ name: "Transcript", extensions: TRANSCRIPT_EXTS }],
      });
      if (!selected) return;
      await importPaths(Array.isArray(selected) ? selected : [selected]);
    } catch (err) {
      toast.error(errorMessage(err));
    }
  }

  const audio = kind === "audio";

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!busy) onOpenChange(next);
      }}
    >
      <DialogContent className="glass-panel sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{audio ? "Import audio" : "Import transcript"}</DialogTitle>
          <DialogDescription>
            {audio
              ? "Drop one or more recordings, or click to choose them."
              : "Drop one or more .vtt, .srt, or .txt files, or click to choose them."}
          </DialogDescription>
        </DialogHeader>

        <button
          ref={canvasRef}
          type="button"
          disabled={busy}
          onClick={() => void pickFiles()}
          onDragOver={(event) => {
            event.preventDefault();
            setHovering(true);
          }}
          onDragLeave={() => setHovering(false)}
          onDrop={(event) => {
            event.preventDefault();
            setHovering(false);
            if (tauriDropRef.current) {
              tauriDropRef.current = false;
              return;
            }
            void importPaths(pathsFromDataTransfer(event.dataTransfer));
          }}
          className={cn(
            "relative flex min-h-56 w-full flex-col items-center justify-center gap-3 rounded-2xl border border-dashed px-6 py-10 text-center transition-colors",
            hovering
              ? "border-primary bg-primary/12"
              : "border-white/20 bg-white/[0.04] hover:border-primary/45 hover:bg-white/[0.07]",
          )}
        >
          {busy ? (
            <>
              <Loader2 className="size-8 animate-spin text-primary" />
              <p className="text-sm font-medium">Adding files…</p>
            </>
          ) : (
            <>
              {audio ? (
                <FileAudio className="size-8 text-primary" />
              ) : (
                <FileText className="size-8 text-primary" />
              )}
              <div>
                <p className="text-sm font-medium">Drop files here or click to browse</p>
                <p className="mt-1 text-xs text-muted-foreground">
                  {audio ? AUDIO_EXTS.join(", ") : TRANSCRIPT_EXTS.map((ext) => `.${ext}`).join(", ")}
                </p>
              </div>
            </>
          )}
        </button>
      </DialogContent>
    </Dialog>
  );
}
