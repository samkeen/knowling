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
                @click="toggleSidebar"
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
        <SideBar ref="sidebarRef"/>
      </main>
    </div>

    <!-- Footer -->
    <footer class="bg-gray-900 text-white py-4 px-6 text-center">
      &copy; {{ currentYear }} My App
    </footer>
  </div>
</template>

<script setup>
import SideBar from './components/SideBar.vue';
import {ref} from "vue";

// Get the current year
const currentYear = ref(new Date().getFullYear())

// Sidebar state
const sidebarOpen = ref(false)
const sidebarWidth = ref(256)

// Sidebar reference
const sidebarRef = ref(null);

// Toggle sidebar
const toggleSidebar = () => {
  sidebarRef.value.toggleSidebar();
}
</script>
