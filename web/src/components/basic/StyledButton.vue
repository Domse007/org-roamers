<script setup lang="ts">
const props = defineProps<{ text: string; bg: string; fg: string }>();
const buttonClicked = () => {
  emit("buttonClicked");
};
const emit = defineEmits(["buttonClicked"]);
</script>

<template>
  <div
    :style="{ backgroundColor: props.bg, color: props.fg }"
    class="styled-button"
    :onclick="buttonClicked"
  >
    {{ props.text }}
  </div>
</template>

<style scoped>
.styled-button {
  border-radius: 5px;
  padding: 5px;
  cursor: pointer;
  user-select: none;
}

.styled-button:hover {
  filter: brightness(125%);
}

.styled-button:active {
  filter: brightness(75%);
}
</style>
