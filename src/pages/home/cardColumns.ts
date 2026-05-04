export interface ColumnLayout {
  columnCount: number;
  cardWidth: number;
}

export function computeColumnLayout(containerWidth: number): ColumnLayout {
  if (containerWidth < 620) {
    return { columnCount: 1, cardWidth: Math.max(containerWidth, 0) };
  }

  const columnGap = 14;
  const minColumnWidth = containerWidth < 920 ? 190 : 210;
  const columnCount = Math.max(1, Math.floor((containerWidth + columnGap) / (minColumnWidth + columnGap)));
  const cardWidth = (containerWidth - (columnCount - 1) * columnGap) / columnCount;

  return { columnCount, cardWidth: Math.round(cardWidth) };
}
