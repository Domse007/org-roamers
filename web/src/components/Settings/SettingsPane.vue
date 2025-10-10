<script setup lang="ts">
import SettingsTheme from "./SettingsTheme.vue";
import SettingsGeneral from "./SettingsGeneral.vue";
import SettingsFilter from "./SettingsFilter.vue";
import { ref, type Ref } from "vue";
import BigButton from "../basic/BigButton.vue";

const settingsPages: string[] = ["General", "Theme", "Filter"];
const activePage: Ref<string> = ref(settingsPages[0]);

const shown: Ref<"none" | ""> = ref("none");

const inv_shown = () => {
  if (shown.value == "none") return "";
  else return "none";
};

const resize = () => {
  if (shown.value == "none") shown.value = "";
  else shown.value = "none";
};

const listButtonColor = (title: string) => {
  if (title == activePage.value) return "var(--surface)";
  else return "var(--clickable)";
};

const switchSettingsPage = (page: MouseEvent) => {
  if (page.target != null) {
    const target = page.target as HTMLElement;
    activePage.value = target.innerHTML;
  }
};

// Props for connection status
const props = defineProps<{
  connectionStatus?: "connecting" | "connected" | "disconnected";
  pendingChanges?: boolean;
  websocketState?: number | null;
  websocketUrl?: string;
}>();
</script>

<template>
  <button
    class="settings-open-button"
    @click="resize"
    :style="{ display: inv_shown() }"
    aria-label="Open Settings"
    title="Settings"
  >
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
      <circle
        cx="12"
        cy="12"
        r="3"
        stroke="currentColor"
        stroke-width="1.5"
      />
      <path
        d="M10.5 4.5H13.5L14 7C14.6 7.2 15.2 7.5 15.7 7.9L18 7L19.5 9.5L17.5 11C17.6 11.7 17.6 12.3 17.5 13L19.5 14.5L18 17L15.7 16.1C15.2 16.5 14.6 16.8 14 17L13.5 19.5H10.5L10 17C9.4 16.8 8.8 16.5 8.3 16.1L6 17L4.5 14.5L6.5 13C6.4 12.3 6.4 11.7 6.5 11L4.5 9.5L6 7L8.3 7.9C8.8 7.5 9.4 7.2 10 7L10.5 4.5Z"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linejoin="round"
      />
    </svg>
  </button>

  <div class="settings-pane" :style="{ display: shown }">
    <div class="settings-header">
      <div class="settings-title">
        <svg width="18" height="18" viewBox="0 0 20 20" fill="none">
          <path
            d="M10 6.5C8.067 6.5 6.5 8.067 6.5 10S8.067 13.5 10 13.5S13.5 11.933 13.5 10S11.933 6.5 10 6.5Z"
            stroke="currentColor"
            stroke-width="1.5"
          />
          <path
            d="M8.5 1.5H11.5L12 3.5C12.5 3.7 13 3.9 13.4 4.2L15.5 3.5L17 6L15.3 7.3C15.4 7.7 15.4 8.3 15.3 8.7L17 10L15.5 12.5L13.4 11.8C13 12.1 12.5 12.3 12 12.5L11.5 14.5H8.5L8 12.5C7.5 12.3 7 12.1 6.6 11.8L4.5 12.5L3 10L4.7 8.7C4.6 8.3 4.6 7.7 4.7 7.3L3 6L4.5 3.5L6.6 4.2C7 3.9 7.5 3.7 8 3.5L8.5 1.5Z"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linejoin="round"
          />
        </svg>
        <span>Settings</span>
      </div>
      <button
        class="settings-close-button"
        @click="resize"
        aria-label="Close Settings"
        title="Close"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path
            d="M1 1l14 14M15 1L1 15"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </div>

    <div class="settings-content">
      <nav class="settings-tabs">
        <button
          v-for="page in settingsPages"
          :key="page"
          :class="[
            'settings-tab',
            { 'settings-tab-active': page === activePage },
          ]"
          @click="activePage = page"
          :aria-selected="page === activePage"
        >
          <div class="settings-tab-icon">
            <svg
              v-if="page === 'General'"
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
            >
              <path
                d="M8 4V2M8 14V12M12 8H14M2 8H4M11.314 4.686L12.728 3.272M3.272 12.728L4.686 11.314M11.314 11.314L12.728 12.728M3.272 3.272L4.686 4.686"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
              <circle
                cx="8"
                cy="8"
                r="3"
                stroke="currentColor"
                stroke-width="1.5"
              />
            </svg>
            <svg
              v-else-if="page === 'Theme'"
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
            >
              <path
                d="M14 8A6 6 0 1 1 8 2V8H14Z"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            <svg
              v-else-if="page === 'Filter'"
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
            >
              <path
                d="M2 3H14L9 9V13L7 14V9L2 3Z"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </div>
          <span class="settings-tab-label">{{ page }}</span>
        </button>
      </nav>

      <div class="settings-panel">
        <SettingsTheme
          v-if="activePage === 'Theme'"
          @redraw-graph="$emit('redrawGraph')"
        />
        <SettingsGeneral
          v-if="activePage === 'General'"
          @toggle-layouter="$emit('toggleLayouter')"
          :connectionStatus="props.connectionStatus"
          :pendingChanges="props.pendingChanges"
          :websocketState="props.websocketState"
          :websocketUrl="props.websocketUrl"
        />
        <SettingsFilter v-if="activePage === 'Filter'" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-open-button {
  position: absolute;
  bottom: 12px;
  left: 12px;
  width: 44px;
  height: 44px;
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
  z-index: 60;
  border: 1px solid color-mix(in srgb, var(--clickable) 120%, white);
}

.settings-open-button:hover {
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--clickable) 90%, white),
    var(--clickable)
  );
  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.25);
  border-color: color-mix(in srgb, var(--clickable) 80%, white);
}

.settings-open-button:active {
  transform: scale(0.98);
}

.settings-pane {
  position: absolute;
  bottom: 12px;
  left: 12px;
  width: min(680px, 90vw);
  height: min(500px, 75vh);
  background: linear-gradient(
    135deg,
    var(--surface),
    color-mix(in srgb, var(--surface) 95%, var(--base))
  );
  border: 1px solid color-mix(in srgb, var(--highlight) 40%, transparent);
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  font-family: var(--font);
  color: var(--text);
  backdrop-filter: blur(8px);
  overflow: hidden;
  z-index: 55;
  display: flex;
  flex-direction: column;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px 10px 16px;
  border-bottom: 1px solid color-mix(in srgb, var(--highlight) 30%, transparent);
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--surface) 98%, var(--highlight)),
    color-mix(in srgb, var(--surface) 95%, var(--base))
  );
  flex-shrink: 0;
}

.settings-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: var(--highlight);
}

.settings-title svg {
  opacity: 0.8;
}

.settings-close-button {
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
  transition: background 0.15s ease;
}

.settings-close-button:hover {
  background: color-mix(in srgb, var(--text) 10%, var(--surface));
}

.settings-close-button:active {
  transform: scale(0.95);
}

.settings-content {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.settings-tabs {
  display: flex;
  flex-direction: column;
  min-width: 140px;
  background: linear-gradient(
    135deg,
    var(--base),
    color-mix(in srgb, var(--base) 95%, var(--surface))
  );
  border-right: 1px solid color-mix(in srgb, var(--highlight) 25%, transparent);
  padding: 6px;
  gap: 2px;
  flex-shrink: 0;
}

.settings-tab {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text);
  font-family: var(--font);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s ease;
  text-align: left;
  position: relative;
}

.settings-tab:before {
  content: "";
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--highlight);
  transform: scaleY(0);
  transition: transform 0.15s ease;
}

.settings-tab:hover {
  background: color-mix(in srgb, var(--clickable) 15%, transparent);
}

.settings-tab:hover:before {
  transform: scaleY(0.5);
}

.settings-tab-active {
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--highlight) 20%, var(--surface)),
    color-mix(in srgb, var(--highlight) 10%, var(--surface))
  );
  color: var(--highlight);
  border: 1px solid color-mix(in srgb, var(--highlight) 30%, transparent);
}

.settings-tab-active:before {
  transform: scaleY(1);
}

.settings-tab-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.7;
  transition: opacity 0.2s ease;
}

.settings-tab-active .settings-tab-icon {
  opacity: 1;
}

.settings-tab-label {
  flex: 1;
}

.settings-panel {
  flex: 1;
  padding: 0;
  overflow-y: auto;
  background: var(--surface);
  min-height: 0;
  min-width: 0;
}

/* Scrollbar styling for settings panel */
.settings-panel::-webkit-scrollbar {
  width: 8px;
}

.settings-panel::-webkit-scrollbar-track {
  background: color-mix(in srgb, var(--base) 50%, transparent);
  border-radius: 4px;
}

.settings-panel::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--text) 30%, transparent);
  border-radius: 4px;
  transition: background 0.2s ease;
}

.settings-panel::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--text) 50%, transparent);
}

/* Responsive design */
@media (max-width: 768px) {
  /* Bottom sheet modal on mobile */
  .settings-pane {
    left: 0;
    right: 0;
    bottom: 0;
    width: 100%;
    height: 90vh;
    border-radius: 12px 12px 0 0;
    border-bottom: none;
    z-index: 40; /* Behind preview frame (50) */
    box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.2);
  }

  .settings-open-button {
    bottom: 12px;
    left: 12px;
    z-index: 40; /* Behind preview frame (50) */
  }

  .settings-header {
    padding: 10px 14px 8px 14px;
  }

  .settings-title {
    font-size: 15px;
  }

  .settings-tabs {
    min-width: 120px;
    padding: 4px;
  }

  .settings-tab {
    padding: 8px 10px;
    font-size: 13px;
  }
}

@media (max-width: 600px) {
  .settings-pane {
    height: 85vh;
  }

  .settings-content {
    flex-direction: column;
  }

  .settings-tabs {
    flex-direction: row;
    min-width: unset;
    border-right: none;
    border-bottom: 1px solid
      color-mix(in srgb, var(--highlight) 25%, transparent);
    overflow-x: auto;
    padding: 6px 8px;
  }

  .settings-tab {
    white-space: nowrap;
    min-width: fit-content;
    padding: 10px 14px;
  }

  .settings-tab:hover,
  .settings-tab-active {
    background: color-mix(in srgb, var(--clickable) 15%, transparent);
  }

  .settings-tab:before {
    left: 0;
    right: 0;
    top: auto;
    bottom: 0;
    width: auto;
    height: 3px;
    transform: scaleX(0);
  }

  .settings-tab:hover:before {
    transform: scaleX(0.5);
  }

  .settings-tab-active:before {
    transform: scaleX(1);
  }
}

/* Dark theme enhancements */
@media (prefers-color-scheme: dark) {
  .settings-pane {
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.3);
  }

  .settings-open-button {
    box-shadow: 0 3px 8px rgba(0, 0, 0, 0.25);
  }

  .settings-open-button:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.35);
  }
}

/* Reduced motion preference */
@media (prefers-reduced-motion: reduce) {
  .settings-open-button,
  .settings-close-button,
  .settings-tab {
    transition: none;
  }
}
</style>
