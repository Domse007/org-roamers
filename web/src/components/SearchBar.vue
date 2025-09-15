<script setup lang="ts">
import { ref, type Ref, inject, onMounted } from "vue";
import {
  type SearchRequestMessage,
  type SearchResponseMessage,
} from "../types.ts";
import SearchSuggestion from "./SearchSuggestion.vue";

const searchSuggestions: Ref<
  { display: string; id: string; tags: string[] }[]
> = ref([]);
const searchterm: Ref<string> = ref("");
const showSuggestions: Ref<boolean> = ref(false);
const selectedIndex: Ref<number> = ref(-1);

// Get WebSocket from parent component
const websocket = inject<Ref<WebSocket | null>>("websocket", ref(null));
const pendingSearchRequests = new Map<
  string,
  (results: { display: string; id: string; tags: string[] }[]) => void
>();

// Generate unique request IDs
const generateRequestId = () =>
  `search_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

const search = async (
  query: string,
): Promise<{ display: string; id: string; tags: string[] }[]> => {
  return new Promise((resolve, reject) => {
    if (!websocket.value || websocket.value.readyState !== WebSocket.OPEN) {
      console.error("WebSocket is not connected");
      resolve([]); // Return empty results if WebSocket is not available
      return;
    }

    if (!query.trim()) {
      resolve([]);
      return;
    }

    const requestId = generateRequestId();
    const searchMessage: SearchRequestMessage = {
      type: "search_request",
      query: query.trim(),
      request_id: requestId,
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
      console.error("Failed to send search request:", error);
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
    console.warn(
      `Received search response for unknown request: ${message.request_id}`,
    );
  }
};

const InputHandler = () => {
  showSuggestions.value = true;
  selectedIndex.value = -1; // Reset selection on new input
  search(searchterm.value).then(
    (results: { display: string; id: string; tags: string[] }[]) => {
      searchSuggestions.value = results;
      console.log("Search results:", searchSuggestions.value);
    },
  );
};

const searchOnLeave = () => {
  setTimeout(() => {
    showSuggestions.value = false;
    selectedIndex.value = -1;
  }, 150);
};

const searchOnFocus = () => {
  if (searchterm.value.trim()) {
    showSuggestions.value = true;
  }
};

// Handle keyboard navigation
const handleKeyDown = (event: KeyboardEvent) => {
  if (!showSuggestions.value || searchSuggestions.value.length === 0) {
    return;
  }

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      selectedIndex.value = Math.min(selectedIndex.value + 1, searchSuggestions.value.length - 1);
      break;
    case 'ArrowUp':
      event.preventDefault();
      selectedIndex.value = Math.max(selectedIndex.value - 1, -1);
      break;
    case 'Enter':
      event.preventDefault();
      if (selectedIndex.value >= 0 && selectedIndex.value < searchSuggestions.value.length) {
        const selectedItem = searchSuggestions.value[selectedIndex.value];
        emit('openNode', selectedItem.id);
        showSuggestions.value = false;
        selectedIndex.value = -1;
        searchterm.value = '';
      }
      break;
    case 'Escape':
      event.preventDefault();
      showSuggestions.value = false;
      selectedIndex.value = -1;
      break;
  }
};

// Handle suggestion click
const onSuggestionClick = (id: string) => {
  emit('openNode', id);
  showSuggestions.value = false;
  selectedIndex.value = -1;
  searchterm.value = '';
};

// Handle suggestion hover
const onSuggestionHover = (index: number) => {
  selectedIndex.value = index;
};

// Export types for parent components
export interface SearchBarMethods {
  handleSearchResponse: (message: SearchResponseMessage) => void;
}

// Expose the search response handler to the parent component
defineExpose<SearchBarMethods>({
  handleSearchResponse,
});

const emit = defineEmits(['openNode']);
</script>

<template>
  <div class="search-wrapper">
    <div class="search-input-container">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 16 16" fill="none">
        <path d="M7 12C9.76142 12 12 9.76142 12 7C12 4.23858 9.76142 2 7 2C4.23858 2 2 4.23858 2 7C2 9.76142 4.23858 12 7 12Z" stroke="currentColor" stroke-width="1.5"/>
        <path d="M15 15L10.5 10.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
      <input
        ref="searchInput"
        class="search-input"
        placeholder="Search nodes..."
        type="search"
        @input="InputHandler"
        @focus="searchOnFocus"
        @blur="searchOnLeave"
        @keydown="handleKeyDown"
        v-model="searchterm"
        autocomplete="off"
        spellcheck="false"
      />
      <button 
        v-if="searchterm.length > 0"
        class="search-clear"
        @click="searchterm = ''; showSuggestions = false; selectedIndex = -1;"
        tabindex="-1"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
    </div>
    
    <div v-if="showSuggestions && (searchSuggestions.length > 0 || searchterm.trim())" class="search-suggestions">
      <div v-if="searchSuggestions.length === 0 && searchterm.trim()" class="search-no-results">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M7 12C9.76142 12 12 9.76142 12 7C12 4.23858 9.76142 2 7 2C4.23858 2 2 4.23858 2 7C2 9.76142 4.23858 12 7 12Z" stroke="currentColor" stroke-width="1.5"/>
          <path d="M15 15L10.5 10.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        <span>No results found for "{{ searchterm }}"</span>
      </div>
      <SearchSuggestion
        v-for="(item, index) in searchSuggestions"
        :key="item.id"
        :display="item.display"
        :id="item.id"
        :tags="item.tags"
        :isSelected="index === selectedIndex"
        @click="onSuggestionClick(item.id)"
        @mouseenter="onSuggestionHover(index)"
      />
    </div>
  </div>
</template>

<style scoped>
.search-wrapper {
  width: min(600px, 45vw);
  position: absolute;
  top: 12px;
  left: 12px;
  z-index: 45;
  font-family: var(--font);
  background: linear-gradient(135deg, 
    color-mix(in srgb, var(--surface) 95%, var(--base)), 
    var(--surface)
  );
  border: 1px solid color-mix(in srgb, var(--highlight) 30%, transparent);
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(6px);
  overflow: hidden;
}

.search-input-container {
  position: relative;
  display: flex;
  align-items: center;
  background: var(--base);
  border-radius: 6px 6px 0 0;
}

.search-icon {
  position: absolute;
  left: 10px;
  color: var(--overlay);
  pointer-events: none;
  z-index: 1;
}

.search-input {
  width: 100%;
  border: none;
  outline: none;
  background: transparent;
  color: var(--text);
  font-family: var(--font);
  font-size: 14px;
  padding: 10px 14px 10px 36px;
  border-radius: 6px 6px 0 0;
}

.search-input::placeholder {
  color: var(--overlay);
}

.search-input:focus {
  box-shadow: inset 0 0 0 1px var(--highlight);
}

.search-clear {
  position: absolute;
  right: 6px;
  width: 22px;
  height: 22px;
  border: none;
  border-radius: 3px;
  background: color-mix(in srgb, var(--overlay) 20%, transparent);
  color: var(--overlay);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.search-clear:hover {
  background: color-mix(in srgb, var(--overlay) 30%, transparent);
  color: var(--text);
}

.search-suggestions {
  max-height: 300px;
  overflow-y: auto;
  border-top: 1px solid color-mix(in srgb, var(--highlight) 20%, transparent);
  background: var(--surface);
}

.search-no-results {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px;
  color: var(--overlay);
  font-size: 13px;
  text-align: center;
  justify-content: center;
}

/* Custom scrollbar for suggestions */
.search-suggestions::-webkit-scrollbar {
  width: 6px;
}

.search-suggestions::-webkit-scrollbar-track {
  background: color-mix(in srgb, var(--base) 50%, transparent);
}

.search-suggestions::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--text) 30%, transparent);
  border-radius: 3px;
}

.search-suggestions::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--text) 50%, transparent);
}

/* Responsive design */
@media (max-width: 768px) {
  .search-wrapper {
    width: min(350px, 90vw);
    top: 8px;
  }
  
  .search-input {
    font-size: 16px; /* Prevents zoom on mobile */
    padding: 9px 12px 9px 32px;
  }
  
  .search-icon {
    left: 8px;
  }
  
  .search-clear {
    right: 4px;
  }
}

@media (max-width: 480px) {
  .search-wrapper {
    width: min(320px, 95vw);
  }
}
</style>
