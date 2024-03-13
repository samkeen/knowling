<script setup>
import {ref, onMounted} from "vue";
import {invoke} from "@tauri-apps/api/tauri";

let notes = ref([]);

async function get_notes() {
  let result = await invoke("get_notes");
  console.log("The all notes result: ", result);
  notes.value = result;
}

onMounted(get_notes);
</script>

<template>
  <h1 class="text-3xl font-bold underline">Notes List</h1>
  <p class="font-mono">Total number of notes: {{ notes.length }}</p>
  <ul>
    <li v-for="note in notes" :key="note.id">
      <p>[{{ note.id }}]::{{ note.text }}
        <router-link :to="`/edit/${note.id}`" class="px-4 py-2 bg-blue-500 text-white rounded">Edit</router-link>
      </p>
    </li>
  </ul>
  <p v-if="notes.length === 0">There are no notes</p>
</template>