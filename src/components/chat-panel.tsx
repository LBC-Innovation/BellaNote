import { useEffect, useMemo, useRef, useState } from "react";
import { Loader2, MessageCircle, Send } from "lucide-react";
import { toast } from "sonner";
import { ChatMarkdown } from "@/components/chat-markdown";
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

function formatChatTime(value?: string) {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  const now = new Date();
  const sameDay = date.toDateString() === now.toDateString();
  const time = date.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });
  if (sameDay) return time;
  return `${date.toLocaleDateString(undefined, { month: "short", day: "numeric" })}, ${time}`;
}

function MessageMeta({ name, at }: { name: string; at?: string }) {
  const stamp = formatChatTime(at);
  return (
    <p className="mb-1 flex items-baseline gap-2">
      <span className="text-[10px] uppercase tracking-[0.14em] text-muted-foreground">{name}</span>
      {stamp ? (
        <time dateTime={at} className="text-[10px] tabular-nums text-muted-foreground/80">
          {stamp}
        </time>
      ) : null}
    </p>
  );
}

function ThinkingBubble({ at }: { at: string }) {
  return (
    <div className="thinking-bubble rounded-2xl bg-black/20 px-3 py-2">
      <MessageMeta name="BellaNote" at={at} />
      <div className="flex items-center gap-2 py-1" aria-live="polite" aria-label="BellaNote is thinking">
        <span className="flex items-center gap-1">
          <span className="chat-dot size-1.5 rounded-full bg-primary" />
          <span className="chat-dot size-1.5 rounded-full bg-primary" style={{ animationDelay: "0.16s" }} />
          <span className="chat-dot size-1.5 rounded-full bg-primary" style={{ animationDelay: "0.32s" }} />
        </span>
        <span className="text-xs text-muted-foreground">Thinking…</span>
      </div>
    </div>
  );
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
  const [thinkingAt, setThinkingAt] = useState<string | null>(null);
  const bottomRef = useRef<HTMLDivElement | null>(null);

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
      setBusy(false);
      setThinkingAt(null);
      return;
    }
    void api.chatScopePreview(scopeType, scopeId).then(setPreview).catch(() => setPreview(null));
    void api.getChatThread(scopeType, scopeId).then((thread) => setMessages(thread.messages));
  }, [scopeType, scopeId]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ block: "end", behavior: "smooth" });
  }, [messages, busy]);

  async function send() {
    if (!scopeId || !question.trim() || busy) return;
    if (!keyConfigured) {
      onNeedKey();
      return;
    }
    if (!preview || preview.readyCount === 0) {
      toast.error("There is no ready transcript in this scope yet.");
      return;
    }
    const text = question.trim();
    const optimistic: ChatTurn = {
      role: "user",
      content: text,
      createdAt: new Date().toISOString(),
    };
    setQuestion("");
    setMessages((current) => [...current, optimistic]);
    setThinkingAt(new Date().toISOString());
    setBusy(true);
    try {
      const thread = await api.askChat(scopeType, scopeId, text);
      setMessages(thread.messages);
    } catch (err) {
      setMessages((current) => current.filter((item) => item !== optimistic));
      setQuestion(text);
      toast.error(errorMessage(err));
    } finally {
      setBusy(false);
      setThinkingAt(null);
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
            disabled={busy}
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
            disabled={!enabled || busy}
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
              key={`${message.role}-${message.createdAt ?? index}`}
              className={cn(
                "rounded-2xl px-3 py-2 text-sm leading-relaxed",
                message.role === "user" ? "bg-primary/10" : "bg-black/20",
              )}
            >
              <MessageMeta
                name={message.role === "user" ? "You" : "BellaNote"}
                at={message.createdAt}
              />
              {message.role === "assistant" ? (
                <ChatMarkdown content={message.content} />
              ) : (
                <div className="whitespace-pre-wrap">{message.content}</div>
              )}
            </div>
          ))}
          {busy && thinkingAt ? <ThinkingBubble at={thinkingAt} /> : null}
          <div ref={bottomRef} />
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
        <Button
          disabled={!scopeId || busy || !question.trim()}
          aria-busy={busy}
          className={cn(busy && "disabled:opacity-100")}
          onClick={() => void send()}
        >
          {busy ? <Loader2 className="animate-spin" /> : <Send />}
          {busy ? "Asking…" : "Ask"}
        </Button>
      </div>
    </section>
  );
}
