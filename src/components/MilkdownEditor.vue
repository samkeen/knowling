<template>
  <Milkdown
      class="editor"
      :class="isEditorFocus || editorContent ? 'active' : ''"
  />
</template>

<script setup>
import {Milkdown, useEditor} from '@milkdown/vue';
import {defaultValueCtx, Editor, editorViewOptionsCtx, rootCtx} from '@milkdown/core';
import {nord} from '@milkdown/theme-nord'
import {commonmark} from '@milkdown/preset-commonmark'
import {useNodeViewFactory, usePluginViewFactory, useWidgetViewFactory} from '@prosemirror-adapter/vue';
import {tooltipFactory} from '@milkdown/plugin-tooltip';
import Tooltip from './Tooltip.vue';
import {ref} from "vue";
import {info} from "tauri-plugin-log-api";
import {listener, listenerCtx} from '@milkdown/plugin-listener'
import {trailing} from "@milkdown/plugin-trailing";
import {clipboard} from "@milkdown/plugin-clipboard";
// import Size from './Size.vue';
// import HeadingAnchor from './HeadingAnchor.vue';
// import Blockquote from './Blockquote.vue';

const tooltip = tooltipFactory('Text');
const emit = defineEmits(['update']);
const props = defineProps({
  initialValue: String,
  readonly: Boolean,
});
const readonly = ref(props.readonly);
info(`READ_ONLY: , ${readonly.value}`)
const editable = () => !readonly.value;
const initEditorContent = ref(props.initialValue);
const pluginViewFactory = usePluginViewFactory();
const isEditorFocus = ref(false)
const editorContent = ref('')

const nodeViewFactory = useNodeViewFactory();
const widgetViewFactory = useWidgetViewFactory()


useEditor((root) => {
  return Editor.make()
      .config(nord)
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
      })

      .use(nord)
      .use(commonmark)
      .use(listener)
      .use(trailing)
      .use(clipboard)
      .use(tooltip)
  // // Add custom node view
  // Pull these from Local::Milkdown Examples
  // .use($view(blockquoteSchema.node, () => nodeViewFactory({
  //   component: Blockquote
  // })))
  // // Add custom plugin view
  // .use($prose(() => new Plugin({
  //   view: pluginViewFactory({
  //     component: Size
  //   })
  // })))
  // .use($prose((ctx) => {
  //   const getAnchorWidget = widgetViewFactory({
  //     as: 'span',
  //     component: HeadingAnchor
  //   })
  //   return new Plugin({
  //     props: {
  //       decorations: (state) => {
  //         const widgets: Decoration[] = []
  //
  //         state.doc.descendants((node, pos) => {
  //           if (node.type === headingSchema.type(ctx)) {
  //             widgets.push(getAnchorWidget(pos + 1, {
  //               id: node.attrs.id,
  //               level: node.attrs.level,
  //               side: -1,
  //             }))
  //           }
  //         })
  //         return DecorationSet.create(state.doc, widgets);
  //       }
  //     }
  //   })
  // }))
})
</script>
