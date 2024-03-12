<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

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
  <div v-for="note in notes" :key="note.id">
    <h2>id: {{ note.id }}</h2>
    <p>text: {{ note.text }}</p>
  </div>
  <p v-if="notes.length === 0">There are no notes</p>
</template>