import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

/** Open (or focus) a dedicated window that plays the artifact’s stored MP4. */
export async function openArtifactVideoWindow(artifactId: string, title?: string) {
  const label = `video-${artifactId}`;
  const existing = await WebviewWindow.getByLabel(label);
  if (existing) {
    await existing.setFocus();
    return;
  }

  const params = new URLSearchParams({
    window: "video",
    id: artifactId,
  });

  await new Promise<void>((resolve, reject) => {
    const win = new WebviewWindow(label, {
      url: `index.html?${params.toString()}`,
      title: title?.trim() || "Video",
      width: 960,
      height: 540,
      minWidth: 480,
      minHeight: 320,
      center: true,
      focus: true,
      resizable: true,
      backgroundColor: "#000000",
    });

    void win.once("tauri://created", () => resolve());
    void win.once("tauri://error", (event) => {
      reject(new Error(typeof event.payload === "string" ? event.payload : "Could not open video window."));
    });
  });
}
