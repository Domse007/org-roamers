<script setup lang="ts">
import { ref, computed, inject, type Ref } from "vue";
import type {
  SearchRequestMessage,
  SearchResponseMessage,
  SearchConfigurationRequestMessage,
  SearchConfigurationResponseMessage,
  SearchResultEntry,
} from "../types";
import ProviderGroup from "./ProviderGroup.vue";

/** Search term entered by the user */
const searchterm = ref<string>("");

/** Whether to show the suggestions dropdown */
const showSuggestions = ref<boolean>(false);

/** Currently selected index across all visible results */
const selectedIndex = ref<number>(-1);

/** Mapping of provider IDs to provider names */
const providerConfig = ref<Map<number, string>>(new Map());

/** Mapping of provider IDs to their search results */
const providerResults = ref<Map<number, SearchResultEntry[]>>(new Map());

/** Current search request ID to track responses */
const currentRequestId = ref<string>("");

/** WebSocket connection injected from parent */
const websocket = inject<Ref<WebSocket | null>>("websocket", ref(null));

/** Reference to the search input element */
const searchInput = ref<HTMLInputElement | null>(null);

/** Flag to track if mouse is down on a search element */
const isMouseDownOnSearch = ref<boolean>(false);

/**
 * Generate a unique request ID for tracking search requests.
 * @returns Unique request ID string
 */
const generateRequestId = (): string =>
  `search_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

/**
 * Initialize search configuration by requesting provider metadata from server.
 * This must be called before performing any searches.
 */
const initializeSearchConfig = () => {
  if (!websocket.value || websocket.value.readyState !== WebSocket.OPEN) {
    console.error("WebSocket is not connected");
    return;
  }

  const configRequest: SearchConfigurationRequestMessage = {
    type: "SearchConfigurationRequest",
  };

  try {
    websocket.value.send(JSON.stringify(configRequest));
  } catch (error) {
    console.error("Failed to request search configuration:", error);
  }
};

/**
 * Handle search configuration response from server.
 * Populates the provider config map with provider IDs and names.
 * @param message - Configuration response containing provider metadata
 */
const handleSearchConfigResponse = (
  message: SearchConfigurationResponseMessage,
) => {
  providerConfig.value.clear();
  for (const [id, name] of message.config) {
    providerConfig.value.set(id, name);
    // Initialize empty results array for each provider
    if (!providerResults.value.has(id)) {
      providerResults.value.set(id, []);
    }
  }
  console.log("Search configuration loaded:", providerConfig.value);
};

/**
 * Perform a search query by sending request to server via WebSocket.
 * Results will be streamed back asynchronously.
 * @param query - Search query string
 */
const search = (query: string) => {
  if (!websocket.value || websocket.value.readyState !== WebSocket.OPEN) {
    console.error("WebSocket is not connected");
    return;
  }

  if (!query.trim()) {
    clearResults();
    return;
  }

  // Clear previous results
  clearResults();

  const requestId = generateRequestId();
  currentRequestId.value = requestId;

  const searchMessage: SearchRequestMessage = {
    type: "search_request",
    query: query.trim(),
    request_id: requestId,
  };

  try {
    websocket.value.send(JSON.stringify(searchMessage));
  } catch (error) {
    console.error("Failed to send search request:", error);
  }
};

/**
 * Handle incoming search result from server.
 * Results are streamed one at a time and grouped by provider.
 * @param message - Search response containing a single result entry
 */
const handleSearchResponse = (message: SearchResponseMessage) => {
  // Ignore results from outdated requests
  if (message.request_id !== currentRequestId.value) {
    return;
  }

  const result = message.results;
  const providerId = result.provider;

  // Get or create results array for this provider
  const results = providerResults.value.get(providerId) || [];
  results.push(result);
  providerResults.value.set(providerId, results);

  // Force reactivity update
  providerResults.value = new Map(providerResults.value);
};

/**
 * Clear all search results and reset state.
 */
const clearResults = () => {
  for (const key of providerResults.value.keys()) {
    providerResults.value.set(key, []);
  }
  providerResults.value = new Map(providerResults.value);
  selectedIndex.value = -1;
};

/**
 * Compute sorted list of provider groups for display.
 * Groups are sorted by provider ID and only include providers with results.
 * @returns Array of provider group data with metadata
 */
const sortedProviderGroups = computed(() => {
  const groups: {
    providerId: number;
    providerName: string;
    results: SearchResultEntry[];
    startIndex: number;
  }[] = [];

  let currentIndex = 0;

  // Sort by provider ID to ensure consistent ordering
  const sortedProviderIds = Array.from(providerResults.value.keys()).sort(
    (a, b) => a - b,
  );

  for (const providerId of sortedProviderIds) {
    const results = providerResults.value.get(providerId) || [];
    if (results.length > 0) {
      groups.push({
        providerId,
        providerName: providerConfig.value.get(providerId) || `Provider ${providerId}`,
        results,
        startIndex: currentIndex,
      });
      // Count only top 10 or all depending on display mode
      // For now we count all for navigation
      currentIndex += results.length;
    }
  }

  return groups;
});

/**
 * Get total count of all visible results across all providers.
 * @returns Total number of results
 */
const totalResultsCount = computed(() => {
  return sortedProviderGroups.value.reduce(
    (sum, group) => sum + group.results.length,
    0,
  );
});

/**
 * Handle input changes in the search field.
 * Triggers a new search query.
 */
const handleInput = () => {
  showSuggestions.value = true;
  selectedIndex.value = -1;
  search(searchterm.value);
};

/**
 * Handle focus event on search input.
 * Requests search configuration and shows suggestions if there's a search term.
 */
const handleFocus = () => {
  // Request configuration on every focus to ensure we have latest providers
  initializeSearchConfig();
  
  if (searchterm.value.trim()) {
    showSuggestions.value = true;
  }
};

/**
 * Handle blur event on search input.
 * Hides suggestions after a short delay to allow clicking on items.
 */
const handleBlur = () => {
  setTimeout(() => {
    // Don't close if we're clicking within the search component
    if (!isMouseDownOnSearch.value) {
      showSuggestions.value = false;
      selectedIndex.value = -1;
    }
  }, 150);
};

/**
 * Handle mousedown on search wrapper to prevent blur from closing.
 */
const handleSearchMouseDown = () => {
  isMouseDownOnSearch.value = true;
};

/**
 * Handle mouseup to reset the flag.
 */
const handleSearchMouseUp = () => {
  isMouseDownOnSearch.value = false;
  // Refocus the input to keep it active
  searchInput.value?.focus();
};

/**
 * Handle keyboard navigation in the search results.
 * Supports arrow keys for navigation, Enter for selection, and Escape to close.
 * @param event - Keyboard event
 */
const handleKeyDown = (event: KeyboardEvent) => {
  if (!showSuggestions.value || totalResultsCount.value === 0) {
    return;
  }

  switch (event.key) {
    case "ArrowDown":
      event.preventDefault();
      selectedIndex.value = Math.min(
        selectedIndex.value + 1,
        totalResultsCount.value - 1,
      );
      break;
    case "ArrowUp":
      event.preventDefault();
      selectedIndex.value = Math.max(selectedIndex.value - 1, -1);
      break;
    case "Enter":
      event.preventDefault();
      if (selectedIndex.value >= 0) {
        const selectedResult = getResultAtIndex(selectedIndex.value);
        if (selectedResult) {
          emit("openNode", selectedResult.id);
          clearSearch();
        }
      }
      break;
    case "Escape":
      event.preventDefault();
      showSuggestions.value = false;
      selectedIndex.value = -1;
      break;
  }
};

/**
 * Get the result entry at a specific global index.
 * @param index - Global index across all provider groups
 * @returns Search result entry or null if not found
 */
const getResultAtIndex = (index: number): SearchResultEntry | null => {
  let currentIndex = 0;
  for (const group of sortedProviderGroups.value) {
    if (index < currentIndex + group.results.length) {
      return group.results[index - currentIndex];
    }
    currentIndex += group.results.length;
  }
  return null;
};

/**
 * Handle click on a search suggestion.
 * Opens the selected node and clears the search.
 * @param id - Node ID to open
 */
const handleSuggestionClick = (id: string) => {
  emit("openNode", id);
  clearSearch();
};

/**
 * Handle hover over a search suggestion.
 * Updates the selected index.
 * @param index - Global index of the hovered item
 */
const handleSuggestionHover = (index: number) => {
  selectedIndex.value = index;
};

/**
 * Clear the search field and hide suggestions.
 */
const clearSearch = () => {
  searchterm.value = "";
  showSuggestions.value = false;
  selectedIndex.value = -1;
  clearResults();
};

/** Emitted events */
const emit = defineEmits<{
  openNode: [id: string];
}>();

/** Methods exposed to parent components */
export interface SearchBarMethods {
  handleSearchResponse: (message: SearchResponseMessage) => void;
  handleSearchConfigResponse: (
    message: SearchConfigurationResponseMessage,
  ) => void;
}

/** Expose methods to parent component */
defineExpose<SearchBarMethods>({
  handleSearchResponse,
  handleSearchConfigResponse,
});
</script>

<template>
  <div class="search-wrapper" @mousedown="handleSearchMouseDown" @mouseup="handleSearchMouseUp">
    <div class="search-input-container">
      <svg
        class="search-icon"
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
      >
        <path
          d="M7 12C9.76142 12 12 9.76142 12 7C12 4.23858 9.76142 2 7 2C4.23858 2 2 4.23858 2 7C2 9.76142 4.23858 12 7 12Z"
          stroke="currentColor"
          stroke-width="1.5"
        />
        <path
          d="M15 15L10.5 10.5"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
        />
      </svg>
      <input
        ref="searchInput"
        class="search-input"
        placeholder="Search nodes..."
        type="search"
        @input="handleInput"
        @focus="handleFocus"
        @blur="handleBlur"
        @keydown="handleKeyDown"
        v-model="searchterm"
        autocomplete="off"
        spellcheck="false"
      />
      <button
        v-if="searchterm.length > 0"
        class="search-clear"
        @click="clearSearch"
        tabindex="-1"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path
            d="M1 1l10 10M11 1L1 11"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </div>

    <div
      v-if="showSuggestions && searchterm.trim()"
      class="search-suggestions"
    >
      <div
        v-if="totalResultsCount === 0"
        class="search-no-results"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path
            d="M7 12C9.76142 12 12 9.76142 12 7C12 4.23858 9.76142 2 7 2C4.23858 2 2 4.23858 2 7C2 9.76142 4.23858 12 7 12Z"
            stroke="currentColor"
            stroke-width="1.5"
          />
          <path
            d="M15 15L10.5 10.5"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          />
        </svg>
        <span>No results found for "{{ searchterm }}"</span>
      </div>

      <ProviderGroup
        v-for="group in sortedProviderGroups"
        :key="group.providerId"
        :provider-name="group.providerName"
        :provider-id="group.providerId"
        :results="group.results"
        :selected-index="selectedIndex"
        :start-index="group.startIndex"
        @click="handleSuggestionClick"
        @hover="handleSuggestionHover"
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
  background: linear-gradient(
    135deg,
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
  max-height: 400px;
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
