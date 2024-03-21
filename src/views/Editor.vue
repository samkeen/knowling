<template>
  <div class="flex h-screen">
    <div class="flex-1 p-4">
      <form @submit.prevent="saveNote">
        <textarea v-model="noteText" class="w-full mb-4 p-2 border h-40" placeholder="Note text"></textarea>
        <button @click="save_note" type="submit" class="bg-blue-500 text-white py-2 px-4">Save</button>
      </form>
    </div>
    <Sidebar :note-id="noteId"/>
  </div>
</template>
<script setup>
import Sidebar from '../components/Sidebar.vue'
import {useRoute} from 'vue-router';
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref} from "vue";

let noteText = ref('');

const route = useRoute();
const noteId = route.params.id || null;
console.log("The note id: ", noteId);

async function save_note() {
  if (noteId) {
    let note = await invoke("save_note", {id: noteId, text: noteText.value})
    console.log("Note updated: ", note.id);
  } else {
    let note = await invoke("save_note", {id: noteId, text: noteText.value})
    // let note = await invoke("new_note", {text: noteText.value})
    console.log("Note created: ", note.id);
  }
}

onMounted(async () => {
  if (noteId) {
    console.log("The note id: ", noteId);
    const note = await invoke("get_note_by_id", {id: noteId});
    noteText.value = note.text;
  } else {
    console.log("The note id is not defined");
  }
});


</script>