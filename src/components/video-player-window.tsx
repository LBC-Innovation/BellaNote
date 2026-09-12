import { useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import * as api from "@/lib/api";
import { errorMessage } from "@/lib/errors";

export function VideoPlayerWindow({ artifactId }: { artifactId: string }) {
  const [src, setSrc] = useState<string | null>(null);
  const [title, setTitle] = useState("Video");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const artifact = await api.getArtifact(artifactId);
        if (cancelled) return;
        const name = artifact.title.trim() || artifact.originalFilename || "Video";
        setTitle(name);
        void getCurrentWindow().setTitle(name).catch(() => undefined);
        const path = await api.getArtifactVideoPath(artifactId);
        if (cancelled) return;
        if (!path) {
          setError("This meeting has no video file.");
          return;
        }
        setSrc(convertFileSrc(path));
      } catch (err) {
        if (!cancelled) setError(errorMessage(err));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [artifactId]);

  return (
    <div className="flex h-full min-h-0 flex-col bg-black text-foreground">
      <div
        data-tauri-drag-region
        className="flex h-10 shrink-0 items-center px-4 text-sm font-medium text-white/90"
      >
        <span className="truncate">{title}</span>
      </div>
      <div className="flex min-h-0 flex-1 items-center justify-center bg-black p-2">
        {error ? (
          <p className="px-6 text-center text-sm text-destructive">{error}</p>
        ) : src ? (
          <video
            className="max-h-full max-w-full rounded-md"
            src={src}
            controls
            autoPlay
            playsInline
          />
        ) : (
          <p className="text-sm text-muted-foreground">Loading video…</p>
        )}
      </div>
    </div>
  );
}
