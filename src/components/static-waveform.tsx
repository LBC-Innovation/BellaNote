import { useEffect, useRef } from "react";
import { cn } from "@/lib/utils";

export type WaveformMarker = {
  id: string;
  ratio: number;
  draft?: boolean;
};

type Props = {
  peaks: number[];
  progress: number;
  markers?: WaveformMarker[];
  onSeek: (ratio: number) => void;
  onMarkerClick?: (id: string) => void;
  className?: string;
};

export function StaticWaveform({
  peaks,
  progress,
  markers = [],
  onSeek,
  onMarkerClick,
  className,
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);

    const mid = height / 2;
    const count = Math.max(peaks.length, 1);
    const barW = Math.max(width / count, 1.2);
    const playedUntil = progress * width;

    for (let i = 0; i < count; i++) {
      const x = (i / count) * width;
      const amp = Math.max(2, (peaks[i] ?? 0) * (height * 0.42));
      ctx.fillStyle = x <= playedUntil ? "oklch(0.84 0.13 178)" : "oklch(0.78 0.04 210 / 0.35)";
      ctx.fillRect(x, mid - amp, Math.max(barW - 0.6, 1), amp * 2);
    }

    ctx.fillStyle = "oklch(0.95 0.02 190)";
    ctx.fillRect(Math.min(playedUntil, width - 1), 4, 1.5, height - 8);
  }, [peaks, progress]);

  function ratioFromEvent(event: React.PointerEvent<HTMLCanvasElement>) {
    const rect = event.currentTarget.getBoundingClientRect();
    return Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width));
  }

  return (
    <div className={cn("relative h-8 w-full", className)}>
      <canvas
        ref={canvasRef}
        className="h-full w-full cursor-pointer rounded-2xl"
        onPointerDown={(event) => {
          event.currentTarget.setPointerCapture(event.pointerId);
          onSeek(ratioFromEvent(event));
        }}
        onPointerMove={(event) => {
          if (event.buttons === 1) onSeek(ratioFromEvent(event));
        }}
      />
      {markers.map((marker) => {
        const left = `${Math.min(100, Math.max(0, marker.ratio * 100))}%`;
        const nodeClass = cn(
          "pointer-events-auto absolute left-0 size-2 -translate-x-1/2 rounded-full border border-white/50",
          marker.draft
            ? "bg-[#8EC5FF]/55"
            : "bg-[#8EC5FF] shadow-[0_0_8px_#8EC5FF] hover:scale-125",
        );
        return (
          <div
            key={marker.id}
            className="pointer-events-none absolute inset-y-0 z-10"
            style={{ left }}
          >
            <div
              className={cn(
                "absolute top-0 bottom-0 w-px -translate-x-1/2",
                marker.draft ? "bg-[#8EC5FF]/50" : "bg-[#8EC5FF]",
              )}
            />
            <button
              type="button"
              title={marker.draft ? "New comment" : "Play from this comment"}
              aria-label={marker.draft ? "Draft comment marker" : "Play from comment"}
              disabled={marker.draft || !onMarkerClick}
              onClick={(event) => {
                event.stopPropagation();
                if (!marker.draft) onMarkerClick?.(marker.id);
              }}
              className={cn(nodeClass, "top-0")}
            />
            <button
              type="button"
              title={marker.draft ? "New comment" : "Play from this comment"}
              aria-label={marker.draft ? "Draft comment marker" : "Play from comment"}
              disabled={marker.draft || !onMarkerClick}
              onClick={(event) => {
                event.stopPropagation();
                if (!marker.draft) onMarkerClick?.(marker.id);
              }}
              className={cn(nodeClass, "bottom-0")}
            />
          </div>
        );
      })}
    </div>
  );
}
