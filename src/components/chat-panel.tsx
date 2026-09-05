import { useEffect, useMemo, useState } from "react";
import { MessageCircle, Send } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Textarea } from "@/components/ui/textarea";
import * as api from "@/lib/api";
import type { ChatScopeType, ChatTurn, ScopePreview } from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { selectedContext } from "@/lib/selection";
import { cn } from "@/lib/utils";
import { useArtifactStore } from "@/store/useArtifactStore";
import { useLibraryStore } from "@/store/useLibraryStore";

function defaultScope(
  selection: ReturnType<typeof useLibraryStore.getState>["selection"],
  artifactId: string | null,
): { type: ChatScopeType; id: string } | null {
  if (artifactId && selection.kind === "group") {
    return { type: "artifact", id: artifactId };
  }
  if (selection.kind === "group") return { type: "meeting_group", id: selection.groupId };
  if (selection.kind === "topic") return { type: "topic", id: selection.topicId };
  if (selection.kind === "organization") return { type: "organization", id: selection.organizationId };
  return null;
}

export function ChatPanel({ keyConfigured, onNeedKey }: { keyConfigured: boolean; onNeedKey: () => void }) {
  const orgs = useLibraryStore((s) => s.orgs);
  const selection = useLibraryStore((s) => s.selection);
  const artifactId = useArtifactStore((s) => s.activeId);
  const { org, topic, group } = selectedContext(orgs, selection);
  const fallback = defaultScope(selection, artifactId);
  const [scopeType, setScopeType] = useState<ChatScopeType>(fallback?.type ?? "meeting_group");
  const [question, setQuestion] = useState("");
  const [preview, setPreview] = useState<ScopePreview | null>(null);
  const [messages, setMessages] = useState<ChatTurn[]>([]);
  const [busy, setBusy] = useState(false);

  const scopeId = useMemo(() => {
    if (scopeType === "organization") return org?.id ?? null;
    if (scopeType === "topic") return topic?.id ?? null;
    if (scopeType === "meeting_group") return group?.id ?? null;
    return artifactId;
  }, [scopeType, org, topic, group, artifactId]);

  useEffect(() => {
    const next = defaultScope(selection, artifactId);
    if (next) setScopeType(next.type);
  }, [selection, artifactId]);

  useEffect(() => {
    if (!scopeId) {
      setPreview(null);
      setMessages([]);
      return;
    }
    void api.chatScopePreview(scopeType, scopeId).then(setPreview).catch(() => setPreview(null));
    void api.getChatThread(scopeType, scopeId).then((thread) => setMessages(thread.messages));
  }, [scopeType, scopeId]);

  async function send() {
    if (!scopeId || !question.trim()) return;
    if (!keyConfigured) {
      onNeedKey();
      return;
    }
    if (!preview || preview.readyCount === 0) {
      toast.error("There is no ready transcript in this scope yet.");
      return;
    }
    setBusy(true);
    try {
      const thread = await api.askChat(scopeType, scopeId, question.trim());
      setMessages(thread.messages);
      setQuestion("");
    } catch (err) {
      toast.error(errorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="glass-panel flex min-h-0 flex-col rounded-3xl p-4">
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <MessageCircle className="size-4 text-primary" />
          <p className="text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
            Chat
          </p>
        </div>
        {scopeId ? (
          <Button
            size="xs"
            variant="ghost"
            onClick={() => {
              void api.newChatThread(scopeType, scopeId).then((thread) => setMessages(thread.messages));
            }}
          >
            New thread
          </Button>
        ) : null}
      </div>

      <div className="mt-3 flex flex-wrap gap-1">
        {(
          [
            ["organization", "Org", Boolean(org)],
            ["topic", "Topic", Boolean(topic)],
            ["meeting_group", "Group", Boolean(group)],
            ["artifact", "This file", Boolean(artifactId)],
          ] as const
        ).map(([value, label, enabled]) => (
          <button
            key={value}
            type="button"
            disabled={!enabled}
            onClick={() => setScopeType(value)}
            className={cn(
              "rounded-full px-2.5 py-1 text-[11px]",
              scopeType === value ? "bg-primary text-primary-foreground" : "bg-white/5 text-muted-foreground",
              !enabled && "opacity-40",
            )}
          >
            {label}
          </button>
        ))}
      </div>

      {preview ? (
        <p className="mt-2 text-[12px] text-muted-foreground">
          {preview.usedCount} ready
          {preview.omittedCount > 0 ? ` · using ${preview.usedCount} most recent · ${preview.omittedCount} omitted` : null}
          {preview.totalCount > preview.readyCount
            ? ` · ${preview.totalCount - preview.readyCount} not ready`
            : null}
        </p>
      ) : (
        <p className="mt-6 text-sm leading-relaxed text-muted-foreground">
          Select a meeting group and add a transcript, then ask across the organization, topic, group,
          or this file.
        </p>
      )}

      <ScrollArea className="mt-3 min-h-0 flex-1">
        <div className="flex flex-col gap-3 pr-2">
          {messages.map((message, index) => (
            <div
              key={`${message.role}-${index}`}
              className={cn(
                "rounded-2xl px-3 py-2 text-sm leading-relaxed",
                message.role === "user" ? "bg-primary/10" : "bg-black/20",
              )}
            >
              <p className="mb-1 text-[10px] uppercase tracking-[0.14em] text-muted-foreground">
                {message.role === "user" ? "You" : "BellaNote"}
              </p>
              <div className="whitespace-pre-wrap">{message.content}</div>
            </div>
          ))}
        </div>
      </ScrollArea>

      <div className="mt-3 flex flex-col gap-2">
        <Textarea
          value={question}
          disabled={!scopeId || busy}
          placeholder={keyConfigured ? "Ask what was decided…" : "Add an OpenAI token in Settings to chat"}
          onChange={(e) => setQuestion(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              void send();
            }
          }}
        />
        <Button disabled={!scopeId || busy || !question.trim()} onClick={() => void send()}>
          <Send />
          Ask
        </Button>
      </div>
    </section>
  );
}
