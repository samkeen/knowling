<script setup>
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import Editor from "./views/Editor.vue";
import {RouterLink, createWebHistory} from "vue-router";
</script>

<template>
  <nav>
    <RouterLink to="/" class="px-4 py-2 bg-blue-500 text-white rounded">Notes</RouterLink>
    <RouterLink to="note" class="px-4 py-2 bg-blue-500 text-white rounded">Editor</RouterLink>
  </nav>
  <div class="container">
    <router-view/>
  </div>
</template>

<style scoped>

</style>
