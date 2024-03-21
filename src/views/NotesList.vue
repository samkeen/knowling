<template>
  <div class="mb-8">
    <h2 class="text-2xl font-bold mb-4">Notes</h2>
    <ul>

      <li v-for="note in notes" :key="note.id" class="mb-4 p-4 bg-white shadow">
        <RouterLink :to="{ name: 'EditNote', params: { id: note.id } }">
          <h3 class="font-bold">{{ note.id }}</h3>
          <p>{{ note.text }}</p>
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
  let result = await invoke("get_notes");
  console.log("The all notes result: ", result);
  notes.value = result;
}

onMounted(get_notes);


</script>