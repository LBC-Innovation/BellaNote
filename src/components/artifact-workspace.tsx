import { useEffect, useMemo, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Check, ChevronDown, ChevronLeft, FileAudio, FileText, FileUp, Link2, Link2Off, ListChecks, Loader2, Pause, Pencil, Play, Plus, RotateCcw, Search, Trash2, X } from "lucide-react";
import { toast } from "sonner";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { ImportDropDialog } from "@/components/import-drop-dialog";
import { StaticWaveform } from "@/components/static-waveform";
import { WorkspaceCard } from "@/components/workspace-card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import * as api from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { formatAddedDate, isFailedArtifact, isImportingArtifact, isLoadableArtifact, isPendingArtifact, transcriptionQuality, type QualityKind } from "@/lib/quality";
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

function hasTranscriptContent(artifact: Artifact) {
  return artifact.status === "ready" && Boolean(artifact.transcript.trim());
}

function ArtifactKindChips({ artifact }: { artifact: Artifact }) {
  const audio = artifact.hasAudio;
  const transcript = hasTranscriptContent(artifact);
  if (!audio && !transcript) return null;
  return (
    <>
      {audio ? (
        <Badge variant="secondary" className="capitalize">
          <FileAudio />
          Audio
        </Badge>
      ) : null}
      {transcript ? (
        <Badge variant="secondary" className="capitalize">
          <FileText />
          Transcript
        </Badge>
      ) : null}
    </>
  );
}

function qualityBadge(kind: QualityKind) {
  if (kind === "failed") return "destructive" as const;
  if (kind === "small" || kind === "medium" || kind === "large") return "default" as const;
  if (kind === "pending") return "outline" as const;
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

function ImportMeetingControl({ onPick }: { onPick: (kind: "audio" | "transcript") => void }) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!open) return;
    function onPointerDown(event: PointerEvent) {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    }
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") setOpen(false);
    }
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  return (
    <div ref={rootRef} className="relative w-48 shrink-0">
      <button
        type="button"
        aria-expanded={open}
        aria-haspopup="listbox"
        onClick={() => setOpen((value) => !value)}
        className={cn(
          "flex h-8 w-full cursor-pointer items-center gap-1.5 border border-white/20 bg-white/[0.06] px-2.5 text-[0.8rem] font-medium backdrop-blur-md transition-colors hover:bg-white/10",
          open ? "rounded-t-xl rounded-b-none border-b-transparent" : "rounded-xl",
        )}
      >
        <FileUp className="size-3.5 shrink-0" />
        <span className="min-w-0 flex-1 text-left">Import Meeting</span>
        {open ? <ChevronDown className="size-3.5 shrink-0" /> : <ChevronLeft className="size-3.5 shrink-0" />}
      </button>
      {open ? (
        <div
          role="listbox"
          className="absolute top-full left-0 z-30 w-full rounded-b-xl border border-t-0 border-white/20 bg-white/[0.06] p-1 backdrop-blur-md"
        >
          <button
            type="button"
            role="option"
            className="flex w-full cursor-pointer items-center gap-2 rounded-lg px-2 py-1.5 text-left text-sm hover:bg-white/8"
            onClick={() => {
              setOpen(false);
              onPick("audio");
            }}
          >
            <FileAudio className="size-4 shrink-0" />
            Audio files
          </button>
          <button
            type="button"
            role="option"
            className="flex w-full cursor-pointer items-center gap-2 rounded-lg px-2 py-1.5 text-left text-sm hover:bg-white/8"
            onClick={() => {
              setOpen(false);
              onPick("transcript");
            }}
          >
            <FileText className="size-4 shrink-0" />
            Transcript files
          </button>
        </div>
      ) : null}
    </div>
  );
}

export function ArtifactWorkspace({ groupId, groupName }: { groupId: string; groupName: string }) {
  const artifacts = useArtifactStore((s) => s.artifacts);
  const activeId = useArtifactStore((s) => s.activeId);
  const setActive = useArtifactStore((s) => s.setActive);
  const load = useArtifactStore((s) => s.load);
  const active = artifacts.find((item) => item.id === activeId) ?? null;
  const [remove, setRemove] = useState<Artifact | null>(null);
  const [removeMany, setRemoveMany] = useState(false);
  const [selecting, setSelecting] = useState(false);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [filesOpen, setFilesOpen] = useState(true);
  const [detailOpen, setDetailOpen] = useState(true);
  const [importKind, setImportKind] = useState<"audio" | "transcript" | null>(null);

  function showImportWait() {
    toast.warning("Please wait, this file is importing", {
      id: "file-importing",
      toasterId: "notice",
      duration: 3500,
      richColors: true,
    });
  }

  useEffect(() => {
    setSelecting(false);
    setSelectedIds([]);
    setRemoveMany(false);
    void load(groupId);
  }, [groupId, load]);

  useEffect(() => {
    const valid = new Set(artifacts.map((item) => item.id));
    setSelectedIds((current) => {
      const next = current.filter((id) => valid.has(id));
      return next.length === current.length ? current : next;
    });
  }, [artifacts]);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex shrink-0 items-center justify-between gap-3">
        <div>
          <h1 className="text-xl font-semibold tracking-tight">{groupName}</h1>
          <p className="text-sm text-muted-foreground">Audio and imported transcripts for this meeting group.</p>
        </div>
        <ImportMeetingControl onPick={setImportKind} />
      </div>

      <div className="mt-4 flex min-h-0 flex-1 flex-col gap-3">
        <WorkspaceCard
          open={filesOpen}
          onOpenChange={(open) => {
            setFilesOpen(open);
            if (!open) {
              setSelecting(false);
              setSelectedIds([]);
            }
          }}
          title="Files"
          meta={artifacts.length === 1 ? "1 file" : `${artifacts.length} files`}
          actions={
            filesOpen && artifacts.length > 0 ? (
              <>
                <Button
                  size="xs"
                  variant={selecting ? "secondary" : "ghost"}
                  aria-pressed={selecting}
                  onClick={() => {
                    setSelecting((value) => !value);
                    setSelectedIds([]);
                  }}
                >
                  <ListChecks />
                  Select multiple
                </Button>
                {selecting ? (
                  <Button
                    size="xs"
                    variant="destructive"
                    disabled={selectedIds.length === 0}
                    onClick={() => setRemoveMany(true)}
                  >
                    <Trash2 />
                    Delete
                  </Button>
                ) : null}
              </>
            ) : null
          }
        >
          <FilesTable
            artifacts={artifacts}
            activeId={activeId}
            selecting={selecting}
            selectedIds={selectedIds}
            onToggleSelected={(id) => {
              setSelectedIds((current) =>
                current.includes(id) ? current.filter((item) => item !== id) : [...current, id],
              );
            }}
            onSelect={setActive}
            onRename={async (id, name) => {
              await api.renameArtifact(id, name);
              await load(groupId);
            }}
            onRetry={async (id) => {
              try {
                await api.retryArtifact(id);
                await load(groupId);
              } catch (err) {
                toast.error(errorMessage(err));
              }
            }}
            onDelete={setRemove}
            onImportingClick={showImportWait}
          />
        </WorkspaceCard>

        <WorkspaceCard
          open={detailOpen}
          onOpenChange={setDetailOpen}
          title={active?.title ?? "Transcript"}
          meta={active ? <ArtifactKindChips artifact={active} /> : undefined}
        >
          {active && isLoadableArtifact(active) ? (
            <ArtifactDetail artifact={active} />
          ) : (
            <div className="flex min-h-0 flex-1 items-center justify-center text-sm text-muted-foreground">
              <Plus className="mr-2 size-4" />
              Select or add a file
            </div>
          )}
        </WorkspaceCard>
      </div>

      <ImportDropDialog
        open={importKind !== null}
        kind={importKind}
        groupId={groupId}
        onOpenChange={(next) => {
          if (!next) setImportKind(null);
        }}
        onImported={() => load(groupId)}
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

      <ConfirmDialog
        open={removeMany}
        title={`Are you sure you want to delete ${selectedIds.length} ${selectedIds.length === 1 ? "file" : "files"}?`}
        description="This removes the files from BellaNote. The originals on disk are left alone."
        confirmLabel="Delete"
        onOpenChange={setRemoveMany}
        onConfirm={async () => {
          const ids = selectedIds;
          try {
            for (const id of ids) {
              await api.deleteArtifact(id);
            }
            setSelectedIds([]);
            setSelecting(false);
            await load(groupId);
          } catch (err) {
            toast.error(errorMessage(err));
            await load(groupId);
          }
        }}
      />
    </div>
  );
}

const filesGridClass = "grid-cols-[minmax(0,1fr)_auto_auto_auto]";

function FileCheckbox({
  checked,
  label,
  onToggle,
}: {
  checked: boolean;
  label: string;
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={checked}
      aria-label={label}
      onClick={(event) => {
        event.stopPropagation();
        onToggle();
      }}
      className={cn(
        "flex size-4 items-center justify-center rounded-[5px] border transition-colors",
        checked
          ? "border-primary bg-primary text-primary-foreground"
          : "border-white/25 bg-white/[0.06] hover:border-primary/45 hover:bg-white/10",
      )}
    >
      {checked ? <Check className="size-2.5" strokeWidth={3} /> : null}
    </button>
  );
}

function FilesTable({
  artifacts,
  activeId,
  selecting,
  selectedIds,
  onToggleSelected,
  onSelect,
  onRename,
  onRetry,
  onDelete,
  onImportingClick,
}: {
  artifacts: Artifact[];
  activeId: string | null;
  selecting: boolean;
  selectedIds: string[];
  onToggleSelected: (id: string) => void;
  onSelect: (id: string) => void;
  onRename: (id: string, name: string) => Promise<void>;
  onRetry: (id: string) => Promise<void>;
  onDelete: (artifact: Artifact) => void;
  onImportingClick: () => void;
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
        <div
          className={cn(
            "grid items-center gap-x-4 px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground",
            filesGridClass,
          )}
        >
          <span>File</span>
          <span className="w-[7.25rem]">Added</span>
          <span className="w-[10rem]">Quality</span>
          <span className="w-14" />
        </div>
        {artifacts.map((item) => (
          <FileRow
            key={item.id}
            item={item}
            selected={item.id === activeId && isLoadableArtifact(item)}
            selecting={selecting}
            checked={selectedIds.includes(item.id)}
            editing={editingId === item.id}
            justSaved={savedId === item.id}
            onToggleSelected={() => onToggleSelected(item.id)}
            onSelect={() => onSelect(item.id)}
            onStartEdit={() => {
              if (isLoadableArtifact(item)) onSelect(item.id);
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
            onRetry={() => onRetry(item.id)}
            onDelete={() => onDelete(item)}
            onImportingClick={onImportingClick}
          />
        ))}
      </div>
    </ScrollArea>
  );
}

function FileRow({
  item,
  selected,
  selecting,
  checked,
  editing,
  justSaved,
  onToggleSelected,
  onSelect,
  onStartEdit,
  onCancelEdit,
  onSave,
  onRetry,
  onDelete,
  onImportingClick,
}: {
  item: Artifact;
  selected: boolean;
  selecting: boolean;
  checked: boolean;
  editing: boolean;
  justSaved: boolean;
  onToggleSelected: () => void;
  onSelect: () => void;
  onStartEdit: () => void;
  onCancelEdit: () => void;
  onSave: (name: string) => Promise<void>;
  onRetry: () => Promise<void>;
  onDelete: () => void;
  onImportingClick: () => void;
}) {
  const quality = transcriptionQuality(item);
  const pending = isPendingArtifact(item);
  const failed = isFailedArtifact(item);
  const blocked = !isLoadableArtifact(item);
  const [retrying, setRetrying] = useState(false);
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
      tabIndex={blocked && !selecting && !isImportingArtifact(item) ? -1 : 0}
      aria-disabled={blocked && !selecting && !isImportingArtifact(item) ? true : undefined}
      aria-pressed={selecting ? checked : undefined}
      onClick={() => {
        if (editing) return;
        if (selecting) {
          onToggleSelected();
          return;
        }
        if (isImportingArtifact(item)) {
          onImportingClick();
          return;
        }
        if (!blocked) onSelect();
      }}
      onKeyDown={(event) => {
        if (editing) return;
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          if (selecting) onToggleSelected();
          else if (isImportingArtifact(item)) onImportingClick();
          else if (!blocked) onSelect();
        }
      }}
      className={cn(
        "grid w-full items-center gap-x-4 rounded-xl border px-4 py-2.5 text-left text-sm transition-colors",
        filesGridClass,
        pending
          ? "border-dashed border-amber-400/40 bg-amber-400/[0.04] text-muted-foreground"
          : failed
            ? "border-dashed border-destructive/35 bg-destructive/[0.04]"
            : selecting && checked
              ? "border-primary/40 bg-primary/18"
              : selected
                ? "border-primary/40 bg-primary/18"
                : "border-transparent hover:bg-white/5",
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
      <span className="flex w-[10rem] items-center gap-1.5">
        <Badge variant={qualityBadge(quality.kind)} className="capitalize">
          {quality.kind === "importing" ? <Loader2 className="animate-spin" /> : null}
          {quality.label}
        </Badge>
        {failed && item.hasAudio ? (
          <Button
            size="xs"
            variant="ghost"
            disabled={retrying}
            aria-label={`Retry ${item.title}`}
            onClick={(event) => {
              event.stopPropagation();
              if (retrying) return;
              setRetrying(true);
              void onRetry().finally(() => setRetrying(false));
            }}
          >
            {retrying ? <Loader2 className="animate-spin" /> : <RotateCcw />}
            Retry
          </Button>
        ) : null}
      </span>
      {selecting ? (
        <span className="flex w-14 items-center justify-end">
          <FileCheckbox
            checked={checked}
            label={`Select ${item.title}`}
            onToggle={onToggleSelected}
          />
        </span>
      ) : (
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
      )}
    </div>
  );
}

function FailedTranscript({ artifact }: { artifact: Artifact }) {
  const refresh = useArtifactStore((state) => state.refresh);
  const [retrying, setRetrying] = useState(false);

  return (
    <div className="flex flex-col items-start gap-3">
      <p className="text-sm text-destructive">
        {artifact.errorMessage || "This file couldn’t be processed. You can try again."}
      </p>
      {artifact.hasAudio ? (
        <Button
          size="sm"
          variant="secondary"
          disabled={retrying}
          onClick={() => {
            if (retrying) return;
            setRetrying(true);
            void api
              .retryArtifact(artifact.id)
              .then(() => refresh())
              .catch((err) => toast.error(errorMessage(err)))
              .finally(() => setRetrying(false));
          }}
        >
          {retrying ? <Loader2 className="animate-spin" /> : <RotateCcw />}
          Retry
        </Button>
      ) : null}
    </div>
  );
}

const PLAYBACK_RATES = [1, 1.25, 1.5, 2] as const;
type PlaybackRate = (typeof PLAYBACK_RATES)[number];
const PLAYBACK_RATE_KEY = "bellanote.playbackRate";

function isPlaybackRate(value: number): value is PlaybackRate {
  return (PLAYBACK_RATES as readonly number[]).includes(value);
}

function readPlaybackRate(): PlaybackRate {
  try {
    const raw = Number(localStorage.getItem(PLAYBACK_RATE_KEY));
    return isPlaybackRate(raw) ? raw : 1;
  } catch {
    return 1;
  }
}

function formatPlaybackRate(rate: PlaybackRate) {
  return `${rate}x`;
}

function PlaybackToolbar({
  disabled,
  playing,
  followPlayback,
  playbackRate,
  currentTimeMs,
  durationMs,
  onTogglePlay,
  onToggleFollow,
  onPlaybackRateChange,
}: {
  disabled: boolean;
  playing: boolean;
  followPlayback: boolean;
  playbackRate: PlaybackRate;
  currentTimeMs: number;
  durationMs: number;
  onTogglePlay: () => void;
  onToggleFollow: () => void;
  onPlaybackRateChange: (rate: PlaybackRate) => void;
}) {
  return (
    <div className="mt-2 flex items-center gap-1.5">
      <Button
        size="icon-sm"
        variant="secondary"
        disabled={disabled}
        aria-label={playing ? "Pause" : "Play"}
        onClick={onTogglePlay}
      >
        {playing ? <Pause /> : <Play />}
      </Button>
      <Button
        size="sm"
        variant={followPlayback ? "secondary" : "ghost"}
        aria-pressed={followPlayback}
        aria-label={followPlayback ? "Stop following playback" : "Follow playback in the transcript"}
        onClick={onToggleFollow}
      >
        {followPlayback ? <Link2 /> : <Link2Off />}
        Follow
      </Button>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            size="sm"
            variant="ghost"
            aria-label={`Playback speed ${formatPlaybackRate(playbackRate)}`}
          >
            {formatPlaybackRate(playbackRate)}
            <ChevronDown />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="min-w-28">
          <DropdownMenuRadioGroup
            value={String(playbackRate)}
            onValueChange={(value) => {
              const next = Number(value);
              if (isPlaybackRate(next)) onPlaybackRateChange(next);
            }}
          >
            {PLAYBACK_RATES.map((rate) => (
              <DropdownMenuRadioItem key={rate} value={String(rate)}>
                {formatPlaybackRate(rate)}
              </DropdownMenuRadioItem>
            ))}
          </DropdownMenuRadioGroup>
        </DropdownMenuContent>
      </DropdownMenu>
      <p className="ml-auto shrink-0 whitespace-nowrap font-mono text-[11px] tabular-nums">
        {formatTimestamp(currentTimeMs)}
        <span className="text-muted-foreground"> / {formatTimestamp(durationMs)}</span>
      </p>
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
  const [playbackRate, setPlaybackRate] = useState<PlaybackRate>(readPlaybackRate);
  const [audioDurationMs, setAudioDurationMs] = useState(artifact.durationMs);
  const [transcriptQuery, setTranscriptQuery] = useState("");

  const activeIndex = useMemo(
    () => activeSegmentIndex(segments, currentTimeMs),
    [segments, currentTimeMs],
  );
  const normalizedQuery = transcriptQuery.trim().toLowerCase();
  const visibleSegments = useMemo(() => {
    const indexed = segments.map((seg, index) => ({ seg, index }));
    if (!normalizedQuery) return indexed;
    return indexed.filter(({ seg }) => seg.text.toLowerCase().includes(normalizedQuery));
  }, [segments, normalizedQuery]);
  const transcriptLines = useMemo(
    () => artifact.transcript.split(/\n/).filter((line) => line.trim().length > 0),
    [artifact.transcript],
  );
  const visibleTranscriptLines = useMemo(() => {
    if (!normalizedQuery) return transcriptLines;
    return transcriptLines.filter((line) => line.toLowerCase().includes(normalizedQuery));
  }, [transcriptLines, normalizedQuery]);
  const canSearchTranscript =
    artifact.status === "ready" && (segments.length > 0 || transcriptLines.length > 0);

  useEffect(() => {
    let cancelled = false;
    setAudioUrl(null);
    setPeaks([]);
    setProgress(0);
    setCurrentTimeMs(0);
    setPlaying(false);
    setTranscriptQuery("");
    setAudioDurationMs(artifact.durationMs);
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
    const syncDuration = () => {
      if (Number.isFinite(audio.duration) && audio.duration > 0) {
        setAudioDurationMs(audio.duration * 1000);
      }
    };
    const onTime = () => {
      const duration = audio.duration || artifact.durationMs / 1000;
      setCurrentTimeMs(audio.currentTime * 1000);
      setProgress(duration ? audio.currentTime / duration : 0);
      syncDuration();
    };
    const applyRate = () => {
      audio.playbackRate = playbackRate;
    };
    const onPlay = () => setPlaying(true);
    const onPause = () => setPlaying(false);
    audio.addEventListener("loadedmetadata", syncDuration);
    audio.addEventListener("loadedmetadata", applyRate);
    audio.addEventListener("durationchange", syncDuration);
    audio.addEventListener("timeupdate", onTime);
    audio.addEventListener("play", onPlay);
    audio.addEventListener("pause", onPause);
    syncDuration();
    applyRate();
    return () => {
      audio.removeEventListener("loadedmetadata", syncDuration);
      audio.removeEventListener("loadedmetadata", applyRate);
      audio.removeEventListener("durationchange", syncDuration);
      audio.removeEventListener("timeupdate", onTime);
      audio.removeEventListener("play", onPlay);
      audio.removeEventListener("pause", onPause);
    };
  }, [audioUrl, artifact.durationMs, playbackRate]);

  useEffect(() => {
    if (!followPlayback || activeIndex < 0 || normalizedQuery) return;
    lineRefs.current[activeIndex]?.scrollIntoView({ block: "center", behavior: "smooth" });
  }, [activeIndex, followPlayback, normalizedQuery]);

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

  function changePlaybackRate(rate: PlaybackRate) {
    setPlaybackRate(rate);
    try {
      localStorage.setItem(PLAYBACK_RATE_KEY, String(rate));
    } catch {
      /* ignore quota / private mode */
    }
    if (audioRef.current) audioRef.current.playbackRate = rate;
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {artifact.hasAudio ? (
        <div className="shrink-0">
          {audioUrl ? <audio ref={audioRef} src={audioUrl} className="hidden" /> : null}
          {peaks.length > 0 ? (
            <StaticWaveform peaks={peaks} progress={progress} onSeek={seek} />
          ) : (
            <div className="flex h-20 items-center rounded-2xl bg-black/20 px-4 text-sm text-muted-foreground">
              Preparing waveform…
            </div>
          )}
          <PlaybackToolbar
            disabled={!audioUrl}
            playing={playing}
            followPlayback={followPlayback}
            playbackRate={playbackRate}
            currentTimeMs={currentTimeMs}
            durationMs={audioDurationMs || artifact.durationMs}
            onTogglePlay={() => {
              const audio = audioRef.current;
              if (!audio) return;
              if (audio.paused) void audio.play();
              else audio.pause();
            }}
            onToggleFollow={() => setFollowPlayback((value) => !value)}
            onPlaybackRateChange={changePlaybackRate}
          />
        </div>
      ) : (
        <p className="shrink-0 text-sm text-muted-foreground">No waveform — this meeting is transcript only.</p>
      )}

      <ScrollArea className="mt-3 min-h-0 flex-1">
        {artifact.status === "transcribing" ? (
          <p className="text-sm text-muted-foreground">
            Transcribing with small.en. This can take a minute on a long lecture.
          </p>
        ) : artifact.status === "queued" ? (
          <p className="text-sm text-muted-foreground">Waiting to transcribe…</p>
        ) : artifact.status === "failed" ? (
          <FailedTranscript artifact={artifact} />
        ) : segments.length > 0 ? (
          visibleSegments.length > 0 ? (
            <div className="flex flex-col gap-1 py-0.5 pr-2">
              {visibleSegments.map(({ seg, index }) => (
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
            <p className="px-1 py-6 text-center text-sm text-muted-foreground">
              No lines match “{transcriptQuery.trim()}”
            </p>
          )
        ) : visibleTranscriptLines.length > 0 ? (
          <div className="flex flex-col gap-1 py-0.5 pr-2">
            {visibleTranscriptLines.map((line, index) => (
              <p key={`${index}-${line.slice(0, 24)}`} className="px-4 py-2.5 text-sm leading-6">
                {line}
              </p>
            ))}
          </div>
        ) : artifact.transcript.trim() && normalizedQuery ? (
          <p className="px-1 py-6 text-center text-sm text-muted-foreground">
            No lines match “{transcriptQuery.trim()}”
          </p>
        ) : (
          <p className="whitespace-pre-wrap text-sm leading-relaxed">{artifact.transcript}</p>
        )}
      </ScrollArea>

      {canSearchTranscript ? (
        <div className="relative mt-2 shrink-0">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={transcriptQuery}
            onChange={(event) => setTranscriptQuery(event.target.value)}
            placeholder="Search transcript"
            aria-label="Search transcript"
            className={cn("pl-8", transcriptQuery ? "pr-8" : null)}
          />
          {transcriptQuery ? (
            <button
              type="button"
              aria-label="Clear search"
              onClick={() => setTranscriptQuery("")}
              className="absolute top-1/2 right-1.5 flex size-6 -translate-y-1/2 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-white/10 hover:text-foreground"
            >
              <X className="size-3.5" />
            </button>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
