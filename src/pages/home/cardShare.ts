import { invoke } from "@tauri-apps/api/core";
import { measuredHomeCardSize } from "./homeCardMetrics";

export async function copyCardElementAsPng(element: HTMLElement): Promise<void> {
  const { toBlob } = await import("html-to-image");

  await waitForFrame();

  const rect = element.getBoundingClientRect();
  const width = Math.round(measuredHomeCardSize.value?.width ?? rect.width);
  const height = Math.round(rect.height);

  const blob = await toBlob(element, {
    backgroundColor: getComputedStyle(element).backgroundColor,
    cacheBust: true,
    height,
    pixelRatio: 1,
    style: {
      width: `${width}px`,
    },
    width,
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
