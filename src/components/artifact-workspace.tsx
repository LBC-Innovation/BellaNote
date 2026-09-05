import { useEffect, useMemo, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Check, FileAudio, FileText, Link2, Link2Off, Loader2, Pause, Pencil, Play, Plus, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { StaticWaveform } from "@/components/static-waveform";
import { WorkspaceCard } from "@/components/workspace-card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import * as api from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { formatAddedDate, transcriptionQuality, type QualityKind } from "@/lib/quality";
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

function qualityBadge(kind: QualityKind) {
  if (kind === "failed") return "destructive" as const;
  if (kind === "small" || kind === "medium" || kind === "large") return "default" as const;
  return "secondary" as const;
}

function activeSegmentIndex(segments: { start_ms: number; end_ms: number }[], timeMs: number) {
  if (segments.length === 0 || timeMs < 0) return -1;
  for (let i = 0; i < segments.length; i++) {
    const seg = segments[i];
    if (timeMs >= seg.start_ms && timeMs < Math.max(seg.end_ms, seg.start_ms + 1)) return i;
  }
  let last = -1;
  for (let i = 0; i < segments.length; i++) {
    if (segments[i].start_ms <= timeMs) last = i;
  }
  return last;
}

export function ArtifactWorkspace({ groupId, groupName }: { groupId: string; groupName: string }) {
  const artifacts = useArtifactStore((s) => s.artifacts);
  const activeId = useArtifactStore((s) => s.activeId);
  const setActive = useArtifactStore((s) => s.setActive);
  const load = useArtifactStore((s) => s.load);
  const active = artifacts.find((item) => item.id === activeId) ?? null;
  const [remove, setRemove] = useState<Artifact | null>(null);
  const [filesOpen, setFilesOpen] = useState(true);
  const [detailOpen, setDetailOpen] = useState(true);

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
      <div className="flex shrink-0 items-center justify-between gap-3">
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

      <div className="mt-4 flex min-h-0 flex-1 flex-col gap-3">
        <WorkspaceCard
          open={filesOpen}
          onOpenChange={setFilesOpen}
          title="Files"
          meta={artifacts.length === 1 ? "1 file" : `${artifacts.length} files`}
        >
          <FilesTable
            artifacts={artifacts}
            activeId={activeId}
            onSelect={setActive}
            onRename={async (id, name) => {
              await api.renameArtifact(id, name);
              await load(groupId);
            }}
            onDelete={setRemove}
          />
        </WorkspaceCard>

        <WorkspaceCard
          open={detailOpen}
          onOpenChange={setDetailOpen}
          title={active?.title ?? "Transcript"}
          meta={
            active
              ? active.hasAudio
                ? "Audio + transcript"
                : "Imported transcript"
              : undefined
          }
        >
          {active ? (
            <ArtifactDetail artifact={active} />
          ) : (
            <div className="flex min-h-0 flex-1 items-center justify-center text-sm text-muted-foreground">
              <Plus className="mr-2 size-4" />
              Select or add a file
            </div>
          )}
        </WorkspaceCard>
      </div>

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

function FilesTable({
  artifacts,
  activeId,
  onSelect,
  onRename,
  onDelete,
}: {
  artifacts: Artifact[];
  activeId: string | null;
  onSelect: (id: string) => void;
  onRename: (id: string, name: string) => Promise<void>;
  onDelete: (artifact: Artifact) => void;
}) {
  const [editingId, setEditingId] = useState<string | null>(null);
  const [savedId, setSavedId] = useState<string | null>(null);

  if (artifacts.length === 0) {
    return (
      <p className="px-2 py-8 text-sm text-muted-foreground">
        Add an audio file or a Zoom/Teams transcript.
      </p>
    );
  }

  return (
    <ScrollArea className="min-h-0 flex-1">
      <div className="flex flex-col gap-1 pr-1">
        <div className="grid grid-cols-[minmax(0,1fr)_auto_auto_auto] items-center gap-x-4 px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
          <span>File</span>
          <span className="w-[7.25rem]">Added</span>
          <span className="w-[6.75rem]">Quality</span>
          <span className="w-14" />
        </div>
        {artifacts.map((item) => (
          <FileRow
            key={item.id}
            item={item}
            selected={item.id === activeId}
            editing={editingId === item.id}
            justSaved={savedId === item.id}
            onSelect={() => onSelect(item.id)}
            onStartEdit={() => {
              onSelect(item.id);
              setEditingId(item.id);
            }}
            onCancelEdit={() => setEditingId(null)}
            onSave={async (name) => {
              await onRename(item.id, name);
              setEditingId(null);
              setSavedId(item.id);
              window.setTimeout(() => {
                setSavedId((current) => (current === item.id ? null : current));
              }, 900);
            }}
            onDelete={() => onDelete(item)}
          />
        ))}
      </div>
    </ScrollArea>
  );
}

function FileRow({
  item,
  selected,
  editing,
  justSaved,
  onSelect,
  onStartEdit,
  onCancelEdit,
  onSave,
  onDelete,
}: {
  item: Artifact;
  selected: boolean;
  editing: boolean;
  justSaved: boolean;
  onSelect: () => void;
  onStartEdit: () => void;
  onCancelEdit: () => void;
  onSave: (name: string) => Promise<void>;
  onDelete: () => void;
}) {
  const quality = transcriptionQuality(item);
  const [draft, setDraft] = useState(item.title);
  const saving = useRef(false);
  const ignoreBlur = useRef(false);

  useEffect(() => {
    if (editing) setDraft(item.title);
  }, [editing, item.title]);

  async function commit() {
    if (ignoreBlur.current) {
      ignoreBlur.current = false;
      return;
    }
    if (saving.current) return;
    const name = draft.trim();
    if (!name || name === item.title) {
      onCancelEdit();
      return;
    }
    saving.current = true;
    try {
      await onSave(name);
    } catch (err) {
      toast.error(errorMessage(err));
    } finally {
      saving.current = false;
    }
  }

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={() => {
        if (!editing) onSelect();
      }}
      onKeyDown={(event) => {
        if (editing) return;
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect();
        }
      }}
      className={cn(
        "grid w-full grid-cols-[minmax(0,1fr)_auto_auto_auto] items-center gap-x-4 rounded-xl border px-4 py-2.5 text-left text-sm transition-colors",
        selected ? "border-primary/40 bg-primary/18" : "border-transparent hover:bg-white/5",
      )}
    >
      <span className="min-w-0">
        {editing ? (
          <Input
            autoFocus
            value={draft}
            maxLength={80}
            aria-label="File title"
            className="h-7 text-sm font-medium"
            onClick={(event) => event.stopPropagation()}
            onChange={(event) => setDraft(event.target.value)}
            onBlur={() => {
              void commit();
            }}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                event.currentTarget.blur();
              }
              if (event.key === "Escape") {
                event.preventDefault();
                ignoreBlur.current = true;
                setDraft(item.title);
                onCancelEdit();
              }
            }}
          />
        ) : (
          <span className="flex min-w-0 items-center gap-1.5">
            <span className={cn("block truncate font-medium", justSaved && "title-saved")}>
              {item.title}
            </span>
            {justSaved ? <Check className="size-3.5 shrink-0 text-primary" /> : null}
          </span>
        )}
        <span className="mt-0.5 block truncate text-[11px] text-muted-foreground">
          {item.originalFilename}
        </span>
      </span>
      <span className="w-[7.25rem] whitespace-nowrap text-muted-foreground">
        {formatAddedDate(item.createdAt)}
      </span>
      <span className="w-[6.75rem]">
        <Badge variant={qualityBadge(quality.kind)} className="capitalize">
          {quality.kind === "importing" ? <Loader2 className="animate-spin" /> : null}
          {quality.label}
        </Badge>
      </span>
      <span
        className="flex w-14 justify-end gap-0.5"
        onClick={(event) => event.stopPropagation()}
      >
        <Button
          size="icon-xs"
          variant="ghost"
          aria-label="Rename file"
          onClick={onStartEdit}
        >
          <Pencil />
        </Button>
        <Button
          size="icon-xs"
          variant="ghost"
          aria-label="Delete file"
          className="text-muted-foreground hover:text-destructive"
          onClick={onDelete}
        >
          <Trash2 />
        </Button>
      </span>
    </div>
  );
}

function ArtifactDetail({ artifact }: { artifact: Artifact }) {
  const segments = useMemo(() => parseSegments(artifact.segmentsJson), [artifact.segmentsJson]);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const lineRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [peaks, setPeaks] = useState<number[]>([]);
  const [progress, setProgress] = useState(0);
  const [currentTimeMs, setCurrentTimeMs] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [followPlayback, setFollowPlayback] = useState(true);

  const activeIndex = useMemo(
    () => activeSegmentIndex(segments, currentTimeMs),
    [segments, currentTimeMs],
  );

  useEffect(() => {
    let cancelled = false;
    setAudioUrl(null);
    setPeaks([]);
    setProgress(0);
    setCurrentTimeMs(0);
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
      setCurrentTimeMs(audio.currentTime * 1000);
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

  useEffect(() => {
    if (!followPlayback || activeIndex < 0) return;
    lineRefs.current[activeIndex]?.scrollIntoView({ block: "center", behavior: "smooth" });
  }, [activeIndex, followPlayback]);

  function seek(ratio: number) {
    const audio = audioRef.current;
    if (!audio) return;
    const duration = audio.duration || artifact.durationMs / 1000;
    audio.currentTime = ratio * duration;
    setCurrentTimeMs(ratio * duration * 1000);
    setProgress(ratio);
  }

  function seekMs(ms: number) {
    const audio = audioRef.current;
    if (!audio || !artifact.hasAudio) return;
    audio.currentTime = ms / 1000;
    setCurrentTimeMs(ms);
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {artifact.hasAudio ? (
        <div className="shrink-0">
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
            <Button
              size="sm"
              variant={followPlayback ? "secondary" : "ghost"}
              aria-pressed={followPlayback}
              onClick={() => setFollowPlayback((value) => !value)}
            >
              {followPlayback ? <Link2 /> : <Link2Off />}
              Follow
            </Button>
          </div>
        </div>
      ) : (
        <p className="shrink-0 text-sm text-muted-foreground">No waveform — this meeting is transcript only.</p>
      )}

      <ScrollArea className="mt-3 min-h-0 flex-1">
        {artifact.status === "transcribing" || artifact.status === "queued" ? (
          <p className="text-sm text-muted-foreground">
            Transcribing with small.en. This can take a minute on a long lecture.
          </p>
        ) : artifact.status === "failed" ? (
          <p className="text-sm text-destructive">{artifact.errorMessage || "Transcription failed."}</p>
        ) : segments.length > 0 ? (
          <div className="flex flex-col gap-1 py-0.5 pr-2">
            {segments.map((seg, index) => (
              <button
                key={`${seg.start_ms}-${index}`}
                ref={(node) => {
                  lineRefs.current[index] = node;
                }}
                type="button"
                className={cn(
                  "w-full appearance-none rounded-xl border text-left transition-colors",
                  index === activeIndex
                    ? "border-primary/40 bg-primary/18"
                    : "border-transparent hover:bg-white/5",
                )}
                onClick={() => seekMs(seg.start_ms)}
              >
                <span className="block px-4 py-2.5 text-sm leading-6">
                  {seg.start_ms > 0 ? (
                    <span className="mr-2 font-mono text-[11px] text-primary">
                      {formatTimestamp(seg.start_ms)}
                    </span>
                  ) : null}
                  {seg.text}
                </span>
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
