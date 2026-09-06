import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

type Props = {
  open: boolean;
  title: string;
  description: string;
  confirmLabel?: string;
  extraOption?: string;
  onOpenChange: (open: boolean) => void;
  onConfirm: (extraChecked: boolean) => Promise<void>;
};

export function ConfirmDialog({
  open,
  title,
  description,
  confirmLabel = "Delete",
  extraOption,
  onOpenChange,
  onConfirm,
}: Props) {
  const [extraChecked, setExtraChecked] = useState(false);

  useEffect(() => {
    if (open) setExtraChecked(false);
  }, [open]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="glass-panel sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        {extraOption ? (
          <label className="flex cursor-pointer items-start gap-2.5 text-sm leading-5 text-foreground">
            <input
              type="checkbox"
              className="mt-0.5 size-4 shrink-0 accent-primary"
              checked={extraChecked}
              onChange={(event) => setExtraChecked(event.target.checked)}
            />
            <span>{extraOption}</span>
          </label>
        ) : null}
        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            onClick={() => {
              void onConfirm(extraChecked).then(() => onOpenChange(false));
            }}
          >
            {confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
