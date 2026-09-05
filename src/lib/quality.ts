import type { Artifact } from "./types";

export type QualityKind =
  | "small"
  | "base"
  | "tiny"
  | "medium"
  | "large"
  | "imported"
  | "importing"
  | "failed"
  | "unknown";

export function transcriptionQuality(artifact: Artifact): { kind: QualityKind; label: string } {
  if (artifact.status === "queued" || artifact.status === "transcribing") {
    return { kind: "importing", label: "Importing" };
  }
  if (artifact.status === "failed") {
    return { kind: "failed", label: "Failed" };
  }
  if (artifact.sourceType === "transcript_import" || artifact.whisperModel === "imported") {
    return { kind: "imported", label: "Imported" };
  }
  const model = (artifact.whisperModel || "small.en").toLowerCase();
  if (model.includes("tiny")) return { kind: "tiny", label: "tiny" };
  if (model.includes("base")) return { kind: "base", label: "base" };
  if (model.includes("small")) return { kind: "small", label: "small" };
  if (model.includes("medium")) return { kind: "medium", label: "medium" };
  if (model.includes("large")) return { kind: "large", label: "large" };
  if (artifact.whisperModel) return { kind: "unknown", label: artifact.whisperModel };
  return { kind: "unknown", label: "—" };
}

export function formatAddedDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "—";
  return date.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });
}
