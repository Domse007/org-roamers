<script setup lang="ts">
const props = defineProps<{ display: string; id: string }>();
</script>

<template>
  <div class="suggestion" @click="$emit('openNode', id)">{{ display }}</div>
</template>

<style scoped>
.suggestion {
  padding: 5px;
  cursor: pointer;
  background-color: var(--base);
}

.suggestion:hover {
  background-color: var(--surface);
}
</style>
