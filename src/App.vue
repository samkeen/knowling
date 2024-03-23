<template>
  <div class="flex flex-col h-screen">
    <AppHeader/>
    <div class="flex-1 overflow-y-auto">
      <router-view/>
    </div>
  </div>
</template>

<script setup>
/**
 * For Layout, see: https://claude.ai/chat/72a74a4e-b343-49f7-a76e-0eabaeb0c1d7
 */
import AppHeader from "./components/AppHeader.vue";
</script>