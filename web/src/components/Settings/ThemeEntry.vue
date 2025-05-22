<script setup lang="ts">
import { onMounted, type Ref, ref } from "vue";
import { themeToArray, setTheme, type Theme } from "../../theme.ts";

const themeProp = defineProps<{ theme: Theme }>();

const themeList: Ref<string[] | null> = ref(null);
const themeName: Ref<string | null> = ref(null);
const themeFlavour: Ref<"dark" | "light" | null> = ref(null);

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
  <div class="theme-entry" :onclick="onClickSetTheme">
    <div class="theme-title">
      <div>{{ themeName }}</div>
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
}
.theme-entry:hover,
.theme-entry:focus {
  filter: brightness(125%);
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
