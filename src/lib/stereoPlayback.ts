/** WebViews play 1-channel (and silent-right) files on the left speaker only. */

function writeAscii(view: DataView, offset: number, text: string) {
  for (let i = 0; i < text.length; i++) {
    view.setUint8(offset + i, text.charCodeAt(i));
  }
}

function floatToI16(sample: number): number {
  const x = Math.max(-1, Math.min(1, sample));
  return Math.round(x * 0x7fff);
}

function channelRms(data: Float32Array): number {
  const step = Math.max(1, Math.floor(data.length / 8_000));
  let sum = 0;
  let n = 0;
  for (let i = 0; i < data.length; i += step) {
    const v = data[i] ?? 0;
    sum += v * v;
    n += 1;
  }
  return n > 0 ? Math.sqrt(sum / n) : 0;
}

export function encodeStereoWav(
  left: Float32Array,
  right: Float32Array,
  sampleRate: number,
): Blob {
  const frames = Math.min(left.length, right.length);
  const dataBytes = frames * 4;
  const buffer = new ArrayBuffer(44 + dataBytes);
  const view = new DataView(buffer);
  writeAscii(view, 0, "RIFF");
  view.setUint32(4, 36 + dataBytes, true);
  writeAscii(view, 8, "WAVE");
  writeAscii(view, 12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 2, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 4, true);
  view.setUint16(32, 4, true);
  view.setUint16(34, 16, true);
  writeAscii(view, 36, "data");
  view.setUint32(40, dataBytes, true);
  let offset = 44;
  for (let i = 0; i < frames; i++) {
    view.setInt16(offset, floatToI16(left[i] ?? 0), true);
    view.setInt16(offset + 2, floatToI16(right[i] ?? 0), true);
    offset += 4;
  }
  return new Blob([buffer], { type: "audio/wav" });
}

function needsLeftCopy(buffer: AudioBuffer): boolean {
  if (buffer.numberOfChannels < 1) return false;
  if (buffer.numberOfChannels === 1) return true;
  const left = buffer.getChannelData(0);
  const right = buffer.getChannelData(1);
  const leftRms = channelRms(left);
  const rightRms = channelRms(right);
  return leftRms > 1e-5 && rightRms < leftRms * 0.05;
}

export async function urlForStereoPlayback(src: string): Promise<string> {
  try {
    const response = await fetch(src);
    if (!response.ok) return src;
    const bytes = await response.arrayBuffer();
    const ctx = new AudioContext();
    try {
      const decoded = await ctx.decodeAudioData(bytes.slice(0));
      if (!needsLeftCopy(decoded)) return src;
      const left = decoded.getChannelData(0);
      const blob = encodeStereoWav(left, left, decoded.sampleRate);
      return URL.createObjectURL(blob);
    } finally {
      await ctx.close();
    }
  } catch {
    return src;
  }
}

export function revokePlaybackUrl(url: string | null) {
  if (url?.startsWith("blob:")) URL.revokeObjectURL(url);
}
