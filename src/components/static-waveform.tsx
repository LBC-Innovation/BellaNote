import { useEffect, useRef } from "react";
import { cn } from "@/lib/utils";

type Props = {
  peaks: number[];
  progress: number;
  onSeek: (ratio: number) => void;
  className?: string;
};

export function StaticWaveform({ peaks, progress, onSeek, className }: Props) {
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
    <canvas
      ref={canvasRef}
      className={cn("h-20 w-full cursor-pointer rounded-2xl", className)}
      onPointerDown={(event) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        onSeek(ratioFromEvent(event));
      }}
      onPointerMove={(event) => {
        if (event.buttons === 1) onSeek(ratioFromEvent(event));
      }}
    />
  );
}
