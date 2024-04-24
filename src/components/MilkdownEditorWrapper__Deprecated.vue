<template>
  <div class="root">
    <MilkdownProvider>
      <MilkdownEditor :initialValue="props.initialValue" :readonly="props.readonly" @update="emit('update', $event)"/>
    </MilkdownProvider>
  </div>
</template>

<script setup>
import {defineEmits, defineProps} from 'vue'
import {MilkdownProvider} from '@milkdown/vue'
import MilkdownEditor from './MilkdownEditor.vue'

const emit = defineEmits(['update']);
const props = defineProps({
  initialValue: String,
  readonly: Boolean
});
</script>

