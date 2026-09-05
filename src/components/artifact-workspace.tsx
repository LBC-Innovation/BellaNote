import { useEffect, useMemo, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FileAudio, FileText, Pause, Play, Plus } from "lucide-react";
import { toast } from "sonner";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { NameDialog } from "@/components/name-dialog";
import { StaticWaveform } from "@/components/static-waveform";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import * as api from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { formatTimestamp, parseSegments } from "@/lib/segments";
import { cn } from "@/lib/utils";
import { useArtifactStore } from "@/store/useArtifactStore";
import type { Artifact } from "@/lib/types";

async function peaksFromUrl(url: string): Promise<number[]> {
  const response = await fetch(url);
  const buffer = await response.arrayBuffer();
  const ctx = new AudioContext();
  const decoded = await ctx.decodeAudioData(buffer.slice(0));
  const channel = decoded.getChannelData(0);
  const bars = 180;
  const windowSize = Math.max(1, Math.floor(channel.length / bars));
  const peaks: number[] = [];
  for (let i = 0; i < bars; i++) {
    let max = 0;
    const start = i * windowSize;
    for (let j = start; j < start + windowSize && j < channel.length; j++) {
      max = Math.max(max, Math.abs(channel[j]));
    }
    peaks.push(max);
  }
  await ctx.close();
  const peak = Math.max(...peaks, 0.001);
  return peaks.map((value) => value / peak);
}

export function ArtifactWorkspace({ groupId, groupName }: { groupId: string; groupName: string }) {
  const artifacts = useArtifactStore((s) => s.artifacts);
  const activeId = useArtifactStore((s) => s.activeId);
  const setActive = useArtifactStore((s) => s.setActive);
  const load = useArtifactStore((s) => s.load);
  const active = artifacts.find((item) => item.id === activeId) ?? null;
  const [rename, setRename] = useState<Artifact | null>(null);
  const [remove, setRemove] = useState<Artifact | null>(null);

  useEffect(() => {
    void load(groupId);
  }, [groupId, load]);

  async function pick(kind: "audio" | "transcript") {
    try {
      const selected = await open({
        multiple: false,
        filters:
          kind === "audio"
            ? [{ name: "Audio", extensions: ["wav", "mp3", "m4a", "aac", "ogg", "flac"] }]
            : [{ name: "Transcript", extensions: ["vtt", "srt", "txt"] }],
      });
      if (!selected || Array.isArray(selected)) return;
      if (kind === "audio") await api.importAudio(groupId, selected);
      else await api.importTranscript(groupId, selected);
      await load(groupId);
    } catch (err) {
      toast.error(errorMessage(err));
    }
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h1 className="text-xl font-semibold tracking-tight">{groupName}</h1>
          <p className="text-sm text-muted-foreground">Audio and imported transcripts for this meeting group.</p>
        </div>
        <div className="flex gap-2">
          <Button size="sm" variant="secondary" onClick={() => void pick("transcript")}>
            <FileText />
            Transcript
          </Button>
          <Button size="sm" onClick={() => void pick("audio")}>
            <FileAudio />
            Audio
          </Button>
        </div>
      </div>

      <div className="mt-4 grid min-h-0 flex-1 grid-cols-[240px_minmax(0,1fr)] gap-3">
        <ScrollArea className="min-h-0 rounded-2xl bg-black/10 p-2">
          {artifacts.length === 0 ? (
            <p className="px-2 py-6 text-sm text-muted-foreground">
              Add an audio file or a Zoom/Teams transcript.
            </p>
          ) : (
            <div className="flex flex-col gap-1">
              {artifacts.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => setActive(item.id)}
                  className={cn(
                    "rounded-xl px-3 py-2 text-left",
                    item.id === activeId ? "bg-primary/15" : "hover:bg-white/5",
                  )}
                >
                  <div className="truncate text-sm font-medium">{item.title}</div>
                  <div className="text-[11px] capitalize text-muted-foreground">{item.status}</div>
                </button>
              ))}
            </div>
          )}
        </ScrollArea>
        {active ? (
          <ArtifactDetail
            artifact={active}
            onRename={() => setRename(active)}
            onDelete={() => setRemove(active)}
          />
        ) : (
          <div className="flex items-center justify-center text-sm text-muted-foreground">
            <Plus className="mr-2 size-4" />
            Select or add a file
          </div>
        )}
      </div>

      <NameDialog
        open={Boolean(rename)}
        title="Rename file"
        description="This name appears in the library and in chat sources."
        confirmLabel="Save"
        initialValue={rename?.title ?? ""}
        onOpenChange={(open) => {
          if (!open) setRename(null);
        }}
        onSubmit={async (name) => {
          if (!rename) return;
          await api.renameArtifact(rename.id, name);
          await load(groupId);
        }}
      />
      <ConfirmDialog
        open={Boolean(remove)}
        title={remove ? `Remove ${remove.title}?` : "Remove"}
        description="This removes the file from BellaNote. The original on disk is left alone."
        confirmLabel="Remove"
        onOpenChange={(open) => {
          if (!open) setRemove(null);
        }}
        onConfirm={async () => {
          if (!remove) return;
          await api.deleteArtifact(remove.id);
          await load(groupId);
        }}
      />
    </div>
  );
}

function ArtifactDetail({
  artifact,
  onRename,
  onDelete,
}: {
  artifact: Artifact;
  onRename: () => void;
  onDelete: () => void;
}) {
  const segments = useMemo(() => parseSegments(artifact.segmentsJson), [artifact.segmentsJson]);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [peaks, setPeaks] = useState<number[]>([]);
  const [progress, setProgress] = useState(0);
  const [playing, setPlaying] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setAudioUrl(null);
    setPeaks([]);
    setProgress(0);
    setPlaying(false);
    if (!artifact.hasAudio) return;
    void api.getArtifactAudioPath(artifact.id).then(async (path) => {
      if (!path || cancelled) return;
      const url = convertFileSrc(path);
      setAudioUrl(url);
      try {
        setPeaks(await peaksFromUrl(url));
      } catch {
        setPeaks([]);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [artifact.id, artifact.hasAudio]);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;
    const onTime = () => {
      const duration = audio.duration || artifact.durationMs / 1000;
      setProgress(duration ? audio.currentTime / duration : 0);
    };
    const onPlay = () => setPlaying(true);
    const onPause = () => setPlaying(false);
    audio.addEventListener("timeupdate", onTime);
    audio.addEventListener("play", onPlay);
    audio.addEventListener("pause", onPause);
    return () => {
      audio.removeEventListener("timeupdate", onTime);
      audio.removeEventListener("play", onPlay);
      audio.removeEventListener("pause", onPause);
    };
  }, [audioUrl, artifact.durationMs]);

  function seek(ratio: number) {
    const audio = audioRef.current;
    if (!audio) return;
    const duration = audio.duration || artifact.durationMs / 1000;
    audio.currentTime = ratio * duration;
    setProgress(ratio);
  }

  function seekMs(ms: number) {
    const audio = audioRef.current;
    if (!audio || !artifact.hasAudio) return;
    audio.currentTime = ms / 1000;
  }

  return (
    <div className="flex min-h-0 flex-col rounded-2xl bg-black/10 p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold tracking-tight">{artifact.title}</h2>
          <p className="text-xs text-muted-foreground">
            {artifact.hasAudio ? "Audio" : "Imported transcript"} · {artifact.originalFilename}
          </p>
        </div>
        <div className="flex gap-2">
          <Button size="xs" variant="ghost" onClick={onRename}>
            Rename
          </Button>
          <Button size="xs" variant="ghost" onClick={onDelete}>
            Remove
          </Button>
        </div>
      </div>

      {artifact.hasAudio ? (
        <div className="mt-4">
          {audioUrl ? <audio ref={audioRef} src={audioUrl} className="hidden" /> : null}
          <div className="flex items-center gap-3">
            <Button
              size="icon-sm"
              variant="secondary"
              disabled={!audioUrl}
              onClick={() => {
                const audio = audioRef.current;
                if (!audio) return;
                if (audio.paused) void audio.play();
                else audio.pause();
              }}
            >
              {playing ? <Pause /> : <Play />}
            </Button>
            <div className="min-w-0 flex-1">
              {peaks.length > 0 ? (
                <StaticWaveform peaks={peaks} progress={progress} onSeek={seek} />
              ) : (
                <div className="flex h-20 items-center rounded-2xl bg-black/20 px-4 text-sm text-muted-foreground">
                  Preparing waveform…
                </div>
              )}
            </div>
          </div>
        </div>
      ) : (
        <p className="mt-3 text-sm text-muted-foreground">No waveform — this meeting is transcript only.</p>
      )}

      <ScrollArea className="mt-4 min-h-0 flex-1">
        {artifact.status === "transcribing" || artifact.status === "queued" ? (
          <p className="text-sm text-muted-foreground">Transcribing with small.en. This can take a minute on a long lecture.</p>
        ) : artifact.status === "failed" ? (
          <p className="text-sm text-destructive">{artifact.errorMessage || "Transcription failed."}</p>
        ) : segments.length > 0 ? (
          <div className="flex flex-col gap-2 pr-2">
            {segments.map((seg, index) => (
              <button
                key={`${seg.start_ms}-${index}`}
                type="button"
                className="rounded-xl px-2 py-1.5 text-left hover:bg-white/5"
                onClick={() => seekMs(seg.start_ms)}
              >
                {seg.start_ms > 0 ? (
                  <span className="mr-2 font-mono text-[11px] text-primary">{formatTimestamp(seg.start_ms)}</span>
                ) : null}
                <span className="text-sm leading-relaxed">{seg.text}</span>
              </button>
            ))}
          </div>
        ) : (
          <p className="whitespace-pre-wrap text-sm leading-relaxed">{artifact.transcript}</p>
        )}
      </ScrollArea>
    </div>
  );
}
