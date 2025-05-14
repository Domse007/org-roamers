<script setup lang="ts">
defineEmits(["dialogClose"]);
</script>

<template>
  <div class="error-dialog">
    <div class="error-title-wrapper">
      <div class="error-title">ERROR</div>
      <div class="error-close-button" :onclick="() => $emit('dialogClose')">
        ×
      </div>
    </div>
    <div class="error-msg"><slot /></div>
  </div>
</template>

<style scoped>
.error-dialog {
  position: absolute;
  background-color: var(--warn);
  top: 0px;
  width: 30%;
  z-index: 150;
  left: 35%;
  margin: 10px;
  padding: 5px;
  border-radius: 5px;
  font-family: var(--font);
}
.error-title-wrapper {
  display: flex;
  justify-content: space-between;
}
.error-close-button {
  border: none;
  cursor: pointer;
  border-radius: 5px;
  background-color: black;
  opacity: 50%;
  color: var(--text);
}
.error-close-button:hover {
  filter: brightness(125%);
}
.error-close-button:active {
  filter: brightness(75%);
}
.error-title {
  color: var(--warn) !important;
  filter: brightness(50%);
  font-size: larger;
}
.error-msg {
  color: var(--text);
}
</style>
