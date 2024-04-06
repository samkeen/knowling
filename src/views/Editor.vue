<template>
  <div class="flex h-full">
    <div class="flex flex-col flex-1 overflow-hidden">
      <EditorToolbar/>
      <div class="flex-1 overflow-y-auto p-4">
        <template v-if="noteLoaded">
          <MilkdownEditorWrapper :initialValue="noteText" @update="handleNoteUpdate"/>
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
 *
 * ## Autosave Functionality
 * 1. When the user types or modifies the content in the Milkdown editor, the update event is emitted, triggering the handleNoteUpdate function.
 * 2. Inside the handleNoteUpdate function, the noteText value is updated with the new content received from the event.
 * 3. The debouncedSaveNote function is called, which is a debounced version of the save_note function.
 * 4. The debounce function from lodash-es is used to create the debouncedSaveNote function. It takes two arguments: the function to be debounced (save_note) and the delay in milliseconds (autosaveDelay).
 * 5. The debounce function works as follows:
 *    - When debouncedSaveNote is called, it starts a timer with the specified delay (1000ms in this case).
 *    - If debouncedSaveNote is called again within the delay period, the previous timer is canceled, and a new timer is started.
 *    - If no further calls to debouncedSaveNote occur within the delay period, the save_note function is finally invoked.
 * 6. This means that the save_note function is not called immediately every time the user types. Instead, it waits for a pause in the user's typing activity. If the user continues typing within the specified delay (1000ms), the previous autosave is canceled, and a new autosave is scheduled.
 * 7. Once the user stops typing and the delay period passes without any further modifications, the save_note function is invoked, saving the note.
 */
import Sidebar from '../components/Sidebar.vue'
import EditorToolbar from "../components/EditorToolbar.vue";
import {useRoute} from 'vue-router';
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref} from "vue";
import {debounce} from 'lodash-es';
import MilkdownEditorWrapper from "../components/MilkdownEditorWrapper.vue";
import {upsertNote} from '../lib/notebook.js';
import {info} from "tauri-plugin-log-api";

let noteText = ref('');
let noteLoaded = ref(false);

const autosaveDelay = 1000; // Adjust the delay as needed (in milliseconds)

const route = useRoute();
let noteId = ref(route.params.id || null);

const debouncedSaveNote = debounce(async () => {
  await handleSaveNote();
}, autosaveDelay);

function handleNoteUpdate(event) {
  noteText.value = event;
  debouncedSaveNote();
}

async function handleSaveNote() {
  const savedNoteId = await upsertNote(noteId.value, noteText.value);
  if (!noteId.value && savedNoteId) {
    noteId.value = savedNoteId;
  }
}

onMounted(async () => {
  if (noteId.value) {
    info("The note id: ", noteId.value);
    try {
      const note = await invoke("get_note_by_id", {id: noteId.value});
      noteText.value = note.text;
      noteLoaded.value = true;
    } catch (error) {
      console.error("Failed getting note:", error);
      // Handle the error as needed, e.g., show a user-friendly message
    }
  } else {
    info("The note id is not defined");
    noteText.value = ''; // Initialize noteText.value with an empty string for new notes
    noteLoaded.value = true;
  }
});
</script>
