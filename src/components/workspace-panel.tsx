import { FileText, FolderPlus, PanelLeftClose, Sparkles } from "lucide-react";
import { ArtifactWorkspace } from "@/components/artifact-workspace";
import { CollapsedRail } from "@/components/collapsed-rail";
import { BreadcrumbTrail } from "@/components/library-panel";
import { Button } from "@/components/ui/button";
import { selectedContext } from "@/lib/selection";
import { useLibraryStore } from "@/store/useLibraryStore";

export function WorkspacePanel({
  collapsed,
  onCollapsedChange,
}: {
  collapsed: boolean;
  onCollapsedChange: (collapsed: boolean) => void;
}) {
  const orgs = useLibraryStore((s) => s.orgs);
  const selection = useLibraryStore((s) => s.selection);
  const { org, topic, group } = selectedContext(orgs, selection);

  if (collapsed) {
    return (
      <div className="flex h-full min-h-0 min-w-0 flex-col">
        <CollapsedRail
          label="Transcript"
          expandLabel="Expand transcript"
          onExpand={() => onCollapsedChange(false)}
          icon={<FileText className="size-4" />}
        />
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 min-w-0 flex-col">
      <section className="glass-panel flex min-h-0 flex-1 flex-col rounded-3xl p-5">
        <div className="flex min-w-0 items-center gap-1">
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label="Collapse transcript"
            onClick={() => onCollapsedChange(true)}
          >
            <PanelLeftClose />
          </Button>
          <div className="min-w-0 flex-1">
            <BreadcrumbTrail />
          </div>
        </div>
        <div
          className={
            group
              ? "mt-4 flex min-h-0 flex-1 flex-col"
              : "mt-6 flex min-h-0 flex-1 flex-col items-center justify-center text-center"
          }
        >
          {!org ? (
            <>
              <div className="mb-4 flex size-12 items-center justify-center rounded-2xl bg-primary/15 text-primary">
                <Sparkles className="size-5" />
              </div>
              <h1 className="max-w-md text-2xl font-semibold tracking-tight">
                Stay present, let Bella capture the context.
              </h1>
              <p className="mt-3 max-w-sm text-sm leading-relaxed text-muted-foreground">
                Create an organization, add a topic, then a meeting group.
              </p>
            </>
          ) : !topic ? (
            <EmptyHint
              title={`Add a topic in ${org.name}`}
              body="Group related meetings — a project, account, or workstream."
            />
          ) : !group ? (
            <EmptyHint
              title={`Add a meeting group in ${topic.name}`}
              body="Recordings and transcripts for a meeting live here."
            />
          ) : (
            <div className="flex min-h-0 w-full flex-1 flex-col items-stretch text-left">
              <ArtifactWorkspace groupId={group.id} groupName={group.name} />
            </div>
          )}
        </div>
      </section>
    </div>
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
