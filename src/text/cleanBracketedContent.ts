const HORIZONTAL_SPACE_PATTERN = /[ \t\f\v\u00a0]$/;

export function cleanBracketedContent(value: string): string {
  let output = "";
  let index = 0;

  while (index < value.length) {
    const openIndex = value.indexOf("[", index);

    if (openIndex === -1) {
      output += value.slice(index);
      break;
    }

    const closeIndex = value.indexOf("]", openIndex + 1);

    if (closeIndex === -1) {
      output += value.slice(index);
      break;
    }

    output += value.slice(index, openIndex);
    output = trimHorizontalSpaceEnd(output);

    const rightStart = skipHorizontalSpace(value, closeIndex + 1);
    const leftChar = output.charAt(output.length - 1);
    const rightChar = value.charAt(rightStart);

    if (isAsciiWordChar(leftChar) && isAsciiWordChar(rightChar)) {
      output += " ";
    }

    index = rightStart;
  }

  return output;
}

function trimHorizontalSpaceEnd(value: string): string {
  let end = value.length;

  while (end > 0 && HORIZONTAL_SPACE_PATTERN.test(value.charAt(end - 1))) {
    end -= 1;
  }

  return value.slice(0, end);
}

function skipHorizontalSpace(value: string, start: number): number {
  let index = start;

  while (index < value.length && HORIZONTAL_SPACE_PATTERN.test(value.charAt(index))) {
    index += 1;
  }

  return index;
}

function isAsciiWordChar(value: string): boolean {
  return /^[A-Za-z0-9]$/.test(value);
}
