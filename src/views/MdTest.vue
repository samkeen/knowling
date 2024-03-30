<script setup lang="ts">
import MilkdownEditorWrapper from '../components/MilkdownEditorWrapper.vue'

let markdownContent = 'Marked down content here';

function save() {
  console.log("Saving:", markdownContent);
}
</script>

<template>
  <MilkdownEditorWrapper :initialValue="markdownContent" @update="markdownContent = $event"/>
  <button @click="save">Save</button>
</template>

<style scoped></style>