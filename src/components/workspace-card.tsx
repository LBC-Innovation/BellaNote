import type { ReactNode } from "react";
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
  meta?: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section
      className={cn(
        "flex min-h-0 flex-col overflow-hidden rounded-2xl bg-black/15",
        open ? "flex-1" : "shrink-0",
      )}
    >
      <div className="group/rail relative flex shrink-0 cursor-pointer items-center">
        <div className="pointer-events-none absolute inset-0 bg-transparent transition-colors group-hover/rail:bg-white/[0.04]" />
        <button
          type="button"
          aria-expanded={open}
          onClick={() => onOpenChange(!open)}
          className="relative z-10 flex min-w-0 flex-1 cursor-pointer items-center gap-2 px-3 py-2.5 text-left"
        >
          <ChevronDown
            className={cn(
              "size-4 shrink-0 text-muted-foreground transition-transform",
              !open && "-rotate-90",
            )}
          />
          <span className="truncate text-sm font-semibold tracking-tight">{title}</span>
          {meta ? (
            typeof meta === "string" ? (
              <span className="truncate text-[11px] text-muted-foreground">{meta}</span>
            ) : (
              <span className="flex min-w-0 items-center gap-1">{meta}</span>
            )
          ) : null}
        </button>
        {actions ? (
          <div
            className="relative z-10 flex shrink-0 items-center gap-1 pr-3"
            onClick={(event) => event.stopPropagation()}
          >
            {actions}
          </div>
        ) : null}
      </div>
      {open ? <div className="flex min-h-0 flex-1 flex-col px-3 pb-3">{children}</div> : null}
    </section>
  );
}
