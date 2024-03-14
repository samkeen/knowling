<script setup>
import {marked} from 'marked'
import {debounce} from 'lodash-es'
import {ref, computed, onMounted} from 'vue'
import {useRoute} from 'vue-router';


// import { ref, onMounted } from "vue";
import {invoke} from "@tauri-apps/api/tauri";


let noteInput = ref('');

async function save_note() {
  let note = await invoke("save_note", {text: noteInput.value});
  console.log(note);
}

// onMounted(get_notes);
const route = useRoute();
const noteId = route.params.id;
console.log("The note id: ", noteId);

const input = ref('# hello')

const output = computed(() => marked(input.value))
const showOutput = ref(true)

const update = debounce((e) => {
  input.value = e.target.value
}, 100)
const toggleOutput = () => {
  showOutput.value = !showOutput.value
}

onMounted(async () => {
  const note = await invoke("get_note_by_id", {noteId: noteId});
  noteInput.value = note.text;
});
</script>

<template>
  <div class="flex justify-between items-center p-4">
    <button @click="toggleOutput" class="px-4 py-2 bg-blue-500 text-white rounded">{{ showOutput ? 'Hide' : 'Show' }}
      Output
    </button>
    <button @click="save_note" class="px-4 py-2 bg-green-500 text-white rounded">Save Note</button>
  </div>
  <div class="flex h-screen">
    <textarea v-model="noteInput"
              class="flex-grow h-full p-4 border-r border-gray-300 overflow-auto resize-none outline-none bg-gray-100 text-sm font-mono"></textarea>
    <!--    <textarea v-model="noteInput" class="flex-grow h-full p-4 border-r border-gray-300 overflow-auto resize-none outline-none bg-gray-100 text-sm font-mono" :value="input" @input="update"></textarea>-->
    <!--    <div class="flex-grow h-full p-4 overflow-auto" v-show="showOutput" v-html="output"></div>-->
  </div>
</template>

