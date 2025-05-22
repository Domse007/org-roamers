<script setup lang="ts">
defineProps<{ display: string; id: string; tags: string[] }>();

const formatTags = (taglist: string[]) => {
  return taglist.join(", ");
};
</script>

<template>
  <div class="suggestion" @click="$emit('openNode', id)">
    <div class="suggestion-title">
      {{ display }}
    </div>
    <div v-if="tags.length > 0">&nbsp;&nbsp;-&nbsp;&nbsp;</div>
    <div class="suggestion-tags">
      {{ formatTags(tags) }}
    </div>
  </div>
</template>

<style scoped>
.suggestion {
  overflow-x: hidden;
  width: 100%;
  white-space: nowrap;
  display: flex;
  padding: 5px;
  cursor: pointer;
  background-color: var(--base);
}

.suggestion:hover {
  background-color: var(--surface);
}

.suggestion-title {
  color: var(--highlight);
}

.suggestion-tags {
  color: var(--highlight-2);
}
</style>
