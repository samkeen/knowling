<script setup>
import {TooltipProvider} from "@milkdown/plugin-tooltip";
import {toggleStrongCommand} from '@milkdown/preset-commonmark';
import {callCommand} from '@milkdown/utils';
import {useInstance} from '@milkdown/vue';
import {usePluginViewContext} from '@prosemirror-adapter/vue';
import {onMounted, onUnmounted, ref, watch} from 'vue';

const {view, prevState} = usePluginViewContext()
const [loading, get] = useInstance()

const divRef = ref();

let tooltipProvider;

onMounted(() => {
  tooltipProvider = new TooltipProvider({
    content: divRef.value,
  })

  tooltipProvider.update(view.value, prevState.value)
})

watch(
    [view, prevState],
    () => {
      tooltipProvider?.update(view.value, prevState.value)
    }
)

onUnmounted(() => {
  tooltipProvider.destroy()
})

const toggleBold = (e) => {
  if (loading.value) return;

  e.preventDefault()

  get().action(callCommand(toggleStrongCommand.key))
}

</script>

<template>
  <div ref="divRef">
    <button
        className="text-gray-600 bg-slate-200 px-2 py-1 rounded-lg hover:bg-slate-300 border hover:text-gray-900"
        @mousedown="toggleBold"
    >
      Bold
    </button>
  </div>
</template>
