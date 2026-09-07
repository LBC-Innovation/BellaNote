import React from "react";
import ReactDOM from "react-dom/client";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";
import { isMac } from "@/lib/platform";
import App from "./App";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <TooltipProvider>
      <App />
      <Toaster theme="dark" position="bottom-right" />
      <Toaster
        theme="dark"
        id="notice"
        className="toaster group notice-toaster"
        position="top-center"
        offset={{ top: isMac() ? 88 : 56 }}
        richColors
      />
    </TooltipProvider>
  </React.StrictMode>,
);

function dismissSplash() {
  const splash = document.getElementById("splash");
  if (!splash) return;
  const remove = () => splash.remove();
  splash.classList.add("is-hidden");
  splash.addEventListener("transitionend", remove, { once: true });
  window.setTimeout(remove, 400);
}

const splashWait = Math.max(0, 3000 - performance.now());
window.setTimeout(() => {
  requestAnimationFrame(() => requestAnimationFrame(dismissSplash));
}, splashWait);
