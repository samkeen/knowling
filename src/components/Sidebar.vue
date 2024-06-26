<template>
  <div class="w-72 pl-4">
    <h2 class="text-xl font-bold mb-4">Related</h2>
    <ul>
      <li v-for="result in relatedNotes" :key="result.note.id">
        <div class="tooltip tooltip-info" :data-tip="result.similarityScore">
          <RouterLink :to="{name: 'EditNote', params: {id:result.note.id}}">{{
              noteTitle(result.note.text)
            }}
          </RouterLink>
        </div>
      </li>
    </ul>
  </div>
</template>

<script setup>
import {RouterLink} from 'vue-router'
import {defineProps, onMounted, ref} from 'vue';
import {getRelatedNotes, noteTitle} from '../lib/notebook.js';
import {debug, info} from "tauri-plugin-log-api";

const props = defineProps({
  noteId: {
    type: String,
    required: false,
  },
});

// Define a reactive variable to store the related notes
const relatedNotes = ref([]);

async function fetchRelatedNotes(noteId) {
  let storedThreshold = parseFloat(localStorage.getItem("similarityScoreThreshold"));
  if (!storedThreshold) {
    debug('No stored similarity score threshold found. Using default value: 0.30');
    storedThreshold = 0.30;
  }
  info(`using similarity score threshold: ${storedThreshold}`);
  relatedNotes.value = await getRelatedNotes(noteId, storedThreshold);
  debug(`Related notes: ${relatedNotes.value}`);
}

onMounted(() => {
  info(`Received note id: ${props.noteId}`);
  if (props.noteId) {
    info(`Getting related notes for: ${props.noteId}`);
    fetchRelatedNotes(props.noteId);
  }
});
</script>
