import { useEffect, useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import * as api from "@/lib/api";
import { errorMessage } from "@/lib/errors";

type Props = {
  open: boolean;
  configured: boolean;
  onOpenChange: (open: boolean) => void;
  onChanged: () => void;
};

export function SettingsDialog({ open, configured, onOpenChange, onChanged }: Props) {
  const [token, setToken] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (open) setToken("");
  }, [open]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="glass-panel sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
          <DialogDescription>
            BellaNote stores the OpenAI token in the OS credential store (macOS Keychain or
            Windows Credential Manager). Chat uses gpt-4o and never uploads audio.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-2">
          <Label htmlFor="openai-token">OpenAI API token</Label>
          <Input
            id="openai-token"
            type="password"
            autoComplete="off"
            placeholder={configured ? "Token saved — paste to replace" : "sk-…"}
            value={token}
            onChange={(e) => setToken(e.target.value)}
          />
        </div>
        <DialogFooter>
          {configured ? (
            <Button
              variant="ghost"
              disabled={busy}
              onClick={() => {
                setBusy(true);
                void api
                  .clearOpenAiKey()
                  .then(() => {
                    onChanged();
                    toast.success("Token cleared");
                  })
                  .catch((err) => toast.error(errorMessage(err)))
                  .finally(() => setBusy(false));
              }}
            >
              Clear
            </Button>
          ) : null}
          <Button
            disabled={busy || !token.trim()}
            onClick={() => {
              setBusy(true);
              void api
                .setOpenAiKey(token.trim())
                .then(() => {
                  onChanged();
                  onOpenChange(false);
                  toast.success("Token saved");
                })
                .catch((err) => toast.error(errorMessage(err)))
                .finally(() => setBusy(false));
            }}
          >
            Save token
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
