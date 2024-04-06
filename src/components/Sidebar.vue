<template>
  <div class="w-72 bg-gray-100 pl-4">
    <h2 class="text-xl font-bold mb-4">Related</h2>
    <ul>
      <li v-for="result in relatedNotes" :key="result.note.id">
        <RouterLink :to="{name: 'EditNote', params: {id:result.note.id}}">{{ noteTitle(result.note.text) }}</RouterLink>
      </li>
    </ul>
  </div>
</template>

<script setup>
import {RouterLink} from 'vue-router'
import {ref, defineProps, onMounted} from 'vue';
import {noteTitle, getRelatedNotes} from '../lib/notebook.js';
import {info} from "tauri-plugin-log-api";

const props = defineProps({
  noteId: {
    type: String,
    required: false,
  },
});

// Define a reactive variable to store the related notes
const relatedNotes = ref([]);

async function fetchRelatedNotes(noteId) {
  relatedNotes.value = await getRelatedNotes(noteId);
  info('Related notes:', relatedNotes.value);
}

onMounted(() => {
  if (props.noteId) {
    info('Getting related notes for:', props.noteId);
    fetchRelatedNotes(props.noteId);
  }
});
</script>