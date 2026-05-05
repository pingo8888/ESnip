export function installContextMenuGuard() {
  document.addEventListener("contextmenu", handleGlobalContextMenu);
}

export function uninstallContextMenuGuard() {
  document.removeEventListener("contextmenu", handleGlobalContextMenu);
}

function handleGlobalContextMenu(event: MouseEvent) {
  if (isContextMenuAllowed(event.target)) {
    return;
  }

  event.preventDefault();
}

function isContextMenuAllowed(target: EventTarget | null) {
  if (!(target instanceof Element)) {
    return false;
  }

  if (target.closest(".note-card")) {
    return true;
  }

  return Boolean(target.closest("input, textarea, select, [contenteditable='true']"));
}
