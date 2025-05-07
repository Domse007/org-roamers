<script setup lang="ts">
import Preview from "./components/Preview.vue";
import Graph from "./components/Graph.vue";
import Search from "./components/Search.vue";
import SettingsPane from "./components/Settings/SettingsPane.vue";
import { type Ref, ref } from "vue";

const previewID: Ref<string> = ref("");
const updatePreviewID = (id: string) => {
  console.log(`Updating ${previewID.value} to ${id}`);
  previewID.value = id;
};

const graphUpdateCount: Ref<number> = ref(0);
const redrawGraph = () => {
  graphUpdateCount.value++;
};
</script>

<template>
  <header></header>
  <main>
    <Search @open-node="updatePreviewID"></Search>
    <Graph @open-node="updatePreviewID" :count="graphUpdateCount"></Graph>
    <Preview :id="previewID"></Preview>
    <SettingsPane @redraw-graph="redrawGraph"></SettingsPane>
  </main>
</template>

<style scoped></style>
