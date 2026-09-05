import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

export function CollapsedRail({
  label,
  expandLabel,
  onExpand,
  icon,
  action,
}: {
  label: string;
  expandLabel: string;
  onExpand: () => void;
  icon?: ReactNode;
  action?: ReactNode;
}) {
  return (
    <section className="glass-panel flex min-h-0 flex-1 flex-col items-center rounded-3xl py-2">
      <button
        type="button"
        aria-label={expandLabel}
        onClick={onExpand}
        className={cn(
          "flex min-h-0 w-full flex-1 flex-col items-center rounded-3xl px-1 py-1",
          "text-muted-foreground transition-colors hover:bg-white/5 hover:text-foreground",
        )}
      >
        {icon ? <span className="mt-1">{icon}</span> : null}
        <span className="flex min-h-0 flex-1 items-center justify-center">
          <span
            className="text-[11px] font-semibold uppercase tracking-[0.18em]"
            style={{ writingMode: "vertical-rl", transform: "rotate(180deg)" }}
          >
            {label}
          </span>
        </span>
      </button>
      {action ? <div className="pb-1">{action}</div> : null}
    </section>
  );
}
