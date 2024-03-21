<template>
  <div class="w-64 bg-gray-100 pl-4">
    <h2 class="text-xl font-bold mb-4">Collections</h2>
    <ul>
      <li v-for="note in relatedNotes" :key="note[0].id">
        {{ note[0].id }}
      </li>
    </ul>
  </div>
</template>

<script setup>
import {ref, defineProps, onMounted} from 'vue'
import {invoke} from "@tauri-apps/api/tauri";

const props = defineProps({
  noteId: {
    type: String,
    required: false
  }
});

// Define a reactive variable to store the related notes
const relatedNotes = ref([]);

// Define the function to get related notes
async function getRelatedNotes(noteId) {
  relatedNotes.value = await invoke("get_note_similarities", {id: noteId});
}

onMounted(() => {
  if (props.noteId) {
    console.log("Getting related notes for: ", props.noteId)
    getRelatedNotes(props.noteId);
  }
});
</script>