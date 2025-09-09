<script setup lang="ts">
import PreviewFrame from "./components/PreviewFrame.vue";
import GraphView from "./components/GraphView.vue";
import SearchBar from "./components/SearchBar.vue";
import SettingsPane from "./components/Settings/SettingsPane.vue";
import ErrorDialog from "./components/ErrorDialog.vue";
import { onMounted, onUnmounted, type Ref, ref } from "vue";
import { type RoamLink, type RoamNode } from "./types.ts";

const connectionStatus: Ref<"connecting" | "connected" | "disconnected"> = ref("connecting");
const pendingChanges: Ref<boolean> = ref(false);

const toggleLayouterRef: Ref<boolean> = ref(false);
const toggleLayouter = () => {
  toggleLayouterRef.value = !toggleLayouterRef.value;
};

const previewID: Ref<string> = ref("");
const updatePreviewID = (id: string) => {
  console.log(`Updating ${previewID.value} to ${id}`);
  previewID.value = id;
};

const graphUpdateCount: Ref<number> = ref(0);
const redrawGraph = () => {
  graphUpdateCount.value++;
};

const graphUpdatesRef: Ref<{
  nodes: RoamNode[];
  links: RoamLink[];
  removedNodes?: string[];
  removedLinks?: RoamLink[];
} | null> = ref(null);

let websocket: WebSocket | null = null;

const connectWebSocket = () => {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.host}/ws`;

  console.log(`Attempting to connect to WebSocket: ${wsUrl}`);
  websocket = new WebSocket(wsUrl);

  websocket.onopen = () => {
    console.log("WebSocket connected successfully");
    console.log("WebSocket readyState:", websocket?.readyState);
    connectionStatus.value = "connected";
    errorMessage.value = null;
  };

  websocket.onmessage = (event) => {
    try {
      const message = JSON.parse(event.data);
      console.log("WebSocket message received:", message.type, message);

      switch (message.type) {
        case "status_update":
          console.log("Status update - pending changes:", message.pending_changes, "visited node:", message.visited_node);
          pendingChanges.value = message.pending_changes || false;
          if (message.updated_links.length > 0 || message.updated_nodes.length > 0) {
            console.log("Status update with graph changes:", message.updated_nodes.length, "nodes,", message.updated_links.length, "links");
            graphUpdatesRef.value = {
              nodes: message.updated_nodes,
              links: message.updated_links,
            };
          } else {
            graphUpdatesRef.value = null;
          }
          if (message.visited_node != null) {
            updatePreviewID(message.visited_node);
          }
          break;

        case "node_visited":
          console.log("Node visited:", message.node_id);
          updatePreviewID(message.node_id);
          break;

        case "graph_update":
          console.log("Graph update received:", {
            new_nodes: message.new_nodes.length,
            updated_nodes: message.updated_nodes.length,
            new_links: message.new_links.length,
            removed_nodes: message.removed_nodes.length,
            removed_links: message.removed_links.length
          });
          // Handle the enhanced graph update format with detailed changes
          const allNodes = [...message.new_nodes, ...message.updated_nodes];
          const allLinks = message.new_links;

          graphUpdatesRef.value = {
            nodes: allNodes,
            links: allLinks,
            // Include removal information for potential future use
            removedNodes: message.removed_nodes,
            removedLinks: message.removed_links,
          };
          break;

        case "ping":
          // Respond to ping with pong
          console.log("Received ping, sending pong");
          if (websocket && websocket.readyState === WebSocket.OPEN) {
            try {
              websocket.send(JSON.stringify({ type: "pong" }));
              console.log("Pong sent successfully");
            } catch (error) {
              console.error("Failed to send pong:", error);
            }
          } else {
            console.warn("Cannot send pong - WebSocket not open. ReadyState:", websocket?.readyState);
          }
          break;

        default:
          console.log("Unknown WebSocket message type:", message.type);
      }
    } catch (error) {
      console.error("Failed to parse WebSocket message:", error);
    }
  };

  websocket.onerror = (error) => {
    console.error("WebSocket error occurred:", error);
    console.log("WebSocket readyState during error:", websocket?.readyState);
    connectionStatus.value = "disconnected";
    errorMessage.value = "WebSocket connection error.";
  };

  websocket.onclose = (event) => {
    console.log("WebSocket closed - Code:", event.code, "Reason:", event.reason, "WasClean:", event.wasClean);
    console.log("Close event details:", {
      code: event.code,
      reason: event.reason,
      wasClean: event.wasClean,
      timeStamp: event.timeStamp
    });
    connectionStatus.value = "disconnected";

    // Different reconnection strategy based on close code
    let reconnectDelay = 3000;
    if (event.code === 1006) { // Abnormal closure
      console.warn("Abnormal closure detected, using longer reconnect delay");
      reconnectDelay = 5000;
    } else if (event.code === 1000) { // Normal closure
      console.log("Normal closure, shorter reconnect delay");
      reconnectDelay = 1000;
    }

    setTimeout(() => {
      console.log(`Attempting reconnection after ${reconnectDelay}ms delay`);
      connectionStatus.value = "connecting";
      connectWebSocket();
    }, reconnectDelay);
  };
};

onMounted(() => {
  console.log("Component mounted, initializing WebSocket connection");
  connectWebSocket();

  // Add periodic connection health check
  setInterval(() => {
    if (websocket) {
      console.log("WebSocket health check - ReadyState:", websocket.readyState, "Status:", connectionStatus.value);
      if (websocket.readyState === WebSocket.CLOSED && connectionStatus.value !== "connecting") {
        console.warn("WebSocket is closed but status shows connected, forcing reconnection");
        connectionStatus.value = "connecting";
        connectWebSocket();
      }
    }
  }, 10000); // Check every 10 seconds
});

onUnmounted(() => {
  console.log("Component unmounting, closing WebSocket");
  if (websocket) {
    websocket.close(1000, "Component unmounting");
  }
});

const errorMessage: Ref<string | null> = ref(null);
const closeError = () => (errorMessage.value = null);
</script>

<template>
  <header>
    <!-- Connection Status Indicator -->
    <div class="status-bar">
      <div class="connection-status" :class="connectionStatus">
        <span v-if="connectionStatus === 'connected'">🟢 Connected ({{ websocket?.readyState }})</span>
        <span v-if="connectionStatus === 'connecting'">🟡 Connecting...</span>
        <span v-if="connectionStatus === 'disconnected'">🔴 Disconnected - Check console for details</span>
      </div>
      <div v-if="pendingChanges" class="pending-changes">
        ⏳ Processing changes...
      </div>
      <div class="debug-info" style="font-size: 10px; color: var(--overlay);">
        WS State: {{ websocket?.readyState || 'null' }} | URL: {{ websocket?.url || 'none' }}
      </div>
    </div>
  </header>
  <main>
    <ErrorDialog v-if="errorMessage != null" @dialog-close="closeError">{{
      errorMessage
    }}</ErrorDialog>
    <SearchBar @open-node="updatePreviewID"></SearchBar>
    <GraphView
      @open-node="updatePreviewID"
      :count="graphUpdateCount"
      :toggle-layouter="toggleLayouterRef"
      :zoom-node="previewID"
      :updates="graphUpdatesRef"
    ></GraphView>
    <PreviewFrame
      :id="previewID"
      @preview-switch="updatePreviewID"
    ></PreviewFrame>
    <SettingsPane
      @redraw-graph="redrawGraph"
      @toggle-layouter="toggleLayouter"
    ></SettingsPane>
  </main>
</template>

<style scoped>
.status-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 30px;
  background-color: var(--surface);
  border-bottom: 1px solid var(--overlay);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 10px;
  font-family: var(--font);
  font-size: 12px;
  z-index: 1000;
}

.connection-status {
  color: var(--text);
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
}

@keyframes pulse {
  from { opacity: 1; }
  to { opacity: 0.5; }
}

/* Adjust main content to account for status bar */
main {
  padding-top: 30px;
}
</style>
