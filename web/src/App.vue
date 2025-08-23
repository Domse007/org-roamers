<script setup lang="ts">
import PreviewFrame from "./components/PreviewFrame.vue";
import GraphView from "./components/GraphView.vue";
import SearchBar from "./components/SearchBar.vue";
import SettingsPane from "./components/Settings/SettingsPane.vue";
import ErrorDialog from "./components/ErrorDialog.vue";
import { onMounted, type Ref, ref } from "vue";
import { type RoamLink, type RoamNode, type ServerStatus } from "./types.ts";
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

const graphUpdatesRef: Ref<{ nodes: RoamNode[]; links: RoamLink[] } | null> =
  ref(null);

onMounted(() => {
  setInterval(() => {
    console.log("Running status check");
    fetch("/status")
      .then((resp) => resp.json())
      .then((json: ServerStatus) => {
        if (json.updated_links.length > 0 || json.updated_nodes.length > 0) {
          graphUpdatesRef.value = {
            nodes: json.updated_nodes,
            links: json.updated_links,
          };
        } else {
          graphUpdatesRef.value = null;
        }
        if (json.visited_node != null) {
          updatePreviewID(json.visited_node);
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
    <SearchBar @open-node="updatePreviewID"></SearchBar>
    <GraphView
      @open-node="updatePreviewID"
      :count="graphUpdateCount"
      :toggle-layouter="toggleLayouterRef"
      :zoom-node="previewID"
      :updates="graphUpdatesRef"
    ></GraphView>
    <PreviewFrame
      :id="previewID"
      @preview-switch="updatePreviewID"
    ></PreviewFrame>
    <SettingsPane
      @redraw-graph="redrawGraph"
      @toggle-layouter="toggleLayouter"
    ></SettingsPane>
  </main>
</template>

<style scoped></style>
