<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  links: { display: string; id: string }[];
  title: string;
  iconType: "incoming" | "outgoing";
  defaultExpanded?: boolean;
}>();

const emit = defineEmits<{
  linkClick: [id: string];
}>();

const expanded = ref(false);

const handleLinkClick = (id: string) => {
  emit("linkClick", id);
};
</script>

<template>
  <div class="links-section" v-if="links.length > 0">
    <button
      class="links-section-header"
      @click="expanded = !expanded"
      :title="
        expanded
          ? `Collapse ${title.toLowerCase()}`
          : `Expand ${title.toLowerCase()}`
      "
    >
      <div class="links-section-header-content">
        <!-- Incoming links icon -->
        <svg
          v-if="iconType === 'incoming'"
          width="14"
          height="14"
          viewBox="0 0 14 14"
          fill="none"
        >
          <path
            d="M9.5 11C9.5 11 9.5 12 8 12C6.5 12 4 12 4 12V8.5"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M7.5 12.5L4 9"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M5.5 7.5V5C5.5 4.5 6 4 6.5 4H11C11.5 4 12 4.5 12 5V9.5C12 10 11.5 10.5 11 10.5H8.5"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>

        <!-- Outgoing links icon -->
        <svg v-else width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path
            d="M4.5 3C4.5 3 4.5 2 6 2C7.5 2 10 2 10 2V5.5"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M6.5 1.5L10 5"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M8.5 6.5V9C8.5 9.5 8 10 7.5 10H3C2.5 10 2 9.5 2 9V4.5C2 4 2.5 3.5 3 3.5H5.5"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>

        <span>{{ title }} ({{ links.length }})</span>
      </div>
      <svg
        width="12"
        height="12"
        viewBox="0 0 12 12"
        fill="none"
        class="links-section-chevron"
        :style="{
          transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)',
        }"
      >
        <path
          d="M3 4.5L6 7.5L9 4.5"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <div class="links-section-links" v-show="expanded">
      <button
        class="links-section-link"
        :key="link.id"
        v-for="link in links"
        @click="handleLinkClick(link.id)"
        :title="`Open ${link.display}`"
      >
        <!-- Incoming links icon (smaller) -->
        <svg
          v-if="iconType === 'incoming'"
          width="12"
          height="12"
          viewBox="0 0 12 12"
          fill="none"
        >
          <path
            d="M9.5 11C9.5 11 9.5 12 8 12C6.5 12 4 12 4 12V8.5"
            stroke="currentColor"
            stroke-width="1"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M7.5 12.5L4 9"
            stroke="currentColor"
            stroke-width="1"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M5.5 7.5V5C5.5 4.5 6 4 6.5 4H11C11.5 4 12 4.5 12 5V9.5C12 10 11.5 10.5 11 10.5H8.5"
            stroke="currentColor"
            stroke-width="1"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>

        <!-- Outgoing links icon (smaller) -->
        <svg v-else width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path
            d="M4.5 3C4.5 3 4.5 2 6 2C7.5 2 10 2 10 2V5.5"
            stroke="currentColor"
            stroke-width="1"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M6.5 1.5L10 5"
            stroke="currentColor"
            stroke-width="1"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M8.5 6.5V9C8.5 9.5 8 10 7.5 10H3C2.5 10 2 9.5 2 9V4.5C2 4 2.5 3.5 3 3.5H5.5"
            stroke="currentColor"
            stroke-width="1"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>

        <span>{{ link.display }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.links-section {
  border-top: 1px solid color-mix(in srgb, var(--highlight) 20%, transparent);
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--base) 98%, var(--surface)),
    var(--base)
  );
  flex-shrink: 0;
}

.links-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: transparent;
  font-size: 13px;
  font-weight: 600;
  color: var(--highlight);
  cursor: pointer;
  transition: all 0.15s ease;
  border-bottom: 1px solid color-mix(in srgb, var(--highlight) 15%, transparent);
}

.links-section-header:hover {
  background: color-mix(in srgb, var(--highlight) 10%, transparent);
}

.links-section-header-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.links-section-header-content svg {
  opacity: 0.7;
}

.links-section-chevron {
  opacity: 0.6;
  transition: transform 0.2s ease;
}

.links-section-links {
  max-height: 200px;
  overflow-y: auto;
  padding: 4px;
  transition: all 0.2s ease;
}

.links-section-link {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  margin: 1px 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--clickable);
  font-family: var(--font);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  transition: all 0.15s ease;
}

.links-section-link:hover {
  background: color-mix(in srgb, var(--clickable) 15%, transparent);
  color: color-mix(in srgb, var(--clickable) 110%, white);
}

.links-section-link:active {
  transform: scale(0.98);
}

.links-section-link svg {
  opacity: 0.6;
  flex-shrink: 0;
}

.links-section-link span {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Custom scrollbar styling */
.links-section-links::-webkit-scrollbar {
  width: 6px;
}

.links-section-links::-webkit-scrollbar-track {
  background: color-mix(in srgb, var(--base) 50%, transparent);
  border-radius: 3px;
}

.links-section-links::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--text) 30%, transparent);
  border-radius: 3px;
  transition: background 0.2s ease;
}

.links-section-links::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--text) 50%, transparent);
}

/* Responsive design */
@media (max-width: 480px) {
  .links-section-links {
    max-height: 120px;
  }

  .links-section-link {
    padding: 6px 8px;
    font-size: 12px;
  }
}
</style>
