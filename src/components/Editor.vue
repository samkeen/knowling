<script setup>
import { marked } from 'marked'
import { debounce } from 'lodash-es'
import { ref, computed } from 'vue'

const input = ref('# hello')

const output = computed(() => marked(input.value))
const showOutput = ref(true)

const update = debounce((e) => {
  input.value = e.target.value
}, 100)
const toggleOutput = () => {
  showOutput.value = !showOutput.value
}
</script>

<template>
  <div class="controls">
    <button @click="toggleOutput">{{ showOutput ? 'Hide' : 'Show' }} Output</button>
  </div>
  <div class="editor">
    <textarea class="input" :value="input" @input="update" :style="{flex: showOutput ? '1' : '2'}"></textarea>
    <div class="output" v-show="showOutput" v-html="output" :style="{flex: showOutput ? '1' : '0'}"></div>
  </div>
</template>

<style>
body {
  margin: 0;
}

.editor {
  height: 100vh;
  display: flex;
}

.input,
.output {
  overflow: auto;
  width: 50%;
  height: 100%;
  box-sizing: border-box;
  padding: 0 20px;
}

.input {
  border: none;
  border-right: 1px solid #ccc;
  resize: none;
  outline: none;
  background-color: #f6f6f6;
  font-size: 14px;
  font-family: 'Monaco', courier, monospace;
  padding: 20px;
}

.output {
  text-align: left;
}


code {
  color: #f66;
}
</style>