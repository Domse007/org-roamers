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
  <button
    class="theme-entry"
    :class="{ active: isActive }"
    @click="onClickSetTheme"
    :aria-label="`Select ${themeName} ${themeFlavour} theme`"
  >
    <div class="theme-info">
      <div class="theme-name">
        {{ themeName }}
        <span v-if="isActive" class="theme-checkmark">✓</span>
      </div>
      <div class="theme-flavour">{{ themeFlavour }}</div>
    </div>
    <div class="theme-colors">
      <div
        v-for="color in themeList"
        :key="color"
        class="theme-color"
        :style="{ backgroundColor: color }"
        :title="color"
      ></div>
    </div>
  </button>
</template>

<style scoped>
.theme-entry {
  width: 100%;
  padding: 12px;
  border: 1px solid color-mix(in srgb, var(--highlight) 20%, transparent);
  border-radius: 4px;
  background: var(--base);
  color: var(--text);
  cursor: pointer;
  transition: all 0.15s ease;
  font-family: var(--font);
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.theme-entry:hover {
  border-color: color-mix(in srgb, var(--highlight) 40%, transparent);
  background: color-mix(in srgb, var(--highlight) 5%, var(--base));
}

.theme-entry:focus {
  outline: none;
  border-color: var(--highlight);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--highlight) 20%, transparent);
}

.theme-entry.active {
  border-color: var(--highlight);
  background: color-mix(in srgb, var(--highlight) 10%, var(--surface));
  box-shadow: 0 0 0 1px var(--highlight);
}

.theme-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.theme-name {
  font-weight: 500;
  font-size: 14px;
  color: var(--text);
  display: flex;
  align-items: center;
  gap: 6px;
}

.theme-checkmark {
  color: var(--highlight);
  font-weight: 600;
}

.theme-flavour {
  font-size: 12px;
  color: var(--overlay);
  text-transform: capitalize;
  padding: 2px 6px;
  background: color-mix(in srgb, var(--overlay) 15%, transparent);
  border-radius: 3px;
}

.theme-colors {
  display: flex;
  gap: 4px;
  justify-content: flex-start;
  flex-wrap: wrap;
}

.theme-color {
  width: 14px;
  height: 14px;
  border-radius: 3px;
  border: 1px solid color-mix(in srgb, var(--text) 20%, transparent);
  flex-shrink: 0;
}

/* Responsive adjustments */
@media (max-width: 600px) {
  .theme-entry {
    padding: 10px;
  }
  
  .theme-name {
    font-size: 13px;
  }
  
  .theme-flavour {
    font-size: 11px;
  }
  
  .theme-color {
    width: 12px;
    height: 12px;
  }
}
</style>
