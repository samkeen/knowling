<template>
  <div class="flex items-center justify-between border-b p-2 sticky top-0 z-10">
    <div class="text-lg font-semibold">
      <RouterLink :to="{name: 'Home'}" class="">✗</RouterLink>
    </div>
    <div class="flex items-center space-x-2">
      <button class="">☆</button>
      <button @click="openQuestionDialog" class="">?</button>
      <div class="relative">
        <button @click="toggleMenu" class="">…</button>
        <div v-if="showMenu" class="absolute right-0 mt-2 py-2 w-48 rounded-md shadow-xl z-20">
          <a @click="handleDeleteNote" href="#"
             class="block px-4 py-2 text-sm">Delete</a>
        </div>
      </div>
    </div>
  </div>
  <div v-if="showQuestionDialog" class="fixed inset-0 flex items-center justify-center z-50">
    <div class="rounded-lg p-6 w-96">
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
import {ref} from 'vue';
import {RouterLink, useRoute, useRouter} from 'vue-router';
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

const route = useRoute();
const router = useRouter();
let noteId = ref(route.params.id || null);

function toggleMenu() {
  showMenu.value = !showMenu.value;
}

function handleDeleteNote() {
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