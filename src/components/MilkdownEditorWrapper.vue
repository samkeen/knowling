<template>
  <div class="root">
    <MilkdownProvider>
      <MilkdownEditor :initialValue="props.initialValue" @update="emit('update', $event)"/>
    </MilkdownProvider>
  </div>
</template>

<script setup>
import {MilkdownProvider} from '@milkdown/vue'
import MilkdownEditor from './MilkdownEditor.vue'

const emit = defineEmits(['update']);
const props = defineProps({
  initialValue: String
});
</script>

