<template>
  <div class="mb-8 ml-8 mt-2">
    <!--    <h2 class="text-2xl font-bold mb-4">Notes</h2>-->
    <div v-for="(group, groupName) in groupedNotes" :key="groupName">
      <h3 class="text-xl font-bold mb-2">{{ groupName }}</h3>
      <ul>
        <li v-for="note in group" :key="note.id" class="p-2">
          <RouterLink :to="{ name: 'EditNote', params: { id: note.id } }">
            <p class="truncate ...">{{ noteTitle(note.text) }}</p>
          </RouterLink>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup>
import {computed, onMounted, ref} from 'vue';
import {invoke} from "@tauri-apps/api/tauri";
import {RouterLink} from "vue-router";
import {noteTitle} from "../lib/notebook.js";
import {info} from "tauri-plugin-log-api";
import {groupNotesByDate} from '../lib/DateUtils.js';

const notes = ref([]);

async function get_notes() {
  try {
    let result = await invoke("get_notes");
    info("The all notes result: ", result);
    notes.value = result;
  } catch (error) {
    console.error("Failed getting notes:", error);
    // Handle the error as needed, e.g., show a user-friendly message
  }
}

const groupedNotes = computed(() => {
  return groupNotesByDate(notes.value);
});

onMounted(get_notes);
</script>