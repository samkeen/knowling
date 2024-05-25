<template>
  <div class="flex items-center justify-between border-b p-2 sticky top-0 z-10">
    <div class="text-lg font-semibold">
      <RouterLink :to="{name: 'Home'}" class="">✗</RouterLink>
    </div>
    <div class="flex items-center space-x-2">
      <button @click="openAddCatModal" class="btn btn-circle btn-xs tooltip tooltip-bottom tooltip-info"
              data-tip="Add Category">+
      </button>
      <div v-if="showModal" class="fixed inset-0 flex items-center justify-center z-50">
        <div class="modal-content bg-base-100 rounded-md shadow-xl p-2">
          <input v-model="newCategory" type="text" placeholder="add category" class="input input-bordered w-full mb-4"/>
          <div class="flex justify-end space-x-2">
            <button @click="handleAddCategory" class="btn btn-sm">Save</button>
            <button @click="closeAddCatModal" class="btn btn-ghost btn-sm">Cancel</button>
          </div>
        </div>
      </div>
      <button class="btn btn-circle btn-xs tooltip tooltip-bottom tooltip-info" data-tip="Add to Favorites">☆</button>
      <button @click="openQuestionDialog" class="btn btn-circle btn-xs tooltip tooltip-bottom tooltip-info"
              data-tip="Ask AI">?
      </button>
      <button @click="handleDeleteNote" class="btn btn-circle btn-xs tooltip tooltip-error tooltip-left"
              data-tip="Delete Note">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd"
                d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z"
                clip-rule="evenodd"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="flex items-center justify-between border-b p-2 sticky top-0">
    <div class="flex space-x-2">
      <span v-for="category in categories" :key="category.id" class="badge badge-ghost flex items-center">
        {{ category.label }}
        <button @click="handleRemoveCategory(category)" class="ml-1 text-xs font-bold">×</button>
      </span>
    </div>
  </div>

  <div v-if="showQuestionDialog" class="fixed inset-0 flex items-center justify-center z-50">
    <div class="rounded-lg p-6 w-96 bg-base-100 border border-base-300">
    <textarea v-model="question" class="w-full h-32 p-2 border rounded"
              placeholder="Enter your question about this note"></textarea>
      <div class="mt-4 flex justify-end">
        <button @click="submitQuestion" class="btn px-4 py-2 rounded" :disabled="isLoading">
          <div v-if="isLoading" class="inline-block">
            <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none"
                 viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          </div>
          <span v-else>Submit</span>
        </button>
        <button @click="closeQuestionDialog" class="btn ml-2 px-4 py-2 rounded">Cancel</button>
      </div>
    </div>
  </div>
  <div v-if="showResponseDialog" class="fixed inset-0 flex items-center justify-center z-50"
       @click.self="closeResponseDialog">
    <div class="p-6 w-2/3 max-h-screen shadow-xl bg-base-100 border rounded">
      <div class="flex justify-between items-center mb-4">
        <h2 class="text-xl font-semibold">Response</h2>
        <div class="flex space-x-2">
          <button @click="copyToClipboard" class="btn btn-sm">
            <span class="sr-only">Copy to Clipboard</span>
            <svg :class="['h-6 w-6 transform duration-300', { 'scale-75': copyClicked }]"
                 xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3"/>
            </svg>
          </button>
          <button @click="closeResponseDialog" class="btn btn-square btn-sm">
            <span class="sr-only">Close</span>
            <svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"
                 stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>
      </div>
      <div class="mb-4 overflow-y-auto max-h-96">
        <div class="card">
          <div class="card-body">
            <MilkdownProvider>
              <ProsemirrorAdapterProvider>
                <MilkdownEditor :initialValue="response" :readonly="true"/>
              </ProsemirrorAdapterProvider>
            </MilkdownProvider>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import {onMounted, ref} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {deleteNote} from '../lib/notebook.js';
import {invoke} from "@tauri-apps/api/tauri";
import {error, info} from "tauri-plugin-log-api";
import {MilkdownProvider} from "@milkdown/vue";
import {ProsemirrorAdapterProvider} from '@prosemirror-adapter/vue';
import MilkdownEditor from "../components/MilkdownEditor.vue";


const showMenu = ref(false);
const showQuestionDialog = ref(false);
const question = ref('');
const showResponseDialog = ref(false);
const response = ref('');
const copyClicked = ref(false);
const isLoading = ref(false);
let showModal = ref(false);
let newCategory = ref('');

const route = useRoute();
const router = useRouter();
let noteId = ref(route.params.id || null);
const categories = ref([]);

onMounted(async () => {
  try {
    let note = await invoke("get_note_by_id", {id: noteId.value});
    categories.value = note.categories || [];
  } catch (error) {
    info(`Failed to load categories for note: ${error}`);
    // Handle the error as needed, e.g., show a user-friendly message
  }
});

const openAddCatModal = () => {
  showModal.value = true;
};

const closeAddCatModal = () => {
  newCategory.value = '';
  showModal.value = false;
};

async function handleAddCategory() {
  // Code to process newCategory
  try {
    console.log("Adding category")
    let updatedNote = await invoke("add_category_to_note", {
      noteId: noteId.value,
      categoryLabel: newCategory.value
    });
    info(`Added cat[${newCategory.value}] to note`);
    categories.value = updatedNote.categories || [];
    newCategory.value = '';
    showModal.value = false;
  } catch (error) {
    info(`Adding cat[${newCategory.value}] to note failed: ${error}`);
    // Handle the error as needed, e.g., show a user-friendly message
  }
}

async function handleRemoveCategory(category) {
  try {
    console.log(`Removing category: ${category.label}`);
    let updatedNote = await invoke("remove_category_from_note", {
      noteId: noteId.value,
      categoryId: category.id
    });
    info(`Removed cat[${category.label}] from note`);
    categories.value = updatedNote.categories || []; // Update the categories array
  } catch (error) {
    info(`Removing cat[${category.label}] from note failed: ${error}`);
    // Handle the error as needed, e.g., show a user-friendly message
  }
}

function toggleMenu() {
  showMenu.value = !showMenu.value;
}

function handleDeleteNote() {
  info(`Deleting note: ${noteId.value}`)
  deleteNote(noteId.value, router);
}

function openQuestionDialog() {
  showQuestionDialog.value = true;
}

function closeQuestionDialog() {
  info(`Closing question dialog...`);
  showQuestionDialog.value = false;
  question.value = '';
}

async function submitQuestion() {
  isLoading.value = true;
  await processQuestion(question.value);
  info(`Completed: processQuestion`);
  closeQuestionDialog();
  info(`Setting isLoading to: false`);
  isLoading.value = false;
}

async function processQuestion(questionText) {
  try {
    info("Sending question to LLM...");
    let responseText = await invoke("prompt_about_note", {prompt: questionText, noteId: noteId.value});
    if (responseText === null || responseText === undefined) {
      throw new Error('Response from invoke is null or undefined');
    }
    info(`LLM response: ${responseText}`);
    response.value = responseText;
    info(`Setting showResponseDialog to: true`);
    showResponseDialog.value = true;
  } catch (err) {
    error(`Failed prompting LLM: ${err}`);
  }
  info(`Processing question: ${questionText}`);
}

function closeResponseDialog() {
  showResponseDialog.value = false;
  response.value = '';
}

function copyToClipboard() {
  navigator.clipboard.writeText(response.value)
      .then(() => {
        console.log('Response copied to clipboard');
        copyClicked.value = true;
        setTimeout(() => {
          copyClicked.value = false;
        }, 200);
      })
      .catch((err) => {
        error(`Failed to copy response: ${err}`);
      });
}
</script>