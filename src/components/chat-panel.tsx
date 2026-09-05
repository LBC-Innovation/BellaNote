import { MessageCircle } from "lucide-react";

export function ChatPanel() {
  return (
    <section className="glass-panel flex flex-col rounded-3xl p-4">
      <div className="flex items-center gap-2">
        <MessageCircle className="size-4 text-primary" />
        <p className="text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
          Chat
        </p>
      </div>
      <p className="mt-6 text-sm leading-relaxed text-muted-foreground">
        After a transcript is ready, you can ask questions against the organization, topic, meeting
        group, or a single file.
      </p>
    </section>
  );
}
