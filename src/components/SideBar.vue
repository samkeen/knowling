<template>
  <div
      class="fixed top-20 right-0 bottom-16 bg-gray-800 text-white transition-transform duration-300 ease-in-out p-6 transform"
      :class="{
    'translate-x-full opacity-0 pointer-events-none': !sidebarOpen,
    'translate-x-0 opacity-100': sidebarOpen,
    'select-none': isResizing
  }"
      :style="{ width: sidebarWidth + 'px' }"
  >
    <!-- Sidebar header -->
    <div class="flex justify-between items-center mb-4">
      <h3 class="text-lg font-bold">Sidebar</h3>
      <!-- Close button -->
      <button
          class="text-gray-400 hover:text-gray-200"
          @click="sidebarOpen = !sidebarOpen"
      >
        <!-- Close icon -->
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
        >
          <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </button>
    </div>
    <!-- Sidebar content -->
    <div :class="{ 'select-none': isResizing }" class="space-y-4">
      <button @click="getRelatedNotes" class="px-4 py-2 bg-blue-500 text-white rounded">Get Related Notes</button>
      <div v-for="note in relatedNotes" :key="note[0].id">
        {{ note[0].id }}
      </div>
    </div>
    <!-- Resize handle -->
    <div
        class="absolute left-0 top-0 h-full w-2 hover:bg-gray-700 cursor-col-resize"
        @mousedown="startResizing"
    ></div>
  </div>
</template>

<script setup>
import {ref, reactive, defineExpose} from 'vue'
import {invoke} from "@tauri-apps/api/tauri";

// Define a reactive variable to store the related notes
const relatedNotes = reactive([]);

// Define the function to get related notes
async function getRelatedNotes() {
  relatedNotes.value = await invoke("get_note_similarities", {id: noteId});
}

// Sidebar state
const sidebarOpen = ref(false)
const sidebarWidth = ref(256)

// Resizing state
const isResizing = ref(false)
const startX = ref(0)
const startWidth = ref(0)

// Start resizing the sidebar
const startResizing = (e) => {
  isResizing.value = true
  startX.value = e.clientX
  startWidth.value = sidebarWidth.value
  document.addEventListener('mousemove', resize)
  document.addEventListener('mouseup', stopResizing)
}

// Resize the sidebar
const resize = (e) => {
  if (isResizing.value) {
    const dx = startX.value - e.clientX
    sidebarWidth.value = startWidth.value + dx
  }
}

// Stop resizing the sidebar
const stopResizing = () => {
  isResizing.value = false
  document.removeEventListener('mousemove', resize)
  document.removeEventListener('mouseup', stopResizing)
}

// Toggle sidebar
const toggleSidebar = () => {
  sidebarOpen.value = !sidebarOpen.value;
}

// Expose the toggleSidebar method
defineExpose({toggleSidebar});
</script>