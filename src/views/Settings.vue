<template>
  <div class="mb-8 ml-8 mt-2">
    <h2 class="text-xl font-bold">Settings</h2>
    <h3 class="text-lg font-bold">Content</h3>
    <div class="mb-4">
      <button @click="exportNotes" class="font-bold py-2 px-4 rounded"
              :disabled="isExporting || isImporting">
        <span v-if="isExporting" class="spinner"></span>
        <span v-else>Export content</span>
      </button>
      <p v-if="exportResult" class="mt-2 text-green-600">{{ exportResult }}</p>
      <p v-if="exportError" class="mt-2 text-red-600">{{ exportError }}</p>
    </div>
    <div>
      <button @click="importNotes" class="font-bold py-2 px-4 rounded"
              :disabled="isImporting || isExporting">
        <span v-if="isImporting" class="spinner"></span>
        <span v-else>Import content</span>
      </button>
      <p v-if="importResult" class="mt-2 text-green-600">{{ importResult }}</p>
      <p v-if="importError" class="mt-2 text-red-600">{{ importError }}</p>
    </div>
    <h3 class="text-lg font-bold mt-4">Theme</h3>
    <div class="flex items-center space-x-4">
      <button @click="setTheme('light')" class="flex items-center space-x-2"
              :class="{'text-blue-500': theme === 'light'}">
        <img src="/light-theme.svg" alt="Light Theme" class="w-6 h-6">
        <span>Light</span>
      </button>
      <button @click="setTheme('dark')" class="flex items-center space-x-2"
              :class="{'text-blue-500': theme === 'dark'}">
        <img src="/dark-theme.svg" alt="Dark Theme" class="w-6 h-6">
        <span>Dark</span>
      </button>
      <button @click="setTheme('system')" class="flex items-center space-x-2"
              :class="{'text-blue-500': theme === 'system'}">
        <img src="/system-theme.svg" alt="System Theme" class="w-6 h-6">
        <span>System</span>
      </button>
    </div>
  </div>

</template>

<script setup>
import {ref, onMounted} from 'vue';
import {invoke} from "@tauri-apps/api/tauri";
import {info, error} from "tauri-plugin-log-api";
import {open} from '@tauri-apps/api/dialog';
import {downloadDir} from '@tauri-apps/api/path';

const exportResult = ref('');
const exportError = ref('');
const isExporting = ref(false);

const importResult = ref('');
const importError = ref('');
const isImporting = ref(false);

async function exportNotes() {
  isExporting.value = true;
  try {
    let result = await invoke("export_notes");
    info("All notes exported:", result);
    exportResult.value = `Exported ${result[0]} file(s) to ${result[1]} successfully.`;
    exportError.value = ''; // Clear any previous error message
  } catch (err) {
    error("Failed exporting notes:", err);
    exportError.value = `Failed to export notes: ${err.message}`;
    exportResult.value = ''; // Clear any previous result message
  } finally {
    isExporting.value = false;
  }
}

async function importNotes() {
  isImporting.value = true;
  try {
    const selectedDirectory = await open({
      directory: true,
      multiple: false,
      defaultPath: await downloadDir(),
    });
    if (selectedDirectory) {
      let result = await invoke("import_notes", {path: selectedDirectory});
      info("Notes imported:", result);
      importResult.value = `Imported ${result} note(s) successfully.`;
      importError.value = ''; // Clear any previous error message
    }
  } catch (err) {
    error("Failed importing notes:", err);
    importError.value = `Failed to import notes: ${err.message}`;
    importResult.value = ''; // Clear any previous result message
  } finally {
    isImporting.value = false;
  }
}

const theme = ref('system');

function setTheme(selectedTheme) {
  theme.value = selectedTheme;
  localStorage.setItem("app-theme", selectedTheme);
  updateHtmlTheme();
}

function updateHtmlTheme() {
  if (theme.value === 'system') {
    if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.documentElement.setAttribute('data-theme', 'dark');
    } else {
      document.documentElement.setAttribute('data-theme', 'light');
    }
  } else {
    document.documentElement.setAttribute('data-theme', theme.value);
  }
}

onMounted(() => {
  const storedTheme = localStorage.getItem("app-theme");
  if (storedTheme) {
    theme.value = storedTheme;
  }
  updateHtmlTheme();
});:xor
</script>
<style scoped>
.spinner {
  display: inline-block;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid #fff;
  border-top-color: transparent;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}
</style>