<script setup lang="ts">
import { ref, watch, type Ref } from "vue";
import { generalSettings } from "../../settings.ts";
const previewScopeChange = () => {
  generalSettings.showEntireFile = !generalSettings.showEntireFile;
};

const timeoutEnabled: Ref<boolean> = ref(!!generalSettings.stopLayoutAfter);
const timeoutTime: Ref<number> = ref(
  !generalSettings.stopLayoutAfter ? 0 : generalSettings.stopLayoutAfter,
);

watch([timeoutEnabled, timeoutTime], () => {
  if (timeoutEnabled.value) generalSettings.stopLayoutAfter = timeoutTime.value;
  else generalSettings.stopLayoutAfter = null;
});
</script>

<template>
  <div id="theme-settings">
    <b style="margin-bottom: 5px">General settings:</b>
    <div>
      <div>
        <input type="checkbox" :onchange="previewScopeChange" />
        Fetch full file
      </div>
      <div style="display: flex">
        <input type="checkbox" v-model="timeoutEnabled" />
        <div>
          Stop layouting after
          <input type="number" v-model="timeoutTime" />
          secs
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
#theme-settings {
  padding: 5px;
}
</style>
