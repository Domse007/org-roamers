<script setup lang="ts">
import hljs from 'highlight.js';
import { nextTick, ref, useTemplateRef, watch, type Ref } from "vue";
import renderMathInElement from "katex/contrib/auto-render";

const props = defineProps<{ id: string }>();
const shown: Ref<"none" | "flex"> = ref("none");

const rendered = ref("");
let current_id: string = "";
const preview_ref = useTemplateRef('preview-ref');

const preview = (id: string) => {
  current_id = id;
  console.log(`Previewing ${id}`);
  fetch(`/org?title=${id}`)
    .then((response) => {
      return response.text();
    })
    .then((html) => {
      rendered.value = html;
      expand();
    });
};

const expand = () => {
  shown.value = "flex";
};
const collapse = () => {
  shown.value = "none"
};

const resize = () => {
  if (rendered.value.length == 0) { return; }
  if (shown.value == "none") {
    expand();
  } else {
    collapse();
  }
};

const katexOptions = {
  delimiters: [
    { left: "$$", right: "$$", display: true },
    { left: "\\(", right: "\\)", display: false },
    { left: "\\begin{equation}", right: "\\end{equation}", display: true },
    { left: "\\begin{align}", right: "\\end{align}", display: true },
    { left: "\\begin{align*}", right: "\\end{align*}", display: true },
    { left: "\\begin{alignat}", right: "\\end{alignat}", display: true },
    { left: "\\begin{gather}", right: "\\end{gather}", display: true },
    { left: "\\begin{CD}", right: "\\end{CD}", display: true },
    { left: "\\begin{algorithm}", right: "\\end{algorithm}", display: true },
    { left: "\\begin{algorithmic}", right: "\\end{algorithmic}", display: true },
    { left: "\\begin{center}", right: "\\end{center}", display: true },
    { left: "\\begin{tikpicture}", right: "\\end{tikzpicture}", display: true },
    { left: "\\begin{center}", right: "\\end{center}", display: true },
    { left: "\\[", right: "\\]", display: true }
  ],
  errorCallback: (message: string, _stack: unknown) => {
    console.log("Trying to process latex on server.");
    const latex = message.substring(36, message.length - 7);
    const encoded = encodeURIComponent(latex);
    const style = window.getComputedStyle(document.body);
    const textColor = style.getPropertyValue('--text');
    const colorEncoded = encodeURIComponent(textColor.substring(1));
    const encodedTitle = encodeURIComponent(current_id);
    fetch(`/latex?tex=${encoded}&color=${colorEncoded}&id=${encodedTitle}`)
      .then((resp) => resp.text())
      .then((svg) => {
        const newHTML = rendered.value.replace(latex, svg);
        rendered.value = newHTML;
      });
  }
};

// Updpate the selector from 'pre code' to 'code' to autodetect inline src
// like src_java[:exports code]{ void main() } which has no <pre></pre>.
hljs.configure({ 'cssSelector': 'code' });

watch(props, () => preview(props.id));
watch(rendered, async () => {
  await nextTick();
  hljs.highlightAll();
  console.log(`Ref: ${preview_ref}`);
  renderMathInElement(preview_ref.value!, katexOptions);
});
</script>

<template>
  <div class="collapse-btn" tabindex="1" :onclick="resize" :style="{ 'position': 'absolute', 'right': '0px' }">></div>
  <div class="org-preview-outerframe" :style="{ 'display': shown }">
    <div class="collapse-btn" tabindex="1" :onclick="resize">></div>
    <div id="org-preview-frame">
      <div id="org-preview" ref="preview-ref" v-html="rendered"></div>
    </div>
  </div>
</template>

<style>
.org-preview-outerframe {
  right: 0px;
  top: 0px;
  bottom: 0px;
  position: absolute;
  z-index: 50;
  display: flex;
  width: 50%;
}

.collapse-btn {
  border-radius: 5px;
  padding: 2px;
  width: 42px;
  min-width: 42px;
  height: 42px;
  min-height: 42px;
  margin: 12px;
  background-color: var(--clickable);
  display: flex;
  text-align: center;
  right: 0px;
  top: 0px;
}

.collapse-btn:hover {
  filter: brightness(125%);
}

.collapse-btn:active {
  filter: brightness(75%);
}

#org-preview-frame {
  background-color: var(--surface);
  border-left: 5px solid var(--clickable);
  color: white;
  float: left;
  width: 100%;
  height: 100%;
  overflow-y: scroll;
  z-index: 50;
}

#org-preview {
  background-color: var(--surface);
  font-family: var(--font);
  padding: 10px;
  overflow: hidden;
}

#org-preview a {
  color: var(--clickable);
}

h1,
h2,
h3,
h4 {
  color: var(--highlight-2);
  border-bottom: 2px solid var(--highlight-2);
}

hr {
  color: var(--highlight);
}

.center {
  text-align: center;
}

.quote {
  margin: unset;
  background-color: var(--base);
  border-left: 2px solid var(--highlight);
}

.quote>p {
  margin-top: 0px;
  margin-bottom: 0px;
  padding: 5px;
}

/* Don't know if pre is enough... */
/* pre:has(.hljs) */
pre {
  background-color: var(--base);
  padding: 5px;
  border-radius: 5px;
  overflow-x: scroll;
}

.hljs-keyword {
  color: var(--keyword) !important;
}

.hljs-string {
  color: var(--string) !important;
}

.hljs-section,
.hljs-selector-class,
.hljs-template-variable,
.hljs-deletion {
  color: #800 !important;
}

.hljs-variable {
  color: var(--ident) !important;
}

.hljs-title {
  color: var(--ident) !important;
}

.hljs-comment {
  color: var(--comment) !important;
}

.hljs-type {
  color: var(--type);
}

.src {
  background-color: var(--base) !important;
  padding: 5px;
  border-radius: 5px;
}

td {
  padding: 5px;
  border: 1px solid var(--highlight);
}

table {
  border-collapse: collapse;
}

/* This is a bit hacky. */
.katex-html {
  display: none;
  visibility: hidden;
}

/* this might be broken for some svgs. */
svg {
  fill: var(--text);
}
</style>
