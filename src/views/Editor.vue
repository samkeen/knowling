<template>
  <div class="flex h-full">
    <div class="flex flex-col flex-1 overflow-hidden">
      <EditorToolbar/>
      <div class="flex-1 overflow-y-auto p-4">
        <template v-if="noteLoaded">
          <MilkdownEditorWrapper :initialValue="noteText" @update="noteText = $event"/>
          <button @click="save_note" type="submit" class="bg-blue-500 text-white py-2 px-4">Save</button>
          <button @click="delete_note" type="submit" class="bg-red-500 text-white py-2 px-4 ml-3">Delete</button>
        </template>
        <template v-else>
          <div class="text-center">Loading note...</div>
        </template>
      </div>
    </div>
    <Sidebar :note-id="noteId"/>
  </div>
</template>

<script setup>
/**
 * For Layout, see: https://claude.ai/chat/72a74a4e-b343-49f7-a76e-0eabaeb0c1d7
 */
import Sidebar from '../components/Sidebar.vue'
import EditorToolbar from "../components/EditorToolbar.vue";
import {useRoute} from 'vue-router';
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref} from "vue";
import MilkdownEditorWrapper from "../components/MilkdownEditorWrapper.vue";


let noteText = ref('');
let noteLoaded = ref(false);

const route = useRoute();
const noteId = route.params.id || null;
console.log("The note id: ", noteId);

async function save_note() {
  if (noteId) {
    try {
      let note = await invoke("save_note", {id: noteId, text: noteText.value})
      console.log("Note updated: ", note.id);
    } catch (error) {
      console.error("Failed saving note:", error);
      // Handle the error as needed, e.g., show a user-friendly message
    }
  } else {
    try {
      let note = await invoke("save_note", {id: noteId, text: noteText.value})
      console.log("Note created: ", note.id);
    } catch (error) {
      console.error("Failed saving note:", error);
      // Handle the error as needed, e.g., show a user-friendly message
    }
  }
}

async function delete_note() {
  if (noteId) {
    try {
      await invoke("delete_note", {id: noteId})
      console.log("Note deleted: ", noteId);
      // This doesn't work for some reason??
      // await router.push("/")
      // So do it old skool
      window.location.href = "/";
    } catch (error) {
      console.error("Failed deleting note:", error);
      // Handle the error as needed, e.g., show a user-friendly message
    }
  } else {
    console.log("The note is not defined: ", noteId);
  }
}

onMounted(async () => {
  if (noteId) {
    console.log("The note id: ", noteId);
    try {
      const note = await invoke("get_note_by_id", {id: noteId});
      noteText.value = note.text;
      noteLoaded.value = true;
    } catch (error) {
      console.error("Failed getting note:", error);
      // Handle the error as needed, e.g., show a user-friendly message
    }
  } else {
    console.log("The note id is not defined");
    noteLoaded.value = true;
  }
});


</script>