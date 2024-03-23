<template>
  <div class="flex flex-col h-screen">
    <AppHeader/>
    <div class="flex-1 overflow-y-auto">
      <div class="p-4">
        <router-view/>
      </div>
    </div>
  </div>
</template>

<script setup>
/**
 * # Layout notes
 * - We wrap the entire content inside a div with class="flex flex-col h-screen". This sets up a flex container with
 *   a column direction and a height of the full viewport.
 * - We place the AppHeader component as the first child of the flex container. This ensures that it stays at the top.
 * - We wrap the router-view and its surrounding div inside another div with class="flex-1 overflow-y-auto".
 *   This gives the content area a flex-grow value of 1, allowing it to take up the remaining vertical space. The
 *   overflow-y-auto class enables vertical scrolling only for the content area.
 */
import AppHeader from "./components/AppHeader.vue";
</script>