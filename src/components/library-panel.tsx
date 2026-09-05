import { useMemo, useState } from "react";
import { ChevronRight, MoreHorizontal, PanelLeft, PanelLeftClose, Plus } from "lucide-react";
import { toast } from "sonner";
import { CollapsedRail } from "@/components/collapsed-rail";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { NameDialog } from "@/components/name-dialog";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ScrollArea } from "@/components/ui/scroll-area";
import * as api from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { cn } from "@/lib/utils";
import { useLibraryStore } from "@/store/useLibraryStore";
import type { LibraryOrganization, LibraryTopic, MeetingGroup } from "@/lib/types";

type DialogKind =
  | { type: "create-org" }
  | { type: "create-topic"; organizationId: string }
  | { type: "create-group"; topicId: string }
  | { type: "rename"; kind: "org" | "topic" | "group"; id: string; name: string }
  | { type: "delete"; kind: "org" | "topic" | "group"; id: string; name: string };

function RowActions({
  onRename,
  onDelete,
}: {
  onRename: () => void;
  onDelete: () => void;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        className="inline-flex size-6 items-center justify-center rounded-md opacity-0 hover:bg-white/10 group-hover:opacity-100"
        aria-label="More"
      >
        <MoreHorizontal className="size-3.5" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onClick={onRename}>Rename</DropdownMenuItem>
        <DropdownMenuItem variant="destructive" onClick={onDelete}>
          Delete
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function TreeButton({
  active,
  indent,
  label,
  meta,
  onClick,
  actions,
}: {
  active: boolean;
  indent: number;
  label: string;
  meta?: string;
  onClick: () => void;
  actions: React.ReactNode;
}) {
  return (
    <div
      className={cn(
        "group flex items-center gap-1 rounded-xl px-2 py-1.5",
        active ? "bg-primary/15 text-foreground" : "hover:bg-white/5",
      )}
      style={{ paddingLeft: 8 + indent * 14 }}
    >
      <button type="button" className="min-w-0 flex-1 text-left" onClick={onClick}>
        <div className="truncate text-sm font-medium">{label}</div>
        {meta ? <div className="truncate text-[11px] text-muted-foreground">{meta}</div> : null}
      </button>
      {actions}
    </div>
  );
}

export function LibraryPanel({
  collapsed,
  onCollapsedChange,
}: {
  collapsed: boolean;
  onCollapsedChange: (collapsed: boolean) => void;
}) {
  const orgs = useLibraryStore((s) => s.orgs);
  const selection = useLibraryStore((s) => s.selection);
  const select = useLibraryStore((s) => s.select);
  const load = useLibraryStore((s) => s.load);
  const [dialog, setDialog] = useState<DialogKind | null>(null);

  const createTitle = useMemo(() => {
    if (!dialog) return "";
    if (dialog.type === "create-org") return "New organization";
    if (dialog.type === "create-topic") return "New topic";
    if (dialog.type === "create-group") return "New meeting group";
    if (dialog.type === "rename") return "Rename";
    return "";
  }, [dialog]);

  async function refreshAfter(run: () => Promise<unknown>) {
    await run();
    await load();
  }

  return (
    <div className="flex h-full min-h-0 min-w-0 flex-col">
      {collapsed ? (
        <CollapsedRail
          label="Library"
          expandLabel="Expand library"
          onExpand={() => onCollapsedChange(false)}
          icon={<PanelLeft className="size-4" />}
          action={
            <Button
              size="icon-sm"
              aria-label="New organization"
              onClick={() => {
                onCollapsedChange(false);
                setDialog({ type: "create-org" });
              }}
            >
              <Plus />
            </Button>
          }
        />
      ) : (
        <section className="glass-panel flex min-h-0 flex-1 flex-col rounded-3xl p-4">
          <div className="flex items-center justify-between gap-2">
            <div className="flex min-w-0 items-center gap-1">
              <Button
                variant="ghost"
                size="icon-xs"
                aria-label="Collapse library"
                onClick={() => onCollapsedChange(true)}
              >
                <PanelLeftClose />
              </Button>
              <p className="text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
                Library
              </p>
            </div>
            <Button size="sm" onClick={() => setDialog({ type: "create-org" })}>
              <Plus />
              Organization
            </Button>
          </div>

          <ScrollArea className="mt-3 min-h-0 flex-1">
            {orgs.length === 0 ? (
              <div className="px-1 pt-6">
                <h2 className="text-base font-semibold tracking-tight">Start with an organization</h2>
                <p className="mt-2 text-sm leading-relaxed text-muted-foreground">
                  Example: Duke, then Competitive Strategies, then Lecture Class 1.
                </p>
              </div>
            ) : (
              <div className="flex flex-col gap-1 pr-2">
                {orgs.map((org) => (
                  <OrgBranch
                    key={org.id}
                    org={org}
                    selection={selection}
                    select={select}
                    onCreateTopic={() => setDialog({ type: "create-topic", organizationId: org.id })}
                    onCreateGroup={(topicId) => setDialog({ type: "create-group", topicId })}
                    onRename={(kind, id, name) => setDialog({ type: "rename", kind, id, name })}
                    onDelete={(kind, id, name) => setDialog({ type: "delete", kind, id, name })}
                  />
                ))}
              </div>
            )}
          </ScrollArea>
        </section>
      )}

      <NameDialog
        open={dialog?.type === "create-org" || dialog?.type === "create-topic" || dialog?.type === "create-group" || dialog?.type === "rename"}
        title={createTitle}
        description={
          dialog?.type === "create-org"
            ? "School, company, or client. Topics live under this."
            : dialog?.type === "create-topic"
              ? "A class or subject inside this organization."
              : dialog?.type === "create-group"
                ? "A lecture, study session, or other bundle of files."
                : "This updates the name everywhere, including chat scope."
        }
        confirmLabel={dialog?.type === "rename" ? "Save" : "Create"}
        initialValue={dialog?.type === "rename" ? dialog.name : ""}
        onOpenChange={(open) => {
          if (!open) setDialog(null);
        }}
        onSubmit={async (name) => {
          if (!dialog) return;
          try {
            if (dialog.type === "create-org") {
              const org = await api.createOrganization(name);
              await load();
              select({ kind: "organization", organizationId: org.id });
            } else if (dialog.type === "create-topic") {
              const topic = await api.createTopic(dialog.organizationId, name);
              await load();
              select({
                kind: "topic",
                organizationId: dialog.organizationId,
                topicId: topic.id,
              });
            } else if (dialog.type === "create-group") {
              const group = await api.createMeetingGroup(dialog.topicId, name);
              await load();
              const org = orgs.find((item) =>
                item.topics.some((topic) => topic.id === dialog.topicId),
              );
              if (org) {
                select({
                  kind: "group",
                  organizationId: org.id,
                  topicId: dialog.topicId,
                  groupId: group.id,
                });
              }
            } else if (dialog.type === "rename") {
              await refreshAfter(() => {
                if (dialog.kind === "org") return api.renameOrganization(dialog.id, name);
                if (dialog.kind === "topic") return api.renameTopic(dialog.id, name);
                return api.renameMeetingGroup(dialog.id, name);
              });
            }
          } catch (err) {
            throw new Error(errorMessage(err));
          }
        }}
      />

      <ConfirmDialog
        open={dialog?.type === "delete"}
        title={dialog?.type === "delete" ? `Delete ${dialog.name}?` : "Delete"}
        description="This removes the item and everything inside it from BellaNote. Original files on disk are not deleted."
        onOpenChange={(open) => {
          if (!open) setDialog(null);
        }}
        onConfirm={async () => {
          if (dialog?.type !== "delete") return;
          try {
            if (dialog.kind === "org") await api.deleteOrganization(dialog.id);
            else if (dialog.kind === "topic") await api.deleteTopic(dialog.id);
            else await api.deleteMeetingGroup(dialog.id);
            await load();
          } catch (err) {
            toast.error(errorMessage(err));
          }
        }}
      />
    </div>
  );
}

function OrgBranch({
  org,
  selection,
  select,
  onCreateTopic,
  onCreateGroup,
  onRename,
  onDelete,
}: {
  org: LibraryOrganization;
  selection: ReturnType<typeof useLibraryStore.getState>["selection"];
  select: (selection: ReturnType<typeof useLibraryStore.getState>["selection"]) => void;
  onCreateTopic: () => void;
  onCreateGroup: (topicId: string) => void;
  onRename: (kind: "org" | "topic" | "group", id: string, name: string) => void;
  onDelete: (kind: "org" | "topic" | "group", id: string, name: string) => void;
}) {
  const active = selection.kind !== "none" && selection.organizationId === org.id && selection.kind === "organization";
  return (
    <div>
      <TreeButton
        active={active}
        indent={0}
        label={org.name}
        onClick={() => select({ kind: "organization", organizationId: org.id })}
        actions={
          <RowActions
            onRename={() => onRename("org", org.id, org.name)}
            onDelete={() => onDelete("org", org.id, org.name)}
          />
        }
      />
      <div className="ml-2">
        {org.topics.map((topic) => (
          <TopicBranch
            key={topic.id}
            orgId={org.id}
            topic={topic}
            selection={selection}
            select={select}
            onCreateGroup={() => onCreateGroup(topic.id)}
            onRename={onRename}
            onDelete={onDelete}
          />
        ))}
        <button
          type="button"
          className="mt-0.5 flex items-center gap-1 rounded-lg px-2 py-1 text-[12px] text-muted-foreground hover:text-foreground"
          style={{ marginLeft: 14 }}
          onClick={onCreateTopic}
        >
          <Plus className="size-3" />
          Topic
        </button>
      </div>
    </div>
  );
}

function TopicBranch({
  orgId,
  topic,
  selection,
  select,
  onCreateGroup,
  onRename,
  onDelete,
}: {
  orgId: string;
  topic: LibraryTopic;
  selection: ReturnType<typeof useLibraryStore.getState>["selection"];
  select: (selection: ReturnType<typeof useLibraryStore.getState>["selection"]) => void;
  onCreateGroup: () => void;
  onRename: (kind: "org" | "topic" | "group", id: string, name: string) => void;
  onDelete: (kind: "org" | "topic" | "group", id: string, name: string) => void;
}) {
  const active =
    (selection.kind === "topic" || selection.kind === "group") && selection.topicId === topic.id && selection.kind === "topic";
  return (
    <div>
      <TreeButton
        active={active}
        indent={1}
        label={topic.name}
        onClick={() => select({ kind: "topic", organizationId: orgId, topicId: topic.id })}
        actions={
          <RowActions
            onRename={() => onRename("topic", topic.id, topic.name)}
            onDelete={() => onDelete("topic", topic.id, topic.name)}
          />
        }
      />
      {topic.groups.map((group) => (
        <GroupRow
          key={group.id}
          orgId={orgId}
          topicId={topic.id}
          group={group}
          selection={selection}
          select={select}
          onRename={onRename}
          onDelete={onDelete}
        />
      ))}
      <button
        type="button"
        className="mt-0.5 flex items-center gap-1 rounded-lg px-2 py-1 text-[12px] text-muted-foreground hover:text-foreground"
        style={{ marginLeft: 28 }}
        onClick={onCreateGroup}
      >
        <Plus className="size-3" />
        Meeting group
      </button>
    </div>
  );
}

function GroupRow({
  orgId,
  topicId,
  group,
  selection,
  select,
  onRename,
  onDelete,
}: {
  orgId: string;
  topicId: string;
  group: MeetingGroup;
  selection: ReturnType<typeof useLibraryStore.getState>["selection"];
  select: (selection: ReturnType<typeof useLibraryStore.getState>["selection"]) => void;
  onRename: (kind: "org" | "topic" | "group", id: string, name: string) => void;
  onDelete: (kind: "org" | "topic" | "group", id: string, name: string) => void;
}) {
  const active = selection.kind === "group" && selection.groupId === group.id;
  const date = new Date(group.occurredAt);
  const meta = Number.isNaN(date.getTime())
    ? undefined
    : date.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });
  return (
    <TreeButton
      active={active}
      indent={2}
      label={group.name}
      meta={meta}
      onClick={() =>
        select({
          kind: "group",
          organizationId: orgId,
          topicId,
          groupId: group.id,
        })
      }
      actions={
        <RowActions
          onRename={() => onRename("group", group.id, group.name)}
          onDelete={() => onDelete("group", group.id, group.name)}
        />
      }
    />
  );
}

export function BreadcrumbTrail() {
  const orgs = useLibraryStore((s) => s.orgs);
  const selection = useLibraryStore((s) => s.selection);
  const parts = [];
  if (selection.kind !== "none") {
    const org = orgs.find((item) => item.id === selection.organizationId);
    if (org) {
      parts.push(org.name);
      if (selection.kind !== "organization") {
        const topic = org.topics.find((item) => item.id === selection.topicId);
        if (topic) {
          parts.push(topic.name);
          if (selection.kind === "group") {
            const group = topic.groups.find((item) => item.id === selection.groupId);
            if (group) parts.push(group.name);
          }
        }
      }
    }
  }
  if (parts.length === 0) return <span className="text-muted-foreground">Library</span>;
  return (
    <div className="flex min-w-0 items-center gap-1 text-sm text-muted-foreground">
      {parts.map((part, index) => (
        <span key={`${part}-${index}`} className="flex min-w-0 items-center gap-1">
          {index > 0 ? <ChevronRight className="size-3 shrink-0" /> : null}
          <span className={cn("truncate", index === parts.length - 1 && "font-medium text-foreground")}>
            {part}
          </span>
        </span>
      ))}
    </div>
  );
}
