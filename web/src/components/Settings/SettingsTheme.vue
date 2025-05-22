<script setup lang="ts">
import { THEMES } from "../../theme_list.ts";
import ThemeEntry from "./ThemeEntry.vue";
</script>

<template>
  <div id="theme-settings">
    <b style="margin-bottom: 5px">List of themes:</b>
    <div v-for="theme in THEMES" :key="theme.name">
      <ThemeEntry
        :theme="theme"
        @redraw-graph="$emit('redrawGraph')"
      ></ThemeEntry>
    </div>
  </div>
</template>

<style scoped>
#theme-settings {
  padding: 5px;
}
</style>
