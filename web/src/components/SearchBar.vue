<script setup lang="ts">
import { ref, type Ref, inject, onMounted } from "vue";
import { type SearchRequestMessage, type SearchResponseMessage } from "../types.ts";
import SearchSuggestion from "./SearchSuggestion.vue";

const searchSuggestions: Ref<
  { display: string; id: string; tags: string[] }[]
> = ref([]);
const searchterm: Ref<string> = ref("");
const showSuggestions: Ref<boolean> = ref(false);

// Get WebSocket from parent component
const websocket = inject<Ref<WebSocket | null>>('websocket', ref(null));
const pendingSearchRequests = new Map<string, (results: { display: string; id: string; tags: string[] }[]) => void>();

// Generate unique request IDs
const generateRequestId = () => `search_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

const search = async (query: string): Promise<{ display: string; id: string; tags: string[] }[]> => {
  return new Promise((resolve, reject) => {
    if (!websocket.value || websocket.value.readyState !== WebSocket.OPEN) {
      console.error('WebSocket is not connected');
      resolve([]); // Return empty results if WebSocket is not available
      return;
    }

    if (!query.trim()) {
      resolve([]);
      return;
    }

    const requestId = generateRequestId();
    const searchMessage: SearchRequestMessage = {
      type: 'search_request',
      query: query.trim(),
      request_id: requestId
    };

    // Store the promise resolver
    pendingSearchRequests.set(requestId, resolve);

    // Set a timeout to clean up if no response is received
    setTimeout(() => {
      if (pendingSearchRequests.has(requestId)) {
        pendingSearchRequests.delete(requestId);
        console.warn(`Search request ${requestId} timed out`);
        resolve([]); // Return empty results on timeout
      }
    }, 5000); // 5 second timeout

    try {
      websocket.value.send(JSON.stringify(searchMessage));
    } catch (error) {
      console.error('Failed to send search request:', error);
      pendingSearchRequests.delete(requestId);
      resolve([]);
    }
  });
};

// Handle search responses from WebSocket
const handleSearchResponse = (message: SearchResponseMessage) => {
  const resolver = pendingSearchRequests.get(message.request_id);
  if (resolver) {
    resolver(message.results);
    pendingSearchRequests.delete(message.request_id);
  } else {
    console.warn(`Received search response for unknown request: ${message.request_id}`);
  }
};

const InputHandler = () => {
  showSuggestions.value = true;
  search(searchterm.value).then((results: { display: string; id: string; tags: string[] }[]) => {
    searchSuggestions.value = results;
    console.log('Search results:', searchSuggestions.value);
  });
};

const searchOnLeave = () => {
  setTimeout(() => (showSuggestions.value = false), 100);
};

// Export types for parent components
export interface SearchBarMethods {
  handleSearchResponse: (message: SearchResponseMessage) => void;
}

// Expose the search response handler to the parent component
defineExpose<SearchBarMethods>({
  handleSearchResponse
});
</script>

<template>
  <div id="search-wrapper">
    <input
      id="search-input"
      placeholder="Search for nodes..."
      type="search"
      @input="InputHandler"
      @focus="InputHandler"
      @blur="searchOnLeave"
      v-model="searchterm"
    />
    <div id="search-suggestion-wrapper">
      <div v-if="showSuggestions">
        <SearchSuggestion
          @open-node="(id) => $emit('openNode', id)"
          v-for="item in searchSuggestions"
          :key="item.id"
          :display="item.display"
          :id="item.id"
          :tags="item.tags"
        >
        </SearchSuggestion>
      </div>
    </div>
  </div>
</template>

<style scoped>
#search-wrapper {
  width: 40%;
  min-width: 400px;
  margin: 5px;
  position: absolute;
  z-index: 100;
  padding: 5px;
  font-family: var(--font);
  background-color: var(--overlay);
  border-radius: 5px;
  transition: top 0.3s ease-in-out;
}

#search-input {
  width: 100%;
  border: 3px solid var(--highlight);
  border-radius: 5px;
  background-color: var(--surface);
  color: var(--text);
  padding: 5px;
}

#search-suggestion-wrapper {
  background-color: var(--surface);
  color: var(--text);
  max-height: 500px;
  overflow-y: scroll;
  overflow-x: hidden;
}
</style>
