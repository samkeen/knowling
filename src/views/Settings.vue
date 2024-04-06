<template>
  <h1 class="text-xl font-bold underline">Settings</h1>
  <h2 class="text-lg font-bold">Content</h2>
  <button @click="exportNotes" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
    Export content
  </button>
</template>

<script setup>
import {invoke} from "@tauri-apps/api/tauri";
import {info} from "tauri-plugin-log-api";

async function exportNotes() {
  try {
    let result = await invoke("export_notes");
    info("All notes exported:", result);
  } catch (error) {
    console.error("Failed exporting notes:", error);
  }
}
</script>