<template>
  <div class="flex flex-col h-screen">
    <AppHeader/>
    <div class="flex-1 overflow-y-auto">
      <router-view :key="route.path"></router-view>
    </div>
  </div>
</template>

<script setup>
/**
 * For Layout, see: https://claude.ai/chat/72a74a4e-b343-49f7-a76e-0eabaeb0c1d7
 *
 * ## Key Attribute in Router-View
 * <This was needed since Editor.vue is used for new and update>
 * When the key attribute is not used, Vue will use the component's constructor
 *  as the key. This can lead to unexpected behavior if the same component is used for multiple routes,
 *  as Vue may not be able to correctly identify which route view needs to be updated.
 */
import AppHeader from "./components/AppHeader.vue";
import {useRoute} from 'vue-router';

const route = useRoute();
</script>