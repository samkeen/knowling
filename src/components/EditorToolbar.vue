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
      <!--      <button class="btn btn-circle btn-xs tooltip tooltip-bottom tooltip-info" data-tip="Add to Favorites">☆</button>-->
      <LlmPrompting :note-id="noteId"/>
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
</template>

<script setup>
import {onMounted, ref} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {deleteNote} from '../lib/notebook.js';
import {invoke} from "@tauri-apps/api/tauri";
import {info} from "tauri-plugin-log-api";
import LlmPrompting from './LlmPrompting.vue';

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

function handleDeleteNote() {
  info(`Deleting note: ${noteId.value}`)
  deleteNote(noteId.value, router);
}
</script>