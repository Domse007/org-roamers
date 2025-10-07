<script setup lang="ts">
import PreviewFrame from "./components/PreviewFrame.vue";
import GraphView from "./components/GraphView.vue";
import SearchBar, { type SearchBarMethods } from "./components/SearchBar.vue";
import SettingsPane from "./components/Settings/SettingsPane.vue";
import ErrorDialog from "./components/ErrorDialog.vue";
import { onMounted, onUnmounted, type Ref, ref, provide } from "vue";
import { type RoamLink, type RoamNode } from "./types.ts";

const connectionStatus: Ref<"connecting" | "connected" | "disconnected"> =
  ref("connecting");
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

const clearGraphUpdates = () => {
  console.log("Clearing graph updates to ensure reactivity");
  graphUpdatesRef.value = null;
};

const graphUpdatesRef: Ref<{
  nodes: RoamNode[];
  links: RoamLink[];
  removedNodes?: string[];
  removedLinks?: RoamLink[];
} | null> = ref(null);

// Reference to SearchBar component to handle search responses
const searchBarRef = ref<SearchBarMethods | null>(null);

const websocket: Ref<WebSocket | null> = ref(null);

// Provide WebSocket to child components
provide("websocket", websocket);

const connectWebSocket = () => {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const wsUrl = `${protocol}//${window.location.host}/ws`;

  console.log(`Attempting to connect to WebSocket: ${wsUrl}`);
  websocket.value = new WebSocket(wsUrl);

  websocket.value.onopen = () => {
    console.log("WebSocket connected successfully");
    console.log("WebSocket readyState:", websocket.value?.readyState);
    connectionStatus.value = "connected";
    errorMessage.value = null;
  };

  websocket.value.onmessage = (event) => {
    try {
      const message = JSON.parse(event.data);
      console.log("WebSocket message received:", message.type, message);

      switch (message.type) {
        case "status_update":
          console.log(
            "Status update - pending changes:",
            message.pending_changes,
            "visited node:",
            message.visited_node,
          );
          pendingChanges.value = message.pending_changes || false;
          if (
            message.updated_links.length > 0 ||
            message.updated_nodes.length > 0
          ) {
            console.log(
              "Status update with graph changes:",
              message.updated_nodes.length,
              "nodes,",
              message.updated_links.length,
              "links",
            );
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
            removed_links: message.removed_links.length,
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
          if (
            websocket.value &&
            websocket.value.readyState === WebSocket.OPEN
          ) {
            try {
              websocket.value.send(JSON.stringify({ type: "pong" }));
              console.log("Pong sent successfully");
            } catch (error) {
              console.error("Failed to send pong:", error);
            }
          } else {
            console.warn(
              "Cannot send pong - WebSocket not open. ReadyState:",
              websocket.value?.readyState,
            );
          }
          break;

        case "search_response":
          // Forward search responses to SearchBar component
          if (searchBarRef.value) {
            searchBarRef.value.handleSearchResponse(message);
          } else {
            console.warn("SearchBar component not available");
          }
          break;

        case "SearchConfigurationResponse":
          // Forward search configuration to SearchBar component
          console.log(
            "Search configuration received:",
            message.config.length,
            "providers",
          );
          if (searchBarRef.value) {
            searchBarRef.value.handleSearchConfigResponse(message);
          } else {
            console.warn("SearchBar component not available");
          }
          break;

        default:
          console.log("Unknown WebSocket message type:", message.type);
      }
    } catch (error) {
      console.error("Failed to parse WebSocket message:", error);
    }
  };

  websocket.value.onerror = (error) => {
    console.error("WebSocket error occurred:", error);
    console.log(
      "WebSocket readyState during error:",
      websocket.value?.readyState,
    );
    connectionStatus.value = "disconnected";
    errorMessage.value = "WebSocket connection error.";
  };

  websocket.value.onclose = (event) => {
    console.log(
      "WebSocket closed - Code:",
      event.code,
      "Reason:",
      event.reason,
      "WasClean:",
      event.wasClean,
    );
    console.log("Close event details:", {
      code: event.code,
      reason: event.reason,
      wasClean: event.wasClean,
      timeStamp: event.timeStamp,
    });
    connectionStatus.value = "disconnected";

    // Different reconnection strategy based on close code
    let reconnectDelay = 3000;
    if (event.code === 1006) {
      // Abnormal closure
      console.warn("Abnormal closure detected, using longer reconnect delay");
      reconnectDelay = 5000;
    } else if (event.code === 1000) {
      // Normal closure
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
    if (websocket.value) {
      console.log(
        "WebSocket health check - ReadyState:",
        websocket.value.readyState,
        "Status:",
        connectionStatus.value,
      );
      if (
        websocket.value.readyState === WebSocket.CLOSED &&
        connectionStatus.value !== "connecting"
      ) {
        console.warn(
          "WebSocket is closed but status shows connected, forcing reconnection",
        );
        connectionStatus.value = "connecting";
        connectWebSocket();
      }
    }
  }, 10000); // Check every 10 seconds
});

onUnmounted(() => {
  console.log("Component unmounting, closing WebSocket");
  if (websocket.value) {
    websocket.value.close(1000, "Component unmounting");
  }
});

const errorMessage: Ref<string | null> = ref(null);
const closeError = () => (errorMessage.value = null);

// Handle errors from child components
const handleError = (error: string) => {
  console.error("Application error:", error);
  errorMessage.value = error;
};
</script>

<template>
  <main>
    <ErrorDialog v-if="errorMessage != null" @dialog-close="closeError">{{
      errorMessage
    }}</ErrorDialog>
    <SearchBar
      ref="searchBarRef"
      @open-node="updatePreviewID"
      @error="handleError"
    ></SearchBar>
    <GraphView
      @open-node="updatePreviewID"
      @updates-processed="clearGraphUpdates"
      @error="handleError"
      :count="graphUpdateCount"
      :toggle-layouter="toggleLayouterRef"
      :zoom-node="previewID"
      :updates="graphUpdatesRef"
    ></GraphView>
    <PreviewFrame
      :id="previewID"
      @preview-switch="updatePreviewID"
      @error="handleError"
    ></PreviewFrame>
    <SettingsPane
      @redraw-graph="redrawGraph"
      @toggle-layouter="toggleLayouter"
      :connectionStatus="connectionStatus"
      :pendingChanges="pendingChanges"
      :websocketState="websocket?.readyState"
      :websocketUrl="websocket?.url"
    ></SettingsPane>
  </main>
</template>

<style scoped></style>
