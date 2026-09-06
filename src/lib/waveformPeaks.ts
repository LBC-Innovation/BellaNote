import * as api from "@/lib/api";
import type { AudioPeaks } from "@/lib/api";

const cache = new Map<string, AudioPeaks>();

export function peekWaveformPeaks(artifactId: string): AudioPeaks | undefined {
  return cache.get(artifactId);
}

export async function loadWaveformPeaks(artifactId: string): Promise<AudioPeaks> {
  const hit = cache.get(artifactId);
  if (hit) return hit;
  const result = await api.getArtifactAudioPeaks(artifactId);
  if (result.peaks.length > 0) {
    cache.set(artifactId, result);
  }
  return result;
}

export function forgetWaveformPeaks(artifactId: string) {
  cache.delete(artifactId);
}
