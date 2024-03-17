<template>
  <!-- Main container -->
  <div class="min-h-screen flex flex-col">
    <!-- Header -->
    <header class="bg-gray-900 text-white py-4 px-6">
      <!-- Navigation -->
      <nav class="flex justify-between items-center">
        <!-- App logo/title -->
        <div class="text-2xl font-bold">NovaNotes</div>
        <!-- Navigation menu items -->
        <div class="flex space-x-4">
          <RouterLink :to="{name: 'Home'}" class="px-4 py-2 bg-blue-500 text-white rounded">Notes</RouterLink>
          <RouterLink :to="{name: 'AddNote'}" class="px-4 py-2 bg-blue-500 text-white rounded">+</RouterLink>
          <RouterLink :to="{name: 'Admin'}" class="px-4 py-2 bg-blue-500 text-white rounded">Admin</RouterLink>
          <!-- Sidebar toggle button -->
          <div class="ml-4">
            <button
                class="bg-gray-800 text-white rounded-full p-2 hover:bg-gray-700"
                @click="sidebarOpen = !sidebarOpen"
            >
              <!-- Hamburger icon -->
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
                    d="M4 6h16M4 12h16M4 18h16"
                />
              </svg>
            </button>
          </div>
        </div>
      </nav>
    </header>


    <div class="flex flex-grow">
      <main class="flex-grow bg-gray-100 py-8 flex relative">
        <div class="flex-grow px-6 transition-all duration-300 ease-in-out"
             :style="{ marginRight: sidebarOpen ? sidebarWidth + 'px' : '0' }">
          <!-- Content placeholder -->
          <router-view/>
        </div>

        <!-- Sidebar -->
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
            <div>Item 1</div>
            <div>Item 2</div>
            <div>Item 3</div>
          </div>
          <!-- Resize handle -->
          <div
              class="absolute left-0 top-0 h-full w-2 hover:bg-gray-700 cursor-col-resize"
              @mousedown="startResizing"
          ></div>
        </div>
      </main>
    </div>

    <!-- Footer -->
    <footer class="bg-gray-900 text-white py-4 px-6 text-center">
      &copy; {{ currentYear }} My App
    </footer>
  </div>
</template>

<script setup>
import {ref} from 'vue'
import {RouterLink, createWebHistory} from "vue-router";
// Get the current year
const currentYear = ref(new Date().getFullYear())

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
</script>
