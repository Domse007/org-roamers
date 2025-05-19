<script setup lang="ts">
import DropDown from "../basic/DropDown.vue";

const props = defineProps<{ tags: string[] }>();
const emit = defineEmits(["settingsChange"]);

const setting = { binOp: "and", direction: "includes", tag: "" };

const propagateSetting = (what: "binOp" | "direction" | "tag", val: string) => {
  if (what == "binOp") setting.binOp = val;
  if (what == "direction") setting.direction = val;
  if (what == "tag") setting.tag = val;
  emit("settingsChange", setting);
};
</script>

<template>
  <div style="margin: 5px">
    <hr style="color: var(--highlight)" />
    <div style="display: flex; margin-bottom: 5px">
      <DropDown
        :items="['and', 'or']"
        :initial="0"
        style="width: 30%; margin-right: 5px"
        @selected="(val) => propagateSetting('binOp', val)"
      >
      </DropDown>
      <DropDown
        :items="['includes', 'excludes']"
        :initial="0"
        style="width: 70%"
        @selected="(val) => propagateSetting('direction', val)"
      >
      </DropDown>
    </div>
    <div>
      <DropDown
        :items="props.tags"
        :initial="null"
        @selected="(val) => propagateSetting('tag', val)"
      >
      </DropDown>
    </div>
    <hr style="color: var(--highlight)" />
  </div>
</template>

<style scoped></style>
