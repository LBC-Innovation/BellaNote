import { useEffect, useRef } from "react";
import * as api from "@/lib/api";

const FALL_RATE = 0.72;

export function LiveMeter({ active, height = 40 }: { active: boolean; height?: number }) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const levelsRef = useRef<number[]>([]);

  useEffect(() => {
    const canvas = canvasRef.current;
    const clear = () => {
      levelsRef.current = [];
      if (!canvas) return;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.clearRect(0, 0, canvas.width, canvas.height);
    };
    if (!active) {
      clear();
      return;
    }

    let raf = 0;
    let cancelled = false;

    const draw = (bands: number[]) => {
      if (!canvas) return;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      const dpr = window.devicePixelRatio || 1;
      const width = canvas.clientWidth;
      const h = canvas.clientHeight;
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(h * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, width, h);

      const count = Math.max(bands.length, 1);
      const gap = 1.5;
      const barW = Math.max(1, (width - gap * (count - 1)) / count);
      const mid = h / 2;
      for (let i = 0; i < count; i++) {
        const amp = Math.max(1.5, (bands[i] ?? 0) * (h * 0.46));
        const x = i * (barW + gap);
        ctx.fillStyle = "oklch(0.84 0.13 178 / 0.85)";
        ctx.fillRect(x, mid - amp, Math.max(barW - 0.4, 1), amp * 2);
      }
    };

    const tick = () => {
      void api
        .getSpectrum()
        .then((next) => {
          if (cancelled) return;
          const prev = levelsRef.current;
          const bands = next.map((value, i) => {
            const incoming = Math.min(1, Math.max(0, value));
            const held = (prev[i] ?? 0) * FALL_RATE;
            return Math.max(incoming, held);
          });
          levelsRef.current = bands;
          draw(bands);
        })
        .catch(() => {
          /* ignore */
        });
    };

    tick();
    const id = window.setInterval(tick, 50);
    const onResize = () => draw(levelsRef.current);
    window.addEventListener("resize", onResize);
    raf = window.requestAnimationFrame(() => draw(levelsRef.current));

    return () => {
      cancelled = true;
      window.clearInterval(id);
      window.cancelAnimationFrame(raf);
      window.removeEventListener("resize", onResize);
      clear();
    };
  }, [active]);

  return (
    <canvas
      ref={canvasRef}
      height={height}
      className="h-10 w-full rounded-2xl bg-black/20"
      aria-hidden
    />
  );
}
