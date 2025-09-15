<script setup lang="ts">
import hljs from "highlight.js";
import {
  nextTick,
  ref,
  useTemplateRef,
  watch,
  onUnmounted,
  type Ref,
} from "vue";
import { getScope } from "../settings.ts";
import { type OrgAsHTMLResponse } from "../types.ts";
import { History } from "../history.ts";
import BigButton from "./basic/BigButton.vue";
import { processLatexPlaceholders } from "../latex-utils.ts";

const props = defineProps<{ id: string }>();
const shown: Ref<"none" | "flex"> = ref("none");
const links: Ref<{ display: string; id: string }[]> = ref([]);

const rendered = ref("");
let current_id: string = "";
let current_latex_blocks: string[] = [];
const preview_ref = useTemplateRef("preview-ref");

const history = new History<string>();

// Resize functionality
const frameWidth = ref(50); // Default width as percentage
const isResizing = ref(false);

const startResize = (event: MouseEvent) => {
  isResizing.value = true;
  document.addEventListener("mousemove", doResize);
  document.addEventListener("mouseup", stopResize);
  document.body.style.cursor = "ew-resize";
  document.body.style.userSelect = "none"; // Prevent text selection during resize
  event.preventDefault();
};

const doResize = (event: MouseEvent) => {
  if (!isResizing.value) return;

  const windowWidth = window.innerWidth;
  const newWidth = ((windowWidth - event.clientX) / windowWidth) * 100;

  // Constrain width between 20% and 80%
  frameWidth.value = Math.max(20, Math.min(80, newWidth));
};

const stopResize = () => {
  isResizing.value = false;
  document.removeEventListener("mousemove", doResize);
  document.removeEventListener("mouseup", stopResize);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
};

const preview = (id: string) => {
  emit("previewSwitch", id);
  current_id = id;
  console.log(`Previewing ${id}`);
  const scope: "file" | "node" = getScope();
  fetch(`/org?id=${id}&scope=${scope}`)
    .then((response) => {
      console.log("Org response status:", response.status);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      return response.json();
    })
    .then((resp: OrgAsHTMLResponse) => {
      console.log("Org response data:", {
        orgLength: resp.org?.length || 0,
        linksCount: resp.links?.length || 0,
        latexBlocksCount: resp.latex_blocks?.length || 0,
      });
      history.push(id);
      rendered.value = resp.org;
      links.value = resp.links;
      current_latex_blocks = resp.latex_blocks || [];
      console.log(
        `Loaded content with ${current_latex_blocks.length} LaTeX blocks`,
      );
      expand();
    })
    .catch((error) => {
      console.error("Failed to load org content:", error);
      rendered.value = `<div class="error">Failed to load content: ${error.message}</div>`;
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

// Helper functions for content management
const getCurrentContent = () => rendered.value;
const updateContent = (newContent: string) => {
  rendered.value = newContent;
};

// Dynamic import for COBOL syntax highlighting to avoid build issues
import("highlightjs-cobol")
  .then((hljsCOBOL) => {
    hljs.registerLanguage("cobol", hljsCOBOL.default);
  })
  .catch((error) => {
    console.warn("Failed to load COBOL syntax highlighting:", error);
  });

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

  // Apply syntax highlighting
  hljs.highlightAll();

  // Process LaTeX placeholders with secure server rendering
  if (preview_ref.value && current_id && current_latex_blocks.length > 0) {
    try {
      console.log(
        "Processing LaTeX placeholders:",
        current_latex_blocks.length,
      );

      await processLatexPlaceholders(
        preview_ref.value,
        current_id,
        current_latex_blocks,
      );

      console.log("LaTeX processing completed");
    } catch (error) {
      console.error("LaTeX processing failed:", error);
    }
  }

  // Configure link handlers
  configureIDLinks("org-preview-id-link");
  configureIDLinks("org-preview-footer-link");
});

// Cleanup event listeners on component unmount
onUnmounted(() => {
  if (isResizing.value) {
    stopResize();
  }
});

const emit = defineEmits(["previewSwitch"]);
</script>

<template>
  <BigButton
    fg="var(--base)"
    bg="var(--clickable)"
    :onclick="resize"
    :style="{
      position: 'absolute',
      right: shown === 'flex' ? frameWidth + '%' : '0px',
      top: '0px',
      zIndex: 52,
    }"
  >
    {{ collapseIcon() }}
  </BigButton>
  <div
    class="org-preview-outerframe"
    :style="{ display: shown, width: frameWidth + '%' }"
  >
    <div
      class="resize-handle"
      @mousedown="startResize"
      :style="{ cursor: isResizing ? 'ew-resize' : 'ew-resize' }"
    ></div>
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
          title="Refresh content"
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
          :key="link.id"
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
}

.resize-handle {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 5px;
  background-color: var(--clickable);
  cursor: ew-resize;
  z-index: 51;
  transition: all 0.2s ease;
}

.resize-handle:hover {
  background-color: var(--highlight);
  width: 7px;
  box-shadow: 0 0 8px rgba(0, 0, 0, 0.2);
}

#org-preview-frame {
  background-color: var(--surface);
  color: white;
  width: 100%;
  height: 100%;
  overflow-y: scroll;
  z-index: 50;
  margin-left: 5px; /* Account for the resize handle */
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

/* Enhanced KaTeX styling */
.katex-display {
  margin: 1em 0 !important;
  text-align: center !important;
  display: block !important;
  visibility: visible !important;
}

.katex-display-wrapper {
  margin: 1em 0 !important;
  text-align: center !important;
  overflow-x: auto !important;
  display: block !important;
  visibility: visible !important;
}

.katex {
  font-size: 1.1em !important;
  color: var(--text) !important;
  display: inline !important;
  visibility: visible !important;
}

.katex-display .katex {
  display: block !important;
}

.katex .base {
  color: var(--text) !important;
}

.katex .mord,
.katex .mop,
.katex .mbin,
.katex .mrel,
.katex .mopen,
.katex .mclose,
.katex .mpunct,
.katex .mspace,
.katex .minner {
  color: var(--text) !important;
}

/* Ensure all KaTeX sub-elements are visible */
.katex * {
  color: var(--text) !important;
  visibility: visible !important;
}

/* Fix potential conflicts with org-mode styling */
#org-preview .katex {
  background: none !important;
  border: none !important;
}

/* Hide MathJax-style elements that might conflict */
.MathJax,
.MathJax_Display,
.math-tex {
  display: none !important;
  visibility: hidden !important;
}

/* Hide MathML elements since we're using HTML-only output */
.katex-mathml {
  display: none !important;
  visibility: hidden !important;
}

/* Make sure KaTeX HTML elements are visible */
.katex-html {
  display: inline !important;
  visibility: visible !important;
}

/* Ensure SVG elements (from server-side LaTeX) use theme colors */
svg {
  fill: var(--text);
  max-width: 100%;
  height: auto;
}

/* Specific styling for LaTeX-rendered SVGs */
svg.org-latex-rendered {
  fill: var(--text) !important;
  color: var(--text) !important;
  max-width: 100%;
  height: auto;
  display: inline-block;
  vertical-align: middle;
}

/* Ensure all paths and elements in LaTeX SVGs use theme color */
svg.org-latex-rendered * {
  fill: var(--text) !important;
  color: var(--text) !important;
}

/* Handle org-mode specific LaTeX classes */
.org-latex {
  color: var(--text);
}

.org-latex-block {
  margin: 1em 0;
  text-align: center;
}

/* Better handling of LaTeX errors and fallbacks */
.katex-error,
.latex-error {
  color: var(--error, #ff6b6b) !important;
  border: 1px solid var(--error, #ff6b6b);
  background-color: var(--error-bg, rgba(255, 107, 107, 0.1));
  padding: 2px 4px;
  border-radius: 3px;
  font-family: monospace;
  font-size: 0.9em;
}

/* LaTeX placeholder styling while loading */
.org-latex-placeholder,
.org-latex-block-placeholder {
  color: var(--comment, #737994);
  font-style: italic;
  background-color: var(--base);
  padding: 2px 4px;
  border-radius: 3px;
  border: 1px dashed var(--comment, #737994);
}

/* Responsive LaTeX rendering */
@media (max-width: 768px) {
  .katex {
    font-size: 0.9em;
  }

  .katex-display-wrapper {
    margin: 0.5em 0;
  }
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
