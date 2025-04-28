<script setup lang="ts">
import Preview from "./components/Preview.vue";
import Graph from "./components/Graph.vue";
import Search from "./components/Search.vue";
import { type Ref, ref } from "vue";

const previewID: Ref<string> = ref("");
const updatePreviewID = (id: string) => {
  console.log(`Updating ${previewID.value} to ${id}`);
  previewID.value = id;
};
</script>

<template>
  <header></header>
  <main>
    <Search @open-node="updatePreviewID"></Search>
    <Graph></Graph>
    <Preview :id="previewID"></Preview>
  </main>
</template>

<style scoped></style>
