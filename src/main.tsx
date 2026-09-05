import React from "react";
import ReactDOM from "react-dom/client";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";
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
        offset={{ top: 56 }}
        richColors
      />
    </TooltipProvider>
  </React.StrictMode>,
);
