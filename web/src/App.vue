<script setup lang="ts">
import Preview from "./components/Preview.vue";
import Graph from "./components/Graph.vue";
import Search from "./components/Search.vue";
import SettingsPane from "./components/Settings/SettingsPane.vue";
import { onMounted, type Ref, ref } from "vue";
import { type ServerStatus } from "./types.ts";
import { STATUS_INTERVAL } from "./settings.ts";

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

onMounted(() => {
  setInterval(() => {
    console.log("Running status check");
    fetch("/status")
      .then((resp) => resp.json())
      .then((text) => JSON.parse(text))
      .then((json: ServerStatus) => {
        if (json.pending_changes) {
          redrawGraph();
        }
      });
  }, STATUS_INTERVAL);
});
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
