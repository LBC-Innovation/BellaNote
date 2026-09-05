import { MessageCircle, Settings2, Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

export default function App() {
  return (
    <div className="flex h-full flex-col text-foreground">
      <header
        className="flex h-12 shrink-0 items-center justify-between px-5"
        data-tauri-drag-region
      >
        <div className="flex items-center gap-2.5 pl-16">
          <span className="text-sm font-semibold tracking-tight">BellaNote</span>
          <Badge variant="secondary" className="font-normal">
            Beautiful Note
          </Badge>
        </div>
        <Button variant="ghost" size="icon-sm" aria-label="Settings" disabled>
          <Settings2 />
        </Button>
      </header>

      <main className="grid min-h-0 flex-1 grid-cols-[280px_minmax(0,1fr)_360px] gap-3 px-3 pb-3">
        <section className="glass-panel flex flex-col rounded-3xl p-4">
          <p className="text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
            Library
          </p>
          <div className="mt-6 flex flex-1 flex-col items-start justify-center gap-2 px-1">
            <h2 className="text-lg font-semibold tracking-tight">Start with an organization</h2>
            <p className="text-sm leading-relaxed text-muted-foreground">
              Create Duke, then a class topic, then a meeting group. Files live inside the group.
            </p>
          </div>
        </section>

        <section className="glass-panel flex flex-col items-center justify-center rounded-3xl p-8 text-center">
          <div className="mb-4 flex size-12 items-center justify-center rounded-2xl bg-primary/15 text-primary">
            <Sparkles className="size-5" />
          </div>
          <h1 className="max-w-md text-2xl font-semibold tracking-tight">
            Stay in the meeting. BellaNote writes the beautiful note.
          </h1>
          <p className="mt-3 max-w-sm text-sm leading-relaxed text-muted-foreground">
            Local transcripts, a quiet workspace, and chat that answers from the files you choose.
          </p>
        </section>

        <section className="glass-panel flex flex-col rounded-3xl p-4">
          <div className="flex items-center gap-2">
            <MessageCircle className="size-4 text-primary" />
            <p className="text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
              Chat
            </p>
          </div>
          <p className="mt-6 text-sm leading-relaxed text-muted-foreground">
            Add a meeting group and a transcript, then ask questions across an organization, topic,
            group, or a single file.
          </p>
        </section>
      </main>
    </div>
  );
}
