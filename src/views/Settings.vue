<template>
  <h1 class="text-xl font-bold underline">Settings</h1>
  <h2 class="text-lg font-bold">Content</h2>
  <button @click="exportNotes" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
    Export content
  </button>
</template>

<script setup>
import {invoke} from "@tauri-apps/api/tauri";
import {open} from "@tauri-apps/api/dialog";

async function exportNotes() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: ".",
    });

    if (selected) {
      const exportPath = selected;
      let result = await invoke("export_notes", {exportPath: exportPath});
      console.log("All notes exported:", result);
    }
  } catch (error) {
    console.error("Failed exporting notes:", error);
  }
}
</script>