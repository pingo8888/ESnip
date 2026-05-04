export function formatHotkeyParts(hotkey: string) {
  return hotkey.split("+").filter(Boolean);
}

export function isHotkeyEvent(event: KeyboardEvent, hotkey: string) {
  const expected = normalizeHotkeyString(hotkey);
  const actual = normalizeHotkeyFromKeyboardEvent(event);
  return Boolean(expected && actual && expected.toLowerCase() === actual.toLowerCase());
}

export function normalizeHotkeyFromKeyboardEvent(event: KeyboardEvent) {
  if (event.metaKey || event.shiftKey) {
    return null;
  }

  const keyToken = extractHotkeyKeyToken(event);
  if (!keyToken) {
    return null;
  }

  const validModifierCombo = (event.altKey && !event.ctrlKey) || (event.ctrlKey && event.altKey);
  if (!validModifierCombo) {
    return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey) {
    parts.push("Ctrl");
  }
  if (event.altKey) {
    parts.push("Alt");
  }
  parts.push(keyToken);
  return parts.join("+");
}

export function normalizeHotkeyString(hotkey: string) {
  const parts = hotkey
    .trim()
    .replace(/\s+/g, "")
    .split("+")
    .filter(Boolean);

  if (parts.length !== 2 && parts.length !== 3) {
    return null;
  }

  let ctrl = false;
  let alt = false;
  for (const modifier of parts.slice(0, -1)) {
    const normalized = modifier.toLowerCase();
    if (normalized === "ctrl" && !ctrl) {
      ctrl = true;
    } else if (normalized === "alt" && !alt) {
      alt = true;
    } else {
      return null;
    }
  }

  if (!((alt && !ctrl) || (ctrl && alt))) {
    return null;
  }

  const keyToken = normalizeKeyToken(parts[parts.length - 1]);
  if (!keyToken) {
    return null;
  }

  return [...(ctrl ? ["Ctrl"] : []), "Alt", keyToken].join("+");
}

function extractHotkeyKeyToken(event: KeyboardEvent) {
  const code = event.code ?? "";
  if (/^Key[A-Z]$/.test(code)) {
    return code.slice(3);
  }
  if (/^Digit[0-9]$/.test(code)) {
    return code.slice(5);
  }
  if (event.key === "Enter") {
    return "Enter";
  }

  if (event.key === "Control" || event.key === "Alt" || event.key === "Shift") {
    return null;
  }
  if (/^[a-z0-9]$/i.test(event.key)) {
    return event.key.toUpperCase();
  }
  return null;
}

function normalizeKeyToken(key: string) {
  if (/^[a-z0-9]$/i.test(key)) {
    return key.toUpperCase();
  }
  if (key.toLowerCase() === "enter") {
    return "Enter";
  }
  return null;
}
