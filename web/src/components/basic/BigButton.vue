<script setup lang="ts">
const props = defineProps<{ bg: string; fg: string }>();
const buttonClicked = () => {
  emit("buttonClicked");
};
const emit = defineEmits(["buttonClicked"]);
</script>

<template>
  <div
    :style="{ backgroundColor: props.bg, color: props.fg }"
    class="big-button"
    :onclick="buttonClicked"
  >
    <slot />
  </div>
</template>

<style scoped>
.big-button {
  border-radius: 5px;
  padding: 2px;
  width: 42px;
  min-width: 42px;
  height: 42px;
  min-height: 42px;
  margin: 12px;
  font-size: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  cursor: pointer;
}

.big-button:hover {
  filter: brightness(125%);
}

.big-button:active {
  filter: brightness(75%);
}
</style>
