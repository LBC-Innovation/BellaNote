import type { ReactNode } from "react";

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
    <section className="group/rail glass-panel relative flex min-h-0 flex-1 cursor-pointer flex-col overflow-hidden rounded-3xl">
      <div className="pointer-events-none absolute inset-0 rounded-[inherit] bg-transparent transition-colors group-hover/rail:bg-white/[0.04]" />
      <button
        type="button"
        aria-label={expandLabel}
        onClick={onExpand}
        className="relative z-10 flex min-h-0 w-full flex-1 cursor-pointer flex-col items-center py-3 text-muted-foreground transition-colors hover:text-foreground"
      >
        {icon ? <span className="mt-0.5">{icon}</span> : null}
        <span className="flex min-h-0 flex-1 items-center justify-center">
          <span
            className="text-[11px] font-semibold uppercase tracking-[0.18em]"
            style={{ writingMode: "vertical-rl", transform: "rotate(180deg)" }}
          >
            {label}
          </span>
        </span>
      </button>
      {action ? <div className="relative z-10 flex justify-center pb-2">{action}</div> : null}
    </section>
  );
}
