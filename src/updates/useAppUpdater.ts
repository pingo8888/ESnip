import { computed, shallowRef, ref } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { t } from "../i18n";

type UpdateStatus = "idle" | "checking" | "available" | "notAvailable" | "downloading" | "installing" | "error";

const currentUpdate = shallowRef<Update | null>(null);
const status = ref<UpdateStatus>("idle");
const message = ref("");
let activeTask: Promise<void> | null = null;

export function useAppUpdater() {
  return {
    checkAndInstallUpdate,
    checkForUpdate,
    hasUpdate: computed(() => Boolean(currentUpdate.value)),
    isBusy: computed(() => ["checking", "downloading", "installing"].includes(status.value)),
    message,
    status,
  };
}

async function checkForUpdate(options: { silent?: boolean } = {}) {
  if (status.value === "checking") {
    return currentUpdate.value;
  }

  status.value = "checking";
  if (!options.silent) {
    message.value = t("updates.checking");
  }

  try {
    const update = await check();
    currentUpdate.value = update;

    if (update) {
      status.value = "available";
      message.value = t("updates.available", { version: update.version });
      return update;
    }

    status.value = "notAvailable";
    if (!options.silent) {
      message.value = t("updates.latest");
    }
    return null;
  } catch (error) {
    status.value = "error";
    if (!options.silent) {
      message.value = formatUpdateError(error);
    }
    return null;
  }
}

async function checkAndInstallUpdate() {
  if (activeTask) {
    return activeTask;
  }

  activeTask = runCheckAndInstall().finally(() => {
    activeTask = null;
  });

  return activeTask;
}

async function runCheckAndInstall() {
  try {
    const update = currentUpdate.value ?? (await checkForUpdate());

    if (!update) {
      return;
    }

    status.value = "downloading";
    message.value = t("updates.downloading", { version: update.version });

    await update.downloadAndInstall();

    status.value = "installing";
    message.value = t("updates.restarting");
    await relaunch();
  } catch (error) {
    status.value = "error";
    message.value = formatUpdateError(error);
  }
}

function formatUpdateError(error: unknown) {
  const detail = error instanceof Error ? error.message : String(error);

  return t("updates.failed", { error: detail });
}
