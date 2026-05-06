const blockedCtrlKeys = new Set(["f", "o", "p", "s", "u"]);

export function installBrowserShortcutGuard() {
  window.addEventListener("keydown", handleBrowserShortcut, true);
}

export function uninstallBrowserShortcutGuard() {
  window.removeEventListener("keydown", handleBrowserShortcut, true);
}

function handleBrowserShortcut(event: KeyboardEvent) {
  if (event.defaultPrevented || event.isComposing) {
    return;
  }

  if (shouldBlockBrowserShortcut(event)) {
    event.preventDefault();
    event.stopPropagation();
  }
}

function shouldBlockBrowserShortcut(event: KeyboardEvent) {
  const key = event.key.toLowerCase();
  const primaryModifier = event.ctrlKey || event.metaKey;

  if (event.key === "F3") {
    return true;
  }

  if (import.meta.env.PROD && (event.key === "F12" || (primaryModifier && event.shiftKey && key === "i"))) {
    return true;
  }

  if (!primaryModifier || event.altKey) {
    return false;
  }

  return blockedCtrlKeys.has(key);
}
