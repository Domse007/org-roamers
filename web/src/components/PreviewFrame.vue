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

// Footer collapse state
const footerExpanded = ref(false);

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
      const errorMsg =
        error.name === "TypeError" && error.message.includes("fetch")
          ? "Server is not responding. Please check if the server is running."
          : `Failed to load content: ${error.message}`;
      emit("error", errorMsg);
      rendered.value = `<div class="error">${errorMsg}</div>`;
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

// Dynamic import for Lisp syntax highlighting and register elisp/emacs-lisp aliases
import("highlight.js/lib/languages/lisp")
  .then((lispLang) => {
    hljs.registerLanguage("lisp", lispLang.default);
    hljs.registerLanguage("elisp", lispLang.default);
    hljs.registerLanguage("emacs-lisp", lispLang.default);
    console.log("Registered lisp, elisp, and emacs-lisp highlighting");
  })
  .catch((error) => {
    console.warn("Failed to load Lisp syntax highlighting:", error);
    // Fallback: try to register aliases if lisp is already available
    try {
      if (hljs.getLanguage("lisp")) {
        hljs.registerAliases("elisp", { languageName: "lisp" });
        hljs.registerAliases("emacs-lisp", { languageName: "lisp" });
        console.log(
          "Registered elisp and emacs-lisp aliases for existing lisp",
        );
      }
    } catch (aliasError) {
      console.warn("Failed to register lisp aliases as fallback:", aliasError);
    }
  });

// Update the selector from 'pre code' to 'code' to autodetect inline src
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

const emit = defineEmits(["previewSwitch", "error"]);
</script>

<template>
  <button
    v-if="shown === 'none'"
    class="preview-toggle-button-floating"
    @click="resize"
    :aria-label="'Open Preview'"
    :title="'Open Preview'"
  >
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
      <path
        d="M6 3L11 8L6 13"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>

  <button
    v-if="shown === 'flex'"
    class="preview-toggle-button-attached"
    @click="resize"
    :style="{
      right: `calc(${frameWidth}% + 8px)`,
    }"
    :aria-label="'Close Preview'"
    :title="'Close Preview'"
  >
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
      <path
        d="M6 3L11 8L6 13"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        transform="rotate(180 8 8)"
      />
    </svg>
  </button>

  <div
    class="org-preview-outerframe"
    :style="{ display: shown, width: frameWidth + '%' }"
  >
    <div
      class="resize-handle"
      @mousedown="startResize"
      :class="{ 'resize-handle-active': isResizing }"
    ></div>

    <div class="org-preview-frame">
      <div class="preview-header">
        <div class="preview-title">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path
              d="M2 3H14V13H2V3Z"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <path
              d="M2 5H14"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <circle cx="4" cy="4" r="0.5" fill="currentColor" />
            <circle cx="6" cy="4" r="0.5" fill="currentColor" />
            <circle cx="8" cy="4" r="0.5" fill="currentColor" />
          </svg>
          <span>Preview</span>
        </div>

        <div class="preview-controls">
          <button
            class="preview-control-button"
            :class="{ 'preview-control-hidden': !history.canGoBack() }"
            @click="preview(history.back()!)"
            title="Go back"
            :disabled="!history.canGoBack()"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path
                d="M8 11L5 7L8 3"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </button>

          <button
            class="preview-control-button"
            :class="{ 'preview-control-hidden': !history.canGoForward() }"
            @click="preview(history.forward()!)"
            title="Go forward"
            :disabled="!history.canGoForward()"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path
                d="M6 3L9 7L6 11"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </button>

          <button
            class="preview-control-button"
            @click="preview(current_id)"
            title="Refresh content"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path
                d="M1 2V6H5"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M13 12V8H9"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M12.5 4.5C11.7 2.9 10.1 1.7 8.2 1.4C5.4 0.9 2.8 2.4 1.9 5"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M1.5 9.5C2.3 11.1 3.9 12.3 5.8 12.6C8.6 13.1 11.2 11.6 12.1 9"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </button>
        </div>
      </div>

      <div
        class="org-preview-content"
        ref="preview-ref"
        v-html="rendered"
      ></div>

      <div class="org-preview-footer" v-if="links.length != 0">
        <button
          class="preview-footer-header"
          @click="footerExpanded = !footerExpanded"
          :title="footerExpanded ? 'Collapse links' : 'Expand links'"
        >
          <div class="preview-footer-header-content">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path
                d="M4.5 3C4.5 3 4.5 2 6 2C7.5 2 10 2 10 2V5.5"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M6.5 1.5L10 5"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M8.5 6.5V9C8.5 9.5 8 10 7.5 10H3C2.5 10 2 9.5 2 9V4.5C2 4 2.5 3.5 3 3.5H5.5"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            <span>Outgoing Links ({{ links.length }})</span>
          </div>
          <svg
            width="12"
            height="12"
            viewBox="0 0 12 12"
            fill="none"
            class="preview-footer-chevron"
            :style="{
              transform: footerExpanded ? 'rotate(180deg)' : 'rotate(0deg)',
            }"
          >
            <path
              d="M3 4.5L6 7.5L9 4.5"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <div class="preview-footer-links" v-show="footerExpanded">
          <button
            class="org-preview-footer-link"
            :key="link.id"
            v-for="link in links"
            @click="preview(link.id)"
            :title="`Open ${link.display}`"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path
                d="M4.5 3C4.5 3 4.5 2 6 2C7.5 2 10 2 10 2V5.5"
                stroke="currentColor"
                stroke-width="1"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M6.5 1.5L10 5"
                stroke="currentColor"
                stroke-width="1"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M8.5 6.5V9C8.5 9.5 8 10 7.5 10H3C2.5 10 2 9.5 2 9V4.5C2 4 2.5 3.5 3 3.5H5.5"
                stroke="currentColor"
                stroke-width="1"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            <span>{{ link.display }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.preview-toggle-button-floating {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 4px;
  background: linear-gradient(
    135deg,
    var(--clickable),
    color-mix(in srgb, var(--clickable) 80%, black)
  );
  color: var(--surface);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
  transition: all 0.15s ease;
  z-index: 52;
  border: 1px solid color-mix(in srgb, var(--clickable) 120%, white);
}

.preview-toggle-button-floating:hover {
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--clickable) 90%, white),
    var(--clickable)
  );
  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.25);
  border-color: color-mix(in srgb, var(--clickable) 80%, white);
}

.preview-toggle-button-floating:active {
  transform: scale(0.98);
}

.preview-toggle-button-attached {
  position: absolute;
  top: 8px;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 4px;
  background: linear-gradient(
    135deg,
    var(--clickable),
    color-mix(in srgb, var(--clickable) 80%, black)
  );
  color: var(--surface);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
  z-index: 53;
  border: 1px solid color-mix(in srgb, var(--clickable) 120%, white);
}

.preview-toggle-button-attached:hover {
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--clickable) 90%, white),
    var(--clickable)
  );
  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.25);
  border-color: color-mix(in srgb, var(--clickable) 80%, white);
}

.preview-toggle-button-attached:active {
  transform: scale(0.98);
}

.org-preview-outerframe {
  position: absolute;
  right: 8px;
  top: 8px;
  bottom: 8px;
  z-index: 50;
  display: flex;
  background: linear-gradient(
    135deg,
    var(--surface),
    color-mix(in srgb, var(--surface) 95%, var(--base))
  );
  border: 1px solid color-mix(in srgb, var(--highlight) 40%, transparent);
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  backdrop-filter: blur(8px);
  overflow: hidden;
}

.resize-handle {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: color-mix(in srgb, var(--highlight) 30%, transparent);
  cursor: ew-resize;
  z-index: 51;
  transition: all 0.2s ease;
}

.resize-handle:hover,
.resize-handle-active {
  background: var(--highlight);
  width: 6px;
  box-shadow: 0 0 8px color-mix(in srgb, var(--highlight) 50%, transparent);
}

.org-preview-frame {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  margin-left: 4px;
  font-family: var(--font);
  color: var(--text);
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px 8px 14px;
  border-bottom: 1px solid color-mix(in srgb, var(--highlight) 30%, transparent);
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--surface) 98%, var(--highlight)),
    color-mix(in srgb, var(--surface) 95%, var(--base))
  );
  flex-shrink: 0;
}

.preview-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--highlight);
}

.preview-title svg {
  opacity: 0.8;
}

.preview-controls {
  display: flex;
  gap: 4px;
}

.preview-control-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: color-mix(in srgb, var(--surface) 80%, transparent);
  color: var(--text);
  cursor: pointer;
  transition: all 0.15s ease;
  opacity: 1;
}

.preview-control-button:hover:not(:disabled) {
  background: color-mix(in srgb, var(--clickable) 20%, var(--surface));
  color: var(--clickable);
}

.preview-control-button:active:not(:disabled) {
  transform: scale(0.95);
}

.preview-control-button:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.preview-control-hidden {
  opacity: 0.2;
}

.org-preview-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  background: var(--surface);
}

.org-preview-footer {
  border-top: 1px solid color-mix(in srgb, var(--highlight) 20%, transparent);
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--base) 98%, var(--surface)),
    var(--base)
  );
  flex-shrink: 0;
}

.preview-footer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: transparent;
  font-size: 13px;
  font-weight: 600;
  color: var(--highlight);
  cursor: pointer;
  transition: all 0.15s ease;
  border-bottom: 1px solid color-mix(in srgb, var(--highlight) 15%, transparent);
}

.preview-footer-header:hover {
  background: color-mix(in srgb, var(--highlight) 10%, transparent);
}

.preview-footer-header-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.preview-footer-header-content svg {
  opacity: 0.7;
}

.preview-footer-chevron {
  opacity: 0.6;
  transition: transform 0.2s ease;
}

.preview-footer-links {
  max-height: 200px;
  overflow-y: auto;
  padding: 4px;
  transition: all 0.2s ease;
}

.org-preview-footer-link {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  margin: 1px 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--clickable);
  font-family: var(--font);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  transition: all 0.15s ease;
}

.org-preview-footer-link:hover {
  background: color-mix(in srgb, var(--clickable) 15%, transparent);
  color: color-mix(in srgb, var(--clickable) 110%, white);
}

.org-preview-footer-link:active {
  transform: scale(0.98);
}

.org-preview-footer-link svg {
  opacity: 0.6;
  flex-shrink: 0;
}

.org-preview-footer-link span {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Custom scrollbar styling */
.org-preview-content::-webkit-scrollbar,
.preview-footer-links::-webkit-scrollbar {
  width: 6px;
}

.org-preview-content::-webkit-scrollbar-track,
.preview-footer-links::-webkit-scrollbar-track {
  background: color-mix(in srgb, var(--base) 50%, transparent);
  border-radius: 3px;
}

.org-preview-content::-webkit-scrollbar-thumb,
.preview-footer-links::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--text) 30%, transparent);
  border-radius: 3px;
  transition: background 0.2s ease;
}

.org-preview-content::-webkit-scrollbar-thumb:hover,
.preview-footer-links::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--text) 50%, transparent);
}

/* Responsive design */
@media (max-width: 768px) {
  .org-preview-outerframe {
    right: 4px;
    top: 4px;
    bottom: 4px;
  }

  .preview-toggle-button-floating {
    top: 4px;
    right: 4px;
    width: 32px;
    height: 32px;
  }

  .preview-toggle-button-attached {
    top: 4px;
    width: 32px;
    height: 32px;
  }

  .preview-header {
    padding: 8px 12px 6px 12px;
  }

  .preview-title {
    font-size: 13px;
  }

  .preview-control-button {
    width: 26px;
    height: 26px;
  }

  .org-preview-content {
    padding: 12px;
  }
}

@media (max-width: 480px) {
  .preview-footer-links {
    max-height: 120px;
  }

  .org-preview-footer-link {
    padding: 6px 8px;
    font-size: 12px;
  }
}
</style>

<!-- Content-specific styles (not scoped to avoid affecting v-html content) -->
<style>
/* Org content styling */
.org-preview-content a {
  color: var(--clickable);
  text-decoration: none;
  border-bottom: 1px solid color-mix(in srgb, var(--clickable) 40%, transparent);
  transition: all 0.15s ease;
}

.org-preview-content a:hover {
  color: color-mix(in srgb, var(--clickable) 110%, white);
  border-bottom-color: var(--clickable);
}

.org-preview-content h1,
.org-preview-content h2,
.org-preview-content h3,
.org-preview-content h4,
.org-preview-content h5,
.org-preview-content h6 {
  color: var(--highlight-2);
  border-bottom: 1px solid
    color-mix(in srgb, var(--highlight-2) 40%, transparent);
  padding-bottom: 4px;
  margin-top: 1.5em;
  margin-bottom: 0.75em;
}

.org-preview-content h1 {
  font-size: 1.5em;
  border-bottom-width: 2px;
}

.org-preview-content h1:first-of-type {
  font-size: 1.8em;
  font-weight: 700;
  text-align: center;
  margin-top: 0.5em;
  margin-bottom: 1.2em;
  padding: 0.8em 1em;
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--highlight-2) 15%, transparent),
    color-mix(in srgb, var(--highlight-2) 8%, transparent)
  );
  border: none;
  border-radius: 6px;
  border-left: 4px solid var(--highlight-2);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  color: var(--highlight);
}

.org-preview-content h2 {
  font-size: 1.3em;
}

.org-preview-content h3 {
  font-size: 1.1em;
}

.org-preview-content hr {
  border: none;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent,
    color-mix(in srgb, var(--highlight) 60%, transparent),
    transparent
  );
  margin: 1.5em 0;
}

.org-preview-content .center {
  text-align: center;
}

.org-preview-content .quote {
  margin: 1em 0;
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--base) 95%, var(--surface)),
    var(--base)
  );
  border-left: 3px solid var(--highlight);
  border-radius: 0 4px 4px 0;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.org-preview-content .quote > p {
  margin: 0;
  padding: 12px 16px;
}

.org-preview-content pre {
  background: linear-gradient(
    135deg,
    var(--base),
    color-mix(in srgb, var(--base) 95%, var(--surface))
  );
  border: 1px solid color-mix(in srgb, var(--highlight) 20%, transparent);
  padding: 12px;
  border-radius: 6px;
  overflow-x: auto;
  margin: 1em 0;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.org-preview-content pre.program-output {
  font-family: "Consolas", "Monaco", "Courier New", monospace;
  font-size: 0.9em;
  line-height: 1.4;
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--base) 90%, black),
    var(--base)
  );
  border-color: color-mix(in srgb, var(--overlay) 40%, transparent);
  color: color-mix(in srgb, var(--text) 95%, white);
  position: relative;
  padding-top: 32px;
}

.org-preview-content pre.program-output::before {
  content: "Output";
  position: absolute;
  top: 8px;
  right: 12px;
  font-size: 0.75em;
  font-weight: 600;
  color: var(--overlay);
  background: color-mix(in srgb, var(--base) 80%, black);
  padding: 3px 8px;
  border-radius: 3px;
  border: 1px solid color-mix(in srgb, var(--overlay) 30%, transparent);
  font-family: var(--font);
  letter-spacing: 0.5px;
  text-transform: uppercase;
}

.org-preview-content .src {
  background: linear-gradient(
    135deg,
    var(--base),
    color-mix(in srgb, var(--base) 95%, var(--surface))
  ) !important;
  border: 1px solid color-mix(in srgb, var(--highlight) 20%, transparent) !important;
  padding: 8px 12px !important;
  border-radius: 4px !important;
  margin: 0.5em 0 !important;
  display: inline-block !important;
}

.org-preview-content table {
  border-collapse: collapse;
  width: 100%;
  margin: 1em 0;
  border-radius: 6px;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.org-preview-content td,
.org-preview-content th {
  padding: 8px 12px;
  border: 1px solid color-mix(in srgb, var(--highlight) 25%, transparent);
  background: color-mix(in srgb, var(--surface) 95%, var(--base));
}

.org-preview-content th {
  background: linear-gradient(
    135deg,
    var(--base),
    color-mix(in srgb, var(--base) 90%, var(--highlight))
  );
  font-weight: 600;
  color: var(--highlight);
}

/* Syntax highlighting improvements */
.org-preview-content .hljs-keyword {
  color: var(--keyword) !important;
  font-weight: 600;
}

.org-preview-content .hljs-string {
  color: var(--string) !important;
}

.org-preview-content .hljs-section,
.org-preview-content .hljs-selector-class,
.org-preview-content .hljs-template-variable,
.org-preview-content .hljs-deletion {
  color: #ff6b6b !important;
}

.org-preview-content .hljs-variable {
  color: var(--ident) !important;
}

.org-preview-content .hljs-title {
  color: var(--ident) !important;
  font-weight: 600;
}

.org-preview-content .hljs-comment {
  color: var(--comment) !important;
  font-style: italic;
}

.org-preview-content .hljs-type {
  color: var(--type) !important;
  font-weight: 500;
}

/* Enhanced KaTeX styling */
.org-preview-content .katex-display {
  margin: 1.5em 0 !important;
  text-align: center !important;
  display: block !important;
  visibility: visible !important;
}

.org-preview-content .katex-display-wrapper {
  margin: 1.5em 0 !important;
  text-align: center !important;
  overflow-x: auto !important;
  display: block !important;
  visibility: visible !important;
  padding: 8px 0;
}

.org-preview-content .katex {
  font-size: 1.1em !important;
  color: var(--text) !important;
  display: inline !important;
  visibility: visible !important;
}

.org-preview-content .katex-display .katex {
  display: block !important;
}

.org-preview-content .katex .base {
  color: var(--text) !important;
}

.org-preview-content .katex .mord,
.org-preview-content .katex .mop,
.org-preview-content .katex .mbin,
.org-preview-content .katex .mrel,
.org-preview-content .katex .mopen,
.org-preview-content .katex .mclose,
.org-preview-content .katex .mpunct,
.org-preview-content .katex .mspace,
.org-preview-content .katex .minner {
  color: var(--text) !important;
}

.org-preview-content .katex * {
  color: var(--text) !important;
  visibility: visible !important;
}

.org-preview-content .katex {
  background: none !important;
  border: none !important;
}

/* Hide conflicting math elements */
.org-preview-content .MathJax,
.org-preview-content .MathJax_Display,
.org-preview-content .math-tex {
  display: none !important;
  visibility: hidden !important;
}

.org-preview-content .katex-mathml {
  display: none !important;
  visibility: hidden !important;
}

.org-preview-content .katex-html {
  display: inline !important;
  visibility: visible !important;
}

/* SVG styling for LaTeX and general content */
.org-preview-content svg {
  fill: var(--text);
  max-width: 100%;
  height: auto;
}

.org-preview-content svg.org-latex-rendered {
  fill: var(--text) !important;
  color: var(--text) !important;
  max-width: 100%;
  height: auto;
  display: inline-block;
  vertical-align: middle;
}

.org-preview-content svg.org-latex-rendered * {
  fill: var(--text) !important;
  color: var(--text) !important;
}

/* Org-mode specific classes */
.org-preview-content .org-latex {
  color: var(--text);
}

.org-preview-content .org-latex-block {
  margin: 1.5em 0;
  text-align: center;
}

/* Error and placeholder styling */
.org-preview-content .katex-error,
.org-preview-content .latex-error {
  color: #ff6b6b !important;
  border: 1px solid #ff6b6b;
  background: color-mix(in srgb, #ff6b6b 10%, var(--surface));
  padding: 4px 6px;
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.9em;
}

.org-preview-content .org-latex-placeholder,
.org-preview-content .org-latex-block-placeholder {
  color: var(--comment);
  font-style: italic;
  background: color-mix(in srgb, var(--base) 80%, var(--surface));
  padding: 4px 6px;
  border-radius: 4px;
  border: 1px dashed color-mix(in srgb, var(--comment) 60%, transparent);
}

/* Clickable org links */
.org-preview-content .org-preview-id-link {
  cursor: pointer;
  color: var(--clickable) !important;
  text-decoration: none !important;
  border-bottom: 1px solid color-mix(in srgb, var(--clickable) 40%, transparent) !important;
  transition: all 0.15s ease !important;
}

.org-preview-content .org-preview-id-link:hover {
  color: color-mix(in srgb, var(--clickable) 110%, white) !important;
  border-bottom-color: var(--clickable) !important;
}

/* Responsive LaTeX */
@media (max-width: 768px) {
  .org-preview-content .katex {
    font-size: 1em !important;
  }

  .org-preview-content .katex-display-wrapper {
    margin: 1em 0 !important;
    padding: 4px 0;
  }
}

/* Dark theme enhancements */
@media (prefers-color-scheme: dark) {
  .org-preview-outerframe {
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.3);
  }

  .preview-toggle-button {
    box-shadow: 0 3px 8px rgba(0, 0, 0, 0.25);
  }

  .preview-toggle-button:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.35);
  }
}

/* Reduced motion preference */
@media (prefers-reduced-motion: reduce) {
  .preview-toggle-button-floating,
  .preview-toggle-button-attached,
  .preview-control-button,
  .org-preview-footer-link,
  .resize-handle {
    transition: none;
  }
}
</style>
