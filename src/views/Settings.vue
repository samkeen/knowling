<template>
  <div class="mb-8 ml-8 mt-2">
    <h2 class="text-xl font-bold mb-5">Settings</h2>
    <h3 class="text-lg font-bold">Content</h3>
    <div class="mb-4">
      <button @click="exportNotes" class="btn btn-outline"
              :disabled="isExporting || isImporting">
        <span v-if="isExporting" class="spinner"></span>
        <span v-else>Export content</span>
      </button>
      <div v-if="exportResult" role="alert" class="alert max-w-fit">
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-info shrink-0 w-6 h-6">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
        </svg>
        <span class="whitespace-nowrap">{{ exportResult }}</span>
        <button @click="closeAlert('export')" class="btn btn-sm ml-4">X</button>
      </div>
      <!--      <p v-if="exportResult" class="mt-2 text-green-600">{{ exportResult }}</p>-->
      <p v-if="exportError" class="mt-2 text-red-600">{{ exportError }}</p>
    </div>
    <div>
      <button @click="importNotes" class="btn btn-outline"
              :disabled="isImporting || isExporting">
        <span v-if="isImporting" class="spinner"></span>
        <span v-else>Import content</span>
      </button>
      <div v-if="importResult" role="alert" class="alert max-w-fit">
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-info shrink-0 w-6 h-6">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
        </svg>
        <span class="whitespace-nowrap">{{ importResult }}</span>
        <button @click="closeAlert('import')" class="btn btn-sm ml-4">X</button>
      </div>
      <p v-if="importError" class="mt-2 text-red-600">{{ importError }}</p>
    </div>
    <h3 class="text-lg font-bold mt-4">Theme</h3>
    <div class="flex items-center space-x-4">
      <button @click="setTheme('light')" class="flex items-center space-x-2"
              :class="{'text-blue-500': theme === 'light'}">
        <img src="/light-theme.svg" alt="Light Theme" class="w-6 h-6 icon">
        <span>Light</span>
      </button>
      <button @click="setTheme('dark')" class="flex items-center space-x-2"
              :class="{'text-blue-500': theme === 'dark'}">
        <img src="/dark-theme.svg" alt="Dark Theme" class="w-6 h-6 icon">
        <span>Dark</span>
      </button>
      <button @click="setTheme('system')" class="flex items-center space-x-2"
              :class="{'text-blue-500': theme === 'system'}">
        <img src="/system-theme.svg" alt="System Theme" class="w-6 h-6 icon">
        <span>System</span>
      </button>
    </div>
    <h3 class="text-lg font-bold mt-4">Similarity score threshold</h3>
    <div class="tooltip w-1/3" :data-tip="similarityScoreThreshold">
      <input type="range" min=".05" max=".99" step="0.01" class="range w-full" v-model="similarityScoreThreshold">
    </div>
  </div>

</template>

<script setup>
import {ref, onMounted, watch} from 'vue';
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

function closeAlert(alertType) {
  if (alertType === 'export') {
    exportResult.value = '';
  } else if (alertType === 'import') {
    importResult.value = '';
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
const similarityScoreThreshold = ref(0.2);

function setTheme(selectedTheme) {
  theme.value = selectedTheme;
  localStorage.setItem("app-theme", selectedTheme);
  updateHtmlTheme();
}

// Store in localStorage whenever the value changes
watch(similarityScoreThreshold, (newThreshold) => {
  info(`Setting similarity score threshold to: ${newThreshold}`);
  localStorage.setItem("similarityScoreThreshold", newThreshold);
});

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
  const storedThreshold = localStorage.getItem("similarityScoreThreshold");
  if (storedThreshold) {
    const parsedThreshold = parseFloat(storedThreshold);
    info(`Read similarity score threshold from storage: ${parsedThreshold}`);
    similarityScoreThreshold.value = parsedThreshold;
  }
});
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