import { readonly, ref } from "vue";

export interface HomeCardSize {
  height: number;
  width: number;
}

const homeCardSize = ref<HomeCardSize | null>(null);

export const measuredHomeCardSize = readonly(homeCardSize);

export function updateMeasuredHomeCardSize(size: HomeCardSize) {
  if (size.width <= 0 || size.height <= 0) {
    return;
  }

  homeCardSize.value = size;
}
