export type HighlightPart = {
  text: string;
  highlighted: boolean;
};

export function parseHighlightTerms(query: string) {
  const terms = query
    .trim()
    .split(/\s+/)
    .filter((term) => term && !term.startsWith("#") && !term.startsWith("!#"));
  const uniqueTerms: string[] = [];

  for (const term of terms) {
    if (!uniqueTerms.some((item) => item.toLocaleLowerCase() === term.toLocaleLowerCase())) {
      uniqueTerms.push(term);
    }
  }

  return uniqueTerms;
}

export function splitHighlightParts(value: string, terms: string[]): HighlightPart[] {
  if (!value || terms.length === 0) {
    return [{ highlighted: false, text: value }];
  }

  const pattern = new RegExp(`(${terms.map(escapeRegExp).join("|")})`, "giu");
  const parts: HighlightPart[] = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(value))) {
    if (match.index > lastIndex) {
      parts.push({
        highlighted: false,
        text: value.slice(lastIndex, match.index),
      });
    }

    parts.push({
      highlighted: true,
      text: match[0],
    });
    lastIndex = pattern.lastIndex;
  }

  if (lastIndex < value.length) {
    parts.push({
      highlighted: false,
      text: value.slice(lastIndex),
    });
  }

  return parts.length > 0 ? parts : [{ highlighted: false, text: value }];
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
