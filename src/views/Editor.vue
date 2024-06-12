<template>
  <div class="flex h-full">
    <div class="flex flex-col flex-1 overflow-hidden">
      <EditorToolbar/>
      <div class="flex-1 overflow-y-auto p-4">
        <div class="root">
          <Editor
              v-on:change="debouncedHandleChange"
              v-bind:initialDocContent="docState.noteObj ? docState.noteObj.text : ''"
          ></Editor>
        </div>
      </div>
    </div>
    <Sidebar :note-id="noteId"/>
  </div>
</template>

<script setup>
/**
 * Handles the application state, including the Markdown code and state management.
 * It also listens for the change event emitted by the Editor component to update
 * the code in the state and save it to local storage.
 * [ ] TODO Utilize keydown event listener
 */
import Editor from "../components/QuillEditor.vue";
import EditorToolbar from "../components/EditorToolbar.vue";
import Sidebar from "../components/Sidebar.vue";
// Import the necessary composition API functions from Vue
import {onMounted, onUnmounted, reactive, ref} from "vue";
import {useRoute} from "vue-router";
import {debug, info} from "tauri-plugin-log-api";
import {invoke} from "@tauri-apps/api/tauri";
import {upsertNote} from "../lib/notebook.js";
import {debounce} from "lodash-es";

const route = useRoute();
let noteId = ref(route.params.id || null);

// Function to get the default code
const loadDocContent = async (noteId) => {
  let noteObj = null;
  if (noteId.value) {
    info(`The note id: ${noteId.value}`);
    try {
      noteObj = await invoke("get_note_by_id", {id: noteId.value});
      console.log("Note loaded:", noteObj);
    } catch (error) {
      console.error("Failed getting note:", error);
    }
  } else {
    info("The note id is not defined");
  }
  return noteObj;
};

const docState = reactive({
  noteId: noteId,
  copied: false,
  /**
   * id: String,
   * text: String,
   * categories: HashSet<Category>,
   * created: i64,
   * modified: i64,
   */
  noteObj: null, // Initialize noteObj as null
});

// Lifecycle hook: Mounted
onMounted(async () => {
  // Load the note content asynchronously
  const noteObj = await loadDocContent(noteId);
  docState.noteObj = noteObj;
  // Add a keydown event listener to the document
  document.addEventListener("keydown", shortcutListener.bind(this));
});

// Lifecycle hook: Unmounted
onUnmounted(() => {
  // Remove the keydown event listener from the document
  document.removeEventListener("keydown", shortcutListener);
});

// Event handler for keydown events
function shortcutListener(e) {
  console.log("shortcut listener", e);
}

// Create a debounced version of the handleChange function
const debouncedHandleChange = debounce(handleChange, 1500);

// Event handler for the 'change' event emitted by the Editor component
async function handleChange(docContent) {
  // docState.noteObj.text = docContent;
  debug(`HANDLED CHANGE: ${docContent}`);
  info(`Saving note with id: ${noteId.value}`);
  const savedNoteId = await upsertNote(noteId.value, docContent);
  // If new note, populate the id
  if (!noteId.value && savedNoteId) {
    noteId.value = savedNoteId;
  }
}
</script>