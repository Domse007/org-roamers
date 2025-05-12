<script setup lang="ts">
import Preview from "./components/Preview.vue";
import Graph from "./components/Graph.vue";
import Search from "./components/Search.vue";
import SettingsPane from "./components/Settings/SettingsPane.vue";
import { type Ref, ref } from "vue";

const toggleLayouterRef: Ref<boolean> = ref(false);
const toggleLayouter = () => {
  toggleLayouterRef.value = !toggleLayouterRef.value;
};

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
    <Graph
      @open-node="updatePreviewID"
      :count="graphUpdateCount"
      :toggle-layouter="toggleLayouterRef"
    ></Graph>
    <Preview :id="previewID"></Preview>
    <SettingsPane
      @redraw-graph="redrawGraph"
      @toggle-layouter="toggleLayouter"
    ></SettingsPane>
  </main>
</template>

<style scoped></style>
