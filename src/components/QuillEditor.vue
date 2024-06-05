<template>
  <!-- Editor container -->
  <div
      name=""
      ref="textEditor"
      autofocus="autofocus"
      class="flex-1 w-full code-editor"
      id="editor"
  ></div>
</template>

<script setup>
/**
 * responsible for rendering the markdown editor using the Quill library. It converts
 * the initial Markdown code to Quill delta format and sets it as the initial content
 * of the editor. It listens for text changes in the editor and emits the change
 * event with the updated Markdown code. The component also handles focusing the editor
 * when the container is clicked.
 * @TODO
 * - [ ] Investigate the focus code
 * - [ ] leverage on change/delta mechanism
 * - [ ] get loaded note text into editor
 * - [ ] bring back editor toolbar
 * - [ ] bring back sidebar
 */

// Import CSS styles for Quill and Quill Markdown
import "quill/dist/quill.bubble.css";
import "quilljs-markdown/dist/quilljs-markdown-common-style.css";

// Import Quill and Quill Markdown libraries
import Quill from "quill";
import QuillMarkdown from "quilljs-markdown";
// Import necessary composition API functions from Vue
import {onMounted, ref, watch} from "vue";
// Import utility functions for markdown conversion
import {deltaToMarkdown} from "quill-delta-to-markdown";
import {MarkdownToQuill} from "md-to-quill-delta";

let initContentLoaded = false;

// Define the component name
const name = "EditorRich";

// Define emitted events
const emit = defineEmits(["change"]);

// Define component props
const props = defineProps({
  initialDocContent: String,
});

// Create a ref for the text editor
const textEditor = ref(null);
let quill;

// This is the mechanism to ensure we get the initial content loaded once the
// reactive var initialDocContent is populated.
watch(
    () => props.initialDocContent,
    (newContent) => {
      if (quill && newContent && !initContentLoaded) {
        initContentLoaded = true;
        console.log("[DEBUG] WATCH newContent: ", newContent)
        // use `quill.setText(newContent);` to render raw markdown
        // quill.setText(newContent);
        // Convert the new content to Quill delta and set it as the editor's content
        const converter = new MarkdownToQuill({});
        const ops = converter.convert(newContent);
        quill.setContents(ops);
      }
    },
    {immediate: true} // Trigger the watcher immediately
);

onMounted(() => {
  // Initialize Quill editor
  const quillOptions = {
    debug: 'warn',
    modules: {
      // toolbar: true,
    },
    // https://quilljs.com/docs/configuration#bounds
    // bounds: '#editor',
    placeholder: 'Compose an epic...',
    theme: 'bubble'
  };
  quill = new Quill("#editor", quillOptions);

  // Enable markdown conversion for Quill
  new QuillMarkdown(quill, {});

  // Listen for text changes in the Quill editor
  quill.on("text-change", () => {
    // Convert the Quill delta to Markdown
    const markdownCode = deltaToMarkdown(quill.getContents().ops);
    // Emit the 'change' event with the Markdown code
    emit("change", markdownCode);
  });

  // Focus the editor when the container is clicked
  if (textEditor.value) {
    const editor = textEditor.value.querySelector(".ql-editor");
    textEditor.value.addEventListener("click", (e) => {
      console.log("CLICK ON: ", e.target)
      console.log("CLICK ON ID: ", e.target.id) // TODO FIX: `e.target.id == ""`
      if (e.target.id !== "editor") {
        return;
      }
      editor.focus();
    });
  }
});
</script>