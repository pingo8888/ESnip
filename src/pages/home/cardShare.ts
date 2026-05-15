import { invoke } from "@tauri-apps/api/core";

export async function copyCardElementAsPng(element: HTMLElement): Promise<void> {
  const { toBlob } = await import("html-to-image");

  await waitForFrame();

  const blob = await toBlob(element, {
    backgroundColor: getComputedStyle(element).backgroundColor,
    cacheBust: true,
    pixelRatio: window.devicePixelRatio || 1,
  });

  if (!blob) {
    throw new Error("Failed to render card PNG.");
  }

  await invoke("copy_png_to_clipboard", {
    pngBase64: await blobToBase64(blob),
  });
}

function waitForFrame() {
  return new Promise<void>((resolve) => {
    requestAnimationFrame(() => resolve());
  });
}

function blobToBase64(blob: Blob) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();

    reader.onerror = () => reject(reader.error ?? new Error("Failed to read card PNG."));
    reader.onload = () => {
      const result = String(reader.result ?? "");
      const [, base64 = ""] = result.split(",", 2);

      if (!base64) {
        reject(new Error("Failed to encode card PNG."));
        return;
      }

      resolve(base64);
    };

    reader.readAsDataURL(blob);
  });
}
