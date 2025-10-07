<script setup lang="ts">
import { ref } from "vue";
import type { SearchResultEntry } from "../types";
import SearchSuggestion from "./SearchSuggestion.vue";

/** Display mode for the provider group results */
type DisplayMode = "top10" | "all" | "collapsed";

/** Props for the ProviderGroup component */
interface Props {
  /** Name of the search provider (e.g., "Default search", "Full text search") */
  providerName: string;
  /** Provider ID used for grouping results */
  providerId: number;
  /** Array of search results for this provider */
  results: SearchResultEntry[];
  /** Currently selected item index across all groups */
  selectedIndex: number;
  /** Starting index of this group in the global list */
  startIndex: number;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  click: [id: string];
  hover: [index: number];
}>();

/** Current display mode for this provider group */
const displayMode = ref<DisplayMode>("top10");

/** Toggle between display modes in cycle: top10 -> all -> collapsed -> top10 */
const cycleDisplayMode = () => {
  if (displayMode.value === "top10") {
    displayMode.value = "all";
  } else if (displayMode.value === "all") {
    displayMode.value = "collapsed";
  } else {
    displayMode.value = "top10";
  }
};

/** Set display mode to show top 10 results */
const showTop10 = () => {
  displayMode.value = "top10";
};

/** Set display mode to show all results */
const showAll = () => {
  displayMode.value = "all";
};

/** Set display mode to collapse (header only) */
const collapse = () => {
  displayMode.value = "collapsed";
};

/**
 * Get the visible results based on current display mode.
 * @returns Array of results to display
 */
const visibleResults = (): SearchResultEntry[] => {
  if (displayMode.value === "collapsed") {
    return [];
  } else if (displayMode.value === "top10") {
    return props.results.slice(0, 10);
  } else {
    return props.results;
  }
};

/**
 * Check if a result is selected.
 * @param index - Local index within this provider group
 * @returns True if this result is selected
 */
const isResultSelected = (index: number): boolean => {
  return props.startIndex + index === props.selectedIndex;
};
</script>

<template>
  <div class="provider-group">
    <div class="provider-header">
      <div class="provider-header-left" @click="cycleDisplayMode">
        <svg v-if="displayMode !== 'collapsed'" class="provider-toggle-icon" width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M12 10L8 6L4 10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        <svg v-else class="provider-toggle-icon" width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M4 6L8 10L12 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        <span class="provider-name">{{ providerName }}</span>
        <span class="provider-count">({{ results.length }})</span>
      </div>
      <div class="provider-controls">
        <button class="provider-control-btn" :class="{ active: displayMode === 'top10' }" @click="showTop10" title="Show top 10 results">
          <svg class="control-icon" width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M1 8C1 8 3.5 3 8 3C12.5 3 15 8 15 8C15 8 12.5 13 8 13C3.5 13 1 8 1 8Z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            <circle cx="8" cy="8" r="2" stroke="currentColor" stroke-width="1.5"/>
          </svg>
        </button>
        <button class="provider-control-btn" :class="{ active: displayMode === 'all' }" @click="showAll" title="Show all results">
          <svg class="control-icon" width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M5 4H14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <path d="M5 8H14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <path d="M5 12H14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <circle cx="2.5" cy="4" r="0.5" fill="currentColor"/>
            <circle cx="2.5" cy="8" r="0.5" fill="currentColor"/>
            <circle cx="2.5" cy="12" r="0.5" fill="currentColor"/>
          </svg>
        </button>
        <button class="provider-control-btn" :class="{ active: displayMode === 'collapsed' }" @click="collapse" title="Collapse">
          <svg class="control-icon" width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M3 8H13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>

    <div v-if="displayMode !== 'collapsed'" class="provider-results">
      <SearchSuggestion
        v-for="(result, index) in visibleResults()"
        :key="result.id"
        :display="result.title"
        :id="result.id"
        :tags="result.tags"
        :isSelected="isResultSelected(index)"
        @click="emit('click', result.id)"
        @mouseenter="emit('hover', startIndex + index)"
      />
      <div v-if="displayMode === 'top10' && results.length > 10" class="provider-show-more" @click="showAll">
        Show {{ results.length - 10 }} more results...
      </div>
    </div>
  </div>
</template>

<style scoped>
.provider-group {
  border-bottom: 1px solid color-mix(in srgb, var(--highlight) 15%, transparent);
}

.provider-group:last-child {
  border-bottom: none;
}

.provider-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: color-mix(in srgb, var(--base) 80%, var(--surface));
  border-bottom: 1px solid color-mix(in srgb, var(--highlight) 10%, transparent);
  user-select: none;
}

.provider-header-left {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  flex: 1;
}

.provider-header-left:hover .provider-name {
  color: var(--highlight);
}

.provider-toggle-icon {
  color: var(--overlay);
  flex-shrink: 0;
  transition: transform 0.2s ease;
}

.provider-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  transition: color 0.2s ease;
}

.provider-count {
  font-size: 11px;
  color: var(--overlay);
  font-weight: 400;
}

.provider-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.provider-control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--overlay);
  cursor: pointer;
  transition: all 0.2s ease;
  padding: 0;
}

.provider-control-btn:hover {
  background: color-mix(in srgb, var(--highlight) 15%, transparent);
  color: var(--text);
}

.provider-control-btn.active {
  background: color-mix(in srgb, var(--highlight) 20%, transparent);
  color: var(--highlight);
}

.control-icon {
  width: 14px;
  height: 14px;
}

.provider-results {
  background: var(--surface);
}

.provider-show-more {
  padding: 10px 12px;
  color: var(--highlight);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  text-align: center;
  border-top: 1px solid color-mix(in srgb, var(--highlight) 8%, transparent);
  background: var(--surface);
}

.provider-show-more:hover {
  background: color-mix(in srgb, var(--highlight) 8%, var(--surface));
}

@media (max-width: 768px) {
  .provider-header {
    padding: 6px 10px;
  }

  .provider-name {
    font-size: 11px;
  }

  .provider-count {
    font-size: 10px;
  }

  .provider-control-btn {
    width: 22px;
    height: 22px;
  }

  .control-icon {
    width: 12px;
    height: 12px;
  }
}
</style>
