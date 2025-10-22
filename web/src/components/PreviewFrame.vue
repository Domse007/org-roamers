<script setup lang="ts">
import hljs from "highlight.js";
import { nextTick, onMounted, onUnmounted, useTemplateRef, watch } from "vue";
import LinksSection from "./LinksSection.vue";
import { processLatexPlaceholders } from "../latex-utils.ts";
import { usePreviewResize } from "../composables/usePreviewResize";
import { usePreviewContent } from "../composables/usePreviewContent";
import { getRouter } from "../router";
import "../styles/preview-frame.css";
import "../styles/preview-content.css";
import "../styles/preview-code.css";
import "../styles/preview-latex.css";
import "../styles/preview-responsive.css";

const props = defineProps<{ id: string }>();
const emit = defineEmits(["previewSwitch", "error"]);

const preview_ref = useTemplateRef("preview-ref");

// Use composables
const { frameWidth, isResizing, startResize, stopResize } = usePreviewResize();
const {
  shown,
  links,
  incomingLinks,
  rendered,
  history,
  preview: previewContent,
  toggle: togglePreview,
  getCurrentId,
  getLatexBlocks,
} = usePreviewContent();

const router = getRouter();

// Wrapper function to handle emits
const preview = (id: string, updateRouter: boolean = true) => {
  emit("previewSwitch", id);
  previewContent(id, (errorMsg) => emit("error", errorMsg), updateRouter);
};

// Handle back button navigation without updating router
const previewFromHistory = (id: string | null) => {
  if (id) {
    // Don't update router since this is from internal history
    preview(id, false);
  }
};

const resize = togglePreview;

const configureIDLinks = (_class: string) => {
  Array.from(document.getElementsByClassName(_class)).forEach((elem: Element) =>
    elem.addEventListener("click", (elem) => {
      if (!elem.target) return;
      const target = <HTMLElement>elem.target;
      preview(target.id);
    }),
  );
};

// Set up router navigation listener
let unregisterRouter: (() => void) | null = null;

onMounted(() => {
  // Register router navigation handler
  unregisterRouter = router.onNavigate((nodeId) => {
    console.log("Router navigation to:", nodeId);
    // Preview without updating router (avoid infinite loop)
    preview(nodeId, false);
  });
});

onUnmounted(() => {
  // Clean up router listener
  if (unregisterRouter) {
    unregisterRouter();
  }
});

watch(props, () => {
  if (props.id) {
    preview(props.id);
  }
});
watch(rendered, async () => {
  await nextTick();

  // Apply syntax highlighting
  hljs.highlightAll();

  // Process LaTeX placeholders with secure server rendering
  const currentId = getCurrentId();
  const latexBlocks = getLatexBlocks();
  if (preview_ref.value && currentId && latexBlocks.length > 0) {
    try {
      console.log("Processing LaTeX placeholders:", latexBlocks.length);

      await processLatexPlaceholders(preview_ref.value, currentId, latexBlocks);

      console.log("LaTeX processing completed");
    } catch (error) {
      console.error("LaTeX processing failed:", error);
    }
  }

  // Configure link handlers
  configureIDLinks("org-preview-id-link");
  configureIDLinks("org-preview-footer-link");
});
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
        d="M10 3L5 8L10 13"
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
            @click="previewFromHistory(history.back())"
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
            @click="previewFromHistory(history.forward())"
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
            @click="preview(getCurrentId())"
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

          <button
            class="preview-control-button preview-close-button-mobile"
            @click="resize"
            title="Close preview"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path
                d="M2 2L12 12M12 2L2 12"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
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

      <!-- Incoming Links Section -->
      <LinksSection
        :links="incomingLinks"
        title="Incoming Links"
        icon-type="incoming"
        @link-click="preview"
      />

      <!-- Outgoing Links Section -->
      <LinksSection
        :links="links"
        title="Outgoing Links"
        icon-type="outgoing"
        @link-click="preview"
      />
    </div>
  </div>
</template>
