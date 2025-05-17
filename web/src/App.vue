<script setup lang="ts">
import Preview from "./components/Preview.vue";
import Graph from "./components/Graph.vue";
import Search from "./components/Search.vue";
import SettingsPane from "./components/Settings/SettingsPane.vue";
import ErrorDialog from "./components/ErrorDialog.vue";
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
      })
      .catch((error) => {
        console.log(error);
        errorMessage.value = "Failed to get response from server.";
      });
  }, STATUS_INTERVAL);
});

const errorMessage: Ref<string | null> = ref(null);
const closeError = () => (errorMessage.value = null);
</script>

<template>
  <header></header>
  <main>
    <ErrorDialog v-if="errorMessage != null" @dialog-close="closeError">{{
      errorMessage
    }}</ErrorDialog>
    <Search @open-node="updatePreviewID"></Search>
    <Graph
      @open-node="updatePreviewID"
      :count="graphUpdateCount"
      :toggle-layouter="toggleLayouterRef"
      :zoom-node="previewID"
    ></Graph>
    <Preview :id="previewID" @preview-switch="updatePreviewID"></Preview>
    <SettingsPane
      @redraw-graph="redrawGraph"
      @toggle-layouter="toggleLayouter"
    ></SettingsPane>
  </main>
</template>

<style scoped></style>
