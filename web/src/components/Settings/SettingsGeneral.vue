<script setup lang="ts">
import { ref, watch, type Ref, inject } from "vue";
import { generalSettings } from "../../settings.ts";
import StyledButton from "../basic/StyledButton.vue";

const previewScopeChange = () => {
  generalSettings.showEntireFile = !generalSettings.showEntireFile;
};

const timeoutEnabled: Ref<boolean> = ref(!!generalSettings.stopLayoutAfter);
const timeoutTime: Ref<number> = ref(
  !generalSettings.stopLayoutAfter ? 0 : generalSettings.stopLayoutAfter,
);

watch([timeoutEnabled, timeoutTime], () => {
  if (timeoutEnabled.value) generalSettings.stopLayoutAfter = timeoutTime.value;
  else generalSettings.stopLayoutAfter = null;
});

// Connection status props (these would be passed from parent)
const props = defineProps<{
  connectionStatus?: "connecting" | "connected" | "disconnected";
  pendingChanges?: boolean;
  websocketState?: number | null;
  websocketUrl?: string;
}>();
</script>

<template>
  <div class="settings-general">
    <!-- Connection Status Section -->
    <section class="settings-section">
      <h3 class="section-title">Connection Status</h3>
      <div class="status-grid">
        <div class="status-item">
          <span class="status-label">Server:</span>
          <div class="connection-status" :class="props.connectionStatus || 'disconnected'">
            <span v-if="(props.connectionStatus || 'disconnected') === 'connected'">
              🟢 Connected
            </span>
            <span v-if="(props.connectionStatus || 'disconnected') === 'connecting'">
              🟡 Connecting...
            </span>
            <span v-if="(props.connectionStatus || 'disconnected') === 'disconnected'">
              🔴 Disconnected
            </span>
          </div>
        </div>
        <div v-if="props.pendingChanges" class="status-item">
          <span class="status-label">Changes:</span>
          <div class="pending-changes">⏳ Processing...</div>
        </div>
        <div class="status-details">
          <span class="debug-info">
            WebSocket State: {{ props.websocketState || "null" }}
          </span>
        </div>
      </div>
    </section>

    <!-- Preview Settings Section -->
    <section class="settings-section">
      <h3 class="section-title">Preview Settings</h3>
      <div class="setting-item">
        <label class="setting-label">
          <input 
            type="checkbox" 
            :checked="!generalSettings.showEntireFile"
            @change="previewScopeChange"
            class="setting-checkbox"
          />
          <span class="setting-text">Show full file content</span>
        </label>
        <p class="setting-description">
          When enabled, preview shows the entire file instead of just the current node.
        </p>
      </div>
    </section>

    <!-- Graph Settings Section -->
    <section class="settings-section">
      <h3 class="section-title">Graph Settings</h3>
      <div class="setting-item">
        <label class="setting-label">
          <input 
            type="checkbox" 
            v-model="timeoutEnabled"
            class="setting-checkbox"
          />
          <span class="setting-text">Auto-stop layout</span>
        </label>
        <div v-if="timeoutEnabled" class="setting-controls">
          <label class="input-group">
            <span>Stop after:</span>
            <input 
              type="number" 
              v-model="timeoutTime"
              min="1"
              max="300"
              class="setting-input"
            />
            <span>seconds</span>
          </label>
        </div>
        <p class="setting-description">
          Automatically stops the graph layout algorithm after the specified time.
        </p>
      </div>
      <div class="setting-item">
        <StyledButton
          text="Restart Layout"
          bg="var(--clickable)"
          fg="var(--base)"
          @button-clicked="$emit('toggleLayouter')"
        />
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings-general {
  padding: 16px;
  height: 100%;
  overflow-y: auto;
  box-sizing: border-box;
}

.settings-section {
  margin-bottom: 24px;
}

.settings-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--highlight);
  margin: 0 0 12px 0;
  padding-bottom: 4px;
  border-bottom: 1px solid color-mix(in srgb, var(--highlight) 20%, transparent);
}

.status-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-label {
  font-weight: 500;
  color: var(--text);
  min-width: 60px;
}

.connection-status {
  font-size: 13px;
  font-weight: 500;
}

.connection-status.connected {
  color: var(--highlight-2);
}

.connection-status.connecting {
  color: var(--warn);
}

.connection-status.disconnected {
  color: var(--warn);
}

.pending-changes {
  color: var(--highlight);
  font-size: 13px;
  font-weight: 500;
  animation: pulse 1.5s ease-in-out infinite alternate;
}

@keyframes pulse {
  from { opacity: 1; }
  to { opacity: 0.5; }
}

.status-details {
  margin-top: 4px;
}

.debug-info {
  font-size: 11px;
  color: var(--overlay);
  font-family: monospace;
}

.setting-item {
  margin-bottom: 16px;
}

.setting-item:last-child {
  margin-bottom: 0;
}

.setting-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-weight: 500;
  color: var(--text);
}

.setting-checkbox {
  width: 16px;
  height: 16px;
  accent-color: var(--highlight);
  cursor: pointer;
}

.setting-text {
  user-select: none;
}

.setting-controls {
  margin-top: 8px;
  margin-left: 24px;
}

.input-group {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--text);
}

.setting-input {
  width: 60px;
  padding: 4px 6px;
  border: 1px solid color-mix(in srgb, var(--highlight) 30%, transparent);
  border-radius: 3px;
  background: var(--base);
  color: var(--text);
  font-family: var(--font);
  font-size: 13px;
}

.setting-input:focus {
  outline: none;
  border-color: var(--highlight);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--highlight) 20%, transparent);
}

.setting-description {
  margin: 6px 0 0 24px;
  font-size: 12px;
  color: var(--overlay);
  line-height: 1.4;
}

/* Responsive adjustments */
@media (max-width: 600px) {
  .settings-general {
    padding: 12px;
  }
  
  .settings-section {
    margin-bottom: 20px;
  }
  
  .section-title {
    font-size: 14px;
  }
  
  .status-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
  
  .status-label {
    min-width: unset;
  }
}
</style>
