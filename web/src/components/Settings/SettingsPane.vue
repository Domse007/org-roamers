<script setup lang="ts">
import SettingsTheme from "./SettingsTheme.vue";
import SettingsGeneral from "./SettingsGeneral.vue";
import SettingsFilter from "./SettingsFilter.vue";
import { ref, type Ref } from "vue";

const settingsPages: string[] = ["General", "Theme", "Filter"];
const activePage: Ref<string> = ref(settingsPages[0]);

const shown: Ref<"none" | ""> = ref("none");

const inv_shown = () => {
  if (shown.value == "none") return "";
  else return "none";
};

const resize = () => {
  if (shown.value == "none") shown.value = "";
  else shown.value = "none";
};

const listButtonColor = (title: string) => {
  if (title == activePage.value) return "var(--surface)";
  else return "var(--clickable)";
};

const switchSettingsPage = (page: any) => {
  activePage.value = page.target.innerHTML;
};
</script>

<template>
  <div
    class="settings-button"
    id="settings-open-button"
    :onclick="resize"
    :style="{ display: inv_shown() }"
  >
    ⚙
  </div>
  <div id="settings-pane" :style="{ display: shown }">
    <div id="settings-pane-header">
      <b>Settings</b>
      <button class="settings-button" aria-label="Close" :onclick="resize">
        &times;
      </button>
    </div>
    <div id="settings-pane-inner">
      <div id="settings-pane-buttons-wrapper">
        <div
          class="settings-button list-button"
          v-for="page in settingsPages"
          :style="{ backgroundColor: listButtonColor(page) }"
          :onclick="switchSettingsPage"
        >
          {{ page }}
        </div>
      </div>
      <div style="flex: 1; width: 100%; overflow: scroll">
        <SettingsTheme
          v-if="activePage == 'Theme'"
          @redraw-graph="$emit('redrawGraph')"
        ></SettingsTheme>
        <SettingsGeneral v-if="activePage == 'General'"></SettingsGeneral>
        <SettingsFilter v-if="activePage == 'Filter'"></SettingsFilter>
      </div>
    </div>
  </div>
</template>

<style scoped>
#settings-pane {
  background-color: var(--surface);
  font-family: var(--font);
  color: var(--text);
  position: absolute;
  bottom: 0px;
  left: 0px;
  margin: 10px;
  padding: 5px;
  border-radius: 2px;
}

#settings-pane-header {
  display: flex;
  justify-content: space-between;
  border-bottom: 2px solid var(--highlight);
  margin-bottom: 5px;
  padding-bottom: 5px;
}

#settings-pane-inner {
  width: 100%;
  display: flex;
  min-height: 400px;
  min-width: 300px;
  height: 50vh;
  width: 40vw;
}

#settings-pane-buttons-wrapper {
  min-width: 125px;
  border-radius: 2px;
  padding: 5px 0px 5px 5px;
  box-sizing: border-box;
  background-color: var(--base);
  height: 100%;
}

.settings-button {
  background: var(--clickable);
  border: none;
  cursor: pointer;
  border-radius: 5px;
  transition:
    background 0.2s,
    color 0.2s;
}

.settings-button:hover,
.settings-button:focus {
  filter: brightness(125%);
}

.list-button {
  padding: 5px;
  margin-bottom: 5px;
  border-radius: 5px 0px 0px 5px;
}

#settings-open-button {
  position: absolute;
  bottom: 10px;
  left: 10px;
  font-size: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 50px;
  min-width: 50px;
  height: 50px;
  min-height: 50px;
}
</style>
