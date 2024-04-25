<template>
  <Milkdown
      class="editor"
      :class="isEditorFocus || editorContent ? 'active' : ''"
  />
</template>

<script setup>
/**
 * This was my original attempt to create a Vue 3 component using Milkdown.
 * Holding here, unused for now
 * original inspiration: https://github.com/Yumamama00/vue3-milkdown-sample/tree/main
 */

import {ref} from 'vue'
// Milkdown core
import {defaultValueCtx, Editor, editorViewOptionsCtx, rootCtx} from '@milkdown/core'
import {useEditor} from '@milkdown/vue'
import {commonmark} from '@milkdown/preset-commonmark'
import {nord} from "@milkdown/theme-nord";
// import {gfm} from '@milkdown/preset-gfm'
// // Milkdown Plugins
// import {history} from '@milkdown/plugin-history'
// import {prism, prismConfig} from '@milkdown/plugin-prism'
import {listener} from '@milkdown/plugin-listener'
import {clipboard} from '@milkdown/plugin-clipboard';
// import {indent} from '@milkdown/plugin-indent'
import {trailing} from '@milkdown/plugin-trailing'
import {tooltipFactory} from '@milkdown/plugin-tooltip'
// // KUN Visual Novel Custom tooltip
// import Tooltip from './plugins/Tooltip.vue'
// // Custom text size calculate
// import Size from './plugins/Size.vue'
// import {$prose} from '@milkdown/utils'
// import {Plugin} from '@milkdown/prose/state'
// KUN Visual Novel style
// import '@/styles/editor/index.scss'
// Syntax highlight
// import c from 'refractor/lang/c'
// import cpp from 'refractor/lang/cpp'
// import csharp from 'refractor/lang/csharp'
// import css from 'refractor/lang/css'
// import go from 'refractor/lang/go'
// import haskell from 'refractor/lang/haskell'
// import python from 'refractor/lang/python'
// import java from 'refractor/lang/java'
// import javascript from 'refractor/lang/javascript'
// import typescript from 'refractor/lang/typescript'
// import jsx from 'refractor/lang/jsx'
// import kotlin from 'refractor/lang/kotlin'
// import r from 'refractor/lang/r'
// import rust from 'refractor/lang/rust'
// import scala from 'refractor/lang/scala'
// import sql from 'refractor/lang/sql'
// import tsx from 'refractor/lang/tsx'
// import markdown from 'refractor/lang/markdown'
import {info} from "tauri-plugin-log-api";
import {usePluginViewFactory} from '@prosemirror-adapter/vue';
import Tooltip from './Tooltip.vue';

const tooltip = tooltipFactory('Text');

// Editor markdown preset
// const value = ref('')
const emit = defineEmits(['update']);
const props = defineProps({
  initialValue: String,
  readonly: Boolean,
});
const readonly = ref(props.readonly);
console.log("READ_ONLY: ", readonly.value)
const editable = () => !readonly.value;
const initEditorContent = ref(props.initialValue);

// const tooltip = tooltipFactory('Text')
const pluginViewFactory = usePluginViewFactory()
const container = ref(null)
const isEditorFocus = ref(false)
const editorContent = ref('')

useEditor((root) =>
        Editor.make()
            .config((ctx) => {
              ctx.set(rootCtx, root)
              ctx.update(editorViewOptionsCtx, (prev) => ({
                ...prev,
                editable,
              }));
              ctx.set(defaultValueCtx, initEditorContent.value)
              ctx.set(tooltip.key, {
                view: pluginViewFactory({
                  component: Tooltip
                }),
              })

              const listener = ctx.get(listenerCtx)
              listener.markdownUpdated((ctx, markdown, prevMarkdown) => {
                if (markdown !== prevMarkdown) {
                  info("[updated]: ", markdown)
                  emit('update', markdown);
                  editorContent.value = markdown
                }
              })
              listener.blur(() => {
                info("[blur]")
                isEditorFocus.value = false
              })
              listener.focus(() => {
                info("[focus]")
                isEditorFocus.value = true
              })

              // ctx.set(prismConfig.key, {
              //   configureRefractor: (refractor) => {
              //     refractor.register(c)
              //     refractor.register(cpp)
              //     refractor.register(csharp)
              //     refractor.register(css)
              //     refractor.register(go)
              //     refractor.register(haskell)
              //     refractor.register(python)
              //     refractor.register(markdown)
              //     refractor.register(java)
              //     refractor.register(javascript)
              //     refractor.register(typescript)
              //     refractor.register(jsx)
              //     refractor.register(kotlin)
              //     refractor.register(r)
              //     refractor.register(rust)
              //     refractor.register(scala)
              //     refractor.register(sql)
              //     refractor.register(tsx)
              //   },
              // })
            })
            .use(nord)
            .use(commonmark)
            .use(listener)
            .use(trailing)
            .use(clipboard)
            .use(tooltip)
    // .use(history)
    // .use(gfm)
    // .use(prism)
    // .use(clipboard)
    // .use(indent)
)
</script>
