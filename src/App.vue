<template>
  <AppHeader/>
  <div class="flex h-screen">
    <div class="flex-1 p-4">
      <router-view/>
    </div>
  </div>
</template>

<script setup>
import AppHeader from "./components/AppHeader.vue";
</script>