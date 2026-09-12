import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { FileAudio, FileText, FileVideo, Loader2 } from "lucide-react";
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
const VIDEO_EXTS = ["mp4"];
const TRANSCRIPT_EXTS = ["vtt", "srt", "txt"];

export type ImportKind = "audio" | "video" | "transcript";

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

function allowedExts(kind: ImportKind) {
  if (kind === "audio") return AUDIO_EXTS;
  if (kind === "video") return VIDEO_EXTS;
  return TRANSCRIPT_EXTS;
}

function rejectMessage(kind: ImportKind, plural: boolean) {
  if (kind === "audio") {
    return plural
      ? "Some files were skipped. Use wav, mp3, m4a, aac, ogg, or flac."
      : "BellaNote needs audio files (wav, mp3, m4a, aac, ogg, or flac).";
  }
  if (kind === "video") {
    return plural
      ? "Some files were skipped. Use MP4 video files."
      : "BellaNote needs MP4 video files.";
  }
  return plural
    ? "Some files were skipped. Use .vtt, .srt, or .txt."
    : "Import a .vtt, .srt, or .txt transcript.";
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
    const allowed = allowedExts(currentKind);
    const accepted = paths.filter((path) => allowed.includes(extensionOf(path)));
    if (accepted.length === 0) {
      busyRef.current = false;
      toast.error(rejectMessage(currentKind, false));
      return;
    }
    if (accepted.length < paths.length) {
      toast.error(rejectMessage(currentKind, true));
    }

    setBusy(true);
    let added = 0;
    let lastError: string | null = null;
    try {
      for (const path of accepted) {
        try {
          if (currentKind === "audio") await api.importAudio(groupIdRef.current, path);
          else if (currentKind === "video") await api.importVideo(groupIdRef.current, path);
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
      const filter =
        kind === "audio"
          ? [{ name: "Audio", extensions: AUDIO_EXTS }]
          : kind === "video"
            ? [{ name: "Video", extensions: VIDEO_EXTS }]
            : [{ name: "Transcript", extensions: TRANSCRIPT_EXTS }];
      const selected = await openFileDialog({
        multiple: true,
        filters: filter,
      });
      if (!selected) return;
      await importPaths(Array.isArray(selected) ? selected : [selected]);
    } catch (err) {
      toast.error(errorMessage(err));
    }
  }

  const title =
    kind === "audio" ? "Import audio" : kind === "video" ? "Import video" : "Import transcript";
  const description =
    kind === "audio"
      ? "Drop one or more recordings, or click to choose them."
      : kind === "video"
        ? "Drop one or more MP4 videos, or click to choose them."
        : "Drop one or more .vtt, .srt, or .txt files, or click to choose them.";
  const extHint =
    kind === "audio"
      ? AUDIO_EXTS.join(", ")
      : kind === "video"
        ? VIDEO_EXTS.map((ext) => `.${ext}`).join(", ")
        : TRANSCRIPT_EXTS.map((ext) => `.${ext}`).join(", ");

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!busy) onOpenChange(next);
      }}
    >
      <DialogContent className="glass-panel sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
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
              {kind === "audio" ? (
                <FileAudio className="size-8 text-primary" />
              ) : kind === "video" ? (
                <FileVideo className="size-8 text-primary" />
              ) : (
                <FileText className="size-8 text-primary" />
              )}
              <div>
                <p className="text-sm font-medium">Drop files here or click to browse</p>
                <p className="mt-1 text-xs text-muted-foreground">{extHint}</p>
              </div>
            </>
          )}
        </button>
      </DialogContent>
    </Dialog>
  );
}
