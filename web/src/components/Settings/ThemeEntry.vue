<script setup lang="ts">
import { onMounted, type Ref, ref, computed } from "vue";
import {
  themeToArray,
  setTheme,
  type Theme,
  currentTheme,
} from "../../theme.ts";

const themeProp = defineProps<{ theme: Theme }>();

const themeList: Ref<string[] | null> = ref(null);
const themeName: Ref<string | null> = ref(null);
const themeFlavour: Ref<"dark" | "light" | null> = ref(null);

const isActive = computed(() => {
  return currentTheme.value?.name === themeProp.theme.name;
});

const onClickSetTheme = () => {
  setTheme(themeProp.theme);
  emit("redrawGraph");
};

onMounted(() => {
  themeList.value = themeToArray(themeProp.theme);
  themeName.value = themeProp.theme.name;
  themeFlavour.value = themeProp.theme.flavour;
});

const emit = defineEmits(["redrawGraph"]);
</script>

<template>
  <div
    class="theme-entry"
    :class="{ active: isActive }"
    :onclick="onClickSetTheme"
  >
    <div class="theme-title">
      <div>{{ themeName }} <span v-if="isActive">✓</span></div>
      <div>{{ themeFlavour }}</div>
    </div>
    <div class="theme-colors">
      <div
        v-for="color in themeList"
        :key="color"
        class="theme-color"
        :style="{ backgroundColor: color }"
      ></div>
    </div>
  </div>
</template>

<style scoped>
.theme-entry {
  margin-bottom: 5px;
  border-radius: 5px;
  padding: 5px;
  display: flex;
  flex-direction: column;
  background-color: var(--overlay);
  cursor: pointer;
  transition: all 0.2s ease;
}
.theme-entry:hover,
.theme-entry:focus {
  filter: brightness(125%);
}
.theme-entry.active {
  border: 2px solid var(--highlight);
  background-color: var(--surface);
}
.theme-title {
  display: flex;
  justify-content: space-between;
  width: 100%;
  margin-bottom: 5px;
}
.theme-colors {
  width: 100%;
  display: flex;
  justify-content: space-between;
}
.theme-color {
  width: 10px;
  height: 10px;
  border-radius: 5px;
}
</style>
