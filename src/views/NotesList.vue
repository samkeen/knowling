<template>
  <div class="mb-8">
    <h2 class="text-2xl font-bold mb-4">Notes</h2>
    <ul>
      <li v-for="note in notes" :key="note.id" class="mb-4 p-4 bg-white">
        <RouterLink :to="{ name: 'EditNote', params: { id: note.id } }">
          <p class="truncate ...">{{ noteTitle(note.text) }}</p>
          <p>Created: {{ formatDate(note.created) }}</p>
          <p>Modified: {{ formatDate(note.modified) }}</p>
        </RouterLink>
      </li>
    </ul>
  </div>
</template>

<script setup>
import {onMounted, ref} from 'vue'
import {invoke} from "@tauri-apps/api/tauri";
import {RouterLink, createWebHistory} from "vue-router";

const notes = ref([]);

async function get_notes() {
  try {
    let result = await invoke("get_notes");
    console.log("The all notes result: ", result);
    notes.value = result;
  } catch (error) {
    console.error("Failed getting notes:", error);
    // Handle the error as needed, e.g., show a user-friendly message
  }
}

function noteTitle(text) {
  const lines = text.split('\n');
  const firstLine = lines[0];
  // remove any leading '#' or spaces
  return firstLine.replace(/^#+\s*/, '');
}

function formatDate(timestamp) {
  return new Date(timestamp * 1000).toLocaleString();
}

onMounted(get_notes);


</script>