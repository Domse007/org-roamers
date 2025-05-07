<script setup lang="ts">
import { generalSettings } from "../../settings.ts";
const previewScopeChange = () => {
  generalSettings.showEntireFile = !generalSettings.showEntireFile;
};
</script>

<template>
  <div id="theme-settings">
    <b style="margin-bottom: 5px">General settings:</b>
    <div>
      <input type="checkbox" :onchange="previewScopeChange" />
      Fetch full file
    </div>
  </div>
</template>

<style scoped>
#theme-settings {
  padding: 5px;
}
</style>
