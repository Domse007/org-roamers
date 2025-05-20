<script setup lang="ts">
import hljs from "highlight.js";
import { nextTick, ref, useTemplateRef, watch, type Ref } from "vue";
import renderMathInElement from "katex/contrib/auto-render";
import { getScope } from "../settings.ts";
import { type OrgAsHTMLResponse } from "../types.ts";
import { History } from "../history.ts";
import BigButton from "./basic/BigButton.vue";

const props = defineProps<{ id: string }>();
const shown: Ref<"none" | "flex"> = ref("none");
const links: Ref<{ display: string; id: string }[]> = ref([]);

const rendered = ref("");
let current_id: string = "";
const preview_ref = useTemplateRef("preview-ref");

const history = new History<string>();

const preview = (id: string) => {
  emit("previewSwitch", id);
  current_id = id;
  console.log(`Previewing ${id}`);
  const scope: "file" | "node" = getScope();
  fetch(`/org?id=${id}&scope=${scope}`)
    .then((response) => response.json())
    .then((text) => JSON.parse(text))
    .then((resp: OrgAsHTMLResponse) => {
      history.push(id);
      rendered.value = resp.org;
      links.value = resp.links;
      expand();
    });
};

const expand = () => {
  shown.value = "flex";
};
const collapse = () => {
  shown.value = "none";
};

const resize = () => {
  if (rendered.value.length == 0) {
    return;
  }
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
    { left: "\\begin{gather*}", right: "\\end{gather*}", display: true },
    { left: "\\begin{CD}", right: "\\end{CD}", display: true },
    { left: "\\begin{algorithm}", right: "\\end{algorithm}", display: true },
    {
      left: "\\begin{algorithmic}",
      right: "\\end{algorithmic}",
      display: true,
    },
    { left: "\\begin{center}", right: "\\end{center}", display: true },
    {
      left: "\\begin{tikzpicture}",
      right: "\\end{tikzpicture}",
      display: true,
    },
    { left: "\\[", right: "\\]", display: true },
  ],
  errorCallback: (message: string, _stack: unknown) => {
    console.log("Trying to process latex on server.");
    let latex = message.substring(36, message.length - 7);
    if (!latex.startsWith("\\begin")) {
      latex = "\\( " + latex + " \\)";
    }
    const encoded = encodeURIComponent(latex);
    const style = window.getComputedStyle(document.body);
    const textColor = style.getPropertyValue("--text");
    const colorEncoded = encodeURIComponent(textColor.substring(1));
    const encodedTitle = encodeURIComponent(current_id);
    fetch(`/latex?tex=${encoded}&color=${colorEncoded}&id=${encodedTitle}`)
      .then((resp) => resp.text())
      .then((svg) => {
        const newHTML = rendered.value.replace(latex, svg);
        rendered.value = newHTML;
      });
  },
};

// Updpate the selector from 'pre code' to 'code' to autodetect inline src
// like src_java[:exports code]{ void main() } which has no <pre></pre>.
hljs.configure({ cssSelector: "code" });

const configureIDLinks = (_class: string) => {
  Array.from(document.getElementsByClassName(_class)).forEach((elem: Element) =>
    elem.addEventListener("click", (elem) => {
      if (!elem.target) return;
      const target = <HTMLElement>elem.target;
      preview(target.id);
    }),
  );
};

const collapseIcon = () => (shown.value == "none" ? "🗁" : "🗀");

watch(props, () => preview(props.id));
watch(rendered, async () => {
  await nextTick();
  hljs.highlightAll();
  console.log(`Ref: ${preview_ref}`);
  renderMathInElement(preview_ref.value!, katexOptions);
  configureIDLinks("org-preview-id-link");
  configureIDLinks("org-preview-footer-link");
});
const emit = defineEmits(["previewSwitch"]);
</script>

<template>
  <BigButton
    fg="var(--base)"
    bg="var(--clickable)"
    :onclick="resize"
    :style="{ position: 'absolute', right: '0px', top: '0px' }"
  >
    {{ collapseIcon() }}
  </BigButton>
  <div class="org-preview-outerframe" :style="{ display: shown }">
    <BigButton fg="var(--base)" bg="var(--clickable)" :onclick="resize">
      &#128448;
    </BigButton>
    <div id="org-preview-frame">
      <div
        style="
          display: flex;
          justify-content: space-between;
          border-bottom: 2px solid var(--clickable);
          background-color: var(--base);
        "
      >
        <div style="display: flex">
          <BigButton
            :style="{ visibility: history.canGoBack() ? 'visible' : 'hidden' }"
            fg="var(--base)"
            bg="var(--clickable)"
            @button-clicked="preview(history.back()!)"
          >
            &hookleftarrow;
          </BigButton>
          <BigButton
            :style="{
              visibility: history.canGoForward() ? 'visible' : 'hidden',
            }"
            fg="var(--base)"
            bg="var(--clickable)"
            @button-clicked="preview(history.forward()!)"
          >
            &hookrightarrow;
          </BigButton>
        </div>
        <BigButton
          fg="var(--base)"
          bg="var(--clickable)"
          @button-clicked="preview(current_id)"
          >&circlearrowleft;
        </BigButton>
      </div>
      <div id="org-preview" ref="preview-ref" v-html="rendered"></div>
      <div id="org-preview-footer" v-if="links.length != 0">
        <div id="org-preview-footer-title">
          Outgoing links:
          <hr />
        </div>
        <a
          class="org-preview-footer-link"
          v-for="link in links"
          :id="link.id"
          >{{ link.display }}</a
        >
      </div>
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

.quote > p {
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

.org-preview-id-link {
  cursor: pointer;
}

#org-preview-footer {
  margin: 10px;
  border-radius: 5px;
  background-color: var(--base);
  font-family: var(--font);
  display: flex;
  flex-direction: column;
}

.org-preview-footer-link {
  color: var(--clickable);
  padding: 5px;
  cursor: pointer;
  user-select: none;
}

.org-preview-footer-link:hover {
  filter: brightness(125%);
}

.org-preview-footer-link:active {
  filter: brightness(75%);
}

#org-preview-footer-title {
  padding-top: 5px;
  padding-left: 5px;
  padding-right: 5px;
  color: var(--highlight);
}
</style>
