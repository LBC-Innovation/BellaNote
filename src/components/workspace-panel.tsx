import { FolderPlus, Sparkles } from "lucide-react";
import { BreadcrumbTrail } from "@/components/library-panel";
import { selectedContext } from "@/lib/selection";
import { useLibraryStore } from "@/store/useLibraryStore";

export function WorkspacePanel() {
  const orgs = useLibraryStore((s) => s.orgs);
  const selection = useLibraryStore((s) => s.selection);
  const { org, topic, group } = selectedContext(orgs, selection);

  return (
    <section className="glass-panel flex min-h-0 flex-col rounded-3xl p-5">
      <BreadcrumbTrail />
      <div className="mt-6 flex min-h-0 flex-1 flex-col items-center justify-center text-center">
        {!org ? (
          <>
            <div className="mb-4 flex size-12 items-center justify-center rounded-2xl bg-primary/15 text-primary">
              <Sparkles className="size-5" />
            </div>
            <h1 className="max-w-md text-2xl font-semibold tracking-tight">
              Stay in the meeting. BellaNote writes the beautiful note.
            </h1>
            <p className="mt-3 max-w-sm text-sm leading-relaxed text-muted-foreground">
              Create an organization to begin the Duke-style tree: organization, topic, then a
              meeting group for files.
            </p>
          </>
        ) : !topic ? (
          <EmptyHint
            title={`Add a topic in ${org.name}`}
            body="Example: Competitive Strategies. Meeting groups and files live under a topic."
          />
        ) : !group ? (
          <EmptyHint
            title={`Add a meeting group in ${topic.name}`}
            body="Example: Lecture Class 1. Audio and transcripts will land here."
          />
        ) : (
          <EmptyHint
            title={group.name}
            body="Audio uploads and imported transcripts will appear here next."
          />
        )}
      </div>
    </section>
  );
}

function EmptyHint({ title, body }: { title: string; body: string }) {
  return (
    <>
      <div className="mb-4 flex size-12 items-center justify-center rounded-2xl bg-primary/10 text-primary">
        <FolderPlus className="size-5" />
      </div>
      <h1 className="max-w-md text-2xl font-semibold tracking-tight">{title}</h1>
      <p className="mt-3 max-w-sm text-sm leading-relaxed text-muted-foreground">{body}</p>
    </>
  );
}
