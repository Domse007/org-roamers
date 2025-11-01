<script setup lang="ts">
import { ref, onMounted, watch } from "vue";

const props = defineProps<{
  filterTags?: string[];
  excludeTags?: string[];
}>();

const availableTags = ref<string[]>([]);
const selectedTags = ref<string[]>(props.filterTags || []);
const excludedTags = ref<string[]>(props.excludeTags || []);
const searchInput = ref("");
const showDropdown = ref(false);

const emit = defineEmits<{
  (e: "filterChange", data: { include: string[]; exclude: string[] }): void;
}>();

watch(
  () => props.filterTags,
  (newTags) => {
    if (newTags) {
      selectedTags.value = [...newTags];
    }
  },
);

watch(
  () => props.excludeTags,
  (newTags) => {
    if (newTags) {
      excludedTags.value = [...newTags];
    }
  },
);

onMounted(async () => {
  try {
    const response = await fetch("/tags", {
      credentials: "include", // Include cookies for authentication
    });
    availableTags.value = await response.json();
  } catch (error) {
    console.error("Failed to load tags:", error);
  }
});

const filteredTags = ref<string[]>([]);
const updateFilteredTags = () => {
  const search = searchInput.value.toLowerCase();
  filteredTags.value = availableTags.value
    .filter((tag) => tag.toLowerCase().includes(search))
    .filter(
      (tag) =>
        !selectedTags.value.includes(tag) && !excludedTags.value.includes(tag),
    )
    .slice(0, 20);
};

const emitChange = () => {
  emit("filterChange", {
    include: selectedTags.value,
    exclude: excludedTags.value,
  });
};

const toggleTag = (tag: string, exclude: boolean = false) => {
  if (exclude) {
    excludedTags.value.push(tag);
  } else {
    selectedTags.value.push(tag);
  }
  emitChange();
  searchInput.value = "";
  showDropdown.value = false;
};

const removeTag = (tag: string) => {
  const includeIndex = selectedTags.value.indexOf(tag);
  if (includeIndex !== -1) {
    selectedTags.value.splice(includeIndex, 1);
  }
  const excludeIndex = excludedTags.value.indexOf(tag);
  if (excludeIndex !== -1) {
    excludedTags.value.splice(excludeIndex, 1);
  }
  emitChange();
};

const toggleTagMode = (tag: string) => {
  const includeIndex = selectedTags.value.indexOf(tag);
  const excludeIndex = excludedTags.value.indexOf(tag);

  if (includeIndex !== -1) {
    selectedTags.value.splice(includeIndex, 1);
    excludedTags.value.push(tag);
  } else if (excludeIndex !== -1) {
    excludedTags.value.splice(excludeIndex, 1);
    selectedTags.value.push(tag);
  }
  emitChange();
};

const clearAll = () => {
  selectedTags.value = [];
  excludedTags.value = [];
  emitChange();
};

const handleInput = () => {
  updateFilteredTags();
  showDropdown.value = searchInput.value.length > 0;
};

const handleFocus = () => {
  updateFilteredTags();
  showDropdown.value = true;
};

const handleBlur = () => {
  setTimeout(() => (showDropdown.value = false), 200);
};
</script>

<template>
  <div class="settings-filter">
    <section class="settings-section">
      <h3 class="section-title">Filter by Tags</h3>
      <p class="section-description">
        Show only nodes with selected tags (OR logic)
      </p>

      <div class="tag-input-container">
        <input
          v-model="searchInput"
          @input="handleInput"
          @focus="handleFocus"
          @blur="handleBlur"
          placeholder="Search tags..."
          class="tag-search"
        />
        <div
          v-if="showDropdown && filteredTags.length > 0"
          class="tag-dropdown"
        >
          <div v-for="tag in filteredTags" :key="tag" class="tag-option-row">
            <span class="tag-option-label" @click="toggleTag(tag, false)">{{
              tag
            }}</span>
            <button
              @click="toggleTag(tag, true)"
              class="exclude-button"
              title="Exclude"
            >
              −
            </button>
          </div>
        </div>
      </div>

      <div v-if="selectedTags.length > 0" class="selected-tags">
        <div
          v-for="tag in selectedTags"
          :key="tag"
          class="tag-chip tag-include"
        >
          <span @click="toggleTagMode(tag)" class="tag-label">{{ tag }}</span>
          <button @click="removeTag(tag)" class="tag-remove">×</button>
        </div>
      </div>

      <div v-if="excludedTags.length > 0" class="excluded-tags">
        <div
          v-for="tag in excludedTags"
          :key="tag"
          class="tag-chip tag-exclude"
        >
          <span @click="toggleTagMode(tag)" class="tag-label">{{ tag }}</span>
          <button @click="removeTag(tag)" class="tag-remove">×</button>
        </div>
      </div>

      <button
        v-if="selectedTags.length > 0"
        @click="clearAll"
        class="clear-button"
      >
        Clear Filters
      </button>
    </section>
  </div>
</template>

<style scoped>
.settings-filter {
  padding: 16px;
  height: 100%;
  overflow-y: auto;
  box-sizing: border-box;
}

.settings-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--highlight);
  margin: 0 0 8px 0;
  padding-bottom: 4px;
  border-bottom: 1px solid color-mix(in srgb, var(--highlight) 20%, transparent);
}

.section-description {
  font-size: 13px;
  color: var(--overlay);
  margin: 0 0 20px 0;
  line-height: 1.4;
}

.tag-input-container {
  position: relative;
  margin-bottom: 16px;
}

.tag-search {
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  background: var(--surface);
  border: 1px solid var(--overlay);
  border-radius: 4px;
  color: var(--text);
  box-sizing: border-box;
}

.tag-search:focus {
  outline: none;
  border-color: var(--highlight);
}

.tag-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--surface);
  border: 1px solid var(--overlay);
  border-radius: 4px;
  max-height: 200px;
  overflow-y: auto;
  z-index: 100;
  margin-top: 4px;
}

.tag-option-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  gap: 8px;
}

.tag-option-row:hover {
  background: var(--overlay);
}

.tag-option-label {
  flex: 1;
  padding: 4px;
  font-size: 13px;
  cursor: pointer;
  color: var(--text);
}

.exclude-button {
  padding: 2px 8px;
  background: var(--warn);
  color: var(--base);
  border: none;
  border-radius: 3px;
  font-size: 14px;
  cursor: pointer;
  font-weight: bold;
}

.exclude-button:hover {
  opacity: 0.8;
}

.selected-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 8px;
}

.excluded-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 16px;
}

.tag-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.tag-include {
  background: var(--highlight);
  color: var(--base);
}

.tag-exclude {
  background: var(--warn);
  color: var(--base);
}

.tag-label {
  cursor: pointer;
  user-select: none;
}

.tag-label:hover {
  opacity: 0.8;
}

.tag-remove {
  background: none;
  border: none;
  color: var(--base);
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  padding: 0;
  margin: 0;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.tag-remove:hover {
  opacity: 0.7;
}

.clear-button {
  width: 100%;
  padding: 8px 16px;
  background: var(--warn);
  color: var(--base);
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
}

.clear-button:hover {
  opacity: 0.9;
}
</style>
