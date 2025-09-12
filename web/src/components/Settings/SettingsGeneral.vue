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
  <div id="theme-settings">
    <!-- Connection Status Section -->
    <div class="connection-section">
      <b style="margin-bottom: 5px">Connection Status:</b>
      <div
        class="connection-status"
        :class="props.connectionStatus || 'disconnected'"
      >
        <span v-if="(props.connectionStatus || 'disconnected') === 'connected'"
          >🟢 Connected ({{ props.websocketState || "unknown" }})</span
        >
        <span v-if="(props.connectionStatus || 'disconnected') === 'connecting'"
          >🟡 Connecting...</span
        >
        <span
          v-if="(props.connectionStatus || 'disconnected') === 'disconnected'"
          >🔴 Disconnected - Check console for details</span
        >
      </div>
      <div v-if="props.pendingChanges" class="pending-changes">
        ⏳ Processing changes...
      </div>
      <div class="debug-info" style="font-size: 10px; color: var(--overlay)">
        WS State: {{ props.websocketState || "null" }} | URL:
        {{ props.websocketUrl || "none" }}
      </div>
    </div>
    <hr :style="{ color: 'var(--highlight)' }" />

    <div>
      <b style="margin-bottom: 5px">Preview Settings:</b>
      <div>
        <input type="checkbox" :onchange="previewScopeChange" />
        Fetch full file
      </div>
      <hr :style="{ color: 'var(--highlight)' }" />
      <b style="margin-bottom: 5px">Graph Settings:</b>
      <div style="display: flex">
        <input type="checkbox" v-model="timeoutEnabled" />
        <div>
          Stop layouting after
          <input type="number" v-model="timeoutTime" />
          secs
        </div>
      </div>
      <div :style="{ padding: '5px' }">
        <StyledButton
          text="Restart layout"
          bg="var(--clickable)"
          fg="var(--text)"
          @button-clicked="$emit('toggleLayouter')"
        >
        </StyledButton>
      </div>
      <hr :style="{ color: 'var(--highlight)' }" />
    </div>
  </div>
</template>

<style scoped>
#theme-settings {
  padding: 5px;
}

.connection-section {
  margin-bottom: 10px;
}

.connection-status {
  color: var(--text);
  font-size: 12px;
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
  animation: pulse 1.5s ease-in-out infinite alternate;
  font-size: 12px;
}

@keyframes pulse {
  from {
    opacity: 1;
  }
  to {
    opacity: 0.5;
  }
}

.debug-info {
  font-size: 10px;
  color: var(--overlay);
}
</style>
