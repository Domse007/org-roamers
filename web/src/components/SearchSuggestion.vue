<script setup lang="ts">
defineProps<{
  display: string;
  id: string;
  tags: string[];
  isSelected?: boolean;
}>();

const formatTags = (taglist: string[]) => {
  return taglist.join(", ");
};
</script>

<template>
  <div
    class="suggestion"
    :class="{ 'suggestion-selected': isSelected }"
    @click="$emit('click')"
    @mouseenter="$emit('mouseenter')"
  >
    <div class="suggestion-content">
      <div class="suggestion-title">
        {{ display }}
      </div>
      <div v-if="tags.length > 0" class="suggestion-tags">
        {{ formatTags(tags) }}
      </div>
    </div>
    <div class="suggestion-arrow">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <path
          d="M4 2L8 6L4 10"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </div>
  </div>
</template>

<style scoped>
.suggestion {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  cursor: pointer;
  border-bottom: 1px solid color-mix(in srgb, var(--highlight) 10%, transparent);
  position: relative;
}

.suggestion:hover,
.suggestion-selected {
  background: color-mix(in srgb, var(--highlight) 8%, var(--surface));
}

.suggestion-selected {
  border-left: 3px solid var(--highlight);
  padding-left: 9px;
}

.suggestion:last-child {
  border-bottom: none;
}

.suggestion-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.suggestion-title {
  color: var(--text);
  font-weight: 500;
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.suggestion-tags {
  color: var(--overlay);
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.suggestion-arrow {
  display: flex;
  align-items: center;
  color: var(--overlay);
  margin-left: 6px;
  flex-shrink: 0;
}

.suggestion-selected .suggestion-arrow {
  color: var(--highlight);
}

.suggestion-selected .suggestion-title {
  color: var(--highlight);
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .suggestion {
    padding: 8px 10px;
  }

  .suggestion-title {
    font-size: 12px;
  }

  .suggestion-tags {
    font-size: 10px;
  }
}
</style>
