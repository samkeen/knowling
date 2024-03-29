<script setup lang="ts">
import MilkdownEditorWrapper from '../components/MilkdownEditorWrapper.vue'

const initialValue = 'Your initial text here';
let markdownContent = '';

function save() {
  console.log("Saving:", markdownContent);
}
</script>

<template>
  <MilkdownEditorWrapper :initialValue="initialValue" @update="markdownContent = $event"/>
  <button @click="save">Save</button>
</template>

<style scoped></style>