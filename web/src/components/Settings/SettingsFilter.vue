<script setup lang="ts">
import { type Ref, ref } from "vue";
import StyledButton from "../basic/StyledButton.vue";
import FilterConstraint from "./FilterConstraint.vue";
import DropDown from "../basic/DropDown.vue";

const tagsRef: Ref<string[]> = ref(["eth", "ddca"]);
const filtersRef: Ref<number> = ref(0);
</script>

<template>
  <div id="theme-settings">
<div style="display: flex; justify-content: space-between;">
    <b style="margin-bottom: 5px">Filter settings:</b>
  <em style="color: var(--warn);">WIP: very broken</em>
</div>
    <div>
      <div :style="{ display: 'flex', marginTop: '5px' }">
        <DropDown
          :items="['includes', 'excludes']"
          :initial="0"
          style="width: 50%; margin-right: 5px"
        ></DropDown>
        <DropDown :items="tagsRef" :initial="0" style="width: 50%"></DropDown>
      </div>
      <hr style="color: var(--highlight)" />
      <div style="margin-bottom: 5px;">
        <div
          style="border-left: 2px solid var(--highlight)"
        >
          <FilterConstraint
            v-for="_i in filtersRef"
            :tags="tagsRef"
            @settings-change="(val) => console.log(val)"
          ></FilterConstraint>
        </div>
        <div style="display: flex; justify-content: center">
          <div
            align="center"
            style="background-color: var(--highlight-2); margin-right: 5px;"
            id="add-constraint"
            :onclick="() => filtersRef++"
          >
            +
          </div>
          <div
            align="center"
            id="add-constraint"
            style="background-color: var(--warn);"
            :onclick="() => filtersRef--"
          >
            -
          </div>
        </div>
      </div>
    </div>
    <div>
      <StyledButton
        text="Apply Filter"
        bg="var(--clickable)"
        fg="var(--text)"
        @button-clicked=""
      >
      </StyledButton>
    </div>
  </div>
</template>

<style scoped>
#theme-settings {
  padding: 5px;
}
#add-constraint {
  border-radius: 5px;
  color: var(--base);
  width: 30px;
  user-select: none;
}

#add-constraint:hover {
  filter: brightness(125%);
}

#add-constraint:active {
  filter: brightness(75%);
}
</style>
