<template>
  <div class="flex items-center justify-between bg-white border-b p-2 sticky top-0 z-10">
    <div class="text-lg font-semibold">✗</div>
    <div class="flex items-center space-x-2">
      <button class="text-gray-600 hover:text-yellow-500">☆</button>
      <div class="relative">
        <button @click="toggleMenu" class="text-gray-600 hover:text-yellow-500">…</button>
        <div v-if="showMenu" class="absolute right-0 mt-2 py-2 w-48 bg-white rounded-md shadow-xl z-20">
          <a @click="handleDeleteNote" href="#"
             class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100">Delete</a>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import {ref} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {deleteNote} from '../lib/notebook.js';

const showMenu = ref(false);
const route = useRoute();
const router = useRouter();
let noteId = ref(route.params.id || null);

function toggleMenu() {
  showMenu.value = !showMenu.value;
}

function handleDeleteNote() {
  deleteNote(noteId.value, router);
}
</script>