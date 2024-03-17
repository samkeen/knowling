<template>
  <h1 class="text-xl font-bold underline">Notes List</h1>
  <p class="font-mono">Total number of notes: {{ notes.length }}</p>
  <ul>
    <li v-for="note in notes" :key="note.id">
      <p>
        <RouterLink :to="{ name: 'EditNote', params: { id: note.id } }">[{{ note.id }}]::{{ note.text }}</RouterLink>
      </p>
    </li>
  </ul>
  <p v-if="notes.length === 0">There are no notes</p>
</template>

<script setup>
import {ref, onMounted} from "vue";
import {RouterLink, createWebHistory} from "vue-router";
import {invoke} from "@tauri-apps/api/tauri";

let notes = ref([]);

async function get_notes() {
  let result = await invoke("get_notes");
  console.log("The all notes result: ", result);
  notes.value = result;
}

onMounted(get_notes);
</script>