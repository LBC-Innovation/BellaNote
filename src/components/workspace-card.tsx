import { ChevronDown } from "lucide-react";
import { cn } from "@/lib/utils";

export function WorkspaceCard({
  open,
  onOpenChange,
  title,
  meta,
  actions,
  children,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  meta?: string;
  actions?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section
      className={cn(
        "flex min-h-0 flex-col overflow-hidden rounded-2xl bg-black/15",
        open ? "flex-1" : "shrink-0",
      )}
    >
      <div className="flex shrink-0 items-center gap-2 px-3 py-2.5">
        <button
          type="button"
          aria-expanded={open}
          onClick={() => onOpenChange(!open)}
          className="flex min-w-0 flex-1 items-center gap-2 rounded-xl px-1 py-0.5 text-left hover:bg-white/5"
        >
          <ChevronDown
            className={cn(
              "size-4 shrink-0 text-muted-foreground transition-transform",
              !open && "-rotate-90",
            )}
          />
          <span className="truncate text-sm font-semibold tracking-tight">{title}</span>
          {meta ? <span className="truncate text-[11px] text-muted-foreground">{meta}</span> : null}
        </button>
        {actions ? <div className="flex shrink-0 items-center gap-1">{actions}</div> : null}
      </div>
      {open ? <div className="flex min-h-0 flex-1 flex-col px-3 pb-3">{children}</div> : null}
    </section>
  );
}
