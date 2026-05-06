<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { installBrowserShortcutGuard, uninstallBrowserShortcutGuard } from "./browserShortcutGuard";
import { installContextMenuGuard, uninstallContextMenuGuard } from "./contextMenuGuard";
import HomePage from "./pages/HomePage.vue";

onMounted(() => {
  installBrowserShortcutGuard();
  installContextMenuGuard();
});

onUnmounted(() => {
  uninstallBrowserShortcutGuard();
  uninstallContextMenuGuard();
});
</script>

<template>
  <HomePage />
</template>
