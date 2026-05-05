<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { installContextMenuGuard, uninstallContextMenuGuard } from "./contextMenuGuard";
import HomePage from "./pages/HomePage.vue";

onMounted(() => {
  installContextMenuGuard();
});

onUnmounted(() => {
  uninstallContextMenuGuard();
});
</script>

<template>
  <HomePage />
</template>
