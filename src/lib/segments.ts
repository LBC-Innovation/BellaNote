import type { TranscriptSegment } from "./types";

export function parseSegments(json: string): TranscriptSegment[] {
  try {
    const value = JSON.parse(json) as TranscriptSegment[];
    return Array.isArray(value) ? value : [];
  } catch {
    return [];
  }
}

export function formatTimestamp(ms: number): string {
  if (!ms || ms < 0) return "00:00";
  const total = Math.floor(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) {
    return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}
